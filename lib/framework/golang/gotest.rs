use std::collections::HashSet;
use std::ops::Range;
use std::sync::LazyLock;

use crate::core::errors::FrameworkError;
use crate::core::metadata::RunnableMeta;
use crate::core::types::Command;
use crate::core::types::CursorPosition;
use crate::core::types::Runnable;
use crate::core::types::Target;
use crate::core::{
    enums::Capability,
    traits::{Framework, FrameworkProvider},
    types::CapabilityDetails,
};

use super::treesitter::operations;
use crate::core::enums::Language as crate_language;
use crate::framework::golang::treesitter::operations::{get_build_tags, parse_tree};
use crate::treesitter::node as crate_treesitter_node;
use tree_sitter::Language;
use tree_sitter::Node;
use tree_sitter::Query;
use tree_sitter::QueryCursor;

pub struct GotestProvider;

static FILE_SUFFIX: &str = "_test.go";

static SEARCH_STRATEGIES: LazyLock<HashSet<CapabilityDetails>> = LazyLock::new(|| {
    let mut res = HashSet::with_capacity(3);
    res.insert(CapabilityDetails {
        capability: Capability::TestRunner,
        search: crate::core::enums::Search::Nearest,
        description: "Test Nearest".to_string(),
    });
    res.insert(CapabilityDetails {
        capability: Capability::TestRunner,
        search: crate::core::enums::Search::Method,
        description: "Test Function".to_string(),
    });
    res.insert(CapabilityDetails {
        capability: Capability::TestRunner,
        search: crate::core::enums::Search::File,
        description: "Test File".to_string(),
    });

    res
});

impl FrameworkProvider for GotestProvider {
    fn create(&self) -> Box<dyn Framework> {
        Box::new(GotestProvider::new())
    }

    fn name(&self) -> &'static str {
        "GoTest"
    }

    fn language(&self) -> crate_language {
        crate_language::Golang
    }

    fn capability(&self) -> Capability {
        Capability::TestRunner
    }
}

impl Framework for GotestProvider {
    fn detect(&self, target: &Target) -> bool {
        if target.category != self.capability() {
            return false;
        }

        target.buffer.filepath.to_string().ends_with(FILE_SUFFIX)
    }

    fn generate_command(&self, runnable: Runnable) -> Command {
        let mut cmd = Command {
            command: "go".to_string(),
            args: vec!["test".to_string(), "-v".to_string()],
        };

        cmd.args.push(runnable.filepath);
        if let Some(meta) = runnable.meta.get_meta() {
            if !meta.build_tags.is_empty() {
                cmd.args
                    .push(format!("-tags={}", meta.build_tags.join(",")));
            }
        }
        cmd
    }

    fn runnables(&self, target: &Target) -> Result<Vec<Runnable>, FrameworkError> {
        /*
         * Goals
         *   - Search set to nearest, return singular test.
         *       If the test contains a subtest, check if that subtest contains the cursory position
         *   - Search set to method, return singular top level test
         *   - Search set to file, return all test names in a file
         *   -
         * */
        let tree = parse_tree::op::execute(target.buffer.content)?;
        let mut walker = tree.walk();
        walker.goto_first_child_for_point(target.buffer.position.to_point());
        let walker_node = walker.node();
        match target.search_strategy {
            crate::core::enums::Search::File => {
                let parent_runnables = self.get_all_test_methods(walker_node, target);
                if parent_runnables.is_none() {
                    return Err(FrameworkError::NotFoundError(
                        "Go Test Function not found no tests in this file".to_string(),
                    ));
                }
                let parent_runnables = parent_runnables.unwrap();
                let mut res: Vec<Runnable> = vec![];
                for parent in parent_runnables.into_iter() {
                    let subtests =
                        golang_subtests::get_sub_tests(walker_node, parent.to_owned(), target);
                    if let Some(sub) = subtests {
                        res.extend(sub);
                    } else {
                        res.push(parent);
                    }
                }
                Ok(res)
            }
            crate::core::enums::Search::Method => {
                let res = self.get_single_test_method(walker_node, target);
                if res.is_none() {
                    return Err(FrameworkError::NotFoundError(
                        "Go Test Function not found at position".to_string(),
                    ));
                }
                Ok(vec![res.unwrap()])
            }
            crate::core::enums::Search::Nearest => {
                let parent_runnable = self.get_single_test_method(walker_node, target);
                if parent_runnable.is_none() {
                    return Err(FrameworkError::NotFoundError(
                        "Go Test Function not found at position".to_string(),
                    ));
                }

                let parent_runnable = parent_runnable.unwrap();
                let subtests =
                    golang_subtests::get_sub_tests(walker_node, parent_runnable.to_owned(), target);
                if subtests.is_none() {
                    return Ok(vec![parent_runnable]);
                }

                Ok(subtests.unwrap())
            }
        }
    }

    fn capabilities(&self) -> &HashSet<CapabilityDetails> {
        LazyLock::force(&SEARCH_STRATEGIES)
    }

    fn search_for_capability(&self, description: &str) -> Option<&CapabilityDetails> {
        let c = LazyLock::force(&SEARCH_STRATEGIES);
        c.iter().find(|s| s.description == description).clone()
    }
}

impl GotestProvider {
    pub fn new() -> Self {
        Self {}
    }

    fn get_test_function_query(&self) -> Option<Query> {
        const QUERY_PATTERN: &str = r#"
            [[((function_declaration 
                    name: (identifier) @test_name
                    parameters: (parameter_list
                        (parameter_declaration
                                 name: (identifier)
                                 type: (pointer_type
                                     (qualified_type
                                      package: (package_identifier) @_param_package
                                      name: (type_identifier) @_param_name))))
                     ) @testfunc
                  (#contains? @test_name "Test"))]]
            "#;
        let query = Query::new(&Language::new(tree_sitter_go::LANGUAGE), QUERY_PATTERN).ok()?;
        Some(query)
    }

    fn get_single_test_method(&self, node: Node, target: &Target) -> Option<Runnable> {
        let current_node_position = node.start_position();
        let query = self.get_test_function_query()?;
        let content = target.buffer.content;
        let test_name_index = query.capture_index_for_name("test_name")?;
        let test_function_index = query.capture_index_for_name("testfunc")?;
        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(&query, node, content.as_bytes());
        for node_matched in query_matches.into_iter() {
            let function_node = node_matched
                .captures
                .iter()
                .filter(|c| c.index == test_function_index)
                .map(|c| c.node)
                .next();

            if function_node.is_none() {
                continue;
            }

            let function_node = function_node.unwrap();

            for m in node_matched
                .captures
                .iter()
                .filter(|c| c.index == test_name_index)
            {
                if m.node.start_position().row <= current_node_position.row
                    && m.node.end_position().row >= current_node_position.row
                {
                    return Some(Runnable {
                        name: crate_treesitter_node::node_text(m.node, content),
                        filepath: target.buffer.filepath.to_string(),
                        range: Range {
                            start: CursorPosition::from_point(function_node.start_position()),
                            end: CursorPosition::from_point(function_node.end_position()),
                        },
                        meta: RunnableMeta::default_golang(),
                    });
                }
            }
        }

        None
    }

    fn get_all_test_methods(&self, node: Node, target: &Target) -> Option<Vec<Runnable>> {
        let content = target.buffer.content;
        const QUERY_PATTERN: &str = r#"
            [[((function_declaration 
                    name: (identifier) @test_name
                    parameters: (parameter_list
                        (parameter_declaration
                                 name: (identifier)
                                 type: (pointer_type
                                     (qualified_type
                                      package: (package_identifier) @_param_package
                                      name: (type_identifier) @_param_name))))
                     ) @testfunc
                  (#contains? @test_name "Test"))]]
            "#;
        let query = Query::new(&Language::new(tree_sitter_go::LANGUAGE), QUERY_PATTERN).ok()?;
        let test_name_index = query.capture_index_for_name("test_name")?;
        let test_function_index = query.capture_index_for_name("testfunc")?;
        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(&query, node, content.as_bytes());

        let mut parent_runnables: Vec<Runnable> = vec![];
        for node_matched in query_matches {
            let function_node = node_matched
                .captures
                .iter()
                .filter(|c| c.index == test_function_index)
                .map(|c| c.node)
                .next();

            if function_node.is_none() {
                continue;
            }

            let function_node = function_node.unwrap();

            for m in node_matched
                .captures
                .iter()
                .filter(|c| c.index == test_name_index)
            {
                parent_runnables.push(Runnable {
                    name: crate_treesitter_node::node_text(m.node, content),
                    filepath: target.buffer.filepath.to_string(),
                    range: Range {
                        start: CursorPosition::from_point(function_node.start_position()),
                        end: CursorPosition::from_point(function_node.end_position()),
                    },
                    meta: RunnableMeta::default_golang(),
                });
            }
        }

        if parent_runnables.is_empty() {
            None
        } else {
            Some(parent_runnables)
        }
    }
}

pub(in crate::framework::golang) mod golang_subtests {
    use std::ops;

    use tree_sitter::{Language, Node, Query, QueryCursor};

    use crate::{
        core::types::{CursorPosition, Runnable, Target},
        treesitter::node::node_text,
    };

    pub(in crate::framework::golang) fn get_sub_tests(
        node: Node,
        parent: Runnable,
        target: &Target,
    ) -> Option<Vec<Runnable>> {
        let mut cursor_position = None;
        if target
            .search_strategy
            .eq(&crate::core::enums::Search::Nearest)
        {
            cursor_position = Some(target.buffer.position);
        }

        let mut res = vec![];
        let finders = [
            get_string_literal_subtests,
            get_in_loop_with_named_subtests,
            get_in_loop_with_unnamed_subtests,
            get_in_loop_typed_subcase_with_unnamed_case_fields,
            get_in_loop_typed_subcase_with_named_case_fields,
            get_out_of_loop_named_subtests,
            get_out_of_loop_unnamed_subtests,
        ];
        for func in finders {
            if let Some(t) = func(
                node,
                target.buffer.content,
                parent.to_owned(),
                cursor_position,
            ) {
                res.extend(t);
            }
        }

        if res.is_empty() {
            None
        } else {
            Some(res)
        }
    }

    // get_string_literal_subtests
    // Example:
    // package golang
    // import (
    //   "testing"
    //
    //   "github.com/stretchr/testify/assert"
    // )
    //
    // func sample_add(a, b int) int {
    //   return a + b
    // }
    //
    // func TestSample(t *testing.T) {
    //     t.Run("case_a", func(t *testing.T){
    //       assert.Equal(t, 1, sample_add(1, 0))
    //     })
    //     t.Run("case_b", func(t *testing.T){
    //       assert.Equal(t, 2, sample_add(1, 2))
    //     })
    // }
    fn get_string_literal_subtests(
        node: Node,
        content: &str,
        parent: Runnable,
        cursor_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        const QUERY_PATTERN: &str = r#"
            [[
              ;; string literal sub test
              (((expression_statement
                  (call_expression
                      function: (selector_expression
                          operand: (identifier) @testing
                            field: (field_identifier) @testing.method (#eq? @testing.method "Run")
                        )
                        arguments: (argument_list
                          (interpreted_string_literal) @test.case.name.value
                            (func_literal
                              parameters: (parameter_list
                                  (parameter_declaration
                                      name: (identifier)
                                        type: (pointer_type
                                          (qualified_type
                                            package: (package_identifier) @test.case.package  
                                              name: (type_identifier) @test.case.package.param 
                        )	
                      )
                                    )
                                )
                            )
                        )
                    )
                ) @test.case
              ))
            ]]
        "#;

        let query = Query::new(&Language::new(tree_sitter_go::LANGUAGE), QUERY_PATTERN).ok()?;
        let subcase_name_index = query.capture_index_for_name("test.case.name.value")?;
        let subcase_index = query.capture_index_for_name("test.case")?;
        let mut cursor = QueryCursor::new();
        cursor.set_point_range(ops::Range {
            start: parent.range.start.to_point(),
            end: parent.range.end.to_point(),
        });
        let query_matches = cursor.matches(&query, node, content.as_bytes());
        let mut runnables = vec![];

        for node_matched in query_matches {
            let subtest_capture = node_matched
                .captures
                .iter()
                .find(|capture| capture.index == subcase_index)
                .map(|capture| capture.node);

            if subtest_capture.is_none() {
                continue;
            }

            let subtest_node = subtest_capture.unwrap();
            if let Some(position) = cursor_position {
                let r = subtest_node.range();
                if !position.in_range(std::ops::Range {
                    start: r.start_point,
                    end: r.end_point,
                }) {
                    continue;
                }
            }
            let subtest = node_matched
                .captures
                .iter()
                .find(|capture| capture.index == subcase_name_index)
                .map(|capture| node_text(capture.node, content));
            if subtest.is_none() {
                continue;
            }
            let subtest = subtest.unwrap();
            let runnable = Runnable {
                name: parent.name.to_owned() + "/" + subtest.replace("\"", "").as_str(),
                filepath: parent.filepath.clone(),
                range: std::ops::Range {
                    start: CursorPosition::from_point(subtest_node.start_position()),
                    end: CursorPosition::from_point(subtest_node.end_position()),
                },
                meta: crate::core::metadata::RunnableMeta::default_golang(),
            };
            runnables.push(runnable);
        }

        if runnables.is_empty() {
            None
        } else {
            Some(runnables)
        }
    }

    // get_in_loop_with_named_subtest
    // Example:
    // func TestTableTest(t *testing.T) {
    // 	for _, tt := range []struct {
    // 		description string
    // 		a           int
    // 		b           int
    // 		expected    int
    // 	}{
    // 		{
    // 			description: "base case",
    // 			a:           0,
    // 			b:           3,
    // 			expected:    3,
    // 		},
    // 		{
    // 			description: "case 1",
    // 			a:           1,
    // 			b:           3,
    // 			expected:    4,
    // 		},
    // 	} {
    // 		t.Run(tt.description, func(t *testing.T) {
    // 			actual := sample_add(tt.a, tt.b)
    // 			assert.Equal(t, tt.expected, actual)
    // 		})
    // 	}
    // }
    fn get_in_loop_with_named_subtests(
        node: Node,
        content: &str,
        parent: Runnable,
        current_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        const QUERY_PATTERN: &str = r#"
        [[
          ((for_statement
            (range_clause
              left: (expression_list
                (identifier)
                (identifier) @test.loop.case.variable
              )
              right: (composite_literal
                type: (slice_type
                  element: (struct_type
                    (field_declaration_list
                      (field_declaration
                      	name: (field_identifier) @test.case.definition.field
                        type: (type_identifier) @test.case.definition.field.type (#eq? @test.case.definition.field.type "string")
                      )
                    )
                  ) @test.case.type
                )
                body: (literal_value
                  (literal_element
                    (literal_value
                      (keyed_element
                        (literal_element
                          (identifier)
                        )  @test.case.field.name (#eq? @test.case.field.name @test.case.definition.field)
                        (literal_element
                          (interpreted_string_literal) @test.case.field.value
                        )
                      )
                    ) @test.case
                  )
                )
              )
            )
            body: (block
              (expression_statement
                (call_expression
                  function: (selector_expression
                    operand: (identifier) @test.loop.test
                    field: (field_identifier) @test.loop.test.method (#eq? @test.loop.test.method "Run")
                  )
                  arguments: (argument_list
                    (selector_expression
                      operand: (identifier) @test.loop.test.variable (#eq? @test.loop.test.variable @test.loop.case.variable)
                      field: (field_identifier) @test.loop.test.variable.field (#eq? @test.loop.test.variable.field @test.case.definition.field)
                    ) 
                  )
                )
              )
            )
          ))
        ]]"#;
        subtest_loop_find_helper(node, content, QUERY_PATTERN, parent, current_position)
    }

    // get_in_loop_with_unnamed_subtests
    // Example:
    // func TestTableTest(t *testing.T) {
    // 	for _, tt := range []struct {
    // 		description string
    // 		a           int
    // 		b           int
    // 		expected    int
    // 	}{
    // 		{
    // 			"base case",
    // 			0,
    // 			3,
    // 			3,
    // 		},
    // 		{
    // 			"case 1",
    // 			1,
    // 			3,
    // 			4,
    // 		},
    // 	} {
    // 		t.Run(tt.description, func(t *testing.T) {
    // 			actual := sample_add(tt.a, tt.b)
    // 			assert.Equal(t, tt.expected, actual)
    // 		})
    // 	}
    // }
    fn get_in_loop_with_unnamed_subtests(
        node: Node,
        content: &str,
        parent: Runnable,
        current_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        const QUERY_PATTERN: &str = r#"
            [[
              ;; query for function name
              ((function_declaration 
                                name: (identifier) @_test.parent.name
                                parameters: (parameter_list
                                    (parameter_declaration
                                             name: (identifier) @_test.parent.var
                                             type: (pointer_type
                                                 (qualified_type
                                                  package: (package_identifier) @_test.param_package
                                                  name: (type_identifier) @_test.param_name))))
                                 ) @testfunc
                              (#contains? @_test.parent.name "Test"))
              ;; query for list table tests (wrapped in loop)
              (for_statement
                (range_clause
                  left: (expression_list
                    (identifier)
                    (identifier) @test.loop.case.variable
                  )
                  right: (composite_literal
                    type: (slice_type
                      element: (struct_type
                        (field_declaration_list
                          (field_declaration
                            name: (field_identifier) @test.case.definition.field
                            type: (type_identifier) @test.case.definition.field.type (#eq? @test.case.definition.field.type "string")
                          )
                        )
                      )
                    )
                    body: (literal_value
                      (literal_element
                        (literal_value
                          (literal_element
                            (interpreted_string_literal) @test.case.field.value
                          )
                        ) 
                      ) @test.case
                    )
                  )
                )
                body: (block
                  (expression_statement
                    (call_expression
                      function: (selector_expression
                        operand: (identifier) @test.loop.test
                        field: (field_identifier) @test.loop.test.method (#eq? @test.loop.test.method "Run")
                      )
                      arguments: (argument_list
                        (selector_expression
                          operand: (identifier) @test.loop.test.variable (#eq? @test.loop.test.variable @test.loop.case.variable)
                          field: (field_identifier) @test.loop.test.variable.field (#eq? @test.loop.test.variable.field @test.case.definition.field)
                        )
                      )
                    )
                  )
                )
              )
            ]]
        "#;
        subtest_loop_find_helper(node, content, QUERY_PATTERN, parent, current_position)
    }

    // get_in_loop_typed_testcase_with_unnamed_case_fields
    // Example:
    // func TestTableTest(t *testing.T) {
    // 	type Scenario struct {
    // 		description string
    // 		a           int
    // 		b           int
    // 		expected    int
    // 	}
    // 	for _, tt := range []Scenario{
    // 		{
    // 			"base case",
    // 			0,
    // 			3,
    // 			3,
    // 		},
    // 		{
    // 			"case 1",
    // 			1,
    // 			3,
    // 			4,
    // 		},
    // 	} {
    // 		t.Run(tt.description, func(t *testing.T) {
    // 			actual := sample_add(tt.a, tt.b)
    // 			assert.Equal(t, tt.expected, actual)
    // 		})
    // 	}
    // }
    fn get_in_loop_typed_subcase_with_unnamed_case_fields(
        node: Node,
        content: &str,
        parent: Runnable,
        current_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        const QUERY_PATTERN: &str = r#"
            [[
              (((type_declaration
                  (type_spec
                      name: (type_identifier) @test.case.variable.name
                        type: (struct_type 
                          (field_declaration_list
                              (field_declaration
                                name: (field_identifier) @test.case.definition.field
                                type: (type_identifier) @test.case.definition.field.type (#eq? @test.case.definition.field.type "string")
                          )
                        )
                    ) @test.case.type
                  )
                ) 
                (for_statement
                    (range_clause
                      left: (expression_list
                        (identifier)
                        (identifier) @test.loop.case.variable
                      )
                      right: (composite_literal
                          type: (slice_type
                          element: (type_identifier) @test.loop.case.variable.type (#eq? @test.loop.case.variable.type @test.case.variable.name)
                        )
                        body: (literal_value
                          (literal_element
                            (literal_value
                              (literal_element
                                (interpreted_string_literal) @test.case.field.value
                              )
                            )
                          ) @test.case
                        )
                      )
                    )
                    body: (block
                      (expression_statement
                        (call_expression
                          function: (selector_expression
                            operand: (identifier) @test.loop.test
                            field: (field_identifier) @test.loop.test.method (#eq? @test.loop.test.method "Run")
                          )
                          arguments: (argument_list
                            (selector_expression
                                operand: (identifier) @test.loop.test.variable (#eq? @test.loop.case.variable @test.loop.test.variable)
                                field: (field_identifier) @test.loop.test.variable.field (#eq? @test.case.definition.field @test.loop.test.variable.field)
                            )
                          )
                        )
                      )
                    )
                  )
              ))
            ]]"#;
        subtest_loop_find_helper(node, content, QUERY_PATTERN, parent, current_position)
    }

    // get_in_loop_typed_testcase_with_named_case_fields
    // Defines a loop with a slice created with an outer variable.
    //
    // Example:
    // func TestSample(t *testing.T) {
    // 	type Scenario struct {
    //     	description string
    //         a			int
    //         b			int
    //         expected	int
    //     }
    //
    // 	for _, tt := range []Scenario{
    // 		{
    // 			description: "base case",
    // 			a:          0,
    // 			b:          3,
    // 			c:          3,
    // 		},
    // 		{
    // 			description: "case 1",
    // 			a:           1,
    // 			b:           3,
    // 			expected:    4,
    // 		},
    // 	}  {
    // 		t.Run(tt.description, func(t *testing.T) {
    // 			actual := sample_add(tt.a, tt.b)
    // 			assert.Equal(t, tt.expected, actual)
    // 		})
    // 	}
    // }
    fn get_in_loop_typed_subcase_with_named_case_fields(
        node: Node,
        content: &str,
        parent: Runnable,
        current_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        const QUERY_PATTERN: &str = r#"
		        [[
              (((type_declaration
                  (type_spec
                      name: (type_identifier) @test.case.variable.name
                        type: (struct_type 
                          (field_declaration_list
                          		(field_declaration
                          			name: (field_identifier) @test.case.definition.field
                          			type: (type_identifier) @test.case.definition.field.type (#eq? @test.case.definition.field.type "string")
                          		)
                        	) 
                    	) @test.case.type
                  	) 
                )
                (for_statement
                	(range_clause
                		left: (expression_list
                			(identifier)
                			(identifier) @test.loop.case.variable
                	)
                		right: (composite_literal
                			type: (slice_type
                			element: (type_identifier) @test.loop.case.variable.type (#eq? @test.loop.case.variable.type @test.case.variable.name)
                		)
                		body: (literal_value
                				(literal_element
                					(literal_value
                						(keyed_element
                							(literal_element
                									(identifier)
                							)  @test.case.field.name
                							(literal_element
                								(interpreted_string_literal) @test.case.field.value
                						)
                					)
                				) @test.case
                			)
                		)
                	)
                )
                body: (block
                		(expression_statement
                			(call_expression
                				function: (selector_expression
                					operand: (identifier) @test.loop.test
                					field: (field_identifier) @test.loop.test.method (#eq? @test.loop.test.method "Run")
                				)
                				arguments: (argument_list
                					(selector_expression
                						operand: (identifier) @test.loop.test.variable (#eq? @test.loop.case.variable @test.loop.test.variable)
                            field: (field_identifier) @test.loop.test.variable.field (#eq? @test.case.definition.field @test.loop.test.variable.field)
                					)
                				)
                			)
                		)
                	)
                )
              ))
            ]]"#;
        subtest_loop_find_helper(node, content, QUERY_PATTERN, parent, current_position)
    }

    // get_out_of_loop_named_subtests
    // Example:
    // func TestTableTest(t *testing.T) {
    // 	scenarios := []struct {
    // 		description string
    // 		a           int
    // 		b           int
    // 		expected    int
    // 	}{
    // 		{
    // 			description: 	"base case",
    // 			a: 				0,
    // 			b: 				3,
    // 			expected:		3,
    // 		},
    // 		{
    // 			description: "case 1",
    // 			a:           1,
    // 			b:           3,
    // 			expected:    4,
    // 		},
    // 	}
    //
    // 	for _, tt := range scenarios {
    // 		t.Run(tt.description, func(t *testing.T) {
    // 			actual := sample_add(tt.a, tt.b)
    // 			assert.Equal(t, tt.expected, actual)
    // 		})
    // 	}
    // }
    fn get_out_of_loop_named_subtests(
        node: Node,
        content: &str,
        parent: Runnable,
        current_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        const QUERY_PATTERN: &str = r#"
        [[
          ((block (
            short_var_declaration (
              (expression_list
                  (identifier) @test.cases.variable.name
                )
                right: (expression_list
                  (composite_literal
                      type: (slice_type
                          element: (struct_type
                              (field_declaration_list
                                   (field_declaration 
                                  name: (field_identifier) @test.case.definition.field
                                    type: (type_identifier) @test.case.definition.field.type (#eq? @test.case.definition.field.type "string")
                                    )
                                ) @test.case.type
                            )
                        )
                      body: (literal_value
                        (literal_element
                          (literal_value
                              (keyed_element
                                  (literal_element
                                      (identifier) @test.case.field.name	(#eq? @test.case.field.name @test.case.definition.field)
                                    )
                                    (literal_element
                                      (interpreted_string_literal) @test.case.field.value
                                    )
                                )
                            ) @test.case
                        )
                      )
                    ) 	
                )
            ))
            (for_statement
              (range_clause
                  left: (expression_list
                      (identifier)
                        (identifier) @test.loop.case.variable
                    )
                    right: (identifier) @test.loop.cases.variable.name (#eq? @test.loop.cases.variable.name @test.cases.variable.name)
                )
                body: (block
                  (expression_statement
                      (call_expression
                          function: (selector_expression
                              operand: (identifier) @test.loop.test
                                field: (field_identifier) @test.loop.test.method (#eq? @test.loop.test.method "Run")
                            )
                            arguments: (argument_list
                              (selector_expression
                                  operand: (identifier) @test.loop.test.variable (#eq? @test.loop.test.variable @test.loop.case.variable)
                                    field: (field_identifier) @test.loop.test.variable.field (#eq? @test.loop.test.variable.field @test.case.definition.field)
                                )
                            )
                        )
                    )
                )
          	)
          ))
        ]]"#;
        subtest_loop_find_helper(node, content, QUERY_PATTERN, parent, current_position)
    }

    // get_out_of_loop_unnamed_subtests
    // example:
    // func TestTableTest(t *testing.T) {
    // 	scenarios := []struct {
    // 		description string
    // 		a           string
    // 		b           int
    // 		expected    int
    // 	}{
    // 		{
    // 			"base case",
    // 			0,
    // 			3,
    // 			3,
    // 		},
    // 		{
    // 			"case 1",
    // 			1,
    // 			3,
    // 			4,
    // 		},
    // 	}
    //
    // 	for _, tt := range scenarios {
    // 		t.Run(tt.description, func(t *testing.T) {
    // 			actual := sample_add(tt.a, tt.b)
    // 			assert.Equal(t, tt.expected, actual)
    // 		})
    // 	}
    // }
    fn get_out_of_loop_unnamed_subtests(
        node: Node,
        content: &str,
        parent: Runnable,
        current_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        const QUERY_PATTERN: &str = r#"
          [[
              ((block (
                short_var_declaration (
                  (expression_list
                      (identifier) @test.cases.variable.name
                    )
                    right: (expression_list
                      (composite_literal
                          type: (slice_type
                              element: (struct_type
                                  (field_declaration_list
                                       (field_declaration 
                                      name: (field_identifier) @test.case.definition.field
                                        type: (type_identifier) @test.case.definition.field.type (#eq? @test.case.definition.field.type "string")
                                        )
                                    ) @test.case.type
                                )
                            )
                          body: (literal_value
                            (literal_element
                              (literal_value
                                     (literal_element
                                        (interpreted_string_literal) @test.case.field.value
                                     )
                                    
                                ) @test.case
                            )
                          )
                        ) 	
                    )
                ))
                (for_statement
                  (range_clause
                      left: (expression_list
                          (identifier)
                            (identifier) @test.loop.case.variable
                        )
                        right: (identifier) @test.loop.cases.variable.name (#eq? @test.loop.cases.variable.name @test.cases.variable.name)
                    )
                    body: (block
                      (expression_statement
                          (call_expression
                              function: (selector_expression
                                  operand: (identifier) @test.loop.test
                                    field: (field_identifier) @test.loop.test.method (#eq? @test.loop.test.method "Run")
                                )
                                arguments: (argument_list
                                  (selector_expression
                                      operand: (identifier) @test.loop.test.variable (#eq? @test.loop.test.variable @test.loop.case.variable)
                                        field: (field_identifier) @test.loop.test.variable.field (#eq? @test.loop.test.variable.field @test.case.definition.field)
                                    )
                                )
                            )
                        )
                    )
              ))
              )
            ]]"#;
        subtest_loop_find_helper(node, content, QUERY_PATTERN, parent, current_position)
    }

    // subtest_loop_find_helper
    // Supports finding subtests in loops under various set ups. The only requirement
    // of this method is that there is singular tree sitter query and the query
    // has the following capture names defined
    // - test.case.field.value
    //      Denotes the name of the subtest
    // - test.case
    //      Denotes the entire subtest. This is helpful for fetching the
    //      nearest sub-test
    fn subtest_loop_find_helper(
        node: Node,
        content: &str,
        query: &str,
        parent: Runnable,
        current_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        let query = Query::new(&Language::new(tree_sitter_go::LANGUAGE), query).ok()?;
        let subcase_name_index = query.capture_index_for_name("test.case.field.value")?;
        let subcase_index = query.capture_index_for_name("test.case")?;
        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(&query, node, content.as_bytes());
        let mut runnables = vec![];

        for node_matched in query_matches {
            let subtest_capture = node_matched
                .captures
                .iter()
                .find(|capture| capture.index == subcase_index)
                .map(|capture| capture.node);

            if subtest_capture.is_none() {
                continue;
            }

            let subtest_node = subtest_capture.unwrap();
            if let Some(position) = current_position {
                let r = subtest_node.range();
                if !position.in_range(std::ops::Range {
                    start: r.start_point,
                    end: r.end_point,
                }) {
                    continue;
                }
            }
            let subtest = node_matched
                .captures
                .iter()
                .find(|capture| capture.index == subcase_name_index)
                .map(|capture| node_text(capture.node, content));
            if subtest.is_none() {
                continue;
            }
            let subtest = subtest.unwrap();
            let runnable = Runnable {
                name: parent.name.to_owned() + "/" + subtest.replace("\"", "").as_str(),
                filepath: parent.filepath.clone(),
                range: std::ops::Range {
                    start: CursorPosition::from_point(subtest_node.start_position()),
                    end: CursorPosition::from_point(subtest_node.end_position()),
                },
                meta: crate::core::metadata::RunnableMeta::default_golang(),
            };
            runnables.push(runnable);
        }

        if runnables.is_empty() {
            None
        } else {
            Some(runnables)
        }
    }
}

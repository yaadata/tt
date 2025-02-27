use std::ops::Range;

use crate::core::errors::FrameworkError;
use crate::core::metadata::RunnableMeta;
use crate::core::types::CursorPosition;
use crate::core::types::Runnable;
use crate::core::types::Target;
use crate::core::{
    enums::{Langauge, ToolCategory},
    traits::{Framework, FrameworkProvider},
};
use crate::treesitter::node as crate_treesitter_node;
use tree_sitter::Language;
use tree_sitter::Node;
use tree_sitter::Query;
use tree_sitter::QueryCursor;

use super::common;
use super::common::utils::get_build_tags;

pub struct GotestProvider;

static FILE_SUFFIX: &str = "_test.go";

impl FrameworkProvider for GotestProvider {
    fn create(&self) -> Box<dyn Framework> {
        Box::new(GotestProvider::new())
    }

    fn name(&self) -> &'static str {
        "gotest"
    }

    fn language(&self) -> Langauge {
        crate::core::enums::Langauge::Golang
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::TestRunner
    }
}

impl GotestProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl Framework for GotestProvider {
    fn detect(&self, target: &Target) -> bool {
        if target.category != self.category() {
            return false;
        }

        target.buffer.filepath.to_string().ends_with(FILE_SUFFIX)
    }

    fn generate_command(&self, runnable: Runnable) -> String {
        "go test some".to_string()
    }

    fn runnable(&self, target: &Target) -> Result<Vec<Runnable>, FrameworkError> {
        let tree = common::utils::parse_tree(target.buffer.content);
        match tree {
            Ok(tree) => {
                let build_tags = get_build_tags(tree.root_node(), target.buffer.content);
                let runnable = get_parent_test(
                    crate_treesitter_node::position_to_nearest_point(&tree, target.buffer.position),
                    target,
                );
                let mut runnables: Vec<Runnable> = vec![];
                if let Some(runnable) = runnable {
                    let r = runnable;
                    runnables.push(r.clone());
                    if let Some(build_tags) = build_tags {
                        for t in build_tags.into_iter() {
                            let mut r = r.clone();
                            r.meta.set_build_tags(t);
                        }
                    }
                }

                Err(FrameworkError::NotFoundError(
                    "no test found at the current position".to_string(),
                ))
            }
            Err(e) => Err(e),
        }
    }
}

pub(crate) fn get_parent_test(node: Option<Node>, target: &Target) -> Option<Runnable> {
    let node = node?;
    let current_node_position = node.start_position();
    let query_pattern = r#"
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
    let content = target.buffer.content;
    let query = Query::new(&Language::new(tree_sitter_go::LANGUAGE), query_pattern).ok()?;
    let test_name_index = query.capture_index_for_name("test_name")?;
    let test_function_index = query.capture_index_for_name("testfunc")?;
    let mut cursor = QueryCursor::new();
    let query_matches = cursor.matches(&query, node, content.as_bytes());
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
            if m.node.start_position().row <= current_node_position.row
                && m.node.end_position().row >= current_node_position.row
            {
                let name = crate_treesitter_node::node_text(m.node, content);
                return Some(Runnable {
                    name,
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

pub(in crate::framework::golang) mod golang_subtests {
    use std::ops;

    use tree_sitter::{Language, Node, Query, QueryCursor};

    use crate::{
        core::types::{CursorPosition, Runnable, Target},
        treesitter::node::node_text,
    };

    pub(in crate::framework::golang) fn get_sub_tests(
        node: Option<Node>,
        parent: Option<Runnable>,
        target: &Target,
    ) -> Option<Vec<Runnable>> {
        let parent = parent?;
        let node = node?;
        let mut cursor_position = None;
        if target
            .search_strategy
            .eq(&crate::core::enums::Search::Nearest)
        {
            cursor_position = Some(target.buffer.position);
        }

        get_string_literal_subtests(node, target.buffer.content, parent, cursor_position)
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
    fn get_in_loop_with_named_subtest(
        node: Node,
        content: &str,
        cursor: Option<CursorPosition>,
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
          ;; query for table tests as a part of the loop
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
        let query = Query::new(&Language::new(tree_sitter_go::LANGUAGE), QUERY_PATTERN).ok()?;
        let function_name_capture_index = query.capture_index_for_name("_test.parent.name")?;
        let subcase_capture_index = query.capture_index_for_name("test.subcase.definition.name")?;
        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(&query, node, content.as_bytes());
        let mut runnables = vec![];

        for node_matched in query_matches {
            let function_name = node_matched
                .captures
                .iter()
                .find(|capture| capture.index == function_name_capture_index)
                .map(|capture| &content[capture.node.byte_range()]);

            let subcase = node_matched
                .captures
                .iter()
                .find(|capture| capture.index == subcase_capture_index)
                .map(|capture| &content[capture.node.byte_range()]);

            if let Some(function_name) = function_name {
                if let Some(casename) = subcase {
                    // let runnable = Runnable {
                    //     name: function_name.to_string() + "/" + casename,
                    //     filepath: "".to_string(),
                    //     meta: crate::core::metadata::RunnableMeta::default_golang(),
                    // };
                    // runnables.push(runnable);
                }
            }
        }

        Some(runnables)
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
        cursor: Option<CursorPosition>,
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
        let query = Query::new(&Language::new(tree_sitter_go::LANGUAGE), QUERY_PATTERN).ok()?;

        let function_name_capture_index = query.capture_index_for_name("_test.parent.name")?;
        let subcase_capture_index = query.capture_index_for_name("test.subcase.definition.name")?;
        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(&query, node, content.as_bytes());
        let mut runnables = vec![];

        for node_matched in query_matches {
            let function_name = node_matched
                .captures
                .iter()
                .find(|capture| capture.index == function_name_capture_index)
                .map(|capture| &content[capture.node.byte_range()]);

            let subcase = node_matched
                .captures
                .iter()
                .find(|capture| capture.index == subcase_capture_index)
                .map(|capture| &content[capture.node.byte_range()]);

            if let Some(function_name) = function_name {
                if let Some(casename) = subcase {
                    // let runnable = Runnable {
                    //     name: function_name.to_string() + "/" + casename,
                    //     filepath: "".to_string(),
                    //     meta: crate::core::metadata::RunnableMeta::default_golang(),
                    // };
                    // runnables.push(runnable);
                }
            }
        }
        Some(runnables)
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
    fn get_in_loop_typed_testcase_with_unnamed_case_fields() -> Option<Vec<Runnable>> {
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
                                (interpreted_string_literal) @test.case.name.value
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
        todo!()
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
    fn get_in_loop_typed_testcase_with_named_case_fields() -> Option<Vec<Runnable>> {
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
                							)  @test.case.name.field
                							(literal_element
                								(interpreted_string_literal) @test.case.name.value
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
        todo!()
    }

    // get_out_loop_named_tabletests
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
    fn get_out_loop_named_tabletests() -> Option<Vec<Runnable>> {
        const QUERY_PATTERN: &str = r#"
        [[
          ;; find function name
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

          ;; query to find out of loop cases
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
                                      (identifier) @test.case.name.field	(#eq? @test.case.name.field @test.case.definition.field)
                                    )
                                    (literal_element
                                      (interpreted_string_literal) @test.case.name.value
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
          ))
          )
        ]]"#;
        todo!()
    }

    // get_out_loop_unnamed_subtests
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
    fn get_out_loop_unnamed_subtests() -> Option<Vec<Runnable>> {
        const QUERY_PATTERN: &str = r#"
          [[
              ;; find function name
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

              ;; query to find out of loop cases
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
                                        (interpreted_string_literal) @test.case.name.value
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
        todo!()
    }
}

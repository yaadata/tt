use crate::core::errors::FrameworkError;
use crate::core::metadata::RunnableMeta;
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
                let runnable = top_level_test_function(
                    crate_treesitter_node::position_to_nearest_point(
                        &tree,
                        target.buffer.position.clone(),
                    ),
                    target,
                );
                let mut runnables: Vec<Runnable> = vec![];
                if let Some(runnable) = runnable {
                    let mut r = runnable;
                    r.meta.set_position(target.buffer.position.clone());
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

pub(crate) fn top_level_test_function(node: Option<Node>, target: &Target) -> Option<Runnable> {
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
    let capture_index = query.capture_index_for_name("test_name")?;
    let mut cursor = QueryCursor::new();
    let query_matches = cursor.matches(&query, node, content.as_bytes());
    for node_matched in query_matches {
        for m in node_matched
            .captures
            .iter()
            .filter(|c| c.index == capture_index)
        {
            if m.node.start_position().row <= current_node_position.row
                && m.node.end_position().row >= current_node_position.row
            {
                let name = crate_treesitter_node::node_text(m.node, content);
                return Some(Runnable {
                    name,
                    filepath: "".to_string(),
                    meta: RunnableMeta::default_golang(),
                });
            }
        }
    }

    None
}

mod golang_table_test {
    use tree_sitter::{Language, Node, Query, QueryCursor};

    use crate::core::types::{Runnable, Target};

    pub(crate) fn get_sub_test_function(
        node: Option<Node>,
        target: &Target,
    ) -> Option<Vec<Runnable>> {
        // match node {
        //     Some(node) => {}
        // }
        todo!()
    }

    fn get_in_loop_subtest(node: Node, content: &str) -> Option<Vec<Runnable>> {
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
                (identifier) @test.subcase.variable
              )
              right: (composite_literal
                type: (slice_type
                  element: (struct_type
                    (field_declaration_list
                      (field_declaration
                        name: (field_identifier)
                        type: (type_identifier)
                      )
                    )
                  )
                )
                body: (literal_value
                  (literal_element
                    (literal_value
                      (keyed_element
                        (literal_element
                          (identifier)
                        )  @test.subcase.definition.field
                        (literal_element
                          (interpreted_string_literal) @test.subcase.definition.name
                        )
                      )
                    ) @test.subcase.definition
                  )
                )
              )
            )
            body: (block
              (expression_statement
                (call_expression
                  function: (selector_expression
                    operand: (identifier) @_test.var
                    field: (field_identifier) &_test.method (#eq? @_test.method "Run")
                  )
                  arguments: (argument_list
                    (selector_expression
                      operand: (identifier)
                      field: (field_identifier) @_test.name
                    ) (#eq? @test.subcase.definition.field @_test.name)
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
                    let runnable = Runnable {
                        name: function_name.to_string() + "/" + casename,
                        filepath: "".to_string(),
                        meta: crate::core::metadata::RunnableMeta::default_golang(),
                    };
                    runnables.push(runnable);
                }
            }
        }

        Some(runnables)
    }

    fn get_in_loop_unnamed_subtests(node: Node, content: &str) -> Option<Vec<Runnable>> {
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
                    (identifier) @test.subcase.variable
                  )
                  right: (composite_literal
                    type: (slice_type
                      element: (struct_type
                        (field_declaration_list
                          (field_declaration
                            name: (field_identifier) @test.subcase.definition.field
                            type: (type_identifier) @field.type (#eq? @field.type "string")
                          )
                        )
                      )
                    )
                    body: (literal_value
                      (literal_element
                        (literal_value
                          (literal_element
                            (interpreted_string_literal) @test.subcase.definition.name
                          )
                          (literal_element)
                        ) @test.subcase.definition
                      )
                    )
                  )
                )
                body: (block
                  (expression_statement
                    (call_expression
                      function: (selector_expression
                        operand: (identifier) @_test.var
                        field: (field_identifier) @_test.method (#eq? @_test.method "Run")
                      )
                      arguments: (argument_list
                        (selector_expression
                          operand: (identifier) @_test.runner.test_name (#eq? @test.subcase.variable @_test.runner.test_name)
                          field: (field_identifier) @_test.runner.test_name (#eq? @test.subcase.definition.field @_test.runner.test_name)
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
                    let runnable = Runnable {
                        name: function_name.to_string() + "/" + casename,
                        filepath: "".to_string(),
                        meta: crate::core::metadata::RunnableMeta::default_golang(),
                    };
                    runnables.push(runnable);
                }
            }
        }
        Some(runnables)
    }

    fn get_in_loop_unnamed_subcase() -> Option<Vec<Runnable>> {
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
                      name: (type_identifier) @test.case.type.name
                        type: (struct_type 
                          (field_declaration_list
                                      (field_declaration
                                        name: (field_identifier) @test.subcase.definition.field
                                        type: (type_identifier) @field.type (#eq? @field.type "string")
                                      )
                                    )
                    )
                  )
                ) @test.case.declaration
                (for_statement
                            (range_clause
                              left: (expression_list
                                (identifier)
                                (identifier) @test.subcase.variable
                              )
                              right: (composite_literal
                                type: (slice_type
                                element: (type_identifier) @test.case.type
                                )
                                body: (literal_value
                                  (literal_element
                                    (literal_value
                                      (literal_element
                                        (interpreted_string_literal) @test.case.definition.name
                                      )
                                      (literal_element)
                                    ) @test.case.definition
                                  )
                                )
                              )
                            )
                            body: (block
                              (expression_statement
                                (call_expression
                                  function: (selector_expression
                                    operand: (identifier) @_test.var
                                    field: (field_identifier) @_test.method (#eq? @_test.method "Run")
                                  )
                                  arguments: (argument_list
                                    (selector_expression
                                      operand: (identifier) @_test.runner.test_name (#eq? @test.subcase.variable @_test.runner.test_name)
                                      field: (field_identifier) @_test.runner.test_name (#eq? @test.subcase.definition.field @_test.runner.test_name)
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

    fn get_in_lop_named_subcase() -> Option<Vec<Runnable>> {
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
                      name: (type_identifier) @test.case.type.name
                        type: (struct_type 
                          (field_declaration_list
                                      (field_declaration
                                        name: (field_identifier) @test.subcase.definition.field
                                        type: (type_identifier) @field.type (#eq? @field.type "string")
                                      )
                                    )
                    )
                  )
                ) @test.case.declaration
                (for_statement
                            (range_clause
                              left: (expression_list
                                (identifier)
                                (identifier) @test.subcase.variable
                              )
                              right: (composite_literal
                                type: (slice_type
                                element: (type_identifier) @test.case.type
                                )
                                body: (literal_value
                                  (literal_element
                                    (literal_value
                                    (keyed_element
                                          (literal_element
                                              (identifier)
                                          )  @test.subcase.definition.field
                                          (literal_element
                                              (interpreted_string_literal) @test.subcase.definition.name
                                          )
                                      )
                                  ) @test.case.definition
                                  )
                                )
                              )
                            )
                            body: (block
                              (expression_statement
                                (call_expression
                                  function: (selector_expression
                                    operand: (identifier) @_test.var
                                    field: (field_identifier) @_test.method (#eq? @_test.method "Run")
                                  )
                                  arguments: (argument_list
                                    (selector_expression
                                      operand: (identifier) @_test.runner.test_name (#eq? @test.subcase.variable @_test.runner.test_name)
                                      field: (field_identifier) @_test.runner.test_name (#eq? @test.subcase.definition.field @_test.runner.test_name)
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

    fn get_out_loop_named_subtests() -> Option<Vec<Runnable>> {
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
                                field: (field_identifier) @test.loop.test.method
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
                                    field: (field_identifier) @test.loop.test.method
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

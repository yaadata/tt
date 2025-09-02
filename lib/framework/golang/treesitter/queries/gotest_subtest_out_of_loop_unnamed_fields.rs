pub(crate) fn query() -> String {
    let res = r#"
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

    res.to_string()
}

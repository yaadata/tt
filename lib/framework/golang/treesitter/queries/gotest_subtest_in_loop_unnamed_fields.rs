// query
//
// Given subtests defined in the loop of within gotest,
//
// This query will find all the subtests given the struct for each
// test implicity defines struct fields.
//
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
//
// This will find the following subtests:
// - "base case"
// - "case 1"
pub(crate) fn query() -> String {
    let res = r#"
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
        ]]"#;

    res.to_string()
}

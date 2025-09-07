// query
//
// Given subtests defined in the loop of within gotest,
//
// This query will find all the subtests given the struct for each
// test explicitly states each field name.
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
//
// This will find the following subtests:
// - "base case"
// - "case 1"
pub(crate) fn query() -> String {
    let res = r#"
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

    res.to_string()
}

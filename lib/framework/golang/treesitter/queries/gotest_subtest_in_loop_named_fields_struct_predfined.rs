// query
//
// Given subtests defined in the loop of within gotest,
//
// This query will find all the subtests given the struct for each
// test implicity defines struct fields.
//
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
//
// This will find the following subtests:
// - "base case"
// - "case 1"
pub(crate) fn query() -> String {
    let res = r#"
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

    res.to_string()
}

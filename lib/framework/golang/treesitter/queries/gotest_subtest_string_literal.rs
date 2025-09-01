// query
//
// Returns a treesitter query that locates subsets as string literals
//
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
//
// If the position is on TestSample, it will find `case_a` and `cas_b` as subtests
pub(crate) fn query() -> String {
    let res = r#"
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

    res.to_string()
}

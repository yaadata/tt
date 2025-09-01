pub(crate) fn query() -> String {
    let res = r#"
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

    res.to_string()
}

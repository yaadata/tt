pub(crate) fn query() -> String {
    let query_pattern = r#"
            [[((source_file 
              (comment) @build_tags
                (package_clause
                  (package_identifier) @package_name
                ))(#any_contains? @build_tags "//go:build" "//+build"))]]
            "#;
    query_pattern.to_string()
}

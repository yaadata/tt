use crate::framework::golang::treesitter::constants;

// query
//
// Given a single import go file, this query checks if that single import is
// the package be queried.
//
// Example:
// import "testing"
//
// and the query is
// [[
// 	(import_declaration
//       (import_spec
//           path: (interpreted_string_literal) @import.path (#eq? @import.path "\"$PACKAGE\"")
//      )
// )
// ]]
//
// by replacing $PACKAGE with "testing", the query will find a match
pub(crate) fn query() -> String {
    let res = format!(
        r#"
        [[
          (import_declaration
                  (import_spec
                      path: (interpreted_string_literal) @import.path (#eq? @import.path "\"{}\"")
                 )
            )
        ]]
    "#,
        constants::PACKAGE
    );

    res.to_string()
}

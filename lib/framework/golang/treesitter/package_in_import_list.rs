use crate::framework::golang::treesitter::constants;

// query
//
// Give a list of packages in a import list, find a given package by replacing
// $PACKAGE in the query query
//
// Example:
// import (
// 	"testing"
// 	"github.com/stretchr/testify/assert"
// )
//
//
// and the query is
// [[
// 	;; imported packages
// 	(import_declaration
//     	(import_spec_list
//         	(import_spec
//             	path: (interpreted_string_literal) @import.path (#eq? @import.path "\"testing\"")
//          ))
//     )
// ]]
//
// by replacing $PACKAGE with "testing", the query will find a match
pub(crate) fn query() -> String {
    let res = format!(
        r#"
        [[
	          (import_declaration
    	          (import_spec_list
        	          (import_spec
            	          path: (interpreted_string_literal) @import.path (#eq? @import.path "\{}\"")
                ))
        )
        ]]
    "#,
        constants::PACKAGE
    );

    res.to_string()
}

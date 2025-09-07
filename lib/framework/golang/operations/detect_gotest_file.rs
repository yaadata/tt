pub(crate) mod op {
    use tree_sitter::{Language, Node, Query, QueryCursor};

    use crate::framework::golang::treesitter::{
        constants, package_in_import_list, package_in_single_import,
    };

    pub fn execute(root: Node, content: &str) -> bool {
        let single_import_query_pattern =
            package_in_single_import::query().replace(constants::PACKAGE, "testing");
        let list_import_query_pattern =
            package_in_import_list::query().replace(constants::PACKAGE, "testing");

        let query = Query::new(
            &Language::new(tree_sitter_go::LANGUAGE),
            single_import_query_pattern.as_str(),
        );

        if let Result::Ok(q) = query {
            let mut cursor = QueryCursor::new();
            let query_matches = cursor.matches(&q, root, content.as_bytes());
            if query_matches.count().gt(&0) {
                return true;
            }
        }
        let query = Query::new(
            &Language::new(tree_sitter_go::LANGUAGE),
            list_import_query_pattern.as_str(),
        );
        println!("part 1");
        if let Result::Ok(q) = query {
            print!("hello");
            let mut cursor = QueryCursor::new();
            let query_matches = cursor.matches(&q, root, content.as_bytes());
            if query_matches.count().gt(&0) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod test {
    use googletest::{
        assert_that, gtest,
        prelude::{anything, eq, ok},
    };

    use crate::framework::golang::operations::parse_tree;

    use super::op;

    #[gtest]
    fn testing_package_found() {
        let content: &str = r#"
        package golang
        import "testing"

        func sample_add(a, b int) int {
          return a + b
        }
        "#;
        let tree = parse_tree::op::execute(content);
        assert_that!(tree, ok(anything()));
        let tree = tree.unwrap();
        let root = tree.root_node();
        // act
        let res = op::execute(root, content);
        // assert
        assert_that!(res, eq(true))
    }
    #[gtest]
    fn testing_package_not_found() {
        let content: &str = r#"
        package golang
        import "context"

        func sample_add(a, b int) int {
          return a + b
        }
        "#;
        let tree = parse_tree::op::execute(content);
        assert_that!(tree, ok(anything()));
        let tree = tree.unwrap();
        let root = tree.root_node();
        // act
        let res = op::execute(root, content);
        // assert
        assert_that!(res, eq(false))
    }
    #[gtest]
    fn testing_package_found_in_import_list() {
        let content: &str = r#"
        package golang
        import (
          "testing"

          "context"
        )

        func sample_add(a, b int) int {
          return a + b
        }
        "#;
        let tree = parse_tree::op::execute(content);
        assert_that!(tree, ok(anything()));
        let tree = tree.unwrap();
        let root = tree.root_node();
        // act
        let res = op::execute(root, content);
        // assert
        assert_that!(res, eq(true))
    }
    #[gtest]
    fn testing_package_not_found_in_import_list() {
        let content: &str = r#"
        package golang
        import (
          "fmt"

          "context"
        )

        func sample_add(a, b int) int {
          return a + b
        }
        "#;
        let tree = parse_tree::op::execute(content);
        assert_that!(tree, ok(anything()));
        let tree = tree.unwrap();
        let root = tree.root_node();
        // act
        let res = op::execute(root, content);
        // assert
        assert_that!(res, eq(false))
    }
}

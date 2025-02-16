#[cfg(test)]
mod test {
    use std::{fs, panic};

    use crate::{
        core::types::{self},
        framework::golang::{common, gotest},
    };

    #[test]
    fn no_test_function() {
        // arrange
        let content = panic::catch_unwind(|| {
            fs::read_to_string("src/fixtures/golang/sample_test.go").unwrap()
        });
        assert!(content.is_ok());
        let content = content.unwrap();
        let tree = common::utils::parse_tree(content.clone().as_str());
        assert!(tree.is_ok());
        let tree = tree.unwrap();
        let mut walker = tree.walk();

        let cursor = types::cursor_position(10, 3);
        walker.goto_first_child_for_point(cursor.to_point());

        // act
        let res = gotest::detect_test_with_query(Some(walker.node()), content);
        // assert
        assert!(res.is_none());
    }

    #[test]
    fn get_test_function_name() {
        // arrange
        let content = panic::catch_unwind(|| {
            fs::read_to_string("src/fixtures/golang/sample_test.go").unwrap()
        });
        assert!(content.is_ok());
        let content = content.unwrap();
        let tree = common::utils::parse_tree(content.clone().as_str());
        assert!(tree.is_ok());
        let tree = tree.unwrap();
        let mut walker = tree.walk();

        let cursor = types::cursor_position(16, 3);
        walker.goto_first_child_for_point(cursor.to_point());

        // act
        let res = gotest::detect_test_with_query(Some(walker.node()), content);
        // assert
        assert!(res.is_some());
    }
}

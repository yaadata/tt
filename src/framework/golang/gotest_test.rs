#[cfg(test)]
mod test {
    use std::{fs, panic};

    use googletest::assert_that;
    use googletest::prelude::*;

    use crate::{
        core::types::{self, Buffer, Target},
        framework::golang::{common, gotest},
    };

    #[test]
    fn no_test_function() {
        // arrange
        let content =
            panic::catch_unwind(|| fs::read_to_string("src/fixtures/golang/none.go").unwrap());
        assert!(content.is_ok());
        let content = content.unwrap();
        let buffer = Buffer::new(
            content.as_str(),
            "src/fixtures/golang/sample_test.go".to_string(),
            types::cursor_position(10, 3),
        );
        let target = Target::new(crate::core::enums::ToolCategory::TestRunner, buffer);
        let tree = common::utils::parse_tree(content.clone().as_str());
        assert!(tree.is_ok());
        let tree = tree.unwrap();
        let mut walker = tree.walk();

        walker.goto_first_child_for_point(target.buffer.position.to_point());

        // act
        let res = gotest::top_level_test_function(Some(walker.node()), &target);
        // assert
        assert!(res.is_none());
    }

    #[gtest]
    fn get_test_function_name() {
        // arrange
        let content =
            panic::catch_unwind(|| fs::read_to_string("src/fixtures/golang/base_test.go").unwrap());
        assert!(content.is_ok());
        let content = content.unwrap();
        let buffer = Buffer::new(
            content.as_str(),
            "src/fixtures/golang/base_test.go".to_string(),
            types::cursor_position(16, 3),
        );
        let target = Target::new(crate::core::enums::ToolCategory::TestRunner, buffer);
        let tree = common::utils::parse_tree(content.clone().as_str());
        assert_that!(tree, ok(anything()));
        let tree = tree.unwrap();
        let mut walker = tree.walk();

        let cursor = types::cursor_position(16, 3);
        walker.goto_first_child_for_point(cursor.to_point());

        // act
        let res = gotest::top_level_test_function(Some(walker.node()), &target);
        // assert
        assert!(res.is_some());
        assert_eq!(res.unwrap().name, "TestSampleAdd");
    }
}

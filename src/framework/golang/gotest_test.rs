#[cfg(test)]
mod test {

    use crate::core::enums;
    use crate::framework::golang::gotest::get_parent_test;
    use crate::framework::golang::gotest::golang_subtests::get_sub_tests;
    use crate::{
        core::types::{self, Buffer, Target},
        framework::golang::{common, gotest},
    };
    use googletest::assert_that;
    use googletest::prelude::*;
    use rstest::rstest;

    #[test]
    fn no_test_function() {
        // arrange
        let content = r#"
        package golang

        func sample_add(a, b int) int {
          return a + b
        }
        "#;
        let buffer = Buffer::new(
            content,
            "src/fixtures/golang/sample_test.go".to_string(),
            types::CursorPosition::new(10, 3),
        );
        let target = Target::new(crate::core::enums::ToolCategory::TestRunner, buffer);
        let tree = common::utils::parse_tree(content);
        assert!(tree.is_ok());
        let tree = tree.unwrap();
        let mut walker = tree.walk();

        walker.goto_first_child_for_point(target.buffer.position.to_point());

        // act
        let res = gotest::get_parent_test(Some(walker.node()), &target);
        // assert
        assert!(res.is_none());
    }

    #[gtest]
    fn get_test_function_name() {
        // arrange
        let content = r#"
        package golang
        import (
          l "testing"

          "github.com/stretchr/testify/assert"
        )

        func sample_add(a, b int) int {
          return a + b
        }

        func TestSample(t *testing.T) {
            t.Run("case_a", func(t *testing.T){
              assert.Equal(t, 1, sample_add(1, 0))
            })
            t.Run("case_b", func(t *testing.T){
              assert.Equal(t, 2, sample_add(1, 2))
            })
        }
        "#;
        let buffer = Buffer::new(
            content,
            "src/fixtures/golang/base_test.go".to_string(),
            types::CursorPosition::new(16, 3),
        );
        let target = Target::new(enums::ToolCategory::TestRunner, buffer);
        let tree = common::utils::parse_tree(content);
        assert_that!(tree, ok(anything()));
        let tree = tree.unwrap();
        let mut walker = tree.walk();

        let cursor = types::CursorPosition::new(16, 3);
        walker.goto_first_child_for_point(cursor.to_point());

        // act
        let res = gotest::get_parent_test(Some(walker.node()), &target);
        // assert
        assert!(res.is_some());
        assert_eq!(res.unwrap().name, "TestSample");
    }

    #[gtest]
    #[rstest]
    #[case(enums::Search::InFile, types::CursorPosition::new(16, 3), 2, vec!["TestSample/case_a", "TestSample/case_b"])]
    #[case(enums::Search::Nearest, types::CursorPosition::new(16, 3), 1, vec![ "TestSample/case_b"])]
    #[case(enums::Search::Nearest, types::CursorPosition::new(13, 3), 1, vec![ "TestSample/case_a"])]
    fn get_sub_test_string_literal(
        #[case] search: enums::Search,
        #[case] position: types::CursorPosition,
        #[case] expected_num_of_tests: usize,
        #[case] expected_test_names: Vec<&str>,
    ) {
        let content = r#"
        package golang
        import (
          "testing"

          "github.com/stretchr/testify/assert"
        )

        func sample_add(a, b int) int {
          return a + b
        }

        func TestSample(t *testing.T) {
            t.Run("case_a", func(t *testing.T){
              assert.Equal(t, 1, sample_add(1, 0))
            })
            t.Run("case_b", func(t *testing.T){
              assert.Equal(t, 2, sample_add(1, 2))
            })
        }

        func TestSecond(t *testing.T) {
            assert.Equal(t, 2, sample_add(1, 2))
        }
        "#;

        let buffer = Buffer::new(content, "run_test.go".to_string(), position);
        let mut target = Target::new(enums::ToolCategory::TestRunner, buffer);
        target.override_search_strategy(search);

        let tree = common::utils::parse_tree(content);
        assert_that!(tree, ok(anything()));
        let tree = tree.unwrap();
        let mut walker = tree.walk();

        walker.goto_first_child_for_point(position.to_point());

        let node = walker.node();
        let parent_runnable = get_parent_test(Some(node), &target);
        assert_that!(parent_runnable.is_some(), eq(true));
        let parent_runnable = parent_runnable.unwrap();
        assert_eq!(parent_runnable.name, "TestSample");

        walker.reset(node);
        // act
        let res = get_sub_tests(Some(node), Some(parent_runnable), &target);

        // assert
        assert_that!(res.is_some(), eq(true));
        let res = res.unwrap();
        assert_that!(res.len(), eq(expected_num_of_tests));
        for ts in expected_test_names {
            let runnable = res.iter().find(|&x| x.name == ts);
            assert_that!(runnable.is_some(), eq(true));
        }
    }
}

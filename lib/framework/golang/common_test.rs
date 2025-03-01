#[cfg(test)]
mod test {
    use googletest::prelude::*;

    use googletest::{
        assert_that,
        prelude::{anything, eq, some},
    };
    use rstest::rstest;

    use crate::framework::golang::common::utils::{get_build_tags, parse_tree};

    const sample_for_build_tag_tests: &str = r#"
    {replace}
    package golang
    import (
      "testing"

      "github.com/stretchr/testify/assert"
    )

    func sample_add(a, b int) int {
      return a + b
    }
    "#;

    #[gtest]
    #[rstest]
    #[case("//+build unix", 1, "unix")]
    #[case("//+build unix,postgres", 1, "unix postgres")]
    #[case("//+build unix postgres", 2, "unix,postgres")]
    #[case("//+build unix postgres !py03", 2, "unix,postgres")]
    fn legacy_build_tags_or_ampersand(
        #[case] tag: &str,
        #[case] size: usize,
        #[case] expected: &str,
    ) {
        let content = sample_for_build_tag_tests.replace("{replace}", tag);
        let tree = parse_tree(content.clone().as_str());
        assert_that!(tree, ok(anything()));
        let tree = tree.unwrap();
        let root = tree.root_node();
        // act
        let res = get_build_tags(root, content.as_str());
        // assert
        expect_that!(res, some(anything()));
        let res = res.unwrap();
        expect_that!(res.len(), eq(size));
        assert_that!(res.join(","), eq(expected))
    }

    #[test]
    fn legacy_single_build_tag_with_negation() {
        // arrange
        let tag = "//+build !unix";
        let content = sample_for_build_tag_tests.replace("{replace}", tag);
        let tree = parse_tree(content.clone().as_str());
        assert_that!(tree, ok(anything()));
        let tree = tree.unwrap();
        let root = tree.root_node();
        // act
        let res = get_build_tags(root, content.as_str());
        // assert
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(0, res.len());
    }

    #[gtest]
    #[rstest]
    #[case("//go:build unix", 1, "unix")]
    #[case("//go:build unix && postgres", 1, "unix postgres")]
    #[case("//go:build (unix && postgres)", 1, "unix postgres")]
    #[case("//go:build ( unix && postgres )", 1, "unix postgres")]
    #[case("//go:build unix || postgres", 2, "unix,postgres")]
    #[case("//go:build ( unix || postgres )", 2, "unix,postgres")]
    #[case("//go:build unix || !postgres", 1, "unix")]
    #[case("//go:build ( unix || !postgres )", 1, "unix")]
    #[case("//go:build ( unix || !postgres ) && mysql", 1, "unix mysql")]
    #[case("//go:build ( unix || !postgres ) || mysql", 2, "unix,mysql")]
    fn modern_build_tags_or_ampersand(
        #[case] tag: &str,
        #[case] size: usize,
        #[case] expected: &str,
    ) {
        let content = sample_for_build_tag_tests.replace("{replace}", tag);
        let tree = parse_tree(content.clone().as_str());
        assert_that!(tree, ok(anything()));
        let tree = tree.unwrap();
        let root = tree.root_node();
        // act
        let res = get_build_tags(root, content.as_str());
        // assert
        expect_that!(res, some(anything()));
        let res = res.unwrap();
        expect_that!(res.len(), eq(size));
        assert_that!(res.join(","), eq(expected))
    }
}

#[cfg(test)]
mod test {
    use googletest::prelude::*;

    use googletest::{
        assert_that,
        prelude::{anything, eq, some},
    };
    use rstest::rstest;

    use crate::framework::golang::common::utils::{legacy_build_tags, modern_build_tags};

    #[gtest]
    #[rstest]
    #[case("//+build unix", "unix")]
    #[case("//+build unix,postgres", "unix postgres")]
    fn legacy_build_tags_single_result(#[case] tag: &str, #[case] expected: &str) {
        // act
        let res = legacy_build_tags(tag);
        // assert
        expect_that!(res, some(anything()));
        let res = res.unwrap();
        expect_that!(res.len(), eq(1));
        assert_that!(res.first().unwrap(), eq(expected))
    }

    #[test]
    fn legacy_multiple_build_tags() {
        // arrange
        let tags = "//+build unix unit py03";
        // act
        let res = legacy_build_tags(tags);
        // assert
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(3, res.len());
    }

    #[test]
    fn legacy_multiple_build_tags_single_negation() {
        // arrange
        let tags = "//+build unix,unit !py03";
        // act
        let res = legacy_build_tags(tags);
        // assert
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(1, res.len());
    }

    #[test]
    fn legacy_single_build_tag_with_negation() {
        // arrange
        let tags = "//+build !unix";
        // act
        let res = legacy_build_tags(tags);
        // assert
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(0, res.len());
    }

    #[gtest]
    #[rstest]
    #[case("//go:build unix", "unix")]
    #[case("//go:build unix && postgres", "unix postgres")]
    #[case("//go:build (unix && postgres)", "unix postgres")]
    #[case("//go:build ( unix && postgres )", "unix postgres")]
    fn modern_build_tags_ampersand(#[case] tag: &str, #[case] expected: &str) {
        // act
        let res = modern_build_tags(tag);
        // assert
        expect_that!(res, some(anything()));
        let res = res.unwrap();
        expect_that!(res.len(), eq(1));
        assert_that!(res.first().unwrap(), eq(expected))
    }

    #[gtest]
    #[rstest]
    #[case("//go:build unix || postgres", 2, "unix,postgres")]
    #[case("//go:build ( unix || postgres )", 2, "unix,postgres")]
    #[case("//go:build unix || !postgres", 1, "unix")]
    #[case("//go:build ( unix || !postgres )", 1, "unix")]
    #[case("//go:build ( unix || !postgres ) && mysql", 1, "unix mysql")]
    #[case("//go:build ( unix || !postgres ) || mysql", 2, "unix,mysql")]
    fn modern_build_tags_or(#[case] tag: &str, #[case] size: usize, #[case] expected: &str) {
        // act
        let res = modern_build_tags(tag);
        // assert
        expect_that!(res, some(anything()));
        let res = res.unwrap();
        expect_that!(res.len(), eq(size));
        assert_that!(res.join(","), eq(expected))
    }
}

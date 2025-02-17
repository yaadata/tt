#[cfg(test)]
mod test {
    use crate::framework::golang::common::utils::{legacy_build_tags, modern_build_tags};

    #[test]
    fn legacy_single_build_tag() {
        // arrange
        let tags = "//+build unix";
        // act
        let res = legacy_build_tags(tags);
        // assert
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(1, res.len());
        assert!(res.first().unwrap().eq("unix"));
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

    #[test]
    fn modern_single_tag() {
        // arrange
        let tags = "//go:build unix";
        // act
        let res = modern_build_tags(tags);
        // assert
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(1, res.len());
        assert!(res.first().unwrap().eq("unix"));
    }
}

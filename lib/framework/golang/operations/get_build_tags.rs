pub(crate) mod op {
    use crate::{framework::golang::treesitter::build_tags, treesitter::node::node_text};
    use tree_sitter::{Language, Node, Query, QueryCursor};

    pub(crate) fn execute(root: Node, content: &str) -> Option<Vec<String>> {
        let query_pattern = build_tags::query();
        let query = Query::new(
            &Language::new(tree_sitter_go::LANGUAGE),
            query_pattern.as_str(),
        );
        if let Result::Ok(q) = query {
            let capture_index = q
                .capture_index_for_name("build_tags")
                .expect("could not find index position of `build_tags` capture");
            let mut cursor = QueryCursor::new();
            let query_matches = cursor.matches(&q, root, content.as_bytes());
            for node_matched in query_matches {
                for m in node_matched.captures.iter() {
                    if m.index != capture_index {
                        continue;
                    }
                    let build_tags = node_text(m.node, content);
                    if build_tags.starts_with("//+build") {
                        return legacy_build_tags(&build_tags);
                    } else {
                        return modern_build_tags(&build_tags);
                    }
                }
            }
        }
        None
    }

    fn legacy_build_tags(tags: &str) -> Option<Vec<String>> {
        let res = tags
            .split(char::is_whitespace)
            .map(|c| c.to_string())
            .filter(|s| s.ne("//+build"))
            .filter(|s| !s.starts_with('!'))
            .map(|s| s.replace(",", " "))
            .collect();

        Some(res)
    }

    fn modern_build_tags(tags: &str) -> Option<Vec<String>> {
        let expr = tags.split_once("//go:build ");
        let expr = expr?
            .1
            .replace("(", " ( ")
            .replace(")", " ) ")
            .replace("&&", " && ")
            .replace("||", " || ")
            .replace("!", " ! ");
        let mut tags: Vec<String> = vec![];
        let mut curr = String::new();
        let mut parenthesis = vec![];
        let mut negation = false;
        for c in expr.split_whitespace() {
            match c {
                "!" => {
                    negation = true;
                }
                "(" => {
                    parenthesis.push('(');
                }
                ")" => {
                    parenthesis.pop();
                    if parenthesis.is_empty() {
                        if negation {
                            negation = false;
                        } else {
                            tags.extend(
                                curr.split(',')
                                    .filter(|s| !s.is_empty())
                                    .map(|s| s.to_string()),
                            );
                        }
                        curr.clear();
                    }
                }
                "&&" => {
                    if curr.is_empty() {
                        let popped = tags.pop().unwrap();
                        curr.push_str(&popped);
                        curr.push(' ');
                    } else {
                        curr.push(' ');
                    }
                }
                "||" => {
                    if !curr.is_empty() {
                        curr.push(',');
                    }
                }
                _ => {
                    if negation {
                        negation = false
                    } else {
                        curr.push_str(c);
                    }
                }
            }
        }

        if !curr.is_empty() {
            tags.extend(
                curr.split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string()),
            );
        }
        Some(tags)
    }
}

#[cfg(test)]
mod test {
    use super::op;
    use crate::framework::golang::operations::parse_tree;
    use googletest::prelude::*;
    use googletest::{
        assert_that,
        prelude::{anything, eq, some},
    };
    use rstest::rstest;

    const SAMPLE_FOR_BUILD_TAG_TESTS: &str = r#"
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
        let content = SAMPLE_FOR_BUILD_TAG_TESTS.replace("{replace}", tag);
        let tree = parse_tree::op::execute(content.clone().as_str());
        assert_that!(tree, ok(anything()));
        let tree = tree.unwrap();
        let root = tree.root_node();
        // act
        let res = op::execute(root, content.as_str());
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
        let content = SAMPLE_FOR_BUILD_TAG_TESTS.replace("{replace}", tag);
        let tree = parse_tree::op::execute(content.clone().as_str());
        assert_that!(tree, ok(anything()));
        let tree = tree.unwrap();
        let root = tree.root_node();
        // act
        let res = op::execute(root, content.as_str());
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
        let content = SAMPLE_FOR_BUILD_TAG_TESTS.replace("{replace}", tag);
        let tree = parse_tree::op::execute(content.clone().as_str());
        assert_that!(tree, ok(anything()));
        let tree = tree.unwrap();
        let root = tree.root_node();
        // act
        let res = op::execute(root, content.as_str());
        // assert
        expect_that!(res, some(anything()));
        let res = res.unwrap();
        expect_that!(res.len(), eq(size));
        assert_that!(res.join(","), eq(expected))
    }
}

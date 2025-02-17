pub(crate) mod utils {
    use tree_sitter::{Language, Node, Parser, Query, QueryCursor, Tree};

    use crate::{core::errors::FrameworkError, treesitter::node::node_text};

    pub(crate) fn parse_tree(content: &str) -> Result<Tree, FrameworkError> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("Error loading Go parser");

        let tree = parser.parse(content, None);
        if tree.is_none() {
            return Err(FrameworkError::ParsingError(
                "failed to parse content to tree".to_string(),
            ));
        }

        Ok(tree.unwrap())
    }

    pub(crate) fn get_build_tags(root: Node, content: &str) -> Option<Vec<String>> {
        let query_pattern = r#"
            [[((source_file 
              (comment) @build_tags
                (package_clause
                  (package_identifier) @package_name
                ))(#any_contains? @build_tags "//go:build" "//+build"))]]
            "#;
        let query = Query::new(&Language::new(tree_sitter_go::LANGUAGE), query_pattern);
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

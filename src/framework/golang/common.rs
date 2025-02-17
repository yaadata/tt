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

    pub(crate) fn build_tags(root: Option<Node>, content: &str) -> Option<Vec<String>> {
        if root.is_none() {
            return None;
        }
        let node = root.unwrap();
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
            let query_matches = cursor.matches(&q, node, content.as_bytes());
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

    pub(crate) fn legacy_build_tags(tags: &str) -> Option<Vec<String>> {
        let res = tags
            .split(char::is_whitespace)
            .map(|c| c.to_string())
            .filter(|s| s.ne("//+build"))
            .filter(|s| !s.starts_with('!'))
            .collect();

        Some(res)
    }

    pub(crate) fn modern_build_tags(tags: &str) -> Option<Vec<String>> {
        let expr = tags.split_once("//go:build ");
        let expr = expr?;
        let mut tags: Vec<String> = vec![];
        let mut curr = String::new();
        let mut parenthesis = vec![];
        for c in expr.1.split(char::is_whitespace) {
            match c.starts_with('(') {
                true => {
                    parenthesis.push('(');
                    let c = c.trim_start_matches('(');
                    if !c.is_empty() {
                        curr.push_str(c);
                    }
                }
                _ => match c.ends_with(')') {
                    true => {
                        parenthesis.pop();
                        let c = c.trim_end_matches(')');
                        if !c.is_empty() {
                            curr.push_str(c);
                        }
                        if parenthesis.is_empty() {
                            tags.push(curr.clone());
                            curr.clear();
                        }
                    }
                    _ => match c {
                        "&&" => {
                            curr.push(',');
                            curr.push_str(c);
                        }
                        "||" => {
                            tags.push(curr.clone());
                            curr.clear();
                        }
                        _ => {
                            curr.push_str(c);
                        }
                    },
                },
            }
        }

        if !curr.is_empty() {
            tags.push(curr);
        }
        Some(tags)
    }
}

pub(crate) mod op {
    use std::ops::Range;

    use tree_sitter::{Language, Node, Query, QueryCursor};

    use crate::{
        core::{
            metadata::RunnableMeta,
            types::{CursorPosition, Runnable, Target},
        },
        framework::golang::treesitter::gotest_test_function,
        treesitter::node,
    };

    pub fn execute(node: Node, target: &Target) -> Option<Runnable> {
        let current_node_position = node.start_position();
        let query_pattern = gotest_test_function::query();
        let query = Query::new(&Language::new(tree_sitter_go::LANGUAGE), &query_pattern).ok()?;
        let content = target.buffer.content;
        let test_name_index = query.capture_index_for_name("test_name")?;
        let test_function_index = query.capture_index_for_name("testfunc")?;
        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(&query, node, content.as_bytes());
        for node_matched in query_matches.into_iter() {
            let function_node = node_matched
                .captures
                .iter()
                .filter(|c| c.index == test_function_index)
                .map(|c| c.node)
                .next();

            if function_node.is_none() {
                continue;
            }

            let function_node = function_node.unwrap();

            for m in node_matched
                .captures
                .iter()
                .filter(|c| c.index == test_name_index)
            {
                if m.node.start_position().row <= current_node_position.row
                    && m.node.end_position().row >= current_node_position.row
                {
                    return Some(Runnable {
                        name: node::node_text(m.node, content),
                        filepath: target.buffer.filepath.to_string(),
                        range: Range {
                            start: CursorPosition::from_point(function_node.start_position()),
                            end: CursorPosition::from_point(function_node.end_position()),
                        },
                        meta: RunnableMeta::default_golang(),
                    });
                }
            }
        }

        None
    }
}

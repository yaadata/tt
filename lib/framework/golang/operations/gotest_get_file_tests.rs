pub(crate) mod op {
    use std::ops::Range;

    use tree_sitter::{Language, Node, Query, QueryCursor};

    use crate::{
        core::{
            metadata::RunnableMeta,
            types::{CursorPosition, Runnable, Target},
        },
        framework::golang::treesitter::queries::gotest_test_function,
        treesitter::node,
    };

    pub fn execute(node: Node, target: &Target) -> Option<Vec<Runnable>> {
        let content = target.buffer.content;
        let query_pattern = gotest_test_function::query();
        let query = Query::new(&Language::new(tree_sitter_go::LANGUAGE), &query_pattern).ok()?;
        let test_name_index = query.capture_index_for_name("test_name")?;
        let test_function_index = query.capture_index_for_name("testfunc")?;
        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(&query, node, content.as_bytes());

        let mut parent_runnables: Vec<Runnable> = vec![];
        for node_matched in query_matches {
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
                parent_runnables.push(Runnable {
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

        if parent_runnables.is_empty() {
            None
        } else {
            Some(parent_runnables)
        }
    }
}

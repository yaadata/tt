pub(crate) mod op {
    use crate::core::types::{CursorPosition, Runnable, Target};
    use crate::treesitter::node::node_text;
    use tree_sitter::{Language, Node, Query, QueryCursor};

    use std::ops;

    use crate::framework::golang::treesitter::{
        gotest_subtest_in_loop_named_fields, gotest_subtest_in_loop_named_fields_struct_predfined,
        gotest_subtest_in_loop_unnamed_fields,
        gotest_subtest_in_loop_unnamed_fields_struct_predefined,
        gotest_subtest_out_of_loop_named_fields, gotest_subtest_out_of_loop_unnamed_fields,
        gotest_subtest_string_literal,
    };

    pub(crate) fn execute(node: Node, parent: Runnable, target: &Target) -> Option<Vec<Runnable>> {
        let mut cursor_position = None;
        if target
            .search_strategy
            .eq(&crate::core::enums::Search::Nearest)
        {
            cursor_position = Some(target.buffer.position);
        }

        let mut res = vec![];
        let finders = [
            get_string_literal_subtests,
            get_in_loop_with_named_subtests,
            get_in_loop_with_unnamed_subtests,
            get_in_loop_typed_subcase_with_unnamed_case_fields,
            get_in_loop_typed_subcase_with_named_case_fields,
            get_out_of_loop_named_subtests,
            get_out_of_loop_unnamed_subtests,
        ];
        for func in finders {
            if let Some(t) = func(
                node,
                target.buffer.content,
                parent.to_owned(),
                cursor_position,
            ) {
                res.extend(t);
            }
        }

        if res.is_empty() {
            None
        } else {
            Some(res)
        }
    }

    fn extract_for_loop_subtests(
        node: Node,
        content: &str,
        query: &str,
        parent: Runnable,
        current_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        let query = Query::new(&Language::new(tree_sitter_go::LANGUAGE), query).ok()?;
        let subcase_name_index = query.capture_index_for_name("test.case.field.value")?;
        let subcase_index = query.capture_index_for_name("test.case")?;
        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(&query, node, content.as_bytes());
        let mut runnables = vec![];

        for node_matched in query_matches {
            let subtest_capture = node_matched
                .captures
                .iter()
                .find(|capture| capture.index == subcase_index)
                .map(|capture| capture.node);

            if subtest_capture.is_none() {
                continue;
            }

            let subtest_node = subtest_capture.unwrap();
            if let Some(position) = current_position {
                let r = subtest_node.range();
                if !position.in_range(std::ops::Range {
                    start: r.start_point,
                    end: r.end_point,
                }) {
                    continue;
                }
            }
            let subtest = node_matched
                .captures
                .iter()
                .find(|capture| capture.index == subcase_name_index)
                .map(|capture| node_text(capture.node, content));
            if subtest.is_none() {
                continue;
            }
            let subtest = subtest.unwrap();
            let runnable = Runnable {
                name: parent.name.to_owned() + "/" + subtest.replace("\"", "").as_str(),
                filepath: parent.filepath.clone(),
                range: std::ops::Range {
                    start: CursorPosition::from_point(subtest_node.start_position()),
                    end: CursorPosition::from_point(subtest_node.end_position()),
                },
                meta: crate::core::metadata::RunnableMeta::default_golang(),
            };
            runnables.push(runnable);
        }

        if runnables.is_empty() {
            None
        } else {
            Some(runnables)
        }
    }

    // get_string_literal_subtests
    // Example:
    // package golang
    // import (
    //   "testing"
    //
    //   "github.com/stretchr/testify/assert"
    // )
    //
    // func sample_add(a, b int) int {
    //   return a + b
    // }
    //
    // func TestSample(t *testing.T) {
    //     t.Run("case_a", func(t *testing.T){
    //       assert.Equal(t, 1, sample_add(1, 0))
    //     })
    //     t.Run("case_b", func(t *testing.T){
    //       assert.Equal(t, 2, sample_add(1, 2))
    //     })
    // }
    fn get_string_literal_subtests(
        node: Node,
        content: &str,
        parent: Runnable,
        cursor_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        let query_pattern = &gotest_subtest_string_literal::query();
        let query = Query::new(&Language::new(tree_sitter_go::LANGUAGE), query_pattern).ok()?;
        let subcase_name_index = query.capture_index_for_name("test.case.name.value")?;
        let subcase_index = query.capture_index_for_name("test.case")?;
        let mut cursor = QueryCursor::new();
        cursor.set_point_range(ops::Range {
            start: parent.range.start.to_point(),
            end: parent.range.end.to_point(),
        });
        let query_matches = cursor.matches(&query, node, content.as_bytes());
        let mut runnables = vec![];

        for node_matched in query_matches {
            let subtest_capture = node_matched
                .captures
                .iter()
                .find(|capture| capture.index == subcase_index)
                .map(|capture| capture.node);

            if subtest_capture.is_none() {
                continue;
            }

            let subtest_node = subtest_capture.unwrap();
            if let Some(position) = cursor_position {
                let r = subtest_node.range();
                if !position.in_range(std::ops::Range {
                    start: r.start_point,
                    end: r.end_point,
                }) {
                    continue;
                }
            }
            let subtest = node_matched
                .captures
                .iter()
                .find(|capture| capture.index == subcase_name_index)
                .map(|capture| node_text(capture.node, content));
            if subtest.is_none() {
                continue;
            }
            let subtest = subtest.unwrap();
            let runnable = Runnable {
                name: parent.name.to_owned() + "/" + subtest.replace("\"", "").as_str(),
                filepath: parent.filepath.clone(),
                range: std::ops::Range {
                    start: CursorPosition::from_point(subtest_node.start_position()),
                    end: CursorPosition::from_point(subtest_node.end_position()),
                },
                meta: crate::core::metadata::RunnableMeta::default_golang(),
            };
            runnables.push(runnable);
        }

        if runnables.is_empty() {
            None
        } else {
            Some(runnables)
        }
    }

    // get_in_loop_with_named_subtest
    // Example:
    // func TestTableTest(t *testing.T) {
    // 	for _, tt := range []struct {
    // 		description string
    // 		a           int
    // 		b           int
    // 		expected    int
    // 	}{
    // 		{
    // 			description: "base case",
    // 			a:           0,
    // 			b:           3,
    // 			expected:    3,
    // 		},
    // 		{
    // 			description: "case 1",
    // 			a:           1,
    // 			b:           3,
    // 			expected:    4,
    // 		},
    // 	} {
    // 		t.Run(tt.description, func(t *testing.T) {
    // 			actual := sample_add(tt.a, tt.b)
    // 			assert.Equal(t, tt.expected, actual)
    // 		})
    // 	}
    // }
    fn get_in_loop_with_named_subtests(
        node: Node,
        content: &str,
        parent: Runnable,
        current_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        let query_pattern = &gotest_subtest_in_loop_named_fields::query();
        extract_for_loop_subtests(node, content, query_pattern, parent, current_position)
    }

    // get_in_loop_with_unnamed_subtests
    // Example:
    // func TestTableTest(t *testing.T) {
    // 	for _, tt := range []struct {
    // 		description string
    // 		a           int
    // 		b           int
    // 		expected    int
    // 	}{
    // 		{
    // 			"base case",
    // 			0,
    // 			3,
    // 			3,
    // 		},
    // 		{
    // 			"case 1",
    // 			1,
    // 			3,
    // 			4,
    // 		},
    // 	} {
    // 		t.Run(tt.description, func(t *testing.T) {
    // 			actual := sample_add(tt.a, tt.b)
    // 			assert.Equal(t, tt.expected, actual)
    // 		})
    // 	}
    // }
    fn get_in_loop_with_unnamed_subtests(
        node: Node,
        content: &str,
        parent: Runnable,
        current_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        let query_pattern = gotest_subtest_in_loop_unnamed_fields::query();
        extract_for_loop_subtests(node, content, &query_pattern, parent, current_position)
    }

    // get_in_loop_typed_testcase_with_unnamed_case_fields
    // Example:
    // func TestTableTest(t *testing.T) {
    // 	type Scenario struct {
    // 		description string
    // 		a           int
    // 		b           int
    // 		expected    int
    // 	}
    // 	for _, tt := range []Scenario{
    // 		{
    // 			"base case",
    // 			0,
    // 			3,
    // 			3,
    // 		},
    // 		{
    // 			"case 1",
    // 			1,
    // 			3,
    // 			4,
    // 		},
    // 	} {
    // 		t.Run(tt.description, func(t *testing.T) {
    // 			actual := sample_add(tt.a, tt.b)
    // 			assert.Equal(t, tt.expected, actual)
    // 		})
    // 	}
    // }
    fn get_in_loop_typed_subcase_with_unnamed_case_fields(
        node: Node,
        content: &str,
        parent: Runnable,
        current_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        let query_pattern = gotest_subtest_in_loop_unnamed_fields_struct_predefined::query();
        extract_for_loop_subtests(node, content, &query_pattern, parent, current_position)
    }

    // get_in_loop_typed_testcase_with_named_case_fields
    // Defines a loop with a slice created with an outer variable.
    //
    // Example:
    // func TestSample(t *testing.T) {
    // 	type Scenario struct {
    //     	description string
    //         a			int
    //         b			int
    //         expected	int
    //     }
    //
    // 	for _, tt := range []Scenario{
    // 		{
    // 			description: "base case",
    // 			a:          0,
    // 			b:          3,
    // 			c:          3,
    // 		},
    // 		{
    // 			description: "case 1",
    // 			a:           1,
    // 			b:           3,
    // 			expected:    4,
    // 		},
    // 	}  {
    // 		t.Run(tt.description, func(t *testing.T) {
    // 			actual := sample_add(tt.a, tt.b)
    // 			assert.Equal(t, tt.expected, actual)
    // 		})
    // 	}
    // }
    fn get_in_loop_typed_subcase_with_named_case_fields(
        node: Node,
        content: &str,
        parent: Runnable,
        current_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        let query_pattern = gotest_subtest_in_loop_named_fields_struct_predfined::query();
        extract_for_loop_subtests(node, content, &query_pattern, parent, current_position)
    }

    // get_out_of_loop_named_subtests
    // Example:
    // func TestTableTest(t *testing.T) {
    // 	scenarios := []struct {
    // 		description string
    // 		a           int
    // 		b           int
    // 		expected    int
    // 	}{
    // 		{
    // 			description: 	"base case",
    // 			a: 				0,
    // 			b: 				3,
    // 			expected:		3,
    // 		},
    // 		{
    // 			description: "case 1",
    // 			a:           1,
    // 			b:           3,
    // 			expected:    4,
    // 		},
    // 	}
    //
    // 	for _, tt := range scenarios {
    // 		t.Run(tt.description, func(t *testing.T) {
    // 			actual := sample_add(tt.a, tt.b)
    // 			assert.Equal(t, tt.expected, actual)
    // 		})
    // 	}
    // }
    fn get_out_of_loop_named_subtests(
        node: Node,
        content: &str,
        parent: Runnable,
        current_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        let query_pattern = gotest_subtest_out_of_loop_named_fields::query();
        extract_for_loop_subtests(node, content, &query_pattern, parent, current_position)
    }

    // get_out_of_loop_unnamed_subtests
    // example:
    // func TestTableTest(t *testing.T) {
    // 	scenarios := []struct {
    // 		description string
    // 		a           string
    // 		b           int
    // 		expected    int
    // 	}{
    // 		{
    // 			"base case",
    // 			0,
    // 			3,
    // 			3,
    // 		},
    // 		{
    // 			"case 1",
    // 			1,
    // 			3,
    // 			4,
    // 		},
    // 	}
    //
    // 	for _, tt := range scenarios {
    // 		t.Run(tt.description, func(t *testing.T) {
    // 			actual := sample_add(tt.a, tt.b)
    // 			assert.Equal(t, tt.expected, actual)
    // 		})
    // 	}
    // }
    fn get_out_of_loop_unnamed_subtests(
        node: Node,
        content: &str,
        parent: Runnable,
        current_position: Option<CursorPosition>,
    ) -> Option<Vec<Runnable>> {
        let query_pattern = gotest_subtest_out_of_loop_unnamed_fields::query();
        extract_for_loop_subtests(node, content, &query_pattern, parent, current_position)
    }
}

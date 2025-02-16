pub(crate) mod utils {
    use tree_sitter::{Parser, Tree};

    use crate::core::errors::FrameworkError;

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
}

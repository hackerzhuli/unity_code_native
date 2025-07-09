//! USS Hover Provider
//!
//! Provides hover information for USS properties, including:
//! - Property descriptions
//! - Documentation URLs
//! - Unity version compatibility

use crate::unity_project_manager::UnityProjectManager;
use crate::uss::definitions::UssDefinitions;
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};
use tree_sitter::{Node, Tree};

/// Provides hover information for USS properties
pub struct UssHoverProvider {
    definitions: UssDefinitions,
}

impl UssHoverProvider {
    /// Creates a new USS hover provider
    pub fn new() -> Self {
        Self {
            definitions: UssDefinitions::new(),
        }
    }

    /// Provides hover information for a position in USS code
    pub fn hover(
        &self,
        tree: &Tree,
        source: &str,
        position: Position,
        unity_manager: &UnityProjectManager,
    ) -> Option<Hover> {
        let node = self.find_property_at_position(tree, source, position)?;
        let property_name = self.extract_property_name(&node, source)?;
        
        if self.definitions.is_predefined_property(&property_name) {
            Some(self.create_hover_content(&property_name, unity_manager))
        } else {
            None
        }
    }

    /// Finds the property node at the given position
    fn find_property_at_position<'a>(
        &self,
        tree: &'a Tree,
        source: &str,
        position: Position,
    ) -> Option<Node<'a>> {
        let root = tree.root_node();
        let byte_offset = self.position_to_byte_offset(source, position)?;
        
        // Find the deepest node at this position
        let mut current = root.descendant_for_byte_range(byte_offset, byte_offset)?;
        
        // Walk up the tree to find a property declaration
        while current.kind() != "property_name" && current.kind() != "declaration" {
            if let Some(parent) = current.parent() {
                current = parent;
            } else {
                return None;
            }
        }
        
        // If we found a declaration, look for the property_name child
        if current.kind() == "declaration" {
            for child in current.children(&mut current.walk()) {
                if child.kind() == "property_name" {
                    return Some(child);
                }
            }
        }
        
        if current.kind() == "property_name" {
            Some(current)
        } else {
            None
        }
    }

    /// Extracts the property name from a node
    fn extract_property_name(&self, node: &Node, source: &str) -> Option<String> {
        let start = node.start_byte();
        let end = node.end_byte();
        
        if start < source.len() && end <= source.len() {
            Some(source[start..end].to_string())
        } else {
            None
        }
    }

    /// Converts LSP position to byte offset
    fn position_to_byte_offset(&self, source: &str, position: Position) -> Option<usize> {
        let mut line = 0;
        let mut col = 0;
        
        for (i, ch) in source.char_indices() {
            if line == position.line as usize && col == position.character as usize {
                return Some(i);
            }
            
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        
        None
    }

    /// Creates hover content for a property
    fn create_hover_content(
        &self,
        property_name: &str,
        unity_manager: &UnityProjectManager,
    ) -> Hover {
        let unity_version_for_docs = unity_manager.get_unity_version_for_docs();
        let description = self.definitions.get_property_description(property_name)
            .unwrap_or("USS property");
        let doc_url = self.definitions.get_property_documentation_url(property_name, &unity_version_for_docs)
            .unwrap_or("https://docs.unity3d.com/Manual/UIE-USS.html".to_string());
        
        let is_inherited = self.definitions.is_property_inherited(property_name);
        let is_animatable = self.definitions.is_property_animatable(property_name);
        
        let mut content = format!("**{}**\n\n", property_name);
        content.push_str(&format!("{}", description));
        
        // Add property characteristics
        let mut characteristics = Vec::new();
        if is_inherited {
            characteristics.push("Inherited");
        }
        if is_animatable {
            characteristics.push("Animatable");
        }
        
        if !characteristics.is_empty() {
            content.push_str(&format!("\n\n*{}*", characteristics.join(", ")));
        }
        
        // Add documentation link
        content.push_str(&format!("\n\n[📖 Documentation]({})", doc_url));
        
        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        }
    }
}

impl Default for UssHoverProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unity_project_manager::UnityProjectManager;
    use std::path::PathBuf;
    use tower_lsp::lsp_types::Position;

    #[test]
    fn test_hover_provider_creation() {
        let provider = UssHoverProvider::new();
        assert!(provider.definitions.is_valid_property("color"));
    }

    #[test]
    fn test_position_to_byte_offset() {
        let provider = UssHoverProvider::new();
        let source = "color: red;\nfont-size: 12px;";
        
        // Test position at start of "color"
        let offset = provider.position_to_byte_offset(source, Position::new(0, 0));
        assert_eq!(offset, Some(0));
        
        // Test position at start of "font-size"
        let offset = provider.position_to_byte_offset(source, Position::new(1, 0));
        assert_eq!(offset, Some(12));
    }

    #[test]
    fn test_extract_property_name() {
        let provider = UssHoverProvider::new();
        let source = "color: red;";
        
        // Test the extract_property_name method with a mock scenario
        // In practice, this would be called with actual tree-sitter nodes
        // For now, we just test that the provider can be created
        assert!(provider.definitions.is_valid_property("color"));
    }

    #[test]
    fn test_create_hover_content() {
        let provider = UssHoverProvider::new();
        let unity_manager = UnityProjectManager::new(PathBuf::from("/test/project"));
        
        let hover = provider.create_hover_content("color", &unity_manager);
        
        if let HoverContents::Markup(content) = hover.contents {
            assert!(content.value.contains("**color**"));
            assert!(content.value.contains("Documentation"));
            assert_eq!(content.kind, MarkupKind::Markdown);
        } else {
            panic!("Expected markup content");
        }
    }

    #[test]
    fn test_hover_for_invalid_property() {
        let provider = UssHoverProvider::new();
        let _unity_manager = UnityProjectManager::new(PathBuf::from("/test/project"));
        
        // Test that invalid properties are not recognized
        assert!(!provider.definitions.is_valid_property("invalid-property"));
        assert!(!provider.definitions.is_predefined_property("invalid-property"));
        
        // Test that custom CSS properties are valid for diagnostics but not for hover
        assert!(provider.definitions.is_valid_property("--button-border-color-what"));
        assert!(!provider.definitions.is_predefined_property("--button-border-color-what"));
        assert!(provider.definitions.is_valid_property("--custom-property"));
        assert!(!provider.definitions.is_predefined_property("--custom-property"));
        
        // Test that predefined properties are recognized by both methods
        assert!(provider.definitions.is_valid_property("color"));
        assert!(provider.definitions.is_predefined_property("color"));
    }
}
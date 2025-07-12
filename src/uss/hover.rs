//! USS Hover Provider
//!
//! Provides hover information for USS properties, including:
//! - Property descriptions
//! - Documentation URLs
//! - Unity version compatibility

use std::path::PathBuf;

use crate::unity_project_manager::UnityProjectManager;
use crate::uss::definitions::UssDefinitions;
use crate::language::tree_utils::{find_node_of_type_at_position};
use crate::uss::import_node::ImportNode;
use crate::uss::url_function_node::UrlFunctionNode;
use crate::uss::uss_utils::convert_uss_string;
use crate::language::asset_url::{project_url_to_path, project_url_to_relative_path, validate_url};
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Url};
use tree_sitter::{Node, Tree};
use crate::uss::constants::*;
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
        source_url: Option<&Url>,
    ) -> Option<Hover> {
        // Priority 1: Check if we are in an import statement
        if let Some(import_node) = find_node_of_type_at_position(tree.root_node(), source, position, NODE_IMPORT_STATEMENT) {
            return self.hover_for_import_statement(import_node, source, unity_manager, source_url);
        }

        // Priority 2: Check if we are in a URL function
        if let Some(call_node) = find_node_of_type_at_position(tree.root_node(), source, position, NODE_CALL_EXPRESSION) {
            if let Some(hover) = self.hover_for_url_function(call_node, source, unity_manager, source_url) {
                return Some(hover);
            }
        }

        // Priority 3: Check if we are in a declaration node (property hover)
        if let Some(declaration_node) = find_node_of_type_at_position(tree.root_node(), source, position, NODE_DECLARATION){
            if let Some(property_name_node) = declaration_node.child(0){
                if property_name_node.kind() == NODE_PROPERTY_NAME{
                    if let Ok(property_name) = property_name_node.utf8_text(source.as_bytes()){
                        if self.definitions.is_predefined_property(&property_name) 
                        {
                            return Some(self.create_hover_content_for_property(&property_name, unity_manager));
                        }
                    }
                }
            }
        }

        return None;
    }

    /// Provides hover information for import statements
    fn hover_for_import_statement(
        &self,
        import_node: Node,
        source: &str,
        unity_manager: &UnityProjectManager,
        source_url: Option<&Url>,
    ) -> Option<Hover> {
        let mut diagnostics = Vec::new();
        let import = ImportNode::from_node(import_node, source, &mut diagnostics)?;
        
        let mut content = String::from("**@import Statement**\n\n");
        
        // Check if the current import resolves to a file and add a link if it exists
        if let Some(import_path) = self.extract_path_from_node(import.argument_node, source) {
            if let Some((file_path, url)) = self.resolve_import_file_path(&import_path, unity_manager, source_url) {
                if let Some(relative_path) = project_url_to_relative_path(&url){
                    content.push_str(&format!("Asset path: `{}`\n\n", relative_path));
                }
        
                if file_path.exists() {
                    if let Ok(file_url) = Url::from_file_path(&file_path) {
                        content.push_str(&format!("📁 [Open file]({})\n\n", file_url));
                    }
                } else {
                    content.push_str("❌ File not found\n\n");
                }
            }
        }

        // Add documentation about what @import does
        content.push_str("Imports styles from another USS file into the current stylesheet.\n\n");
        
        // Add usage examples
        content.push_str("**Examples:**\n");
        content.push_str("```css\n");
        content.push_str("@import url(\"project:///Assets/UI/styles/common.uss\");\n");
        content.push_str("@import \"./common.uss\";\n");
        content.push_str("```\n\n");
        
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        })
    }

    /// Provides hover information for URL functions
    fn hover_for_url_function(
        &self,
        call_node: Node,
        source: &str,
        unity_manager: &UnityProjectManager,
        source_url: Option<&Url>,
    ) -> Option<Hover> {
        let url_function = UrlFunctionNode::from_node(call_node, source, None, source_url, None)?;
        
        let mut content = format!("**url()**\n\n");

        // Try to resolve the file path and check if it exists
        if let Some((file_path, url)) = self.resolve_import_file_path(url_function.url(), unity_manager, source_url) {
            if let Some(relative_path) = project_url_to_relative_path(&url){
                content.push_str(&format!("Asset path: `{}`\n\n", relative_path));
            }
        
            if file_path.exists() {
                // Create a file:// URL for the resolved path
                if let Ok(file_url) = Url::from_file_path(&file_path) {
                    content.push_str(&format!("[📂 Open File]({})", file_url));
                }
            } else {
                content.push_str(&format!("❌ File not found"));
            }
        } else {
            content.push_str(&format!("⚠️ Could not resolve file path"));
        }
        
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        })
    }

    /// Extracts a path string from a tree-sitter node (string or url function)
    fn extract_path_from_node(&self, node: Node, source: &str) -> Option<String> {
        match node.kind() {
            NODE_STRING_VALUE => {
                let raw_string = node.utf8_text(source.as_bytes()).ok()?;
                convert_uss_string(raw_string).ok()
            }
            NODE_CALL_EXPRESSION => {
                // This should be a url() function
                let url_function = UrlFunctionNode::from_node(node, source, None, None, None)?;
                Some(url_function.url().to_string())
            }
            _ => None,
        }
    }

    /// Resolves an import file path to an absolute file system path
    fn resolve_import_file_path(
        &self,
        import_path: &str,
        unity_manager: &UnityProjectManager,
        source_url: Option<&Url>,
    ) -> Option<(PathBuf, Url)> {
        // Try to validate the URL first to get a proper project URL
        if let Ok(validation_result) = validate_url(import_path, source_url) {
            // Convert project URL to file system path
            let project_root = unity_manager.project_path();
            if let Some(path) = project_url_to_path(project_root, &validation_result.url){
                return Some((path, validation_result.url));
            }
        }
        None
    }

    /// Creates hover content for a property
    fn create_hover_content_for_property(
        &self,
        property_name: &str,
        unity_manager: &UnityProjectManager,
    ) -> Hover {
        let unity_version_for_docs = unity_manager.get_unity_version_for_docs()
            .unwrap_or_else(|| "6000.0".to_string());
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
        use crate::language::tree_utils::position_to_byte_offset;
        let source = "color: red;\nfont-size: 12px;";
        
        // Test position at start of "color"
        let offset = position_to_byte_offset(source, Position::new(0, 0));
        assert_eq!(offset, Some(0));
        
        // Test position at start of "font-size"
        let offset = position_to_byte_offset(source, Position::new(1, 0));
        assert_eq!(offset, Some(12));
    }

    #[test]
    fn test_create_hover_content() {
        let provider = UssHoverProvider::new();
        let unity_manager = UnityProjectManager::new(PathBuf::from("/test/project"));
        
        let hover = provider.create_hover_content_for_property("color", &unity_manager);
        
        if let HoverContents::Markup(content) = hover.contents {
            assert!(content.value.contains("**color**"));
            assert!(content.value.contains("Documentation"));
            assert_eq!(content.kind, MarkupKind::Markdown);
        } else {
            panic!("Expected markup content");
        }
    }

    #[test]
    fn test_extract_path_from_string_node() {
        let provider = UssHoverProvider::new();
        // This test would need a proper tree-sitter node, which is complex to create in unit tests
        // For now, we'll just test that the provider can be created
        assert!(provider.definitions.is_valid_property("color"));
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
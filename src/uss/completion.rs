//! USS Completion Provider
//!
//! Provides auto-completion for USS properties and values.
//! Supports completion for property values after ':' with automatic semicolon insertion.

use std::collections::HashSet;
use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Tree};
use url::Url;

use crate::language::tree_utils::{position_to_byte_offset, find_node_of_type_at_position};
use crate::unity_project_manager::UnityProjectManager;
use crate::uss::constants::*;
use crate::uss::definitions::UssDefinitions;
use crate::uss::value_spec::{ValueType, ValueFormat};
use crate::uss::parser::UssParser;

/// USS completion provider
pub struct UssCompletionProvider {
    pub(crate) definitions: UssDefinitions,
}

/// Context for completion
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionContext {
    /// Completing property values after ':'
    PropertyValue {
        property_name: String,
        /// Text already typed after the colon
        partial_value: String,
    },
    /// Completing selectors
    Selector,
    /// Completing property names
    Property,
    /// Completing pseudo-classes after ':'
    PseudoClass,
    /// Completing inside function arguments
    FunctionArgument,
    /// Unknown context
    Unknown,
}

impl UssCompletionProvider {
    /// Create a new USS completion provider
    pub fn new() -> Self {
        Self {
            definitions: UssDefinitions::new(),
        }
    }

    /// Provide completion items for the given position
    pub fn complete(
        &self,
        tree: &Tree,
        content: &str,
        position: Position,
        _unity_manager: &UnityProjectManager,
        _source_url: Option<&Url>,
    ) -> Vec<CompletionItem> {
        let context = self.get_completion_context(tree, content, position);
        
        // Debug logging
        log::info!("Completion context: {:?}", context);

        match context {
            CompletionContext::PropertyValue { property_name, partial_value } => {
                log::info!("Property value completion for '{}' with partial '{}'", property_name, partial_value);
                self.complete_property_value(&property_name, &partial_value)
            }
            CompletionContext::PseudoClass => {
                log::info!("Pseudo classes completion");
                self.complete_pseudo_classes()
            }
            _ => {
                log::info!("No completion context matched");
                Vec::new()
            }
        }
    }

    /// Determine the completion context at the given position
    pub(super) fn get_completion_context(&self, tree: &Tree, content: &str, position: Position) -> CompletionContext {

        // Check if we're in a declaration context (after ':')
        if position.character > 0 {
            // The tree have trouble finding the right node if we're looking at the cursor
            // We need to go back one character to be safe, the one that the user just typed
            let last_pos = Position::new(position.line, position.character - 1);
            if let Some(declaration_node) = find_node_of_type_at_position(tree.root_node(), content, last_pos, NODE_DECLARATION) {
                log::info!("Found declaration node, analyzing declaration context");
                return self.analyze_declaration_context(declaration_node, content, position);
            }
        }

        CompletionContext::Unknown
    }

    /// Analyze completion context within a declaration
    fn analyze_declaration_context(&self, declaration_node: Node, content: &str, position: Position) -> CompletionContext {
        // Extract property name from the declaration
        let property_name = if let Some(prop_node) = declaration_node.child_by_field_name("property_name") {
            prop_node.utf8_text(content.as_bytes()).unwrap_or("").to_string()
        } else {
            // Fallback: look for first child that looks like a property name
            for i in 0..declaration_node.child_count() {
                if let Some(child) = declaration_node.child(i) {
                    if child.kind() == "property_name" {
                        if let Ok(text) = child.utf8_text(content.as_bytes()) {
                            return CompletionContext::PropertyValue {
                                property_name: text.to_string(),
                                partial_value: self.extract_partial_value(&declaration_node, content, position),
                            };
                        }
                    }
                }
            }
            return CompletionContext::Unknown;
        };

        CompletionContext::PropertyValue {
            property_name,
            partial_value: self.extract_partial_value(&declaration_node, content, position),
        }
    }

    /// Analyze completion context within ERROR nodes (incomplete declarations)
    fn analyze_error_context(&self, error_node: Node, content: &str, position: Position) -> CompletionContext {
        // For incomplete declarations like "color: ", the parser creates an ERROR node
        // We need to extract the property name from the content manually
        
        let error_text = error_node.utf8_text(content.as_bytes()).unwrap_or("");
        
        // Look for pattern: "property_name: [partial_value]"
        if let Some(colon_pos) = error_text.find(':') {
            let property_name = error_text[..colon_pos].trim();
            let after_colon = error_text[colon_pos + 1..].trim();
            
            // Validate that this looks like a CSS property
            if !property_name.is_empty() && property_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                return CompletionContext::PropertyValue {
                    property_name: property_name.to_string(),
                    partial_value: after_colon.to_string(),
                };
            }
        }
        
        CompletionContext::Unknown
    }

    /// Analyze completion context within selectors
    fn analyze_selector_context(&self, _selectors_node: Node, content: &str, position: Position) -> CompletionContext {
        let byte_offset = position_to_byte_offset(content, position).unwrap_or(0);
        
        // Check if the character before cursor is ':'
        if byte_offset > 0 {
            if let Some(char_before) = content.chars().nth(byte_offset - 1) {
                if char_before == ':' {
                    return CompletionContext::PseudoClass;
                }
            }
        }

        CompletionContext::Selector
    }

    /// Extract partial value after colon in declaration
    fn extract_partial_value(&self, declaration_node: &Node, content: &str, position: Position) -> String {
        let byte_offset = position_to_byte_offset(content, position).unwrap_or(0);
        
        // Look for value nodes (plain_value, etc.)
        for i in 0..declaration_node.child_count() {
            if let Some(child) = declaration_node.child(i) {
                if child.kind() == "plain_value" || child.kind().contains("value") {
                    // Found a value node, extract its text
                    if let Ok(value_text) = child.utf8_text(content.as_bytes()) {
                        // If cursor is within or after this value node, return the text
                        if byte_offset >= child.start_byte() {
                            return value_text.trim().to_string();
                        }
                    }
                }
            }
        }
        
        // Fallback: try to find colon and extract text after it
        for i in 0..declaration_node.child_count() {
            if let Some(child) = declaration_node.child(i) {
                if child.kind() == ":" {
                    // Found the colon, extract text from after colon to cursor position
                    let colon_end = child.end_byte();
                    if byte_offset > colon_end {
                        let partial_bytes = &content.as_bytes()[colon_end..byte_offset];
                        if let Ok(partial_str) = std::str::from_utf8(partial_bytes) {
                            return partial_str.trim().to_string();
                        }
                    }
                    break;
                }
            }
        }
        
        String::new()
    }

    /// Complete property values for a given property
    pub(super) fn complete_property_value(&self, property_name: &str, partial_value: &str) -> Vec<CompletionItem> {
        // Implement simplified auto-completion logic:
        // 1. If partial_value is empty (right after ':'), show all common values
        // 2. If partial_value is not empty, only show completions for keyword-only properties
        
        if partial_value.is_empty() {
            // Right after colon - show all common values
            return self.get_all_common_values_for_property(property_name);
        }
        
        // Check if this is a keyword-only property
        if let Some(valid_keywords) = self.definitions.get_valid_keyword_values_for_property(property_name) {
            // This is a keyword-only property, filter keywords that match partial input
            let mut items = Vec::new();
            let partial_lower = partial_value.to_lowercase();
            
            for keyword in valid_keywords {
                if keyword.to_lowercase().starts_with(&partial_lower) {
                    let item = CompletionItem {
                        label: keyword.to_string(),
                        kind: Some(CompletionItemKind::VALUE),
                        detail: Some(format!("Keyword value for {}", property_name)),
                        insert_text: Some(format!("{} ", keyword)), // Add space after value
                        insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                        additional_text_edits: Some(vec![
                            self.create_semicolon_edit(partial_value)
                        ].into_iter().filter_map(|x| x).collect()),
                        ..Default::default()
                    };
                    items.push(item);
                }
            }
            
            return items;
        }
        
        // For non-keyword-only properties, don't provide completions after typing
        Vec::new()
    }
    
    /// Get all common values for a property (used when partial_value is empty)
    fn get_all_common_values_for_property(&self, property_name: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Get property info and suggestions
        if let Some(property_info) = self.definitions.get_property_info(property_name) {
            let suggestions = self.get_value_suggestions(&property_info.value_spec, "");
            
            for suggestion in suggestions {
                let item = CompletionItem {
                    label: suggestion.clone(),
                    kind: Some(CompletionItemKind::VALUE),
                    detail: Some(format!("Value for {}", property_name)),
                    insert_text: Some(format!("{} ", suggestion)), // Add space after value
                    insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                    additional_text_edits: Some(vec![
                        self.create_semicolon_edit("")
                    ].into_iter().filter_map(|x| x).collect()),
                    documentation: if !property_info.description.is_empty() {
                        Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!("**{}**\n\n{}", property_name, property_info.description),
                        }))
                    } else {
                        None
                    },
                    ..Default::default()
                };
                items.push(item);
            }
        }

        // Always add common CSS values that work with most properties
        self.add_common_values(&mut items, "");

        items
    }

    /// Get value suggestions from a ValueSpec
    fn get_value_suggestions(&self, value_spec: &crate::uss::value_spec::ValueSpec, partial_value: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        let partial_lower = partial_value.to_lowercase();

        // Get suggestions from all possible formats
        for format in &value_spec.formats {
            suggestions.extend(self.get_format_suggestions(format, &partial_lower));
        }

        // Remove duplicates and sort
        let mut unique_suggestions: Vec<String> = suggestions.into_iter().collect::<HashSet<_>>().into_iter().collect();
        unique_suggestions.sort();
        
        // Filter by partial match
        if !partial_value.is_empty() {
            unique_suggestions.retain(|s| s.to_lowercase().starts_with(&partial_lower));
        }

        unique_suggestions
    }

    /// Get suggestions from a specific ValueFormat
    fn get_format_suggestions(&self, format: &ValueFormat, partial_lower: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        for entry in &format.entries {
            for value_type in &entry.types {
                suggestions.extend(self.get_type_suggestions(*value_type, partial_lower));
            }
        }

        suggestions
    }

    /// Get suggestions for a specific ValueType
    fn get_type_suggestions(&self, value_type: ValueType, partial_lower: &str) -> Vec<String> {
        match value_type {
            ValueType::Keyword(keyword) => {
                if keyword.to_lowercase().starts_with(partial_lower) {
                    vec![keyword.to_string()]
                } else {
                    vec![]
                }
            }
            ValueType::Color => {
                let mut colors = Vec::new();
                // Add color keywords
                for (color_name, _hex) in &self.definitions.valid_color_keywords {
                    if color_name.to_lowercase().starts_with(partial_lower) {
                        colors.push(color_name.to_string());
                    }
                }
                // Add common color functions
                if "rgb".starts_with(partial_lower) {
                    colors.push("rgb(255, 255, 255)".to_string());
                }
                if "rgba".starts_with(partial_lower) {
                    colors.push("rgba(255, 255, 255, 1.0)".to_string());
                }
                if "hsl".starts_with(partial_lower) {
                    colors.push("hsl(0, 100%, 50%)".to_string());
                }
                if "hsla".starts_with(partial_lower) {
                    colors.push("hsla(0, 100%, 50%, 1.0)".to_string());
                }
                colors
            }
            ValueType::Length => {
                let mut lengths = Vec::new();
                if partial_lower.is_empty() || "0".starts_with(partial_lower) {
                    lengths.push("0".to_string());
                }
                // Add common length values
                for value in ["10px", "20px", "50px", "100px", "100%", "50%", "auto"] {
                    if value.to_lowercase().starts_with(partial_lower) {
                        lengths.push(value.to_string());
                    }
                }
                lengths
            }
            ValueType::Number => {
                let mut numbers = Vec::new();
                if partial_lower.is_empty() || "0".starts_with(partial_lower) {
                    numbers.push("0".to_string());
                }
                for value in ["1", "2", "0.5", "1.5"] {
                    if value.starts_with(partial_lower) {
                        numbers.push(value.to_string());
                    }
                }
                numbers
            }
            ValueType::Asset => {
                let mut assets = Vec::new();
                if "url".starts_with(partial_lower) {
                    assets.push("url(\"path/to/asset\")".to_string());
                }
                if "resource".starts_with(partial_lower) {
                    assets.push("resource(\"AssetName\")".to_string());
                }
                assets
            }
            ValueType::String => {
                if partial_lower.is_empty() {
                    vec!["\"text\"".to_string()]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }

    /// Complete pseudo-classes
    pub(super) fn complete_pseudo_classes(&self) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        for pseudo_class in &self.definitions.valid_pseudo_classes {
            // Remove the leading ':' since it's already typed
            let label = pseudo_class.strip_prefix(':').unwrap_or(pseudo_class);
            
            items.push(CompletionItem {
                label: label.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Pseudo-class".to_string()),
                insert_text: Some(label.to_string()),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..Default::default()
            });
        }

        items
    }

    /// Add common CSS values that work with most properties
    fn add_common_values(&self, items: &mut Vec<CompletionItem>, partial_value: &str) {
        let common_values = ["inherit", "initial", "unset", "auto", "none"];
        let partial_lower = partial_value.to_lowercase();

        for value in common_values {
            if partial_value.is_empty() || value.to_lowercase().starts_with(&partial_lower) {
                items.push(CompletionItem {
                    label: value.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some("Common CSS value".to_string()),
                    insert_text: Some(format!("{} ", value)), // Add space after value
                    insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                    additional_text_edits: Some(vec![
                        self.create_semicolon_edit(partial_value)
                    ].into_iter().filter_map(|x| x).collect()),
                    ..Default::default()
                });
            }
        }
    }

    /// Create a text edit to add a semicolon at the end of the declaration if needed
    fn create_semicolon_edit(&self, _partial_value: &str) -> Option<TextEdit> {
        // For now, we'll implement this as a placeholder
        // In a full implementation, we'd need to check if a semicolon already exists
        // and determine the correct position to insert it
        None
    }
}

impl Default for UssCompletionProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uss::parser::UssParser;

    #[test]
    fn test_completion_context_detection_right_after_colon() {
        let provider = UssCompletionProvider::new();
        let mut parser = UssParser::new().unwrap();
        
        // Test content: "color: " where cursor is right after colon and space
        let content = "color: ";
        let tree = parser.parse(content, None).unwrap();
        let position = Position { line: 0, character: 6 }; // Right after space (0-indexed, so 6 is the 7th character)
        
        let context = provider.get_completion_context(&tree, content, position);
        
        match context {
            CompletionContext::PropertyValue { property_name, partial_value } => {
                assert_eq!(property_name, "color");
                assert_eq!(partial_value, ""); // Should be empty right after colon
            }
            _ => panic!("Expected PropertyValue context, got: {:?}", context),
        }
    }
    
    #[test]
    fn test_completion_context_detection_with_partial_value() {
        let provider = UssCompletionProvider::new();
        let mut parser = UssParser::new().unwrap();
        
        // Test content: "display: f" where cursor is after 'f'
        let content = "display: f";
        let tree = parser.parse(content, None).unwrap();
        let position = Position { line: 0, character: 9 }; // After 'f' (0-indexed, content is 10 chars long 0-9)
        
        let context = provider.get_completion_context(&tree, content, position);
        
        match context {
            CompletionContext::PropertyValue { property_name, partial_value } => {
                assert_eq!(property_name, "display");
                assert_eq!(partial_value, "f"); // Should extract 'f' as partial value
            }
            _ => panic!("Expected PropertyValue context, got: {:?}", context),
        }
    }
    
    #[test]
    fn test_simplified_completion_behavior() {
        let provider = UssCompletionProvider::new();
        
        // Test 1: Right after colon should provide all values
        let completions_empty = provider.complete_property_value("color", "");
        assert!(!completions_empty.is_empty(), "Should provide completions right after colon");
        
        // Test 2: For keyword-only property, should provide filtered keywords
        let completions_display = provider.complete_property_value("display", "f");
        assert!(!completions_display.is_empty(), "Should provide filtered keywords for display property");
        let labels: Vec<String> = completions_display.iter().map(|c| c.label.clone()).collect();
        assert!(labels.contains(&"flex".to_string()), "Should include 'flex' for display:f");
        
        // Test 3: For non-keyword-only property, should not provide completions after typing
        let completions_color = provider.complete_property_value("color", "r");
        assert!(completions_color.is_empty(), "Should not provide completions for color after typing");
    }
}

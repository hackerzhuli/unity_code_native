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
        
        match context {
            CompletionContext::PropertyValue { property_name, partial_value } => {
                self.complete_property_value(&property_name, &partial_value)
            }
            CompletionContext::PseudoClass => {
                self.complete_pseudo_classes()
            }
            _ => Vec::new(),
        }
    }

    /// Determine the completion context at the given position
    pub(super) fn get_completion_context(&self, tree: &Tree, content: &str, position: Position) -> CompletionContext {
        let byte_offset = match position_to_byte_offset(content, position) {
            Some(offset) => offset,
            None => return CompletionContext::Unknown,
        };

        let root = tree.root_node();
        
        // Find the deepest node at this position
        let node_at_position = match root.descendant_for_byte_range(byte_offset, byte_offset) {
            Some(node) => node,
            None => return CompletionContext::Unknown,
        };

        // Check if we're in a declaration context (after ':')
        if let Some(declaration_node) = find_node_of_type_at_position(node_at_position, content, position, NODE_DECLARATION) {
            return self.analyze_declaration_context(declaration_node, content, position);
        }

        // Check if we're typing a pseudo-class (after ':')
        if let Some(selectors_node) = find_node_of_type_at_position(node_at_position, content, position, NODE_SELECTORS) {
            return self.analyze_selector_context(selectors_node, content, position);
        }

        CompletionContext::Unknown
    }

    /// Analyze completion context within a declaration
    fn analyze_declaration_context(&self, declaration_node: Node, content: &str, position: Position) -> CompletionContext {
        // Use the same pattern as hover.rs: check first child for property name
        if let Some(property_name_node) = declaration_node.child(0) {
            if property_name_node.kind() == NODE_PROPERTY_NAME {
                if let Ok(property_name) = property_name_node.utf8_text(content.as_bytes()) {
                    // Now we need to determine what part of the value we're completing
                    // For now, return PropertyValue context with empty partial_value
                    return CompletionContext::PropertyValue {
                        property_name: property_name.to_string(),
                        partial_value: String::new(),
                    };
                }
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

    /// Complete property values for a given property
    pub(super) fn complete_property_value(&self, property_name: &str, partial_value: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Get property info
        if let Some(property_info) = self.definitions.get_property_info(property_name) {
            // Get value suggestions from the property's value spec
            let suggestions = self.get_value_suggestions(&property_info.value_spec, partial_value);
            
            for suggestion in suggestions {
                let mut item = CompletionItem {
                    label: suggestion.clone(),
                    kind: Some(CompletionItemKind::VALUE),
                    detail: Some(format!("Value for {}", property_name)),
                    insert_text: Some(format!("{} ", suggestion)), // Add space after value
                    insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                    additional_text_edits: Some(vec![
                        // Add semicolon at the end if not present
                        self.create_semicolon_edit(partial_value)
                    ].into_iter().filter_map(|x| x).collect()),
                    ..Default::default()
                };

                // Add documentation if available
                if !property_info.description.is_empty() {
                    item.documentation = Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("**{}**\n\n{}", property_name, property_info.description),
                    }));
                }

                items.push(item);
            }
        }

        // Always add common CSS values that work with most properties
        self.add_common_values(&mut items, partial_value);

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

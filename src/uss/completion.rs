//! USS Completion Provider
//!
//! Provides auto-completion for USS properties and values.
//! Supports completion for property values after ':' with automatic semicolon insertion.

use std::collections::HashSet;
use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Tree};
use url::Url;

use crate::language::tree_utils::{find_node_at_position, find_node_of_type_at_position};
use crate::unity_project_manager::UnityProjectManager;
use crate::uss::constants::*;
use crate::uss::definitions::UssDefinitions;
use crate::uss::value_spec::{ValueFormat, ValueType};

/// USS completion provider
pub struct UssCompletionProvider {
    pub(crate) definitions: UssDefinitions,
}

#[derive(Debug, Clone)]
pub(super) struct CompletionContext<'a> {
    pub t: CompletionType,
    pub current_node: Option<Node<'a>>,
    pub position: Position,
}

/// Context for completion
#[derive(Debug, Clone, PartialEq)]
pub(super) enum CompletionType {
    /// Completing property values after ':'
    PropertyValue { property_name: String },
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

        if let Some(current_node) = context.current_node {
            match context.t {
                CompletionType::PropertyValue { property_name } => {
                    self.complete_property_value(&property_name, current_node, content)
                }
                CompletionType::PseudoClass => {
                    log::info!("Pseudo classes completion");
                    self.complete_pseudo_classes()
                }
                _ => {
                    log::info!("No completion context matched");
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        }
    }

    /// Determine the completion context at the given position
    pub(super) fn get_completion_context<'a>(
        &self,
        tree: &'a Tree,
        content: &str,
        position: Position,
    ) -> CompletionContext<'a> {
        // Check if we're in a declaration context (after ':')
        if position.character > 0 {
            // The tree have trouble finding the right node if we're looking at the cursor
            // We need to go back one character to be safe, the one that the user just typed
            let last_pos = Position::new(position.line, position.character - 1);

            if let Some(current_node) = find_node_at_position(tree.root_node(), last_pos) {
                if let Some(declaration_node) = find_node_of_type_at_position(
                    tree.root_node(),
                    content,
                    last_pos,
                    NODE_DECLARATION,
                ) {
                    //log::info!("Found declaration node, analyzing declaration context");
                    return self.analyze_declaration_context(
                        declaration_node,
                        content,
                        current_node,
                        position,
                    );
                }
            }
        }

        return CompletionContext {
            t: CompletionType::Unknown,
            current_node: None,
            position,
        };
    }

    /// Analyze completion context within a declaration
    fn analyze_declaration_context<'a>(
        &self,
        declaration_node: Node<'a>,
        content: &str,
        current_node: Node<'a>,
        position: Position,
    ) -> CompletionContext<'a> {
        if let Some(property_name_node) = declaration_node.child(0) {
            if property_name_node.kind() == NODE_PROPERTY_NAME {
                let property_name = property_name_node
                    .utf8_text(content.as_bytes())
                    .unwrap_or("")
                    .to_string();

                return CompletionContext {
                    t: CompletionType::PropertyValue { property_name },
                    current_node: Some(current_node),
                    position,
                };
            }
        }

        return CompletionContext {
            t: CompletionType::Unknown,
            current_node: Some(current_node),
            position,
        };
    }

    /// Complete property values for a given property
    ///
    /// ### Parameters
    /// `current_node`: The current node right before the cursor position
    pub(super) fn complete_property_value(
        &self,
        property_name: &str,
        current_node: Node,
        content: &str,
    ) -> Vec<CompletionItem> {
        // Implement simple auto-completion logic:
        // 1. If current node is colon, show all common values
        // 2. If current node is the first value node after property and the property is keyword-only or is color, we try to show a list of keywords that still matches what is already in the node

        log::info!(
            "Current node is of type {} with content {} and parent is of type {}",
            current_node.kind(),
            current_node.utf8_text(content.as_bytes()).unwrap_or(""),
            current_node.parent().unwrap().kind()
        );

        if current_node.kind() == NODE_COLON {
            // Right after colon - show all common values
            return self.get_all_common_values_for_property(property_name);
        }

        // check if we are the first value node(ie. the previous node is colon and the node before that is property name)
        let mut is_first_value_node = false;
        if let Some(prev_sibling) = current_node.prev_sibling() {
            if prev_sibling.kind() == NODE_COLON {
                if let Some(prev_prev_sibling) = prev_sibling.prev_sibling() {
                    if prev_prev_sibling.kind() == NODE_PROPERTY_NAME {
                        is_first_value_node = true;
                    }
                }
            }
        }

        log::info!(
            "Are we at first node after colon for property {} ? {}",
            property_name,
            is_first_value_node
        );

        if !is_first_value_node {
            return Vec::new();
        }

        // keywords that we want to filter against
        let valid_keywords = self.get_keywors_for_keyword_based_property(property_name);

        if valid_keywords.is_empty() {
            return Vec::new();
        }

        // try to find what still matches and return that
        let mut items = Vec::new();
        let partial_value = current_node.utf8_text(content.as_bytes()).unwrap_or("");
        if partial_value.is_empty() {
            return Vec::new();
        }

        let partial_lower = partial_value.to_lowercase();

        for keyword in valid_keywords {
            if keyword.starts_with(&partial_lower) {
                let item = CompletionItem {
                    label: keyword.to_string(),
                    kind: Some(CompletionItemKind::VALUE),
                    detail: Some(format!("Keyword value for {}", property_name)),
                    insert_text: Some(format!("{} ", keyword)), // Add space after value
                    insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                    additional_text_edits: Some(
                        vec![self.create_semicolon_edit(partial_value)]
                            .into_iter()
                            .filter_map(|x| x)
                            .collect(),
                    ),
                    ..Default::default()
                };
                items.push(item);
            }
        }

        // For non-keyword-only properties, don't provide completions after typing
        Vec::new()
    }

    /// if a property is a single value and keyword only or is a color then get the valid keywords for them
    fn get_keywors_for_keyword_based_property(&self, property_name: &str) -> Vec<&str> {
        let mut valid_keywords: Vec<&str> = Vec::new();

        // Check if this is a keyword-only property
        if let Some(property_info) = self.definitions.get_property_info(property_name) {
            if property_info.value_spec.formats.len() > 0
                && property_info.value_spec.formats[0].entries.len() > 0
            {
                if property_info.value_spec.is_keyword_only() {
                    for t in property_info.value_spec.formats[0].entries[0].types.iter() {
                        if let ValueType::Keyword(keyword) = t {
                            valid_keywords.push(*keyword);
                        }
                    }
                }

                // check if it is a color property
                if property_info.value_spec.is_color_only() {
                    self.definitions
                        .valid_color_keywords
                        .iter()
                        .for_each(|(keyword, _)| {
                            valid_keywords.push(*keyword);
                        });
                }
            }
        }
        valid_keywords
    }

    /// Get all common values for a property (used when partial_value is empty)
    fn get_all_common_values_for_property(&self, property_name: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        let mut suggestions: Vec<&str> = Vec::new();

        if let Some(property_info) = self.definitions.get_property_info(property_name) {
            suggestions.extend_from_slice(
                self.get_keywors_for_keyword_based_property(property_name)
                    .as_slice(),
            );

            for suggestion in suggestions {
                let item = CompletionItem {
                    label: suggestion.to_string(),
                    kind: Some(CompletionItemKind::VALUE),
                    detail: Some(format!("Value for {}", property_name)),
                    insert_text: Some(format!("{} ", suggestion)), // Add space after value
                    insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                    additional_text_edits: Some(
                        vec![self.create_semicolon_edit("")]
                            .into_iter()
                            .filter_map(|x| x)
                            .collect(),
                    ),
                    documentation: if !property_info.description.is_empty() {
                        Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!(
                                "**{}**\n\n{}",
                                property_name, property_info.description
                            ),
                        }))
                    } else {
                        None
                    },
                    ..Default::default()
                };
                items.push(item);
            }
        }

        items
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
                    additional_text_edits: Some(
                        vec![self.create_semicolon_edit(partial_value)]
                            .into_iter()
                            .filter_map(|x| x)
                            .collect(),
                    ),
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

//! USS Completion Provider
//!
//! Provides auto-completion for USS properties and values.
//! Supports completion for property values after ':' with automatic semicolon insertion.

use std::collections::HashSet;
use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Tree};
use url::Url;

use crate::language::tree_utils::{
    find_node_at_position, find_node_of_type_at_position, get_node_depth,
};
use crate::language::url_completion::UrlCompletionProvider;
use crate::unity_project_manager::UnityProjectManager;
use crate::uss::constants::*;
use crate::uss::definitions::UssDefinitions;
use crate::uss::value_spec::ValueType;
use crate::uxml_schema_manager::UxmlSchemaManager;

// Import additional constants for selector completion
use crate::uss::constants::*;

/// USS completion provider
pub struct UssCompletionProvider {
    pub(crate) definitions: UssDefinitions,
    url_completion_provider: Option<UrlCompletionProvider>,
    uxml_schema_manager: Option<UxmlSchemaManager>,
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
    /// Completing property names
    Property,
    /// Completing pseudo-classes after ':'
    PseudoClass,
    /// Completing class selectors after '.'
    ClassSelector,
    /// Completing ID selectors after '#'
    IdSelector,
    /// Completing tag selectors
    TagSelector,
    /// Completing URL inside url() function or an import statement without url function
    UrlString {
        /// The partial URL string being typed
        url_string: String,
        /// The cursor position within the URL string
        cursor_position: usize,
    },
    /// Completing import statement structure after @import
    ImportStatement,
    /// Unknown context
    Unknown,
}

impl UssCompletionProvider {
    /// Create a new USS completion provider
    pub fn new() -> Self {
        Self {
            definitions: UssDefinitions::new(),
            url_completion_provider: None,
            uxml_schema_manager: None,
        }
    }

    /// Create a new USS completion provider with URL completion support
    pub fn new_with_project_root(project_root: &std::path::Path) -> Self {
        Self {
            definitions: UssDefinitions::new(),
            url_completion_provider: Some(UrlCompletionProvider::new(project_root)),
            uxml_schema_manager: Some(UxmlSchemaManager::new(
                project_root.join("UIElementsSchema"),
            )),
        }
    }

    /// Provide completion items for the given position
    pub fn complete(
        &self,
        tree: &Tree,
        content: &str,
        position: Position,
        unity_manager: &UnityProjectManager,
        source_url: Option<&Url>,
        uxml_element_names: Option<&std::collections::HashSet<String>>,
    ) -> Vec<CompletionItem> {
        let context = self.get_completion_context(tree, content, position);

        if let Some(current_node) = context.current_node {
            match context.t {
                CompletionType::PropertyValue { property_name } => {
                    self.complete_property_value(&property_name, current_node, content)
                }
                CompletionType::Property => {
                    self.complete_property_names(context.current_node, content)
                }
                CompletionType::PseudoClass => {
                    self.complete_pseudo_classes_with_filter(current_node, content)
                }
                CompletionType::ClassSelector => {
                    self.complete_class_selectors(tree, content, current_node)
                }
                CompletionType::IdSelector => {
                    self.complete_id_selectors(tree, content, current_node)
                }
                CompletionType::TagSelector => {
                    self.complete_tag_selectors(current_node, content, uxml_element_names)
                }
                CompletionType::UrlString {
                    url_string,
                    cursor_position,
                } => self.complete_url_function(&url_string, cursor_position, source_url),
                CompletionType::ImportStatement => self.complete_import_statement(),
                _ => Vec::new(),
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
            // Note that we have a limitation here, we can get the `:` in the tree right after the user typed it
            // If user type spaces after that, then we can't locate the node the cursor is at because there is no node for just spaces after colon
            // That means we don't have a good way to get auto completion for that case now
            // So in that case we don't provide auto completion at all
            let last_pos = Position::new(position.line, position.character - 1);

            if let Some(current_node) = find_node_at_position(tree.root_node(), last_pos) {
                // Check for incomplete @import statements first
                if let Some(import_context) =
                    self.analyze_incomplete_import_context(current_node, content, position)
                {
                    return import_context;
                }

                // Check for selector completion context first
                if let Some(selector_context) =
                    self.analyze_selector_context(tree, content, current_node, position)
                {
                    return selector_context;
                }

                // Check if we're inside an import statement
                if let Some(import_context) =
                    self.analyze_import_context(tree, content, current_node, position)
                {
                    return import_context;
                }

                if let Some(declaration_node) = find_node_of_type_at_position(
                    tree.root_node(),
                    content,
                    last_pos,
                    NODE_DECLARATION,
                ) {
                    return self.analyze_declaration_context(
                        declaration_node,
                        content,
                        current_node,
                        position,
                    );
                }

                // Check if we're typing a property name within a block
                if let Some(_block_node) =
                    find_node_of_type_at_position(tree.root_node(), content, last_pos, NODE_BLOCK)
                {
                    // Check if current node is a property name being typed
                    // Note: incomplete property names are parsed as "attribute_name" in ERROR nodes
                    if current_node.kind() == NODE_ATTRIBUTE_NAME
                        && current_node.parent().map(|p| p.kind()) == Some(NODE_ERROR)
                    {
                        return CompletionContext {
                            t: CompletionType::Property,
                            current_node: Some(current_node),
                            position,
                        };
                    }
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
                // Check if we're still typing the property name
                // Note: incomplete property names might be parsed as "attribute_name" in ERROR nodes
                if current_node.kind() == NODE_PROPERTY_NAME
                    || (current_node.kind() == "attribute_name"
                        && current_node.parent().map(|p| p.kind()) == Some(NODE_ERROR))
                {
                    return CompletionContext {
                        t: CompletionType::Property,
                        current_node: Some(current_node),
                        position,
                    };
                }

                // Check if we're inside a URL function
                if let Some(url_context) =
                    self.analyze_url_function_context(current_node, content, position)
                {
                    return url_context;
                }

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

        if !is_first_value_node {
            return Vec::new();
        }

        // keywords that we want to filter against
        let valid_keywords = self.get_keywords_for_keyword_based_property(property_name);

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
                    ..Default::default()
                };
                items.push(item);
            }
        }

        items
    }

    /// if a property is a single value and keyword only or is a color then get the valid keywords for them
    fn get_keywords_for_keyword_based_property(&self, property_name: &str) -> Vec<&str> {
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

        suggestions.extend_from_slice(
            self.get_keywords_for_keyword_based_property(property_name)
                .as_slice(),
        );

        for suggestion in suggestions {
            // now all suggestions are keywords so we can get docs from keyword itself
            let mut doc = None;
            if let Some(k) = self.definitions.get_keyword_info(suggestion) {
                doc = Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: k.doc.to_string(),
                }));
            }
            let item = CompletionItem {
                label: suggestion.to_string(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some(format!("Value for {}", property_name)),
                insert_text: Some(format!(" {};", suggestion)), // add space before and semicolon after to complete it, we only offer complete suggetions after property
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                documentation: doc, // for now we don't have docs for keywords
                ..Default::default()
            };
            items.push(item);
        }

        items
    }

    /// Complete property names
    fn complete_property_names(
        &self,
        current_node: Option<tree_sitter::Node>,
        content: &str,
    ) -> Vec<CompletionItem> {
        let partial_text = if let Some(node) = current_node {
            node.utf8_text(content.as_bytes())
                .unwrap_or("")
                .to_lowercase()
        } else {
            String::new()
        };

        // Only provide completions if user has typed at least one character
        if partial_text.is_empty() {
            return Vec::new();
        }

        self.definitions
            .get_all_property_names()
            .iter()
            .filter(|name| name.starts_with(&partial_text))
            .map(|name| {
                let property_info = self.definitions.get_property_info(name);
                let description = property_info
                    .as_ref()
                    .map(|info| info.description.to_string())
                    .unwrap_or_else(|| format!("USS property: {}", name));

                let documentation_url = property_info
                    .as_ref()
                    .map(|info| info.documentation_url.clone());

                CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    detail: Some(description),
                    documentation: documentation_url.map(|url| {
                        Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!("[Documentation]({})", url),
                        })
                    }),
                    insert_text: Some(format!("{}: ", name)),
                    ..Default::default()
                }
            })
            .collect()
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

    /// Complete pseudo-classes with partial matching
    fn complete_pseudo_classes_with_filter(
        &self,
        current_node: Node,
        content: &str,
    ) -> Vec<CompletionItem> {
        let node_text = current_node.utf8_text(content.as_bytes()).unwrap_or("");

        // If current node is a colon, we want to show all pseudo-classes
        // If current node is a class_name (partial pseudo-class), filter by that text
        let partial_text = if current_node.kind() == NODE_COLON {
            String::new() // Show all pseudo-classes when cursor is right after colon
        } else if current_node.kind() == NODE_ATTRIBUTE_NAME {
            // If we're at an attribute_name that's actually a partial pseudo-class, use its text
            node_text.to_lowercase()
        } else {
            node_text.to_lowercase()
        };

        let mut items = Vec::new();

        for pseudo_class in &self.definitions.valid_pseudo_classes {
            let label = pseudo_class;
            let label_lower = label.to_lowercase();

            // Filter based on partial text
            if partial_text.is_empty() {
                // Show all pseudo-classes when just typed ':'
                items.push(CompletionItem {
                    label: label.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some("Pseudo-class".to_string()),
                    insert_text: Some(label.to_string()),
                    insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                    ..Default::default()
                });
            } else if label_lower.starts_with(&partial_text) && label_lower != partial_text {
                // Only show pseudo-classes that start with partial_text but are not exact matches
                items.push(CompletionItem {
                    label: label.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some("Pseudo-class".to_string()),
                    insert_text: Some(label.to_string()),
                    insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                    ..Default::default()
                });
            }
        }

        items
    }

    /// Analyze if we're in a selector completion context
    fn analyze_selector_context<'a>(
        &self,
        tree: &'a Tree,
        content: &str,
        current_node: Node<'a>,
        position: Position,
    ) -> Option<CompletionContext<'a>> {
        // Check if we're in a selector context (not inside a block)
        if find_node_of_type_at_position(
            tree.root_node(),
            content,
            Position::new(position.line, position.character.saturating_sub(1)),
            NODE_BLOCK,
        )
        .is_some()
        {
            return None; // We're inside a declaration block, not in selector context
        }

        // Check if current node is a class selector being typed
        if current_node.kind() == NODE_CLASS_SELECTOR
            || (current_node.kind() == NODE_CLASS_NAME
                && current_node.parent().map(|p| p.kind()) == Some(NODE_CLASS_SELECTOR))
        {
            return Some(CompletionContext {
                t: CompletionType::ClassSelector,
                current_node: Some(current_node),
                position,
            });
        }

        // partial pseudo class being typed after a selector
        if current_node.kind() == NODE_CLASS_NAME {
            if let Some(parent) = current_node.parent() {
                if parent.kind() == NODE_PSEUDO_CLASS_SELECTOR {
                    if let Some(prev) = current_node.prev_sibling() {
                        if prev.kind() == NODE_COLON {
                            return Some(CompletionContext {
                                t: CompletionType::PseudoClass,
                                current_node: Some(current_node),
                                position,
                            });
                        }
                    }
                }
            }
        }

        // Check if current node is an ID selector being typed
        if current_node.kind() == NODE_ID_SELECTOR
            || (current_node.kind() == NODE_ID_NAME
                && current_node.parent().map(|p| p.kind()) == Some(NODE_ID_SELECTOR))
        {
            return Some(CompletionContext {
                t: CompletionType::IdSelector,
                current_node: Some(current_node),
                position,
            });
        }

        // Check if current node is a tag selector being typed
        if current_node.kind() == NODE_TAG_NAME {
            return Some(CompletionContext {
                t: CompletionType::TagSelector,
                current_node: Some(current_node),
                position,
            });
        }

        // Check if current node is attribute_name that might be a partial pseudo-class FIRST
        // This happens when user types something like ".button:h" - the "h" becomes attribute_name
        if current_node.kind() == NODE_ATTRIBUTE_NAME {
            if let Some(parent) = current_node.parent() {
                if parent.kind() == NODE_ERROR {
                    // Check if there's a colon and selector before this attribute_name
                    // For pseudo-class, we expect: selector -> colon -> attribute_name
                    if let Some(prev_sibling) = current_node.prev_sibling() {
                        if prev_sibling.kind() == NODE_COLON {
                            // Found colon, now check if there's a selector before the colon
                            if let Some(prev_prev_sibling) = prev_sibling.prev_sibling() {
                                if prev_prev_sibling.kind() == NODE_CLASS_SELECTOR
                                    || prev_prev_sibling.kind() == NODE_ID_SELECTOR
                                    || prev_prev_sibling.kind() == NODE_TAG_NAME
                                {
                                    return Some(CompletionContext {
                                        t: CompletionType::PseudoClass,
                                        current_node: Some(current_node),
                                        position,
                                    });
                                }
                            }
                        }
                    }

                    // Otherwise, it's a partial tag name
                    return Some(CompletionContext {
                        t: CompletionType::TagSelector,
                        current_node: Some(current_node),
                        position,
                    });
                }
            }
        }

        // Check if we're directly on a '.' or '#' token
        if current_node.kind() == "." {
            return Some(CompletionContext {
                t: CompletionType::ClassSelector,
                current_node: Some(current_node),
                position,
            });
        }

        if current_node.kind() == "#" {
            return Some(CompletionContext {
                t: CompletionType::IdSelector,
                current_node: Some(current_node),
                position,
            });
        }

        // Check if we're typing a pseudo-class after ':'
        if current_node.kind() == NODE_COLON {
            // Check if this colon is part of a selector (not a property declaration)
            // We can check if the colon's parent is a pseudo_class_selector or if it's in an ERROR node in selector context
            if let Some(parent) = current_node.parent() {
                let parent_text = parent.utf8_text(content.as_bytes()).unwrap_or("");

                // If parent is ERROR, check if there's a selector before this colon
                if parent.kind() == NODE_ERROR {
                    // Check previous sibling - pseudo-class must come after a selector
                    if let Some(prev_sibling) = current_node.prev_sibling() {
                        if prev_sibling.kind() == NODE_CLASS_SELECTOR
                            || prev_sibling.kind() == NODE_ID_SELECTOR
                            || prev_sibling.kind() == NODE_TAG_NAME
                        {
                            return Some(CompletionContext {
                                t: CompletionType::PseudoClass,
                                current_node: Some(current_node),
                                position,
                            });
                        }
                    }

                    // parent is also just colon, then check parent's prev sibling
                    if parent_text == ":" {
                        if let Some(parent_prev_sibling) = parent.prev_sibling() {
                            if parent_prev_sibling.kind() == NODE_SELECTORS {
                                return Some(CompletionContext {
                                    t: CompletionType::PseudoClass,
                                    current_node: Some(current_node),
                                    position,
                                });
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Complete class selectors
    fn complete_class_selectors(
        &self,
        tree: &Tree,
        content: &str,
        current_node: Node,
    ) -> Vec<CompletionItem> {
        let partial_text = self.extract_partial_selector_text(current_node, content, '.');

        let existing_classes = self.extract_class_selectors_from_document(tree, content);

        existing_classes
            .into_iter()
            .filter(|class_name| {
                if partial_text.is_empty() {
                    true // Show all classes when just typed '.'
                } else {
                    // Exclude exact matches and only show classes that start with partial_text but are longer
                    let class_lower = class_name.to_lowercase();
                    let partial_lower = partial_text.to_lowercase();
                    class_lower.starts_with(&partial_lower) && class_lower != partial_lower
                }
            })
            .map(|class_name| CompletionItem {
                label: class_name.clone(),
                kind: Some(CompletionItemKind::COLOR),
                detail: Some("Class selector".to_string()),
                insert_text: Some(class_name),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..Default::default()
            })
            .collect()
    }

    /// Complete ID selectors
    fn complete_id_selectors(
        &self,
        tree: &Tree,
        content: &str,
        current_node: Node,
    ) -> Vec<CompletionItem> {
        let partial_text = self.extract_partial_selector_text(current_node, content, '#');

        let existing_ids = self.extract_id_selectors_from_document(tree, content);

        existing_ids
            .into_iter()
            .filter(|id_name| {
                if partial_text.is_empty() {
                    true // Show all IDs when just typed '#'
                } else {
                    // Exclude exact matches and only show IDs that start with partial_text but are longer
                    let id_lower = id_name.to_lowercase();
                    let partial_lower = partial_text.to_lowercase();
                    id_lower.starts_with(&partial_lower) && id_lower != partial_lower
                }
            })
            .map(|id_name| CompletionItem {
                label: id_name.clone(),
                kind: Some(CompletionItemKind::CONSTANT),
                detail: Some("ID selector".to_string()),
                insert_text: Some(id_name),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..Default::default()
            })
            .collect()
    }

    /// Complete tag selectors
    /// Complete tag selectors using real UXML schema data
    fn complete_tag_selectors(
        &self,
        current_node: Node,
        content: &str,
        uxml_element_names: Option<&std::collections::HashSet<String>>,
    ) -> Vec<CompletionItem> {
        let partial_text = current_node
            .utf8_text(content.as_bytes())
            .unwrap_or("")
            .to_lowercase();

        let mut items = Vec::new();

        if let Some(element_names) = uxml_element_names {
            // Use real schema data from extracted element names
            for element_name in element_names {
                if element_name.to_lowercase().starts_with(&partial_text) {
                    items.push(CompletionItem {
                        label: element_name.clone(),
                        kind: Some(CompletionItemKind::CLASS),
                        detail: Some("UXML Element".to_string()),
                        insert_text: Some(element_name.clone()),
                        insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!("**UXML Element:** `{}`", element_name),
                        })),
                        ..Default::default()
                    });
                }
            }
        } else {
            // Fallback to hardcoded list if element names are not available
            let unity_tags = vec!["Button", "Label", "Slider", "Dropdown"];

            for tag_name in unity_tags {
                if tag_name.to_lowercase().starts_with(&partial_text) {
                    items.push(CompletionItem {
                        label: tag_name.to_string(),
                        kind: Some(CompletionItemKind::CLASS),
                        detail: Some("Unity UI element (fallback)".to_string()),
                        insert_text: Some(tag_name.to_string()),
                        insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                        ..Default::default()
                    });
                }
            }
        }

        items
    }

    /// Extract partial selector text (without the prefix character)
    fn extract_partial_selector_text(
        &self,
        current_node: Node,
        content: &str,
        prefix: char,
    ) -> String {
        let node_text = current_node.utf8_text(content.as_bytes()).unwrap_or("");

        if current_node.kind() == NODE_CLASS_NAME || current_node.kind() == NODE_ID_NAME {
            // We're in the name part of the selector
            return node_text.to_string();
        }

        if current_node.kind() == NODE_ERROR && node_text.starts_with(prefix) {
            // We're in an incomplete selector, extract the part after the prefix
            return node_text.chars().skip(1).collect();
        }

        // Handle case where we're directly on the prefix token
        if (current_node.kind() == "." && prefix == '.')
            || (current_node.kind() == "#" && prefix == '#')
        {
            // Just typed the prefix, return empty string to show all completions
            return String::new();
        }

        String::new()
    }

    /// Extract all class selectors from the document
    fn extract_class_selectors_from_document(&self, tree: &Tree, content: &str) -> Vec<String> {
        let (class_names, _) = self.collect_all_selectors_from_document(tree, content);
        class_names.into_iter().collect()
    }

    /// Extract all ID selectors from the document
    fn extract_id_selectors_from_document(&self, tree: &Tree, content: &str) -> Vec<String> {
        let (_, id_names) = self.collect_all_selectors_from_document(tree, content);
        id_names.into_iter().collect()
    }

    /// Analyze if we're inside an import statement and return appropriate context
    fn analyze_import_context<'a>(
        &self,
        tree: &'a Tree,
        content: &str,
        current_node: Node<'a>,
        position: Position,
    ) -> Option<CompletionContext<'a>> {
        // Check if we're inside an import statement
        if let Some(_) = find_node_of_type_at_position(
            tree.root_node(),
            content,
            Position::new(position.line, position.character.saturating_sub(1)),
            NODE_IMPORT_STATEMENT,
        ) {
            // First check if we're inside a url() function within the import
            if let Some(url_context) =
                self.analyze_url_function_context(current_node, content, position)
            {
                return Some(url_context);
            }

            // Check if we're inside a string node (direct import path)
            if current_node.kind() == NODE_STRING_VALUE {
                // Ensure the string node is a direct child of the import statement
                if let Some(parent) = current_node.parent() {
                    if parent.kind() == NODE_IMPORT_STATEMENT {
                        // Use the safer string extraction method
                        if let Some((url_string, cursor_offset)) = self
                            .extract_url_string_from_current_node(current_node, content, position)
                        {
                            return Some(CompletionContext {
                                t: CompletionType::UrlString {
                                    url_string,
                                    cursor_position: cursor_offset,
                                },
                                current_node: Some(current_node),
                                position,
                            });
                        }
                    }
                }
            }
        }

        None
    }

    /// Analyze if we're in an incomplete @import statement context
    fn analyze_incomplete_import_context<'a>(
        &self,
        current_node: Node<'a>,
        content: &str,
        position: Position,
    ) -> Option<CompletionContext<'a>> {
        let kind = current_node.kind();
        // First is this node an incomplete import keyword or a complete import keyword
        // If it's just @, then it is error node, if incomplete it is at keyword node, if it is complete, it is import node
        if kind == NODE_ERROR || kind == NODE_AT_KEYWORD || kind == NODE_IMPORT {
            // first check if we are at the top level, eg, not inside of any blocks
            if get_node_depth(current_node) == 2 {
                // this node should be inside of an error node(for just @ or @import) or an at rule(for incomplete import keyword, eg. @i)
                if let Some(parent) = current_node.parent(){
                    let parent_kind = parent.kind();
                    if parent_kind == NODE_ERROR || parent_kind == NODE_AT_RULE {
                        let text = current_node.utf8_text(content.as_bytes()).unwrap_or("");
                        if "@import".starts_with(text) {
                            return Some(CompletionContext {
                                t: CompletionType::ImportStatement,
                                current_node: Some(current_node),
                                position,
                            });
                        }
                    }
                }
            }
        }

        None
    }

    /// Analyze if we're inside a URL function and return appropriate context
    fn analyze_url_function_context<'a>(
        &self,
        current_node: Node<'a>,
        content: &str,
        position: Position,
    ) -> Option<CompletionContext<'a>> {
        // Only offer completions if current node is a string value
        if current_node.kind() != NODE_STRING_VALUE {
            return None;
        }

        // Validate the hierarchy: string_node -> arguments -> call_expression (url function)
        if !self.is_string_in_url_function_arguments(current_node, content) {
            return None;
        }

        // Extract URL string and cursor position from the current string node
        if let Some((url_string, cursor_pos)) =
            self.extract_url_string_from_current_node(current_node, content, position)
        {
            return Some(CompletionContext {
                t: CompletionType::UrlString {
                    url_string,
                    cursor_position: cursor_pos,
                },
                current_node: Some(current_node),
                position,
            });
        }

        None
    }

    /// Extract URL string and cursor position from the current string node
    fn extract_url_string_from_current_node(
        &self,
        string_node: Node,
        content: &str,
        position: Position,
    ) -> Option<(String, usize)> {
        let string_content = string_node.utf8_text(content.as_bytes()).unwrap_or("");

        // Check if string has proper quotes and extract content safely
        let url_string = if string_content.len() >= 2 {
            let first_char = string_content.chars().next()?;
            let last_char = string_content.chars().last()?;

            // Verify the string starts and ends with matching quotes
            if (first_char == '"' && last_char == '"') || (first_char == '\'' && last_char == '\'')
            {
                let inner_content = &string_content[1..string_content.len() - 1];

                // Don't provide completions if the string contains backslashes (escape sequences)
                if inner_content.contains('\\') {
                    return None;
                }

                inner_content.to_string()
            } else {
                // Malformed string, don't provide completions
                return None;
            }
        } else {
            // String too short to have quotes, don't provide completions
            return None;
        };

        // Calculate cursor position within the URL string
        let string_start = string_node.start_position();

        let cursor_offset = if position.line as usize == string_start.row {
            if position.character as usize > string_start.column + 1 {
                // +1 for opening quote
                position.character as usize - string_start.column - 1
            } else {
                0
            }
        } else {
            0
        };

        Some((url_string, cursor_offset))
    }

    /// Check if the current string node is properly inside URL function arguments
    /// Validates hierarchy: string_node -> arguments -> call_expression (url function)
    fn is_string_in_url_function_arguments(&self, string_node: Node, content: &str) -> bool {
        // Check immediate parent is arguments node
        let arguments_node = match string_node.parent() {
            Some(parent) if parent.kind() == NODE_ARGUMENTS => parent,
            _ => return false,
        };

        // Check arguments node's parent is call_expression
        let call_expression_node = match arguments_node.parent() {
            Some(parent) if parent.kind() == NODE_CALL_EXPRESSION => parent,
            _ => return false,
        };

        // Check if the call_expression is a URL function
        if let Some(function_name_node) = call_expression_node.child(0) {
            let function_name = function_name_node
                .utf8_text(content.as_bytes())
                .unwrap_or("");
            function_name == "url"
        } else {
            false
        }
    }

    /// Complete URL function arguments
    fn complete_url_function(
        &self,
        url_string: &str,
        cursor_position: usize,
        source_url: Option<&url::Url>,
    ) -> Vec<CompletionItem> {
        if let Some(provider) = &self.url_completion_provider {
            provider.complete_url(url_string, cursor_position, source_url)
        } else {
            Vec::new()
        }
    }

    /// Complete import statement structure
    fn complete_import_statement(&self) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Provide @import completion with project scheme (recommended)
        items.push(CompletionItem {
            label: "@import url(\"project:///Assets\");".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Import statement with project scheme (recommended)".to_string()),
            insert_text: Some("@import url(\"project:///Assets$1\");$0".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Import file using project scheme. This is the recommended way to reference assets.".to_string(),
            })),
            ..Default::default()
        });

        // Provide @import completion with empty URL for relative paths
        items.push(CompletionItem {
            label: "@import url(\"\");".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Import statement with empty URL".to_string()),
            insert_text: Some("@import url(\"$1\");$0".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value:
                    "Import file with any URL."
                        .to_string(),
            })),
            ..Default::default()
        });

        // Provide @import completion with absolute path
        items.push(CompletionItem {
            label: "@import url(\"/Assets\");".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Import statement with absolute path".to_string()),
            insert_text: Some("@import url(\"/Assets$1\");$0".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Import file using absolute path.".to_string(),
            })),
            ..Default::default()
        });

        items
    }

    /// Collect all selectors from the document, separating classes and IDs
    fn collect_all_selectors_from_document(
        &self,
        tree: &Tree,
        content: &str,
    ) -> (HashSet<String>, HashSet<String>) {
        let mut class_names = HashSet::new();
        let mut id_names = HashSet::new();
        self.collect_selectors_recursive(
            tree.root_node(),
            content,
            &mut class_names,
            &mut id_names,
        );
        (class_names, id_names)
    }

    /// Recursively collect selector names from the syntax tree
    fn collect_selectors_recursive(
        &self,
        node: Node,
        content: &str,
        class_collector: &mut HashSet<String>,
        id_collector: &mut HashSet<String>,
    ) {
        // Collect class names that are children of class selectors
        if node.kind() == NODE_CLASS_NAME {
            if let Some(parent) = node.parent() {
                if parent.kind() == NODE_CLASS_SELECTOR {
                    if let Ok(name) = node.utf8_text(content.as_bytes()) {
                        if !name.is_empty() {
                            class_collector.insert(name.to_string());
                        }
                    }
                }
            }
        }

        // Collect ID names that are children of ID selectors
        if node.kind() == NODE_ID_NAME {
            if let Some(parent) = node.parent() {
                if parent.kind() == NODE_ID_SELECTOR {
                    if let Ok(name) = node.utf8_text(content.as_bytes()) {
                        if !name.is_empty() {
                            id_collector.insert(name.to_string());
                        }
                    }
                }
            }
        }

        for child in node.children(&mut node.walk()) {
            self.collect_selectors_recursive(child, content, class_collector, id_collector);
        }
    }
}

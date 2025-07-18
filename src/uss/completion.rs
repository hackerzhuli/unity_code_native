//! USS Completion Provider
//!
//! Provides auto-completion for USS properties and values.
//! Supports completion for property values after ':' with automatic semicolon insertion.

use std::collections::HashSet;
use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Tree};
use url::Url;

use crate::language::tree_utils::{find_node_at_position, find_node_by_type, find_node_of_type_at_position, get_node_depth, node_to_range};
use crate::language::url_completion::UrlCompletionProvider;
use crate::uss::constants::*;
use crate::uss::definitions::UssDefinitions;

/// USS completion provider
pub struct UssCompletionProvider {
    pub(crate) definitions: UssDefinitions,
    url_completion_provider: Option<UrlCompletionProvider>,
}

#[derive(Debug, Clone)]
pub(super) struct CompletionContext<'a> {
    pub t: CompletionType,
    pub current_node: Option<Node<'a>>,
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
        }
    }

    /// Create a new USS completion provider with URL completion support
    pub fn new_with_project_root(project_root: &std::path::Path) -> Self {
        Self {
            definitions: UssDefinitions::new(),
            url_completion_provider: Some(UrlCompletionProvider::new(project_root)),
        }
    }

    /// Provide completion items for the given position
    pub fn complete(
        &self,
        tree: &Tree,
        content: &str,
        position: Position,
        source_url: Option<&Url>,
        uxml_element_names: Option<&std::collections::HashSet<String>>,
        unity_manager: Option<&crate::unity_project_manager::UnityProjectManager>,
    ) -> Vec<CompletionItem> {
        let context = self.get_completion_context(tree, content, position);

        let mut unity_version = "6000.0".to_string();
        if let Some(u) = unity_manager {
            if let Some(v) = u.get_unity_version_for_docs() {
                unity_version = v;
            }
        }

        if let Some(current_node) = context.current_node {
            match context.t {
                CompletionType::PropertyValue { property_name } => {
                    self.complete_property_value(&property_name, current_node, content, unity_version.as_str())
                }
                CompletionType::Property => self.complete_property_names(
                    context.current_node,
                    content,
                    unity_version.as_str(),
                ),
                CompletionType::PseudoClass => self.complete_pseudo_classes_with_filter(
                    current_node,
                    content,
                    unity_version.as_str(),
                ),
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
                CompletionType::ImportStatement => {
                    self.complete_import_statement(current_node, content)
                }
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
                if let Some(import_context) =
                    self.analyze_incomplete_import_context(current_node, content)
                {
                    return import_context;
                }

                if let Some(import_context) =
                    self.analyze_import_context(tree, content, current_node, position)
                {
                    return import_context;
                }

                if let Some(selector_context) =
                    self.analyze_selector_context(tree, content, current_node, position)
                {
                    return selector_context;
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

                if let Some(value) = self.analyze_property_name_context(tree, content, last_pos, current_node) {
                    return value;
                }

            }
        }

        return CompletionContext {
            t: CompletionType::Unknown,
            current_node: None,
        };
    }

    fn analyze_property_name_context<'a>(&self, tree: &Tree, content: &str, last_pos: Position, current_node: Node<'a>) -> Option<CompletionContext<'a>> {
        // Check if we're typing a property name within a block
        if let Some(_block_node) =
            find_node_of_type_at_position(tree.root_node(), content, last_pos, NODE_BLOCK)
        {
            // Check if current node is a property name being typed
            // Note: incomplete property names are parsed as "attribute_name" in ERROR nodes
            // New version css parser 0.23
            // incomplete property names are parsed as identifier name in error nodes
            let current_node_kind = current_node.kind();
            let parent_node_kind = match current_node.parent() {
                None => "",
                Some(p) => p.kind(),
            };

            if current_node_kind == NODE_IDENTIFIER && parent_node_kind == NODE_ERROR {
                return Some(CompletionContext {
                    t: CompletionType::Property,
                    current_node: Some(current_node),
                });
            } else if current_node_kind == NODE_COMMA && parent_node_kind == NODE_ERROR {
                // user just typed a comma after a comma seperated list (without semicolon yet)
                if let Some(parent) = current_node.parent() {
                    if let Some(parent_prev) = parent.prev_sibling() {
                        if parent_prev.kind() == NODE_DECLARATION {
                            if let Some(dec_last_node) =
                                parent_prev.child(parent_prev.child_count() - 1)
                            {
                                if dec_last_node.kind() != NODE_SEMICOLON {
                                    if let Some(property_name_node) = parent_prev.child(0) {
                                        if let Ok(property_name) =
                                            property_name_node.utf8_text(content.as_bytes())
                                        {
                                            if let Some(property_info) = self
                                                .definitions
                                                .get_property_info(property_name)
                                            {
                                                if property_info
                                                    .value_spec
                                                    .allows_multiple_values
                                                {
                                                    return Some(CompletionContext {
                                                        t: CompletionType::PropertyValue {
                                                            property_name: property_name
                                                                .to_string(),
                                                        },
                                                        current_node: Some(current_node),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else if current_node_kind == NODE_TAG_NAME && parent_node_kind == NODE_DESCENDANT_SELECTOR {
                // Tag name inside of block, is actually user typing another property name before another property
                // uss doesn't support nested rules, we're in a block, so it can't be a selector
                // tree sitter css parser thinks it is a selector, but it's not
                let parent = current_node.parent().unwrap();
                if parent.child_count() == 2 {
                    if let Some(next_sib) = current_node.next_sibling() {
                        if next_sib.kind() == NODE_TAG_NAME {
                            if let Some(parent_next_sib) = parent.next_sibling() {
                                if parent_next_sib.kind() == NODE_COLON {
                                    // now we know user is typing a property name
                                    return Some(CompletionContext {
                                        t: CompletionType::Property,
                                        current_node: Some(current_node)
                                    })
                                }
                            }
                        }
                    }
                }
            }
        }

        if Self::is_incomplete_property_name_parsed_as_tag_name(current_node) {
            return Some(CompletionContext {
                t: CompletionType::Property,
                current_node: Some(current_node)
            })
        }

        None
    }

    /// check if current node  a case where it is parsed as tag name, but actually the user typing a property name
    fn is_incomplete_property_name_parsed_as_tag_name(current_node: Node) -> bool {
        // another type of incomplete property name in css parser version 0.23
        // there is no block node
        // we are the first property name after `{`, and it is not parsed as a block
        if current_node.kind() == NODE_TAG_NAME {
            if let Some(parent) = current_node.parent() {
                if parent.kind() == NODE_DESCENDANT_SELECTOR {
                    if let Some(parent_parent) = parent.parent() {
                        if parent_parent.kind() == NODE_SELECTORS {
                            if let Some(parent_parent_sibling) = parent_parent.prev_sibling() {
                                if parent_parent_sibling.kind() == "{" {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
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
                };
            }
        }

        return CompletionContext {
            t: CompletionType::Unknown,
            current_node: Some(current_node),
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
        unity_version: &str,
    ) -> Vec<CompletionItem> {
        // value just started, with colon or comma
        let mut is_colon_or_comma = false;
        if current_node.kind() == NODE_COLON || current_node.kind() == NODE_COMMA {
            // Right after colon - show all common values
            is_colon_or_comma = true;
        }

        // check if we are the first value node(ie. the previous node is colon and the node before that is property name)
        let mut is_first_value_node = is_colon_or_comma;
        if !is_colon_or_comma {
            if let Some(prev_sibling) = current_node.prev_sibling() {
                if prev_sibling.kind() == NODE_COLON {
                    if let Some(prev_prev_sibling) = prev_sibling.prev_sibling() {
                        if prev_prev_sibling.kind() == NODE_PROPERTY_NAME {
                            is_first_value_node = true;
                        }
                    }
                } else if prev_sibling.kind() == NODE_COMMA {
                    is_first_value_node = true;
                }
            }
        }

        if !is_first_value_node {
            return Vec::new();
        }

        let property_info_option = self.definitions.get_property_info(property_name);
        if property_info_option.is_none() {
            return Vec::new();
        }

        let property_info = property_info_option.unwrap();

        // keywords that we want to filter against
        let valid_values = self
            .definitions
            .get_simple_completions_for_property(property_name);

        if valid_values.is_empty() {
            return Vec::new();
        }

        // try to find what still matches and return that
        let mut items = Vec::new();
        let partial_value = if is_colon_or_comma {
            ""
        } else {
            current_node.utf8_text(content.as_bytes()).unwrap_or("")
        };
        let partial_lower = partial_value.to_lowercase();

        for value in valid_values {
            if partial_lower.is_empty() || value.starts_with(&partial_lower) {
                // add one space if user just typed colon or comma
                let mut text = if is_colon_or_comma {
                    format!(" {}", value)
                } else {
                    format!("{}", value)
                };

                // add a semicolon for a value that doesn't have multiple values(ie. comma separated values)
                if !property_info.value_spec.allows_multiple_values {
                    text.push(';');
                }

                // Check if this value is a keyword and get its documentation
                // Special case: for transition-property, treat values as property names first
                let mut documentation = if property_name == "transition-property" {
                    self.definitions.get_property_info(value)
                        .map(|property_info| {
                            let doc_content = property_info.create_documentation(value, unity_version);
                            Documentation::MarkupContent(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: doc_content,
                            })
                        })
                }else {
                    None
                };

                if documentation.is_none() {
                    documentation = self.definitions.get_keyword_info(value)
                        .map(|keyword_info| {
                            let doc_content = keyword_info.create_documentation(Some(property_name));
                            Documentation::MarkupContent(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: doc_content,
                            })
                        })
                }

                // Check if this value is a color keyword to provide color preview
                let kind = if self.definitions.is_valid_color_keyword(value) {
                    CompletionItemKind::COLOR
                } else {
                    CompletionItemKind::VALUE
                };

                let mut item = CompletionItem {
                    label: value.to_string(),
                    kind: Some(kind),
                    documentation,
                    insert_text: Some(text),
                    insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                    ..Default::default()
                };

                // Add color information for VS Code color preview
                if let Some((r, g, b)) = self.definitions.get_color_rgb(value) {
                    // Set the color information that VS Code can use for color preview
                    item.data = Some(serde_json::json!({
                        "color": {
                            "red": r as f64 / 255.0,
                            "green": g as f64 / 255.0,
                            "blue": b as f64 / 255.0,
                            "alpha": 1.0
                        }
                    }));
                }
                items.push(item);
            }
        }

        items
    }

    /// Complete property names
    fn complete_property_names(
        &self,
        current_node: Option<tree_sitter::Node>,
        content: &str,
        unity_version: &str,
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
            .get_all_properties()
            .keys()
            .filter(|name| name.starts_with(&partial_text))
            .map(|name| {
                let property_info = self.definitions.get_property_info(name);
                let documentation = property_info
                    .as_ref()
                    .map(|info| info.create_documentation(name, unity_version));

                CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    documentation: documentation.map(|doc| {
                        Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: doc,
                        })
                    }),
                    // no colon because if user type the colon
                    // that will trigger next round of auto completion for value, which is prefered
                    insert_text: Some(format!("{}", name)),
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
        unity_version: &str,
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

        for &pseudo_class in &self.definitions.valid_pseudo_classes {
            // Filter based on partial text
            if partial_text.is_empty()
                || (pseudo_class.starts_with(&partial_text) && pseudo_class != partial_text)
            {
                if let Some(info) = self.definitions.get_pseudo_class_info(pseudo_class) {
                    // Create documentation with description and link
                    let documentation_value = info.create_documentation(&unity_version);

                    items.push(CompletionItem {
                        label: pseudo_class.to_string(),
                        kind: Some(CompletionItemKind::KEYWORD),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: documentation_value,
                        })),
                        insert_text: Some(pseudo_class.to_string()),
                        insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                        ..Default::default()
                    });
                }
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
        if Self::is_class_selector_being_typed(current_node)
        {
            return Some(CompletionContext {
                t: CompletionType::ClassSelector,
                current_node: Some(current_node),
            });
        }

        // Check if current node is an ID selector being typed
        if current_node.kind() == NODE_ID_SELECTOR
            || (current_node.kind() == NODE_ID_NAME
                && current_node.parent().map(|p| p.kind()) == Some(NODE_ID_SELECTOR))
        {
            return Some(CompletionContext {
                t: CompletionType::IdSelector,
                current_node: Some(current_node),
            });
        }

        if Self::is_tag_selector_being_typed(current_node){
            return Some(CompletionContext {
                t: CompletionType::TagSelector,
                current_node: Some(current_node),
            });
        }

        if Self::is_pseudo_class_being_typed(current_node) {
            return Some(CompletionContext {
                t: CompletionType::PseudoClass,
                current_node: Some(current_node),
            });
        }

        // Check if we're directly on a '.' or '#' token
        if current_node.kind() == "." {
            return Some(CompletionContext {
                t: CompletionType::ClassSelector,
                current_node: Some(current_node),
            });
        }

        if current_node.kind() == "#" {
            return Some(CompletionContext {
                t: CompletionType::IdSelector,
                current_node: Some(current_node),
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
                                });
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// check if current node is a class selector being typed
    fn is_class_selector_being_typed(current_node: Node) -> bool {
        let kind = current_node.kind();
        if kind == NODE_CLASS_SELECTOR {
            return true;
        }

        if let Some(parent) = current_node.parent() {
            let parent_kind = parent.kind();
            if kind == NODE_CLASS_NAME
               && parent_kind == NODE_CLASS_SELECTOR{
                return true;
            }

            if let Some(parent_parent) = parent.parent() {
                if kind == NODE_IDENTIFIER && parent_kind == NODE_CLASS_NAME && parent_parent.kind() == NODE_CLASS_SELECTOR{
                    return true;
                }
            }
        }

        false
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
                kind: Some(CompletionItemKind::CLASS),
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
        // In new version 0.23 the selector names are in identifier node
        if current_node.kind() == NODE_CLASS_NAME || current_node.kind() == NODE_ID_NAME || current_node.kind() == NODE_IDENTIFIER {
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
            if current_node.kind() == NODE_STRING_CONTENT {
                if let Some(parent) = current_node.parent() {
                    if parent.kind() == NODE_STRING_VALUE{
                        // Ensure the string node is a direct child of the import statement
                        if let Some(parent_parent) = parent.parent() {
                            if parent_parent.kind() == NODE_IMPORT_STATEMENT {
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
                                    });
                                }
                            }
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
    ) -> Option<CompletionContext<'a>> {
        let kind = current_node.kind();
        // First is this node an incomplete import keyword or a complete import keyword
        // If it's just @, then it is error node, if incomplete it is at keyword node, if it is complete, it is import node
        if kind == NODE_ERROR || kind == NODE_AT_KEYWORD || kind == NODE_IMPORT {
            // first check if we are at the top level, eg, not inside of any blocks
            if get_node_depth(current_node) == 2 {
                // this node should be inside of an error node(for just @ or @import) or an at rule(for incomplete import keyword, eg. @i)
                if let Some(parent) = current_node.parent() {
                    let parent_kind = parent.kind();
                    if parent_kind == NODE_ERROR || parent_kind == NODE_AT_RULE {
                        let text = current_node.utf8_text(content.as_bytes()).unwrap_or("");
                        if "@import".starts_with(text) {
                            return Some(CompletionContext {
                                t: CompletionType::ImportStatement,
                                current_node: Some(current_node),
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
        if current_node.kind() != NODE_STRING_CONTENT {
            return None;
        }

        let string_value_node = current_node.parent()?;

        // Validate the hierarchy: string_node -> arguments -> call_expression (url function)
        if !self.is_string_in_url_function_arguments(string_value_node, content) {
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
        let url_string = string_content.to_string();

        // Calculate cursor position within the URL string
        let string_start = string_node.start_position();

        let cursor_offset = if position.line as usize == string_start.row {
            if position.character as usize >= string_start.column {
                position.character as usize - string_start.column
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
    fn is_string_in_url_function_arguments(&self, string_value_node: Node, content: &str) -> bool {
        // Check immediate parent is arguments node
        let arguments_node = match string_value_node.parent() {
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

    /// Complete import statement structure with specific range
    fn complete_import_statement<'a>(
        &self,
        current_node: Node<'a>,
        content: &str,
    ) -> Vec<CompletionItem> {
        let text_edit_range = node_to_range(current_node, content);
        let mut items = Vec::new();

        // Determine if we should use text_edit (when we have a valid range) or insert_text
        let use_text_edit = text_edit_range.start.line != 0
            || text_edit_range.start.character != 0
            || text_edit_range.end.line != 0
            || text_edit_range.end.character != 0;

        // Provide @import completion with project scheme (recommended)
        let mut item1 = CompletionItem {
            label: "@import url(\"project:///Assets\");".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Import statement with project scheme (recommended)".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            filter_text: Some("@import".to_string()),
            sort_text: Some("0001".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Import file using project scheme. This is the recommended way to reference assets.".to_string(),
            })),
            ..Default::default()
        };

        if use_text_edit {
            item1.text_edit = Some(CompletionTextEdit::Edit(TextEdit {
                range: text_edit_range,
                new_text: "@import url(\"project:///Assets$1\");$0".to_string(),
            }));
        } else {
            item1.insert_text = Some("@import url(\"project:///Assets$1\");$0".to_string());
        }
        items.push(item1);

        // Provide @import completion with empty URL for relative paths
        let mut item2 = CompletionItem {
            label: "@import url(\"\");".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Import statement with empty URL".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            filter_text: Some("@import".to_string()),
            sort_text: Some("0002".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Import file with any URL.".to_string(),
            })),
            ..Default::default()
        };

        if use_text_edit {
            item2.text_edit = Some(CompletionTextEdit::Edit(TextEdit {
                range: text_edit_range,
                new_text: "@import url(\"$1\");$0".to_string(),
            }));
        } else {
            item2.insert_text = Some("@import url(\"$1\");$0".to_string());
        }
        items.push(item2);

        // Provide @import completion with absolute path
        let mut item3 = CompletionItem {
            label: "@import url(\"/Assets\");".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Import statement with absolute path".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            filter_text: Some("@import".to_string()),
            sort_text: Some("0003".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Import file using absolute path.".to_string(),
            })),
            ..Default::default()
        };

        if use_text_edit {
            item3.text_edit = Some(CompletionTextEdit::Edit(TextEdit {
                range: text_edit_range,
                new_text: "@import url(\"/Assets$1\");$0".to_string(),
            }));
        } else {
            item3.insert_text = Some("@import url(\"/Assets$1\");$0".to_string());
        }
        items.push(item3);

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

    /// if current node is a pseudo class being typed
    fn is_pseudo_class_being_typed(current_node:Node) -> bool {
        if current_node.kind() == NODE_IDENTIFIER {
            if let Some(parent) = current_node.parent(){
                if parent.kind() == NODE_CLASS_NAME {
                    if let Some(parent_prev) = parent.prev_sibling() {
                        return parent_prev.kind() == NODE_COLON
                    }
                }
            }
        }

        false
    }

    /// Check if current node is a tag selector being typed
    fn is_tag_selector_being_typed(current_node: Node) -> bool {
        if current_node.kind() == NODE_TAG_NAME {
            // make sure this is not a property name
            if !Self::is_incomplete_property_name_parsed_as_tag_name(current_node){
                return true;
            }
        }
        else if current_node.kind() == NODE_IDENTIFIER {
            if let Some(parent) = current_node.parent() {
                if let Some(parent_parent) = parent.parent(){
                    // parent parent is root node
                    if parent_parent.parent().is_none() && parent.kind() == NODE_ERROR {
                        return true;
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
#[path = "completion_tests.rs"]
mod completion_tests;

#[cfg(test)]
#[path = "completion_tests_declaration.rs"]
mod completion_tests_declaration;

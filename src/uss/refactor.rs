//! USS Refactoring functionality
//!
//! Provides code actions for refactoring USS files, including renaming selectors.

use tower_lsp::lsp_types::*;
use tree_sitter::Node;
use crate::language::tree_utils::{node_to_range, find_node_at_position};
use crate::uss::document::UssDocument;

/// USS Refactor provider for code actions
pub struct UssRefactorProvider {
    // Future: could add configuration options here
}

impl UssRefactorProvider {
    /// Create a new USS refactor provider
    pub fn new() -> Self {
        Self {}
    }

    /// Find all references to a class or id selector in the document
    pub fn find_selector_references(
        &self,
        tree: &tree_sitter::Tree,
        content: &str,
        selector_name: &str,
        selector_type: SelectorType,
    ) -> Vec<Range> {
        let mut references = Vec::new();
        self.find_selector_references_recursive(
            tree.root_node(),
            content,
            selector_name,
            selector_type,
            &mut references,
        );
        references
    }

    /// Recursively find selector references in the syntax tree
    fn find_selector_references_recursive(
        &self,
        node: Node,
        content: &str,
        selector_name: &str,
        selector_type: SelectorType,
        references: &mut Vec<Range>,
    ) {
        let node_kind = node.kind();
        
        match selector_type {
            SelectorType::Class => {
                // Look for class_selector nodes
                if node_kind == "class_selector" {
                    // Find the class_name child
                    if let Some(class_name_node) = self.find_child_by_kind(node, "class_name") {
                        // Get the identifier child which contains the actual class name
                        if let Some(identifier_node) = self.find_child_by_kind(class_name_node, "identifier") {
                            if let Ok(text) = identifier_node.utf8_text(content.as_bytes()) {
                                if text == selector_name {
                                    references.push(node_to_range(identifier_node, content));
                                }
                            }
                        }
                    }
                }
            }
            SelectorType::Id => {
                // Look for id_selector nodes
                if node_kind == "id_selector" {
                    // Find the id_name child which directly contains the ID name
                    if let Some(id_name_node) = self.find_child_by_kind(node, "id_name") {
                        if let Ok(text) = id_name_node.utf8_text(content.as_bytes()) {
                            if text == selector_name {
                                references.push(node_to_range(id_name_node, content));
                            }
                        }
                    }
                }
            }
        }
        
        // Recursively check children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.find_selector_references_recursive(
                    child,
                    content,
                    selector_name,
                    selector_type,
                    references,
                );
            }
        }
    }
    
    /// Helper function to find a child node by its kind
    fn find_child_by_kind<'a>(&self, node: Node<'a>, target_kind: &str) -> Option<Node<'a>> {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == target_kind {
                    return Some(child);
                }
            }
        }
        None
    }

    /// Generate workspace edit for renaming a selector
    pub fn rename_selector(
        &self,
        tree: &tree_sitter::Tree,
        content: &str,
        uri: &Url,
        old_name: &str,
        new_name: &str,
        selector_type: SelectorType,
    ) -> Option<WorkspaceEdit> {
        let references = self.find_selector_references(tree, content, old_name, selector_type);
        
        if references.is_empty() {
            return None;
        }

        let text_edits: Vec<TextEdit> = references
            .into_iter()
            .map(|range| TextEdit {
                range,
                new_text: new_name.to_string(),
            })
            .collect();

        let mut changes = std::collections::HashMap::new();
        changes.insert(uri.clone(), text_edits);

        Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        })
    }

    /// Generate code actions for the given range
    pub fn get_code_actions(
        &self,
        document: &UssDocument,
        uri: &Url,
        range: Range,
    ) -> Option<Vec<CodeActionOrCommand>> {
        let mut actions = Vec::new();
        
        if let Some(tree) = document.tree() {
            // Find selector at the current position
            let start_position = range.start;
            
            if let Some(node) = find_node_at_position(tree.root_node(), start_position) {
                let mut current = node;
                
                // Walk up the tree to find a selector
                loop {
                    let node_kind = current.kind();
                    
                    if node_kind == "class_selector" || node_kind == "id_selector" {
                        // Extract the selector name
                        let selector_text = if let Ok(text) = current.utf8_text(document.content().as_bytes()) {
                            text.to_string()
                        } else {
                            break;
                        };
                        
                        let (selector_type, selector_name) = if node_kind == "class_selector" {
                            if selector_text.starts_with('.') {
                                (SelectorType::Class, &selector_text[1..])
                            } else {
                                break;
                            }
                        } else if node_kind == "id_selector" {
                            if selector_text.starts_with('#') {
                                (SelectorType::Id, &selector_text[1..])
                            } else {
                                break;
                            }
                        } else {
                            break;
                        };
                        
                        // Create rename action
                        let action = CodeActionOrCommand::CodeAction(CodeAction {
                            title: format!("Rename {} '{}'", 
                                if matches!(selector_type, SelectorType::Class) { "class" } else { "id" },
                                selector_name
                            ),
                            kind: Some(CodeActionKind::REFACTOR),
                            diagnostics: None,
                            edit: None,
                            command: Some(Command {
                                title: "Rename Selector".to_string(),
                                command: "uss.renameSelector".to_string(),
                                arguments: Some(vec![
                                    serde_json::to_value(&uri).unwrap(),
                                    serde_json::to_value(selector_name).unwrap(),
                                    serde_json::to_value(match selector_type {
                                        SelectorType::Class => "class",
                                        SelectorType::Id => "id",
                                    }).unwrap(),
                                ]),
                            }),
                            is_preferred: Some(true),
                            disabled: None,
                            data: None,
                        });
                        
                        actions.push(action);
                        break;
                    }
                    
                    if let Some(parent) = current.parent() {
                        current = parent;
                    } else {
                        break;
                    }
                }
            }
        }
        
        if actions.is_empty() {
            None
        } else {
            Some(actions)
        }
    }

    /// Prepare rename operation by finding the selector at the given position
    pub fn prepare_rename(
        &self,
        document: &UssDocument,
        position: Position,
    ) -> Option<PrepareRenameResponse> {
        if let Some(tree) = document.tree() {
            // Find selector at the current position
            if let Some(node) = find_node_at_position(tree.root_node(), position) {
                let mut current = node;
                
                // Walk up the tree to find a selector
                loop {
                    let node_kind = current.kind();
                    
                    if node_kind == "class_selector" || node_kind == "id_selector" {
                        // Extract the selector name and range
                        let selector_text = if let Ok(text) = current.utf8_text(document.content().as_bytes()) {
                            text.to_string()
                        } else {
                            break;
                        };
                        
                        let (_, selector_name) = if node_kind == "class_selector" {
                            if selector_text.starts_with('.') {
                                (SelectorType::Class, &selector_text[1..])
                            } else {
                                break;
                            }
                        } else if node_kind == "id_selector" {
                            if selector_text.starts_with('#') {
                                (SelectorType::Id, &selector_text[1..])
                            } else {
                                break;
                            }
                        } else {
                            break;
                        };
                        
                        // Get the range of the selector name (without the . or # prefix)
                        let mut range = node_to_range(current, document.content());
                        
                        // Adjust range to exclude the prefix (. or #)
                        range.start.character += 1;
                        
                        return Some(PrepareRenameResponse::RangeWithPlaceholder {
                            range,
                            placeholder: selector_name.to_string(),
                        });
                    }
                    
                    if let Some(parent) = current.parent() {
                        current = parent;
                    } else {
                        break;
                    }
                }
            }
        }
        
        None
    }

    /// Handle rename operation by finding the selector and generating workspace edit
    pub fn handle_rename(
        &self,
        document: &UssDocument,
        uri: &Url,
        position: Position,
        new_name: &str,
    ) -> Option<WorkspaceEdit> {
        if let Some(tree) = document.tree() {
            // Find selector at the current position
            if let Some(node) = find_node_at_position(tree.root_node(), position) {
                let mut current = node;
                
                // Walk up the tree to find a selector
                loop {
                    let node_kind = current.kind();
                    
                    if node_kind == "class_selector" || node_kind == "id_selector" {
                        // Extract the selector name
                        let selector_text = if let Ok(text) = current.utf8_text(document.content().as_bytes()) {
                            text.to_string()
                        } else {
                            break;
                        };
                        
                        let (selector_type, old_name) = if node_kind == "class_selector" {
                            if selector_text.starts_with('.') {
                                (SelectorType::Class, &selector_text[1..])
                            } else {
                                break;
                            }
                        } else if node_kind == "id_selector" {
                            if selector_text.starts_with('#') {
                                (SelectorType::Id, &selector_text[1..])
                            } else {
                                break;
                            }
                        } else {
                            break;
                        };
                        
                        // Generate workspace edit for renaming
                        return self.rename_selector(
                            tree,
                            document.content(),
                            uri,
                            old_name,
                            new_name,
                            selector_type,
                        );
                    }
                    
                    if let Some(parent) = current.parent() {
                        current = parent;
                    } else {
                        break;
                    }
                }
            }
        }
        
        None
    }
}

/// Type of CSS selector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorType {
    /// Class selector (.class-name)
    Class,
    /// ID selector (#id-name)
    Id,
}

impl Default for UssRefactorProvider {
    fn default() -> Self {
        Self::new()
    }
}
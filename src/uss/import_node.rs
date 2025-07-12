//! Import Node Validation
//!
//! Provides validation for USS @import statements.
//! Validates basic structure: @import followed by string or function argument and semicolon.

use tower_lsp::lsp_types::*;
use tree_sitter::Node;
use crate::language::tree_utils::node_to_range;
use crate::uss::constants::*;

/// Represents a validated USS import statement
#[derive(Debug, Clone)]
pub struct ImportNode<'a> {
    /// The argument node (string or function) following @import
    pub argument_node: Node<'a>,
}

impl<'a> ImportNode<'a> {
    /// Attempts to create an ImportNode from a tree-sitter node
    /// 
    /// Returns Some(ImportNode) if the node represents a valid import statement structure:
    /// - Node must be an import statement
    /// - Nodes themselves must not contain errors
    /// - Must have exactly one argument node after @import
    /// - Argument must be either a string or function
    /// - Must be followed by a semicolon
    /// 
    /// # Arguments
    /// * `node` - The tree-sitter node to validate
    /// * `content` - The source code content
    /// * `diagnostics` - Optional vector to push validation errors to (no diagnostics will be pushed if nodes themselves contain errors)
    pub fn from_node(
        node: Node<'a>,
        content: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<Self> {
        // Check if this is an import statement
        if node.kind() != NODE_IMPORT_STATEMENT {
            return None;
        }

        // If the syntax tree has error nodes, return None without pushing diagnostics
        if node.has_error() {
            return None;
        }

        // Import statement structure: import_statement -> @import + (string_value | call_expression) + semicolon
        // Find the value child that contains the import path (either string or url() function)
        let mut import_value_node = None;

        // first node must be @import, second node is url function or a string, third node must be ; to end the statement, and nothing after that
        // the tree says it is a import statement, so we assume tree sitter already checked so no need to check that
        if node.child_count() > 1 {
            import_value_node = Some(node.child(1).unwrap());
        } else {
            // Missing argument
            let range = node_to_range(node, content);
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("missing-import-argument".to_string())),
                source: Some("uss".to_string()),
                message: "Import statement is missing an argument".to_string(),
                ..Default::default()
            });
            return None;
        }

        // we expect the third child to be a ";"
        if node.child_count() > 2 {
            let semi_node = node.child(2).unwrap();
            if semi_node.kind() != NODE_SEMICOLON {
                let range = node_to_range(semi_node, content);
                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("missing-semicolon".to_string())),
                    source: Some("uss".to_string()),
                    message: format!(
                        "Import statement is expecting a semicolon, but found {}",
                        semi_node.utf8_text(content.as_bytes()).unwrap_or("None")
                    ),
                    ..Default::default()
                });
            }
        } else {
            // Missing semicolon
            let range = node_to_range(node, content);
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("missing-semicolon".to_string())),
                source: Some("uss".to_string()),
                message: "Import statement is missing a semicolon".to_string(),
                ..Default::default()
            });
        }

        // Check if we have more than 3 children (multiple arguments)
        if node.child_count() > 3 {
            let extra_node = node.child(3).unwrap();
            let range = node_to_range(extra_node, content);
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("multiple-import-arguments".to_string())),
                source: Some("uss".to_string()),
                message: "Import statement should have only one argument".to_string(),
                ..Default::default()
            });
        }

        if let Some(value_node) = import_value_node {
            // Validate that the argument is a string or url() function
            match value_node.kind() {
                NODE_STRING_VALUE | NODE_CALL_EXPRESSION => {
                    // Valid argument type - create ImportNode
                    Some(ImportNode {
                        argument_node: value_node,
                    })
                }
                _ => {
                    let range = node_to_range(value_node, content);
                    diagnostics.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("invalid-import-argument".to_string())),
                        source: Some("uss".to_string()),
                        message: "Import path must be a string or url() function".to_string(),
                        ..Default::default()
                    });
                    None
                }
            }
        } else {
            None
        }
    }
}

//! Function Node Validation
//!
//! Provides validation for USS function calls.
//! Validates basic structure and extracts function name and arguments.

use tower_lsp::lsp_types::*;
use tree_sitter::Node;
use crate::language::tree_utils::node_to_range;
use crate::uss::constants::*;

/// Represents a validated USS function call
#[derive(Debug, Clone)]
pub struct FunctionNode<'a> {
    /// The function name
    pub function_name: String,
    /// The argument nodes (excluding parentheses and commas)
    pub argument_nodes: Vec<Node<'a>>,
}

impl<'a> FunctionNode<'a> {
    /// Attempts to create a FunctionNode from a tree-sitter node
    /// 
    /// Returns Some(FunctionNode) if the node represents a valid function call structure:
    /// - Node must be a call_expression
    /// - Nodes themselves must not contain errors
    /// - Extracts function name and all argument nodes
    /// - Returns None if nested function calls are detected (USS doesn't support nested functions)
    /// 
    /// # Arguments
    /// * `node` - The tree-sitter node to validate
    /// * `content` - The source code content
    /// * `diagnostics` - Optional vector to collect diagnostics
    pub fn from_node(
        node: Node<'a>,
        content: &str,
        diagnostics: Option<&mut Vec<Diagnostic>>,
    ) -> Option<Self> {
        // Check if this is a call expression
        if node.kind() != NODE_CALL_EXPRESSION {
            return None;
        }

        // If the syntax tree has error nodes, return None
        if node.has_error() {
            return None;
        }

        // Extract function name
        let function_name = if let Some(function_name_node) = node.child(0) {
            function_name_node
                .utf8_text(content.as_bytes())
                .unwrap_or("")
                .to_string()
        } else {
            return None;
        };

        // Extract arguments and check for nested functions
        let mut argument_nodes = Vec::new();
        
        if let Some(args_node) = node.child(1) {
            if args_node.kind() != NODE_ARGUMENTS {
                return None;
            }

            // Parse arguments structure: arguments -> "(" + arg1 + "," + arg2 + ... + ")"
            // We skip parentheses and commas, only collecting actual argument nodes
            for i in 0..args_node.child_count() {
                if let Some(child) = args_node.child(i) {
                    match child.kind() {
                        NODE_OPEN_PAREN | NODE_CLOSE_PAREN | "," => {
                            // Skip structural elements
                            continue;
                        }
                        _ => {
                            // This is an actual argument
                            argument_nodes.push(child);
                            
                            // Check for nested function calls - if found, return None
                            if Self::has_nested_function_calls(child) {
                                if let Some(diag) = diagnostics {
                                    let range = node_to_range(node, content);
                                    diag.push(Diagnostic {
                                        range,
                                        severity: Some(DiagnosticSeverity::ERROR),
                                        code: Some(NumberOrString::String("nested-functions-not-supported".to_string())),
                                        source: Some("uss".to_string()),
                                        message: "Nested function calls are not supported in USS".to_string(),
                                        ..Default::default()
                                    });
                                }
                                return None;
                            }
                        }
                    }
                }
            }
        }

        Some(FunctionNode {
            function_name,
            argument_nodes,
        })
    }

    /// Get the number of arguments
    pub fn argument_count(&self) -> usize {
        self.argument_nodes.len()
    }

    /// Check if this is a specific function by name
    pub fn is_function(&self, name: &str) -> bool {
        self.function_name == name
    }

    /// Get argument text at specific index
    pub fn get_argument_text(&self, index: usize, content: &str) -> Option<String> {
        self.argument_nodes.get(index)
            .and_then(|node| node.utf8_text(content.as_bytes()).ok())
            .map(|s| s.to_string())
    }

    /// Check if a node contains nested function calls
    fn has_nested_function_calls(node: Node) -> bool {
        // Check if this node itself is a function call
        if node.kind() == NODE_CALL_EXPRESSION {
            return true;
        }
        
        // Recursively check all children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if Self::has_nested_function_calls(child) {
                    return true;
                }
            }
        }
        
        false
    }
}

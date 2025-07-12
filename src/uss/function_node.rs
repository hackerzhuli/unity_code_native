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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uss::parser::UssParser;
    use crate::language::tree_utils::find_node_by_type;

    #[test]
    fn test_url_function_single_argument() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = "background-image: url(\"image.png\");";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let result = FunctionNode::from_node(call_node, source, None);
            
            assert!(result.is_some(), "Expected valid FunctionNode");
            
            if let Some(func) = result {
                assert_eq!(func.function_name, "url");
                assert_eq!(func.argument_count(), 1);
                assert!(func.is_function("url"));
                
                let arg_text = func.get_argument_text(0, source).unwrap();
                assert_eq!(arg_text, "\"image.png\"");
            }
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_rgb_function_multiple_arguments() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = "color: rgb(255, 128, 0);";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let result = FunctionNode::from_node(call_node, source, None);
            
            assert!(result.is_some(), "Expected valid FunctionNode");
            
            if let Some(func) = result {
                assert_eq!(func.function_name, "rgb");
                assert_eq!(func.argument_count(), 3);
                assert!(func.is_function("rgb"));
                
                assert_eq!(func.get_argument_text(0, source).unwrap(), "255");
                assert_eq!(func.get_argument_text(1, source).unwrap(), "128");
                assert_eq!(func.get_argument_text(2, source).unwrap(), "0");
            }
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_function_no_arguments() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = "background-image: url();";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let result = FunctionNode::from_node(call_node, source, None);
            
            assert!(result.is_some(), "Expected valid FunctionNode");
            
            if let Some(func) = result {
                assert_eq!(func.function_name, "url");
                assert_eq!(func.argument_count(), 0);
            }
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_non_call_expression_returns_none() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = ".test { color: red; }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        // Try to create FunctionNode from a rule_set node
        if let Some(rule_node) = find_node_by_type(root, "rule_set") {
            let result = FunctionNode::from_node(rule_node, source, None);
            
            assert!(result.is_none(), "Expected None for non-call-expression node");
        }
    }

    #[test]
    fn test_function_with_syntax_errors_returns_none() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Malformed CSS that will have error nodes
        let source = "background-image: url(\"image.png\" {{{;";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        // Check if there are any error nodes in the tree
        fn has_error_nodes(node: tree_sitter::Node) -> bool {
            if node.is_error() || node.kind() == "ERROR" {
                return true;
            }
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if has_error_nodes(child) {
                        return true;
                    }
                }
            }
            false
        }
        
        if has_error_nodes(root) {
            if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
                let result = FunctionNode::from_node(call_node, source, None);
                
                assert!(result.is_none(), "Expected None when syntax tree has errors");
            }
        }
    }

    #[test]
    fn test_calc_function_complex_arguments() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = "width: calc(100% - 20px);";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let result = FunctionNode::from_node(call_node, source, None);
            
            assert!(result.is_some(), "Expected valid FunctionNode");
            
            if let Some(func) = result {
                assert_eq!(func.function_name, "calc");
                // calc typically has one complex expression argument
                assert!(func.argument_count() >= 1);
            }
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_nested_function_calls_rejected() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test nested url() inside calc()
        let source = "width: calc(url(\"image.png\") + 10px);";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let mut diagnostics = Vec::new();
            let result = FunctionNode::from_node(call_node, source, Some(&mut diagnostics));
            
            assert!(result.is_none(), "Expected None for nested function calls");
            assert!(!diagnostics.is_empty(), "Expected diagnostic for nested functions");
            
            let diagnostic = &diagnostics[0];
            assert_eq!(diagnostic.code, Some(NumberOrString::String("nested-functions-not-supported".to_string())));
            assert_eq!(diagnostic.message, "Nested function calls are not supported in USS");
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_multiple_nested_functions_rejected() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test multiple nested functions
        let source = "background: linear-gradient(rgb(255, 0, 0), url(\"bg.png\"));";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let mut diagnostics = Vec::new();
            let result = FunctionNode::from_node(call_node, source, Some(&mut diagnostics));
            
            assert!(result.is_none(), "Expected None for nested function calls");
            assert!(!diagnostics.is_empty(), "Expected diagnostic for nested functions");
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_deeply_nested_functions_rejected() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test deeply nested functions: calc(rgb(url("test")))
        let source = "color: calc(rgb(url(\"test.png\")));";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let mut diagnostics = Vec::new();
            let result = FunctionNode::from_node(call_node, source, Some(&mut diagnostics));
            
            assert!(result.is_none(), "Expected None for deeply nested function calls");
            assert!(!diagnostics.is_empty(), "Expected diagnostic for nested functions");
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_function_with_mixed_arguments() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test function with string, number, and identifier arguments (no nesting)
        // Using simple values instead of var() to avoid nested function detection
        let source = "transform: translate3d(10px, 20%, 5em);";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let result = FunctionNode::from_node(call_node, source, None);
            
            assert!(result.is_some(), "Expected valid FunctionNode for mixed arguments");
            
            if let Some(func) = result {
                assert_eq!(func.function_name, "translate3d");
                assert_eq!(func.argument_count(), 3);
            }
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_css_var_function_rejected() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test that var() functions are also rejected as nested functions
        let source = "transform: translate3d(10px, 20%, var(--offset));";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let mut diagnostics = Vec::new();
            let result = FunctionNode::from_node(call_node, source, Some(&mut diagnostics));
            
            // This should be rejected because var() is a nested function call
            assert!(result.is_none(), "Expected None for function with var() argument");
            assert!(!diagnostics.is_empty(), "Expected diagnostic for nested var() function");
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_function_with_complex_expressions() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test function with complex mathematical expressions (no function nesting)
        let source = "width: calc(100vw - 2 * 20px + 5%);";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let result = FunctionNode::from_node(call_node, source, None);
            
            assert!(result.is_some(), "Expected valid FunctionNode for complex expressions");
            
            if let Some(func) = result {
                assert_eq!(func.function_name, "calc");
                assert!(func.argument_count() >= 1);
            }
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_function_with_parentheses_in_strings() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test function with parentheses inside string arguments (should not be confused with function calls)
        let source = "background-image: url(\"path/to/file(1).png\");";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let result = FunctionNode::from_node(call_node, source, None);
            
            assert!(result.is_some(), "Expected valid FunctionNode for string with parentheses");
            
            if let Some(func) = result {
                assert_eq!(func.function_name, "url");
                assert_eq!(func.argument_count(), 1);
                let arg_text = func.get_argument_text(0, source).unwrap();
                assert_eq!(arg_text, "\"path/to/file(1).png\"");
            }
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_function_with_empty_string_argument() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = "background-image: url(\"\");";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let result = FunctionNode::from_node(call_node, source, None);
            
            assert!(result.is_some(), "Expected valid FunctionNode for empty string");
            
            if let Some(func) = result {
                assert_eq!(func.function_name, "url");
                assert_eq!(func.argument_count(), 1);
                let arg_text = func.get_argument_text(0, source).unwrap();
                assert_eq!(arg_text, "\"\"");
            }
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_function_with_single_quotes() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = "background-image: url('image.png');";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let result = FunctionNode::from_node(call_node, source, None);
            
            assert!(result.is_some(), "Expected valid FunctionNode for single quotes");
            
            if let Some(func) = result {
                assert_eq!(func.function_name, "url");
                assert_eq!(func.argument_count(), 1);
                let arg_text = func.get_argument_text(0, source).unwrap();
                assert_eq!(arg_text, "'image.png'");
            }
        } else {
            panic!("Expected to find call_expression node");
        }
    }
}
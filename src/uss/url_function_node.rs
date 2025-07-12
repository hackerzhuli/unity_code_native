//! URL Function Node Validation
//!
//! Provides specialized validation for USS url() function calls.
//! Extracts and validates the URL string argument.

use tower_lsp::lsp_types::*;
use tree_sitter::Node;
use crate::language::tree_utils::node_to_range;
use crate::uss::function_node::FunctionNode;
use crate::uss::uss_utils::{convert_uss_string, UssStringError};
use crate::uss::constants::NODE_STRING_VALUE;

/// Represents a validated USS url() function call with extracted URL string
#[derive(Debug, Clone)]
pub struct UrlFunctionNode {
    /// The extracted URL string (without quotes and with escapes processed)
    pub url_string: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uss::parser::UssParser;
    use crate::language::tree_utils::find_node_by_type;
    use crate::uss::constants::NODE_CALL_EXPRESSION;

    #[test]
    fn test_valid_url_function() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = "background-image: url(\"image.png\");";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let mut diagnostics = Vec::new();
            let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics));
            
            assert!(result.is_some(), "Expected valid UrlFunctionNode");
            
            if let Some(url_func) = result {
                assert_eq!(url_func.url(), "image.png");
                assert!(!url_func.is_empty());
            }
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_url_function_with_single_quotes() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = "background-image: url('image.png');";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let mut diagnostics = Vec::new();
            let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics));
            
            assert!(result.is_some(), "Expected valid UrlFunctionNode with single quotes");
            
            if let Some(url_func) = result {
                assert_eq!(url_func.url(), "image.png");
            }
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_url_function_with_escapes() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = r#"background-image: url("path\\to\\file.png");"#;
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let mut diagnostics = Vec::new();
            let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics));
            
            assert!(result.is_some(), "Expected valid UrlFunctionNode with escapes");
            
            if let Some(url_func) = result {
                assert_eq!(url_func.url(), "path\\to\\file.png");
            }
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_url_function_empty_string() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = "background-image: url(\"\");";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let mut diagnostics = Vec::new();
            let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics));
            
            assert!(result.is_some(), "Expected valid UrlFunctionNode with empty string");
            
            if let Some(url_func) = result {
                assert_eq!(url_func.url(), "");
                assert!(url_func.is_empty());
            }
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_non_url_function_rejected() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = "color: rgb(255, 0, 0);";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let mut diagnostics = Vec::new();
            let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics));
            
            assert!(result.is_none(), "Expected None for rgb() function");
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_url_function_no_arguments() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = "background-image: url();";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let mut diagnostics = Vec::new();
            let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics));
            
            assert!(result.is_none(), "Expected None for url() with no arguments");
            assert!(!diagnostics.is_empty());
            assert!(diagnostics[0].message.contains("expects exactly 1 argument"));
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_url_function_too_many_arguments() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = "background-image: url(\"image.png\", \"fallback.png\");";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let mut diagnostics = Vec::new();
            let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics));
            
            assert!(result.is_none(), "Expected None for url() with too many arguments");
            assert!(!diagnostics.is_empty());
            assert!(diagnostics[0].message.contains("expects exactly 1 argument"));
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_url_function_non_string_argument() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = "background-image: url(123);";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let mut diagnostics = Vec::new();
            let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics));
            
            assert!(result.is_none(), "Expected None for url() with non-string argument");
            assert!(!diagnostics.is_empty());
            assert!(diagnostics[0].message.contains("expects a string argument"));
        } else {
            panic!("Expected to find call_expression node");
        }
    }

    #[test]
    fn test_url_function_with_hex_escapes() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test hex escape: \26 = & (ampersand)
        let source = r#"background-image: url("test\26 file.png");"#;
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let mut diagnostics = Vec::new();
            let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics));
            
            assert!(result.is_some(), "Expected valid UrlFunctionNode with hex escapes");
            
            if let Some(url_func) = result {
                assert_eq!(url_func.url(), "test&file.png");
            }
        } else {
            panic!("Expected to find call_expression node");
        }
    }
}


impl<'a> UrlFunctionNode {
    /// Attempts to create a UrlFunctionNode from a tree-sitter node
    /// 
    /// Returns Some(UrlFunctionNode) if:
    /// - The node represents a valid url() function call
    /// - Has exactly one string argument
    /// - The string can be successfully parsed
    /// 
    /// # Arguments
    /// * `node` - The tree-sitter node to validate
    /// * `content` - The source code content
    /// * `diagnostics` - Optional vector to collect diagnostics
    pub fn from_node(
        node: Node<'a>,
        content: &str,
        mut diagnostics: Option<&mut Vec<Diagnostic>>,
    ) -> Option<UrlFunctionNode> {
        // First validate as a general function
        let function_node = FunctionNode::from_node(node, content, diagnostics.as_deref_mut())?;
        
        // Check if it's a url function
        if !function_node.is_function("url") {
            return None;
        }
        
        // Check argument count
        if function_node.argument_count() != 1 {
            if let Some(diag) = diagnostics.as_deref_mut() {
                let range = node_to_range(node, content);
                diag.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("url-invalid-argument-count".to_string())),
                    source: Some("uss".to_string()),
                    message: format!("url() function expects exactly 1 argument, found {}", function_node.argument_count()),
                    ..Default::default()
                });
            }
            return None;
        }
        
        // Get the argument node
        let arg_node = function_node.argument_nodes[0];
        
        // Check if it's a string value
        if arg_node.kind() != NODE_STRING_VALUE {
            if let Some(diag) = diagnostics.as_deref_mut() {
                let range = node_to_range(arg_node, content);
                diag.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("url-invalid-argument-type".to_string())),
                    source: Some("uss".to_string()),
                    message: format!("url() function expects a string argument, found {}", arg_node.kind()),
                    ..Default::default()
                });
            }
            return None;
        }
        
        // Extract the raw string text
        let raw_string = arg_node.utf8_text(content.as_bytes()).ok()?;
        
        // Parse the string using uss_utils
        let url_string = match convert_uss_string(raw_string) {
            Ok(s) => s,
            Err(err) => {
                if let Some(diag) = diagnostics.as_deref_mut() {
                    let range = node_to_range(arg_node, content);
                    diag.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("url-string-parse-error".to_string())),
                        source: Some("uss".to_string()),
                        message: format!("Failed to parse URL string: {}", err.message),
                        ..Default::default()
                    });
                }
                return None;
            }
        };
        
        Some(UrlFunctionNode {
            url_string,
        })
    }
    
    /// Get the extracted URL string
    pub fn url(&self) -> &str {
        &self.url_string
    }
    
    /// Check if the URL is empty
    pub fn is_empty(&self) -> bool {
        self.url_string.is_empty()
    }
    

}
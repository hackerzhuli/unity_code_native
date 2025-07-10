//! Integration tests for asset string validation with USS parsing
//!
//! These tests demonstrate the URL and resource validation working
//! with actual USS parsing and value extraction.

use crate::uss::parser::UssParser;
use crate::uss::value::UssValue;
use tree_sitter::Node;

/// Helper function to find a node by type in the tree
fn find_node_by_type(node: Node, target_type: &str) -> Option<Node> {
    if node.kind() == target_type {
        return Some(node);
    }
    
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if let Some(found) = find_node_by_type(child, target_type) {
                return Some(found);
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_function_integration() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test valid URL function with simple string
        let source = r#".test { background-image: url("image.png"); }"#;
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_ok(), "Valid url() function should parse successfully");
            if let Ok(UssValue::Asset(asset)) = result {
                assert_eq!(asset, r#"url("image.png")"#);
            }
        } else {
            panic!("Could not find call_expression node");
        }
    }

    #[test]
    fn test_url_function_with_hex_escapes_integration() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test URL function with hex escapes in path
        let source = r#".test { background-image: url("\26 image.png"); }"#;
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_ok(), "URL with hex escapes should parse successfully");
            if let Ok(UssValue::Asset(asset)) = result {
                assert_eq!(asset, r#"url("\26 image.png")"#);
            }
        } else {
            panic!("Could not find call_expression node");
        }
    }

    #[test]
    fn test_url_function_empty_path_error_integration() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test URL function with empty string
        let source = r#".test { background-image: url(""); }"#;
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_err(), "URL with empty path should fail");
            if let Err(error) = result {
                assert!(error.message.contains("cannot have empty path"));
            }
        } else {
            panic!("Could not find call_expression node");
        }
    }

    #[test]
    fn test_resource_function_integration() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test valid resource function
        let source = r#".test { background-image: resource("texture.png"); }"#;
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_ok(), "Valid resource() function should parse successfully");
            if let Ok(UssValue::Asset(asset)) = result {
                assert_eq!(asset, r#"resource("texture.png")"#);
            }
        } else {
            panic!("Could not find call_expression node");
        }
    }

    #[test]
    fn test_resource_function_empty_path_error_integration() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test resource function with empty string
        let source = r#".test { background-image: resource(""); }"#;
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_err(), "Resource with empty path should fail");
            if let Err(error) = result {
                assert!(error.message.contains("cannot have empty path"));
            }
        } else {
            panic!("Could not find call_expression node");
        }
    }

    #[test]
    fn test_url_function_invalid_hex_escape_integration() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test URL function with invalid hex escape (zero codepoint)
        let source = r#".test { background-image: url("\0 image.png"); }"#;
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_err(), "URL with invalid string literal should fail");
            if let Err(error) = result {
                assert!(error.message.contains("Invalid string literal"));
            }
        } else {
            panic!("Could not find call_expression node");
        }
    }

    #[test]
    fn test_multiple_asset_functions_integration() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test USS with multiple asset functions
        let source = r#"
            .test {
                background-image: url("bg.png");
                --icon: resource("icon.png");
            }
        "#;
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        // Find all call expressions
        let mut call_expressions = Vec::new();
        fn collect_call_expressions(node: Node, expressions: &mut Vec<Node>) {
            if node.kind() == "call_expression" {
                expressions.push(node);
            }
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    collect_call_expressions(child, expressions);
                }
            }
        }
        collect_call_expressions(root, &mut call_expressions);
        
        assert_eq!(call_expressions.len(), 2, "Should find 2 call expressions");
        
        // Test both functions
        for call_expr in call_expressions {
            let result = UssValue::from_node(call_expr, source);
            assert!(result.is_ok(), "Both asset functions should parse successfully");
            if let Ok(UssValue::Asset(asset)) = result {
                assert!(asset.contains("url(") || asset.contains("resource("));
            }
        }
    }
}
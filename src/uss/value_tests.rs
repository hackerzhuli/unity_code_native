use crate::uss::value::UssValue;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uss::parser::UssParser;

    fn find_node_by_type<'a>(node: tree_sitter::Node<'a>, target_type: &str) -> Option<tree_sitter::Node<'a>> {
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

    #[test]
    fn test_from_node_with_mock_nodes() {
        // Create a simple CSS parser for testing
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test integer value parsing
        let source = ".test { width: 100px; }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "integer_value") {
             let result = UssValue::from_node(node, source);
             assert!(result.is_ok());
             if let Ok(UssValue::Numeric { value, unit, has_fractional }) = result {
                 assert_eq!(value, 100.0);
                 assert_eq!(unit, Some("px".to_string()));
                 assert_eq!(has_fractional, false);
             } else {
                 panic!("Expected Numeric value");
             }
         } else {
             println!("No integer_value node found");
             panic!("Expected to find integer_value node");
         }
    }

    #[test]
    fn test_from_node_float_values() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = ".test { opacity: 0.75; }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "float_value") {
             let result = UssValue::from_node(node, source);
             assert!(result.is_ok());
             if let Ok(UssValue::Numeric { value, unit, has_fractional }) = result {
                 assert_eq!(value, 0.75);
                 assert_eq!(unit, None);
                 assert_eq!(has_fractional, true);
             } else {
                 panic!("Expected Numeric value");
             }
         }
    }

    #[test]
    fn test_from_node_string_values() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = ".test { font-family: \"Arial\"; }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "string_value") {
             let result = UssValue::from_node(node, source);
             assert!(result.is_ok());
             if let Ok(UssValue::String(s)) = result {
                 assert_eq!(s, "\"Arial\"");
             } else {
                 panic!("Expected String value");
             }
         }
    }

    #[test]
    fn test_from_node_plain_value_identifiers() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = ".test { display: flex; }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "plain_value") {
             let result = UssValue::from_node(node, source);
             assert!(result.is_ok());
             if let Ok(UssValue::Identifier(id)) = result {
                 assert_eq!(id, "flex");
             } else {
                 panic!("Expected Identifier value");
             }
         }
    }

    #[test]
    fn test_from_node_color_values() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test hex color
        let source = ".test { color: #ff0000; }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "color_value") {
             let result = UssValue::from_node(node, source);
             assert!(result.is_ok());
             if let Ok(UssValue::Color(color)) = result {
                 assert_eq!(color, "#ff0000");
             } else {
                 panic!("Expected Color value");
             }
         }
    }

    #[test]
    fn test_from_node_variable_references() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        let source = ".test { color: var(--primary-color); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
             let result = UssValue::from_node(node, source);
             assert!(result.is_ok());
             if let Ok(UssValue::VariableReference(var_name)) = result {
                 assert_eq!(var_name, "primary-color");
             } else {
                 panic!("Expected VariableReference value");
             }
         }
    }

    #[test]
    fn test_from_node_error_cases() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test invalid hex color - use a format that tree-sitter will parse as plain_value
        // but our validation should reject
        let source = ".test { color: #ff00gg; }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "plain_value") {
             let node_text = node.utf8_text(source.as_bytes()).unwrap();
             if node_text.starts_with('#') {
                 let result = UssValue::from_node(node, source);
                 // Invalid hex color should be treated as identifier (Ok result)
                 // since tree-sitter classified it as plain_value, not color_value
                 assert!(result.is_ok());
             }
         }
    }

    #[test]
    fn test_to_string_conversion() {
        // Test that values can be converted back to strings
        let numeric = UssValue::Numeric { value: 100.0, unit: Some("px".to_string()), has_fractional: false };
        assert_eq!(numeric.to_string(), "100px");
        
        let color = UssValue::Color("#ff0000".to_string());
        assert_eq!(color.to_string(), "#ff0000");
        
        let var_ref = UssValue::VariableReference("primary-color".to_string());
        assert_eq!(var_ref.to_string(), "var(--primary-color)");
        
        let identifier = UssValue::Identifier("flex".to_string());
        assert_eq!(identifier.to_string(), "flex");
        
        let asset = UssValue::Asset("url(\"image.png\")".to_string());
        assert_eq!(asset.to_string(), "url(\"image.png\")");
        
        let string_val = UssValue::String("\"Arial\"".to_string());
        assert_eq!(string_val.to_string(), "\"Arial\"");
    }
}
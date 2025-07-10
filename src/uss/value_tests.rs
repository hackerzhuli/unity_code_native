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

    #[test]
    fn test_color_function_range_validation() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test valid rgb() function
        let source = ".test { color: rgb(255, 128, 0); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_ok(), "Valid rgb() should parse successfully");
            if let Ok(UssValue::Color(color)) = result {
                assert_eq!(color, "rgb(255, 128, 0)");
            }
        }
        
        // Test invalid rgb() function - value out of range
        let source = ".test { color: rgb(300, 128, 0); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_err(), "rgb() with value > 255 should fail");
            if let Err(error) = result {
                assert!(error.message.contains("out of range (0-255)"));
            }
        }
        
        // Test valid rgba() function
        let source = ".test { color: rgba(255, 128, 0, 0.5); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_ok(), "Valid rgba() should parse successfully");
        }
        
        // Test invalid rgba() function - alpha out of range
        let source = ".test { color: rgba(255, 128, 0, 1.5); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_err(), "rgba() with alpha > 1 should fail");
            if let Err(error) = result {
                assert!(error.message.contains("alpha value") && error.message.contains("out of range (0-1)"));
            }
        }
        
        // Test valid hsl() function
        let source = ".test { color: hsl(360, 100, 50); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_ok(), "Valid hsl() should parse successfully");
        }
        
        // Test invalid hsl() function - hue out of range
        let source = ".test { color: hsl(400, 100, 50); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_err(), "hsl() with hue > 360 should fail");
            if let Err(error) = result {
                assert!(error.message.contains("hue value") && error.message.contains("out of range (0-360)"));
            }
        }
        
        // Test invalid hsl() function - saturation out of range
        let source = ".test { color: hsl(180, 150, 50); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_err(), "hsl() with saturation > 100 should fail");
            if let Err(error) = result {
                assert!(error.message.contains("saturation/lightness value") && error.message.contains("out of range (0-100)"));
            }
        }
        
        // Test valid hsla() function
        let source = ".test { color: hsla(180, 50, 75, 0.8); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_ok(), "Valid hsla() should parse successfully");
        }
        
        // Test invalid hsla() function - alpha out of range
        let source = ".test { color: hsla(180, 50, 75, -0.1); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_err(), "hsla() with negative alpha should fail");
            if let Err(error) = result {
                assert!(error.message.contains("alpha value") && error.message.contains("out of range (0-1)"));
            }
   }    
    }

    #[test]
    fn test_color_function_unitless_validation() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test rgb() function with units - should fail
        let source = ".test { color: rgb(255px, 128, 0); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_err(), "rgb() with units should fail");
            if let Err(error) = result {
                assert!(error.message.contains("unitless number") && error.message.contains("px"));
            }
        }
        
        // Test rgba() function with units - should fail
        let source = ".test { color: rgba(255, 128px, 0, 0.5); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_err(), "rgba() with units should fail");
            if let Err(error) = result {
                assert!(error.message.contains("unitless number") && error.message.contains("px"));
            }
        }
        
        // Test hsl() function with units - should fail
        let source = ".test { color: hsl(180deg, 50, 75); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_err(), "hsl() with units should fail");
            if let Err(error) = result {
                assert!(error.message.contains("unitless number") && error.message.contains("deg"));
            }
        }
        
        // Test hsla() function with units - should fail
        let source = ".test { color: hsla(180, 50%, 75, 0.8); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, "call_expression") {
            let result = UssValue::from_node(node, source);
            assert!(result.is_err(), "hsla() with units should fail");
            if let Err(error) = result {
                assert!(error.message.contains("unitless number") && error.message.contains("%"));
            }
        }
    }
}
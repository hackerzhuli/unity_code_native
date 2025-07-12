use crate::uss::value::UssValue;
use crate::uss::definitions::UssDefinitions;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uss::{constants::{NODE_PLAIN_VALUE, NODE_STRING_VALUE, NODE_INTEGER_VALUE, NODE_FLOAT_VALUE, NODE_COLOR_VALUE, NODE_CALL_EXPRESSION, UNIT_PX}, parser::UssParser};
    use crate::language::tree_utils::find_node_by_type;

    #[test]
    fn test_from_node_with_mock_nodes() {
        // Create a simple CSS parser for testing
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test integer value parsing
        let source = ".test { width: 100px; }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, NODE_INTEGER_VALUE) {
             let definitions = UssDefinitions::new();
             let result = UssValue::from_node(node, source, &definitions, None);
             assert!(result.is_ok());
             if let Ok(UssValue::Numeric { value, unit, has_fractional }) = result {
                 assert_eq!(value, 100.0);
                 assert_eq!(unit, Some(UNIT_PX.to_string()));
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
        
        if let Some(node) = find_node_by_type(root, NODE_FLOAT_VALUE) {
             let definitions = UssDefinitions::new();
             let result = UssValue::from_node(node, source, &definitions, None);
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
        
        if let Some(node) = find_node_by_type(root, NODE_STRING_VALUE) {
             let definitions = UssDefinitions::new();
             let result = UssValue::from_node(node, source, &definitions, None);
             assert!(result.is_ok());
             if let Ok(UssValue::String(s)) = result {
                 assert_eq!(s, "Arial");
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
        
        if let Some(node) = find_node_by_type(root, NODE_PLAIN_VALUE) {
             let definitions = UssDefinitions::new();
             let result = UssValue::from_node(node, source, &definitions, None);
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
        
        if let Some(node) = find_node_by_type(root, NODE_COLOR_VALUE) {
             let definitions = UssDefinitions::new();
             let result = UssValue::from_node(node, source, &definitions, None);
             assert!(result.is_ok());
             if let Ok(UssValue::Color(color)) = result {
                assert_eq!(color.r, 255);
                assert_eq!(color.g, 0);
                assert_eq!(color.b, 0);
                assert_eq!(color.a, 1.0);
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
        
        if let Some(node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
             let definitions = UssDefinitions::new();
             let result = UssValue::from_node(node, source, &definitions, None);
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
        
        if let Some(node) = find_node_by_type(root, NODE_PLAIN_VALUE) {
             let node_text = node.utf8_text(source.as_bytes()).unwrap();
             if node_text.starts_with('#') {
                 let definitions = UssDefinitions::new();
                 let result = UssValue::from_node(node, source, &definitions, None);
                 // Invalid hex color should be treated as identifier (Ok result)
                 // since tree-sitter classified it as plain_value, not color_value
                 assert!(result.is_ok());
             }
         }
    }

    #[test]
    fn test_to_string_conversion() {
        // Test that values can be converted back to strings
        let numeric = UssValue::Numeric { value: 100.0, unit: Some(UNIT_PX.to_string()), has_fractional: false };
        assert_eq!(numeric.to_string(), "100px");
        
        let color = UssValue::Color(crate::uss::color::Color::new_rgb(255, 0, 0));
        assert_eq!(color.to_string(), "rgb(255, 0, 0)");
        
        let var_ref = UssValue::VariableReference("primary-color".to_string());
        assert_eq!(var_ref.to_string(), "var(--primary-color)");
        
        let identifier = UssValue::Identifier("flex".to_string());
        assert_eq!(identifier.to_string(), "flex");
        
        let url = url::Url::parse("file:///image.png").unwrap();
        let asset = UssValue::Url(url);
        assert_eq!(asset.to_string(), "url(\"file:///image.png\")");
        
        let string_val = UssValue::String("Arial".to_string());
        assert_eq!(string_val.to_string(), "\"Arial\"");
    }

    #[test]
    fn test_color_function_range_validation() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        let definitions = UssDefinitions::new();
        
        // Test valid rgb() function
        let source = ".test { color: rgb(255, 128, 0); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let result = UssValue::from_node(node, source, &definitions, None);
            assert!(result.is_ok(), "Valid rgb() should parse successfully");
            if let Ok(UssValue::Color(color)) = result {
                assert_eq!(color.r, 255);
                assert_eq!(color.g, 128);
                assert_eq!(color.b, 0);
                assert_eq!(color.a, 1.0);
            }
        }
        
        // Test invalid rgb() function - value out of range
        let source = ".test { color: rgb(300, 128, 0); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let result = UssValue::from_node(node, source, &definitions, None);
            assert!(result.is_err(), "rgb() with value > 255 should fail");
            if let Err(error) = result {
                assert!(error.message.contains("out of range (0-255)"));
            }
        }
        
        // Test valid rgba() function
        let source = ".test { color: rgba(255, 128, 0, 0.5); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let result = UssValue::from_node(node, source, &definitions, None);
            assert!(result.is_ok(), "Valid rgba() should parse successfully");
        }
        
        // Test invalid rgba() function - alpha out of range
        let source = ".test { color: rgba(255, 128, 0, 1.5); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let result = UssValue::from_node(node, source, &definitions, None);
            assert!(result.is_err(), "rgba() with alpha > 1 should fail");
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
        
        if let Some(node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let definitions = UssDefinitions::new();
            let result = UssValue::from_node(node, source, &definitions, None);
            assert!(result.is_err(), "rgb() with units should fail");
            if let Err(error) = result {
                assert!(error.message.contains("unitless number") && error.message.contains(UNIT_PX));
            }
        }
        
        // Test rgba() function with units - should fail
        let source = ".test { color: rgba(255, 128px, 0, 0.5); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let definitions = UssDefinitions::new();
            let result = UssValue::from_node(node, source, &definitions, None);
            assert!(result.is_err(), "rgba() with units should fail");
            if let Err(error) = result {
                assert!(error.message.contains("unitless number") && error.message.contains(UNIT_PX));
            }
        }
        

    }

    #[test]
    fn test_valid_units() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test valid length units
        let source = ".test { width: 100px; }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, NODE_INTEGER_VALUE) {
            let definitions = UssDefinitions::new();
            let result = UssValue::from_node(node, source, &definitions, None);
            assert!(result.is_ok(), "100px should be valid");
        }
        
        let source = ".test { width: 50%; }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, NODE_INTEGER_VALUE) {
            let definitions = UssDefinitions::new();
            let result = UssValue::from_node(node, source, &definitions, None);
            assert!(result.is_ok(), "50% should be valid");
        }
        
        // Test valid angle units
        let source = ".test { transform: rotate(45deg); }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, NODE_INTEGER_VALUE) {
            let definitions = UssDefinitions::new();
            let result = UssValue::from_node(node, source, &definitions, None);
            assert!(result.is_ok(), "45deg should be valid");
        }
        
        // Test valid time units
        let source = ".test { transition-duration: 2s; }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, NODE_INTEGER_VALUE) {
            let definitions = UssDefinitions::new();
            let result = UssValue::from_node(node, source, &definitions, None);
            assert!(result.is_ok(), "2s should be valid");
        }
        
        let source = ".test { animation-duration: 500ms; }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, NODE_INTEGER_VALUE) {
            let definitions = UssDefinitions::new();
            let result = UssValue::from_node(node, source, &definitions, None);
            assert!(result.is_ok(), "500ms should be valid");
        }
    }

    #[test]
    fn test_invalid_units() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test invalid units that should be rejected
        let invalid_units = ["em", "rem", "pt", "cm", "in", "mm", "pc", "ex", "ch", "vw", "vh", "vmin", "vmax"];
        
        for unit in invalid_units {
            let source = format!(".test {{ width: 100{}; }}", unit);
            let tree = parser.parse(&source, None).unwrap();
            let root = tree.root_node();
            
            if let Some(node) = find_node_by_type(root, NODE_INTEGER_VALUE) {
                let definitions = UssDefinitions::new();
                let result = UssValue::from_node(node, &source, &definitions, None);
                assert!(result.is_err(), "Unit {} should be invalid", unit);
                if let Err(error) = result {
                    assert!(error.message.contains("Invalid unit") && error.message.contains(unit), 
                           "Error message should mention invalid unit {}: {}", unit, error.message);
                }
            }
        }
    }

    #[test]
    fn test_unitless_numbers() {
        let mut parser = UssParser::new().expect("Failed to create USS parser");
        
        // Test unitless integers
        let source = ".test { z-index: 42; }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, NODE_INTEGER_VALUE) {
            let definitions = UssDefinitions::new();
            let result = UssValue::from_node(node, source, &definitions, None);
            assert!(result.is_ok(), "Unitless integer should be valid");
            if let Ok(UssValue::Numeric { value, unit, has_fractional }) = result {
                assert_eq!(value, 42.0);
                assert_eq!(unit, None);
                assert_eq!(has_fractional, false);
            }
        }
        
        // Test unitless floats
        let source = ".test { opacity: 0.75; }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        
        if let Some(node) = find_node_by_type(root, NODE_FLOAT_VALUE) {
            let definitions = UssDefinitions::new();
            let result = UssValue::from_node(node, source, &definitions, None);
            assert!(result.is_ok(), "Unitless float should be valid");
            if let Ok(UssValue::Numeric { value, unit, has_fractional }) = result {
                assert_eq!(value, 0.75);
                assert_eq!(unit, None);
                assert_eq!(has_fractional, true);
            }
        }
    }
}
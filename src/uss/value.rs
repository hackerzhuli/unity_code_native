use tree_sitter::Node;

use crate::uss::definitions::UssDefinitions;


/// A concrete USS value that represents a single valid value in USS
#[derive(Debug, Clone, PartialEq)]
pub enum UssValue {
    /// Numeric values (numbers, lengths, angles, times) with optional unit and fractional indicator
    Numeric { value: f64, unit: Option<String>, has_fractional: bool },
    /// String literals
    String(String),
    /// Color values (hex, named colors, rgb functions)
    Color(String),
    /// Keyword values or property names
    Identifier(String),
    /// Asset references (url(), resource()) - kept as-is
    Asset(String),
    /// Variable references (var(--variable-name))
    VariableReference(String),
}

impl UssValue {
    /// Convert the UssValue back to a string representation
    pub fn to_string(&self) -> String {
        match self {
            UssValue::Numeric { value, unit, .. } => {
                if let Some(unit) = unit {
                    format!("{}{}", value, unit)
                } else {
                    value.to_string()
                }
            }
            UssValue::String(s) => s.clone(),
            UssValue::Color(c) => c.clone(),
            UssValue::Identifier(k) => k.clone(),
            UssValue::Asset(a) => a.clone(),
            UssValue::VariableReference(var_name) => format!("var(--{})", var_name),
        }
    }

    /// Validate that a color function has the expected number of numeric arguments
    /// 
    /// Expected structure: function_name(arg1, arg2, ..., argN)
    /// Where args_node contains: (, arg1, ,, arg2, ,, ..., ,, argN, )
    fn validate_color_function_args(node: Node, expected_arg_count: usize) -> bool {
        if let Some(args_node) = node.child(1) {
            // Calculate expected child count: ( + args + commas + )
            // For N arguments: 1 (open paren) + N (args) + (N-1) (commas) + 1 (close paren) = 2*N + 1
            let expected_child_count = 2 * expected_arg_count + 1;
            
            if args_node.child_count() == expected_child_count {
                let children: Vec<_> = (0..expected_child_count).map(|i| args_node.child(i)).collect();
                
                // Check opening parenthesis
                if let Some(Some(open)) = children.first() {
                    if open.kind() != "(" {
                        return false;
                    }
                } else {
                    return false;
                }
                
                // Check closing parenthesis
                if let Some(Some(close)) = children.last() {
                    if close.kind() != ")" {
                        return false;
                    }
                } else {
                    return false;
                }
                
                // Check arguments and commas
                for i in 0..expected_arg_count {
                    let arg_index = 1 + i * 2; // Arguments are at indices 1, 3, 5, ...
                    
                    if let Some(Some(arg_node)) = children.get(arg_index) {
                        if !matches!(arg_node.kind(), "integer_value" | "float_value") {
                            return false;
                        }
                    } else {
                        return false;
                    }
                    
                    // Check comma (except for the last argument)
                    if i < expected_arg_count - 1 {
                        let comma_index = 2 + i * 2; // Commas are at indices 2, 4, 6, ...
                        if let Some(Some(comma_node)) = children.get(comma_index) {
                            if comma_node.kind() != "," {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                }
                
                return true;
            }
        }
        false
    }

    /// Parse a USS value from a tree-sitter node
    /// 
    /// Returns None if:
    /// - There is any error detected, ideally all errors should be detected by this function
    pub fn from_node(node: Node, content: &str) -> Option<Self> {
        // Return None immediately if the node has parsing errors
        if node.has_error() {
            return None;
        }
        let node_kind = node.kind();
        let node_text = node.utf8_text(content.as_bytes()).ok()?;
        
        match node_kind {
            "integer_value" | "float_value" => {
                let has_fractional = node_kind == "float_value";
                
                // Check for unit child - must have 0 or 1 child, otherwise it's malformed
                let unit = match node.child_count() {
                    0 => None,
                    1 => {
                        let child = node.child(0).unwrap();
                        if child.kind() == "unit" {
                            child.utf8_text(content.as_bytes()).ok().map(|s| s.to_string())
                        } else {
                            return None; // Invalid child type
                        }
                    }
                    _ => return None, // More than 1 child is an error
                };
                
                // Tree-sitter provides the full text (e.g., "32px") in the parent node
                // and the unit (e.g., "px") as a separate child node
                // We need to extract just the numeric part
                let value_text = if let Some(unit_str) = &unit {
                    // Remove the unit suffix to get just the numeric part
                    &node_text[..node_text.len() - unit_str.len()]
                } else {
                    // No unit, the entire text is the numeric value
                    node_text
                };
                
                // Parse the numeric value
                value_text.parse::<f64>()
                    .ok()
                    .map(|value| UssValue::Numeric { value, unit, has_fractional })
            }
            "plain_value" => {
                // Handle plain value types - tree-sitter already classified valid colors as color_value
                // If we're here with a # prefix, it's an invalid color that should be treated as identifier
                Some(UssValue::Identifier(node_text.to_string()))
            }
            "string_value" => {
                Some(UssValue::String(node_text.to_string()))
            }
            "color_value" => {
                Some(UssValue::Color(node_text.to_string()))
            }
            "call_expression" => {
                // Handle function calls like url(), rgb(), var(), etc.
                // Structure: call_expression -> function_name + arguments
                let function_name_text = if let Some(function_name_node) = node.child(0) {
                    if function_name_node.has_error() {
                        return None;
                    }
                    function_name_node.utf8_text(content.as_bytes()).ok()?.to_string()
                } else {
                    return None;
                };
                
                match function_name_text.as_str() {
                    "var" => {
                        // Extract variable name from var(--variable-name)
                        // Structure: arguments -> "(" + plain_value + ")"
                        if let Some(args_node) = node.child(1) { // arguments node
                            // var() must have exactly 3 children: (, variable, )
                            if args_node.child_count() == 3 {
                                let open_paren = args_node.child(0);
                                let var_node = args_node.child(1);
                                let close_paren = args_node.child(2);
                                
                                if let (Some(open), Some(var), Some(close)) = (open_paren, var_node, close_paren) {
                                    if open.kind() == "(" && close.kind() == ")" && var.kind() == "plain_value" {
                                        let var_name_text = var.utf8_text(content.as_bytes()).ok()?;
                                        if var_name_text.starts_with("--") && var_name_text.len() > 2 {
                                            // Remove the -- prefix for internal storage
                                            let var_name = &var_name_text[2..];
                                            // Validate variable name contains only valid characters
                                            if var_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                                                return Some(UssValue::VariableReference(var_name.to_string()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        None
                    }
                    "url" | "resource" => {
                        // Validate that url/resource functions have exactly one string argument
                        if let Some(args_node) = node.child(1) {
                            // Should have exactly 3 children: (, string_value, )
                            if args_node.child_count() == 3 {
                                let open_paren = args_node.child(0);
                                let string_node = args_node.child(1);
                                let close_paren = args_node.child(2);
                                
                                if let (Some(open), Some(string), Some(close)) = (open_paren, string_node, close_paren) {
                                    if open.kind() == "(" && close.kind() == ")" && string.kind() == "string_value" {
                                        return Some(UssValue::Asset(node_text.to_string()));
                                    }
                                }
                            }
                        }
                        None
                    }
                    "rgb" | "hsl" => {
                        if Self::validate_color_function_args(node, 3) {
                            Some(UssValue::Color(node_text.to_string()))
                        } else {
                            None
                        }
                    }
                    "rgba" | "hsla" => {
                        if Self::validate_color_function_args(node, 4) {
                            Some(UssValue::Color(node_text.to_string()))
                        } else {
                            None
                        }
                    }
                    _ => {
                        // Unknown function - return None instead of treating as identifier
                        None
                    }
                }
            }
            _ => None,
        }
    }
    

}
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

    /// Parse a USS value from a tree-sitter node
    /// 
    /// Returns None if:
    /// - The node contains parsing errors (detected via node.has_error())
    /// - The node text cannot be extracted
    /// - The value format is invalid (e.g., malformed hex colors, invalid numeric values)
    /// - Required child nodes are missing for complex structures
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
                // Handle various plain value types - trust tree-sitter's classification
                if node_text.starts_with('#') {
                    // Hex color - validate format strictly
                    let hex_part = &node_text[1..];
                    if (hex_part.len() == 3 || hex_part.len() == 6) && 
                       hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
                        Some(UssValue::Color(node_text.to_string()))
                    } else {
                        // Invalid hex color format
                        None
                    }
                } else {
                    // Check if it's a color keyword
                    let definitions = UssDefinitions::new();
                    if definitions.is_valid_color_keyword(node_text) {
                        Some(UssValue::Color(node_text.to_string()))
                    } else if node_text.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                        // Could be a keyword or property name (allow alphanumeric, hyphens, underscores)
                        Some(UssValue::Identifier(node_text.to_string()))
                    } else {
                        // Contains invalid characters for an identifier
                        None
                    }
                }
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
                            if args_node.has_error() {
                                return None;
                            }
                            let mut cursor = args_node.walk();
                            for arg_child in args_node.children(&mut cursor) {
                                if arg_child.kind() == "plain_value" {
                                    if arg_child.has_error() {
                                        return None;
                                    }
                                    let var_name_text = arg_child.utf8_text(content.as_bytes()).ok()?;
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
                        None
                    }
                    "url" | "resource" => Some(UssValue::Asset(node_text.to_string())),
                    "rgb" | "rgba" | "hsl" | "hsla" => Some(UssValue::Color(node_text.to_string())),
                    _ => Some(UssValue::Identifier(node_text.to_string())),
                }
            }
            _ => None,
        }
    }
    

}
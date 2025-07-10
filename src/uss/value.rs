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
                let has_fractional = node_kind == "float_value" || node_text.contains('.');
                
                // Check if it has a unit child
                let mut unit = None;
                if node.child_count() > 0 {
                    for i in 0..node.child_count() {
                        if let Some(child) = node.child(i) {
                            if child.kind() == "unit" {
                                unit = Some(child.utf8_text(content.as_bytes()).ok()?.to_string());
                                break;
                            }
                        }
                    }
                }
                
                // Extract the numeric part from the full text
                let (value_str, _) = Self::extract_value_and_unit(node_text);
                if let Ok(value) = value_str.parse::<f64>() {
                    Some(UssValue::Numeric { value, unit, has_fractional })
                } else {
                    None
                }
            }
            "plain_value" => {
                // Handle various plain value types
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
                    // Try to parse as numeric value with optional unit
                    let (value_str, unit) = Self::extract_value_and_unit(&node_text);
                    if let Ok(value) = value_str.parse::<f64>() {
                        let has_fractional = value_str.contains('.');
                        Some(UssValue::Numeric { value, unit, has_fractional })
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
    
    /// Extract numeric value and unit from a string like "10px" -> ("10", Some("px"))
    pub fn extract_value_and_unit(text: &str) -> (&str, Option<String>) {
        // Find where the numeric part ends and unit begins
        let mut split_pos = 0;
        
        for (i, ch) in text.char_indices() {
            if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' {
                split_pos = i + ch.len_utf8();
            } else {
                break;
            }
        }
        
        if split_pos == text.len() {
            // No unit found
            (text, None)
        } else {
            // Split into value and unit parts
            let (value_part, unit_part) = text.split_at(split_pos);
            (value_part, Some(unit_part.to_string()))
        }
    }
}
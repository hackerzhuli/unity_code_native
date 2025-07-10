use tree_sitter::Node;

use crate::uss::definitions::UssDefinitions;

/// Error type for USS value parsing
#[derive(Debug, Clone, PartialEq)]
pub struct UssValueError {
    /// The node that caused the error
    pub node_kind: String,
    /// The text content of the problematic node
    pub node_text: String,
    /// The byte range of the node in the source
    pub byte_range: (usize, usize),
    /// Descriptive error message
    pub message: String,
}

impl UssValueError {
    fn new(node: Node, content: &str, message: String) -> Self {
        let node_text = node.utf8_text(content.as_bytes())
            .unwrap_or("<invalid utf8>")
            .to_string();
        
        Self {
            node_kind: node.kind().to_string(),
            node_text,
            byte_range: (node.start_byte(), node.end_byte()),
            message,
        }
    }
}

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
    /// Returns an error with specific details if:
    /// - There is any parsing error detected
    /// - The node structure is invalid
    /// - The content cannot be parsed as a valid USS value
    pub fn from_node(node: Node, content: &str) -> Result<Self, UssValueError> {
        // Return error immediately if the node has parsing errors
        if node.has_error() {
            return Err(UssValueError::new(node, content, "Node has parsing errors".to_string()));
        }
        let node_kind = node.kind();
        let node_text = node.utf8_text(content.as_bytes())
            .map_err(|_| UssValueError::new(node, content, "Invalid UTF-8 in node text".to_string()))?;
        
        match node_kind {
            "integer_value" | "float_value" => {
                let has_fractional = node_kind == "float_value";
                
                // Check for unit child - must have 0 or 1 child, otherwise it's malformed
                let unit = match node.child_count() {
                    0 => None,
                    1 => {
                        let child = node.child(0).unwrap();
                        if child.kind() == "unit" {
                            child.utf8_text(content.as_bytes())
                                .map_err(|_| UssValueError::new(child, content, "Invalid UTF-8 in unit text".to_string()))?
                                .to_string()
                                .into()
                        } else {
                            return Err(UssValueError::new(child, content, format!("Expected unit child, found {}", child.kind())));
                        }
                    }
                    _ => return Err(UssValueError::new(node, content, format!("Numeric value has {} children, expected 0 or 1", node.child_count()))),
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
                let value = value_text.parse::<f64>()
                    .map_err(|_| UssValueError::new(node, content, format!("Cannot parse '{}' as numeric value", value_text)))?;
                
                Ok(UssValue::Numeric { value, unit, has_fractional })
            }
            "plain_value" => {
                // Handle plain value types - tree-sitter already classified valid colors as color_value
                // If we're here with a # prefix, it's an invalid color that should be treated as identifier
                Ok(UssValue::Identifier(node_text.to_string()))
            }
            "string_value" => {
                Ok(UssValue::String(node_text.to_string()))
            }
            "color_value" => {
                Ok(UssValue::Color(node_text.to_string()))
            }
            "call_expression" => {
                // Handle function calls like url(), rgb(), var(), etc.
                // Structure: call_expression -> function_name + arguments
                let function_name_node = node.child(0)
                    .ok_or_else(|| UssValueError::new(node, content, "Call expression missing function name".to_string()))?;
                
                if function_name_node.has_error() {
                    return Err(UssValueError::new(function_name_node, content, "Function name has parsing errors".to_string()));
                }
                
                let function_name_text = function_name_node.utf8_text(content.as_bytes())
                    .map_err(|_| UssValueError::new(function_name_node, content, "Invalid UTF-8 in function name".to_string()))?
                    .to_string();
                
                match function_name_text.as_str() {
                    "var" => {
                        // Extract variable name from var(--variable-name)
                        // Structure: arguments -> "(" + plain_value + ")"
                        let args_node = node.child(1)
                            .ok_or_else(|| UssValueError::new(node, content, "var() function missing arguments".to_string()))?;
                        
                        // var() must have exactly 3 children: (, variable, )
                        if args_node.child_count() != 3 {
                            return Err(UssValueError::new(args_node, content, format!("var() arguments have {} children, expected 3", args_node.child_count())));
                        }
                        
                        let open_paren = args_node.child(0)
                            .ok_or_else(|| UssValueError::new(args_node, content, "var() missing opening parenthesis".to_string()))?;
                        let var_node = args_node.child(1)
                            .ok_or_else(|| UssValueError::new(args_node, content, "var() missing variable argument".to_string()))?;
                        let close_paren = args_node.child(2)
                            .ok_or_else(|| UssValueError::new(args_node, content, "var() missing closing parenthesis".to_string()))?;
                        
                        if open_paren.kind() != "(" {
                            return Err(UssValueError::new(open_paren, content, format!("Expected '(', found {}", open_paren.kind())));
                        }
                        if close_paren.kind() != ")" {
                            return Err(UssValueError::new(close_paren, content, format!("Expected ')', found {}", close_paren.kind())));
                        }
                        if var_node.kind() != "plain_value" {
                            return Err(UssValueError::new(var_node, content, format!("Expected plain_value for variable name, found {}", var_node.kind())));
                        }
                        
                        let var_name_text = var_node.utf8_text(content.as_bytes())
                            .map_err(|_| UssValueError::new(var_node, content, "Invalid UTF-8 in variable name".to_string()))?;
                        
                        if !var_name_text.starts_with("--") {
                            return Err(UssValueError::new(var_node, content, "Variable name must start with '--'".to_string()));
                        }
                        if var_name_text.len() <= 2 {
                            return Err(UssValueError::new(var_node, content, "Variable name cannot be empty after '--'".to_string()));
                        }
                        
                        // Remove the -- prefix for internal storage
                        let var_name = &var_name_text[2..];
                        // Validate variable name contains only valid characters
                        if !var_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                            return Err(UssValueError::new(var_node, content, format!("Invalid characters in variable name '{}'", var_name)));
                        }
                        
                        Ok(UssValue::VariableReference(var_name.to_string()))
                    }
                    "url" | "resource" => {
                        // Validate that url/resource functions have exactly one string argument
                        let args_node = node.child(1)
                            .ok_or_else(|| UssValueError::new(node, content, format!("{}() function missing arguments", function_name_text)))?;
                        
                        // Should have exactly 3 children: (, string_value, )
                        if args_node.child_count() != 3 {
                            return Err(UssValueError::new(args_node, content, format!("{}() arguments have {} children, expected 3", function_name_text, args_node.child_count())));
                        }
                        
                        let open_paren = args_node.child(0)
                            .ok_or_else(|| UssValueError::new(args_node, content, format!("{}() missing opening parenthesis", function_name_text)))?;
                        let string_node = args_node.child(1)
                            .ok_or_else(|| UssValueError::new(args_node, content, format!("{}() missing string argument", function_name_text)))?;
                        let close_paren = args_node.child(2)
                            .ok_or_else(|| UssValueError::new(args_node, content, format!("{}() missing closing parenthesis", function_name_text)))?;
                        
                        if open_paren.kind() != "(" {
                            return Err(UssValueError::new(open_paren, content, format!("Expected '(', found {}", open_paren.kind())));
                        }
                        if close_paren.kind() != ")" {
                            return Err(UssValueError::new(close_paren, content, format!("Expected ')', found {}", close_paren.kind())));
                        }
                        if string_node.kind() != "string_value" {
                            return Err(UssValueError::new(string_node, content, format!("Expected string_value for {}() argument, found {}", function_name_text, string_node.kind())));
                        }
                        
                        Ok(UssValue::Asset(node_text.to_string()))
                    }
                    "rgb" | "hsl" => {
                        if Self::validate_color_function_args(node, 3) {
                            Ok(UssValue::Color(node_text.to_string()))
                        } else {
                            Err(UssValueError::new(node, content, format!("Invalid arguments for {}() function, expected 3 numeric arguments", function_name_text)))
                        }
                    }
                    "rgba" | "hsla" => {
                        if Self::validate_color_function_args(node, 4) {
                            Ok(UssValue::Color(node_text.to_string()))
                        } else {
                            Err(UssValueError::new(node, content, format!("Invalid arguments for {}() function, expected 4 numeric arguments", function_name_text)))
                        }
                    }
                    _ => {
                        // Unknown function
                        Err(UssValueError::new(node, content, format!("Unknown function '{}'", function_name_text)))
                    }
                }
            }
            _ => Err(UssValueError::new(node, content, format!("Unsupported node type '{}'", node_kind))),
        }
    }
    

}
use tree_sitter::Node;
use url::Url;

use crate::language::asset_url::{validate_url};
use crate::uss::uss_utils::convert_uss_string;
use crate::uss::definitions::UssDefinitions;
use crate::uss::color::Color;



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
    /// Diagnostic severity level
    pub severity: tower_lsp::lsp_types::DiagnosticSeverity,
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
            severity: tower_lsp::lsp_types::DiagnosticSeverity::ERROR,
        }
    }
    
    fn new_with_severity(node: Node, content: &str, message: String, severity: tower_lsp::lsp_types::DiagnosticSeverity) -> Self {
        let node_text = node.utf8_text(content.as_bytes())
            .unwrap_or("<invalid utf8>")
            .to_string();
        
        Self {
            node_kind: node.kind().to_string(),
            node_text,
            byte_range: (node.start_byte(), node.end_byte()),
            message,
            severity,
        }
    }
}

/// A concrete USS value that represents a single valid value in USS
#[derive(Debug, Clone, PartialEq)]
pub enum UssValue {
    /// Numeric values (numbers, lengths, angles, times) with optional unit and fractional indicator
    Numeric { value: f64, unit: Option<String>, has_fractional: bool },
    /// String literals, content is value of the string
    String(String),
    /// Color value, note that USS doesn't support color functions like hls
    Color(Color),
    /// Keyword values or property names, content is the identifier
    Identifier(String),
    /// a url asset reference, from url(), content is the actual parsed url
    Url(Url),
    /// a resource asset reference, from resource(), content is the actual parsed url
    Resource(Url),
    /// Variable references (var(--variable-name)), content is the name of variable with -- removed
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
            UssValue::String(s) => format!("\"{}\"", s),
            UssValue::Color(color) => color.to_string(),
            UssValue::Identifier(k) => k.clone(),
            UssValue::Url(url) => format!("url(\"{}\")", url.as_str()),
            UssValue::Resource(url) => format!("resource(\"{}\")", url.as_str()),
            UssValue::VariableReference(var_name) => format!("var(--{})", var_name),
        }
    }

    /// Validate general function argument structure and extract argument nodes
    /// 
    /// Expected structure: function_name(arg1, arg2, ..., argN)
    /// Where args_node contains: (, arg1, ,, arg2, ,, ..., ,, argN, )
    /// 
    /// Returns a vector of argument nodes if structure is valid
    fn validate_function_args_structure<'a>(node: Node<'a>, content: &'a str) -> Result<Vec<Node<'a>>, UssValueError> {
        let args_node = node.child(1)
            .ok_or_else(|| UssValueError::new(node, content, "Function missing arguments".to_string()))?;
        
        if args_node.child_count() == 0 {
            return Err(UssValueError::new(args_node, content, "Function has no arguments".to_string()));
        }
        
        // Check if child count is odd (should be: ( + args + commas + ))
        if args_node.child_count() % 2 == 0 {
            return Err(UssValueError::new(args_node, content, "Invalid argument structure".to_string()));
        }
        
        let children: Vec<_> = (0..args_node.child_count()).map(|i| args_node.child(i)).collect();
        
        // Check opening parenthesis
        if let Some(Some(open)) = children.first() {
            if open.kind() != "(" {
                return Err(UssValueError::new(*open, content, "Expected opening parenthesis '('".to_string()));
            }
        } else {
            return Err(UssValueError::new(args_node, content, "Missing opening parenthesis".to_string()));
        }
        
        // Check closing parenthesis
        if let Some(Some(close)) = children.last() {
            if close.kind() != ")" {
                return Err(UssValueError::new(*close, content, "Expected closing parenthesis ')'".to_string()));
            }
        } else {
            return Err(UssValueError::new(args_node, content, "Missing closing parenthesis".to_string()));
        }
        
        let mut arg_nodes = Vec::new();
        
        // Extract argument nodes (skip first and last which are parentheses)
        for i in (1..children.len() - 1).step_by(2) {
            let arg_node = if let Some(Some(arg_node)) = children.get(i) {
                *arg_node
            } else {
                return Err(UssValueError::new(args_node, content, format!("Missing argument at position {}", (i + 1) / 2)));
            };
            
            arg_nodes.push(arg_node);
            
            // Check comma (except for the last argument)
            if i < children.len() - 2 {
                let comma_index = i + 1;
                if let Some(Some(comma_node)) = children.get(comma_index) {
                    if comma_node.kind() != "," {
                        return Err(UssValueError::new(*comma_node, content, "Expected comma separator".to_string()));
                    }
                } else {
                    return Err(UssValueError::new(args_node, content, format!("Missing comma after argument {}", (i + 1) / 2)));
                }
            }
        }
        
        Ok(arg_nodes)
    }
    
    /// Parse and validate color function arguments structure
    /// 
    /// Expected structure: function_name(arg1, arg2, ..., argN)
    /// Where args_node contains: (, arg1, ,, arg2, ,, ..., ,, argN, )
    /// 
    /// Returns a vector of parsed numeric values if all arguments are valid
    fn parse_color_function_args(node: Node, content: &str) -> Result<Vec<f32>, UssValueError> {
        let arg_nodes = Self::validate_function_args_structure(node, content)?;
        
        let mut parsed_args = Vec::new();
        
        for (i, arg_node) in arg_nodes.iter().enumerate() {
            if !matches!(arg_node.kind(), "integer_value" | "float_value") {
                return Err(UssValueError::new(*arg_node, content, format!("Argument {} must be a number, found {}", i + 1, arg_node.kind())));
            }
            
            // Check if the numeric value has a unit - color functions expect unitless numbers
            if arg_node.child_count() > 0 {
                let child = arg_node.child(0).unwrap();
                if child.kind() == "unit" {
                    let unit_text = child.utf8_text(content.as_bytes())
                        .map_err(|_| UssValueError::new(child, content, "Invalid UTF-8 in unit text".to_string()))?;
                    return Err(UssValueError::new(*arg_node, content, format!("Argument {} must be a unitless number, found number with unit '{}'", i + 1, unit_text)));
                }
            }
            
            // Parse the argument value
            let arg_text = arg_node.utf8_text(content.as_bytes())
                .map_err(|_| UssValueError::new(*arg_node, content, "Invalid UTF-8 in argument".to_string()))?;
            let arg_value = arg_text.parse::<f32>()
                .map_err(|_| UssValueError::new(*arg_node, content, format!("Cannot parse '{}' as numeric value", arg_text)))?;
            
            parsed_args.push(arg_value);
        }
        
        Ok(parsed_args)
    }

    /// Parse a USS value from a tree-sitter node
    /// 
    /// Returns an error with specific details if:
    /// - There is any parsing error detected
    /// - The node structure is invalid
    /// - The content cannot be parsed as a valid USS value
    /// 
    /// ### Parameters
    /// 
    /// - `node`: The tree-sitter node to parse
    /// - `content`: The full content of the USS file
    /// - `definitions`: The USS definitions to use for validation
    /// - `source_url`: The URL of the USS file, note that this must be a absolute path and with project scheme, other schemes are not going to produce valid url values for uss
    pub fn from_node(node: Node, content: &str, definitions: &UssDefinitions, source_url: Option<&Url>) -> Result<Self, UssValueError> {
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
                
                // Validate unit if present
                if let Some(unit_str) = &unit {
                    if !definitions.is_valid_unit(unit_str) {
                        return Err(UssValueError::new(node, content, format!("Invalid unit '{}'. Valid units are: px, %, deg, rad, grad, turn, s, ms", unit_str)));
                    }
                }
                
                Ok(UssValue::Numeric { value, unit, has_fractional })
            }
            "plain_value" => {
                // Handle plain value types - tree-sitter already classified valid colors as color_value
                // If we're here with a # prefix, it's an invalid color that should be treated as identifier
                Ok(UssValue::Identifier(node_text.to_string()))
            }
            "string_value" => {
                // Convert USS string literal to actual string value
                let converted_string = convert_uss_string(node_text)
                    .map_err(|uss_err| UssValueError::new(node, content, format!("Invalid string literal: {}", uss_err.message)))?;
                
                Ok(UssValue::String(converted_string))
            }
            "color_value" => {
                // Parse hex color value using the centralized Color::from_hex method
                if let Some(color) = Color::from_hex(node_text) {
                    Ok(UssValue::Color(color))
                } else {
                    Err(UssValueError::new(node, content, format!("Invalid hex color format: {}", node_text)))
                }
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
                        let arg_nodes = Self::validate_function_args_structure(node, content)?;
                        
                        if arg_nodes.len() != 1 {
                            return Err(UssValueError::new(node, content, format!("var() function expects 1 argument, found {}", arg_nodes.len())));
                        }
                        
                        let var_node = arg_nodes[0];
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
                    "url" => {
                        // Validate that url function has exactly one string argument
                        let arg_nodes = Self::validate_function_args_structure(node, content)?;
                        
                        if arg_nodes.len() != 1 {
                            return Err(UssValueError::new(node, content, format!("url() function expects 1 argument, found {}", arg_nodes.len())));
                        }
                        
                        let string_node = arg_nodes[0];
                        if string_node.kind() != "string_value" {
                            return Err(UssValueError::new(string_node, content, format!("Expected string_value for url() argument, found {}", string_node.kind())));
                        }
                        
                        let string_text = string_node.utf8_text(content.as_bytes())
                            .map_err(|_| UssValueError::new(string_node, content, "Invalid UTF-8 in string argument".to_string()))?;
                        
                        // Convert USS string literal to actual string value
                        let converted_string = convert_uss_string(string_text)
                            .map_err(|uss_err| UssValueError::new(string_node, content, format!("Invalid string literal: {}", uss_err.message)))?;
                        
                        // Validate and parse the URL
                        let url = validate_url(&converted_string, source_url)
                            .map_err(|e| UssValueError::new_with_severity(string_node, content, e.message, e.severity))?;

                        Ok(UssValue::Url(url))
                    }
                    "resource" => {
                        // Validate that resource function has exactly one string argument
                        let arg_nodes = Self::validate_function_args_structure(node, content)?;
                        
                        if arg_nodes.len() != 1 {
                            return Err(UssValueError::new(node, content, format!("resource() function expects 1 argument, found {}", arg_nodes.len())));
                        }
                        
                        let string_node = arg_nodes[0];
                        if string_node.kind() != "string_value" {
                            return Err(UssValueError::new(string_node, content, format!("Expected string_value for resource() argument, found {}", string_node.kind())));
                        }
                        
                        let string_text = string_node.utf8_text(content.as_bytes())
                            .map_err(|_| UssValueError::new(string_node, content, "Invalid UTF-8 in string argument".to_string()))?;
                        
                        // Convert USS string literal to actual string value
                        let converted_string = convert_uss_string(string_text)
                            .map_err(|uss_err| UssValueError::new(string_node, content, format!("Invalid string literal: {}", uss_err.message)))?;
                        
                        // For resource functions, we use a fixed base URL since Unity's resource system
                        // doesn't resolve relative paths in the same way as regular URLs
                        let resource_base = Url::parse("project:///Assets/Resources/").ok();
                        let url = validate_url(&converted_string, resource_base.as_ref())
                            .map_err(|e| UssValueError::new_with_severity(string_node, content, e.message, e.severity))?;

                        Ok(UssValue::Resource(url))
                    }
                    "rgb" => {
                        let args = Self::parse_color_function_args(node, content)?;
                        if args.len() != 3 {
                            return Err(UssValueError::new(node, content, format!("rgb() function expects 3 arguments, found {}", args.len())));
                        }
                        // Validate RGB range (0-255)
                        for (i, &value) in args.iter().enumerate() {
                            if value < 0.0 || value > 255.0 {
                                return Err(UssValueError::new(node, content, format!("rgb() argument {} value {} is out of range (0-255)", i + 1, value)));
                            }
                        }
                        let color = Color::new_rgb(args[0] as u8, args[1] as u8, args[2] as u8);
                        Ok(UssValue::Color(color))
                    }

                    "rgba" => {
                        let args = Self::parse_color_function_args(node, content)?;
                        if args.len() != 4 {
                            return Err(UssValueError::new(node, content, format!("rgba() function expects 4 arguments, found {}", args.len())));
                        }
                        // Validate RGBA ranges: RGB (0-255), alpha (0-1)
                        for i in 0..3 {
                            if args[i] < 0.0 || args[i] > 255.0 {
                                return Err(UssValueError::new(node, content, format!("rgba() RGB component {} value {} is out of range (0-255)", i + 1, args[i])));
                            }
                        }
                        if args[3] < 0.0 || args[3] > 1.0 {
                            return Err(UssValueError::new(node, content, format!("rgba() alpha value {} is out of range (0-1)", args[3])));
                        }
                        let color = Color::new_rgba(args[0] as u8, args[1] as u8, args[2] as u8, args[3]);
                        Ok(UssValue::Color(color))
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

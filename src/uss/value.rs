use tree_sitter::Node;
use url::Url;

use crate::language::asset_url::{validate_url, validate_url_complete};
use crate::uss::uss_utils::convert_uss_string;
use crate::uss::definitions::UssDefinitions;
use crate::uss::color::Color;
use crate::uss::constants::*;
use crate::uss::function_node::FunctionNode;
use crate::uss::url_function_node::UrlFunctionNode;



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

    /// Parse and validate color function arguments using FunctionNode
    /// 
    /// Returns a vector of parsed numeric values if all arguments are valid
    fn parse_color_function_args(node: Node, content: &str) -> Result<Vec<f32>, UssValueError> {
        // Use FunctionNode to validate basic structure
        let function_node = FunctionNode::from_node(node, content, None)
            .ok_or_else(|| UssValueError::new(node, content, "Invalid function structure".to_string()))?;
        
        let mut parsed_args = Vec::new();
        
        for (i, arg_node) in function_node.argument_nodes.iter().enumerate() {
            if !matches!(arg_node.kind(), NODE_INTEGER_VALUE | NODE_FLOAT_VALUE) {
                return Err(UssValueError::new(*arg_node, content, format!("Argument {} must be a number, found {}", i + 1, arg_node.kind())));
            }
            
            // Check if the numeric value has a unit - color functions expect unitless numbers
            if arg_node.child_count() > 0 {
                let child = arg_node.child(0).unwrap();
                if child.kind() == NODE_UNIT {
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
            NODE_INTEGER_VALUE | NODE_FLOAT_VALUE => {
                let has_fractional = node_kind == NODE_FLOAT_VALUE;
                
                // Check for unit child - must have 0 or 1 child, otherwise it's malformed
                let unit = match node.child_count() {
                    0 => None,
                    1 => {
                        let child = node.child(0).unwrap();
                        if child.kind() == NODE_UNIT {
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
            NODE_PLAIN_VALUE => {
                // Handle plain value types - tree-sitter already classified valid colors as color_value
                // If we're here with a # prefix, it's an invalid color that should be treated as identifier
                Ok(UssValue::Identifier(node_text.to_string()))
            }
            NODE_STRING_VALUE => {
                // Convert USS string literal to actual string value
                let converted_string = convert_uss_string(node_text)
                    .map_err(|uss_err| UssValueError::new(node, content, format!("Invalid string literal: {}", uss_err.message)))?;
                
                Ok(UssValue::String(converted_string))
            }
            NODE_COLOR_VALUE => {
                // Parse hex color value using the centralized Color::from_hex method
                if let Some(color) = Color::from_hex(node_text) {
                    Ok(UssValue::Color(color))
                } else {
                    Err(UssValueError::new(node, content, format!("Invalid hex color format: {}", node_text)))
                }
            }
            NODE_CALL_EXPRESSION => {
                // Use FunctionNode to validate structure and extract function name and arguments
                let function_node = FunctionNode::from_node(node, content, None)
                    .ok_or_else(|| UssValueError::new(node, content, "Invalid function call structure".to_string()))?;
                
                match function_node.function_name.as_str() {
                    "var" => {
                        // var() function: var(custom-property-name, fallback-value?)
                        if function_node.argument_nodes.is_empty() || function_node.argument_nodes.len() > 2 {
                            return Err(UssValueError::new(node, content, "var() function expects 1 or 2 arguments".to_string()));
                        }
                        
                        // First argument must be an identifier (custom property name)
                        let var_name_node = function_node.argument_nodes[0];
                        if var_name_node.kind() != NODE_PLAIN_VALUE {
                            return Err(UssValueError::new(var_name_node, content, "First argument of var() must be a plain value".to_string()));
                        }
                        
                        let var_name_text = var_name_node.utf8_text(content.as_bytes())
                            .map_err(|_| UssValueError::new(var_name_node, content, "Invalid UTF-8 in variable name".to_string()))?;
                        
                        if !var_name_text.starts_with("--") {
                            return Err(UssValueError::new(var_name_node, content, "Variable name must start with '--'".to_string()));
                        }
                        
                        if var_name_text.len() <= 2 {
                            return Err(UssValueError::new(var_name_node, content, "Variable name cannot be empty after '--'".to_string()));
                        }
                        
                        // Remove the -- prefix for internal storage
                        let var_name = &var_name_text[2..];
                        // Validate variable name contains only valid characters
                        if !var_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                            return Err(UssValueError::new(var_name_node, content, format!("Invalid characters in variable name '{}'", var_name)));
                        }
                        
                        // Note: UssValue::VariableReference only stores the variable name
                        // Fallback value handling would need to be implemented at a higher level
                        // if needed in the future
                        
                        Ok(UssValue::VariableReference(var_name.to_string()))
                    }
                    "url" => {
                        // Use UrlFunctionNode for basic validation, then validate URL
                        let url_function = UrlFunctionNode::from_node(node, content, None, None, None, true)
                            .ok_or_else(|| UssValueError::new(node, content, "Invalid url() function structure".to_string()))?;
                        
                        // Validate and parse the URL
                        let result = validate_url_complete(&url_function.url_string, source_url, true)
                            .map_err(|e| UssValueError::new(node, content, e.message))?;
                        
                        Ok(UssValue::Url(result.url))
                    }
                    "resource" => {
                        // resource() function: resource("path/to/resource")
                        if function_node.argument_nodes.len() != 1 {
                            return Err(UssValueError::new(node, content, "resource() function expects exactly 1 argument".to_string()));
                        }
                        
                        let arg_node = function_node.argument_nodes[0];
                        if !matches!(arg_node.kind(), NODE_STRING_VALUE | NODE_PLAIN_VALUE) {
                            return Err(UssValueError::new(arg_node, content, "resource() argument must be a string or plain value".to_string()));
                        }
                        
                        let resource_string = if arg_node.kind() == NODE_STRING_VALUE {
                            // Convert USS string literal to actual string value
                            convert_uss_string(arg_node.utf8_text(content.as_bytes())
                                .map_err(|_| UssValueError::new(arg_node, content, "Invalid UTF-8 in resource string".to_string()))?)
                                .map_err(|uss_err| UssValueError::new(arg_node, content, format!("Invalid string literal: {}", uss_err.message)))?
                        } else {
                            // Plain value - use as-is
                            arg_node.utf8_text(content.as_bytes())
                                .map_err(|_| UssValueError::new(arg_node, content, "Invalid UTF-8 in resource path".to_string()))?
                                .to_string()
                        };
                        
                        // For resource functions, we use a fixed base URL since Unity's resource system
                        // doesn't resolve relative paths in the same way as regular URLs
                        let resource_base = Url::parse("project:///Assets/Resources/").ok();
                        let result = validate_url(&resource_string, resource_base.as_ref())
                            .map_err(|e| UssValueError::new(node, content, e.message))?;
                        
                        Ok(UssValue::Resource(result.url))
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
                        Err(UssValueError::new(node, content, format!("Unknown function '{}'", function_node.function_name)))
                    }
                }
            }
            _ => Err(UssValueError::new(node, content, format!("Unsupported node type '{}'", node_kind))),
        }
    }
}

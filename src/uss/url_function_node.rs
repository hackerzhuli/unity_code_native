//! URL Function Node Validation
//!
//! Provides specialized validation for USS url() function calls.
//! Extracts and validates the URL string argument.

use tower_lsp::lsp_types::*;
use tree_sitter::Node;
use crate::language::asset_url::validate_url;
use crate::language::tree_utils::node_to_range;
use crate::uss::function_node::FunctionNode;
use crate::uss::uss_utils::convert_uss_string;
use crate::uss::constants::{NODE_STRING_VALUE, NODE_PLAIN_VALUE};

/// Represents a URL found in USS code along with its location range
/// Used for future asset validation (file existence checks, etc.)
#[derive(Debug, Clone)]
pub struct UrlReference {
    /// The URL found in the USS code
    pub url: Url,
    /// The LSP range of the URL (for url() functions, this is just the argument range, not including the function name)
    pub range: Range,
}

/// Represents a validated USS url() function call with extracted URL string
#[derive(Debug, Clone)]
pub struct UrlFunctionNode<'a> {
    /// The extracted URL string (the actual value, without quotes and with escapes processed)
    pub url_string: String,
    /// The argument node containing the URL
    pub argument_node: Node<'a>,
}

impl<'a> UrlFunctionNode<'a> {
    /// Attempts to create a UrlFunctionNode from a tree-sitter node
    /// 
    /// Returns Some(UrlFunctionNode) if:
    /// - The node represents a valid url() function call
    /// - Has exactly one string argument
    /// - The string can be successfully parsed
    /// 
    /// # Arguments
    /// * `node` - The tree-sitter node to validate
    /// * `content` - The source code content
    /// * `source_url` - The URL of source code where the syntax tree is from, this is only needed for collection url references
    /// * `url_references` - Optional vector to collect URL references
    /// * `diagnostics` - Optional vector to collect diagnostics
    pub fn from_node(
        node: Node<'a>,
        content: &str,
        mut diagnostics: Option<&mut Vec<Diagnostic>>,
        source_url: Option<&Url>,
        url_references: Option<&mut Vec<UrlReference>>
    ) -> Option<UrlFunctionNode<'a>> {
        // First validate as a general function
        let function_node = FunctionNode::from_node(node, content, diagnostics.as_deref_mut())?;
        
        // Check if it's a url function
        if !function_node.is_function("url") {
            return None;
        }
        
        // Check argument count
        if function_node.argument_count() != 1 {
            if let Some(diag) = diagnostics.as_deref_mut() {
                let range = node_to_range(node, content);
                diag.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("url-invalid-argument-count".to_string())),
                    source: Some("uss".to_string()),
                    message: format!("url() function expects exactly 1 argument, found {}", function_node.argument_count()),
                    ..Default::default()
                });
            }
            return None;
        }
        
        // Get the argument node
        let arg_node = function_node.argument_nodes[0];
        
        // Handle both string values and plain values (identifiers)
        let url_string = match arg_node.kind() {
            NODE_STRING_VALUE => {
                // Extract the raw string text and parse it
                let raw_string = arg_node.utf8_text(content.as_bytes()).ok()?;
                match convert_uss_string(raw_string) {
                    Ok(s) => s,
                    Err(err) => {
                        if let Some(diag) = diagnostics.as_deref_mut() {
                            let range = node_to_range(arg_node, content);
                            diag.push(Diagnostic {
                                range,
                                severity: Some(DiagnosticSeverity::ERROR),
                                code: Some(NumberOrString::String("url-string-parse-error".to_string())),
                                source: Some("uss".to_string()),
                                message: format!("Failed to parse URL string: {}", err.message),
                                ..Default::default()
                            });
                        }
                        return None;
                    }
                }
            },
            NODE_PLAIN_VALUE => {
                // For plain values (identifiers), treat them as unquoted strings
                let raw_text = arg_node.utf8_text(content.as_bytes()).ok()?;
                // Process escapes for plain values by wrapping in quotes and calling convert_uss_string
                let quoted_text = format!("\"{}\"", raw_text);
                match convert_uss_string(&quoted_text) {
                    Ok(s) => s,
                    Err(err) => {
                        if let Some(diag) = diagnostics.as_deref_mut() {
                            let range = node_to_range(arg_node, content);
                            diag.push(Diagnostic {
                                range,
                                severity: Some(DiagnosticSeverity::ERROR),
                                code: Some(NumberOrString::String("url-plain-value-parse-error".to_string())),
                                source: Some("uss".to_string()),
                                message: format!("Failed to parse URL plain value: {}", err.message),
                                ..Default::default()
                            });
                        }
                        return None;
                    }
                }
            },
            _ => {
                // Invalid argument type
                if let Some(diag) = diagnostics.as_deref_mut() {
                    let range = node_to_range(arg_node, content);
                    diag.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("url-invalid-argument-type".to_string())),
                        source: Some("uss".to_string()),
                        message: format!("url() function expects a string or identifier argument, found {}", arg_node.kind()),
                        ..Default::default()
                    });
                }
                return None;
            }
        };

        if url_references.is_some(){
            if let Ok(validation_result) =  validate_url(url_string.as_str(), source_url) {
                let arg_range = node_to_range(arg_node, content);
                url_references.unwrap().push(UrlReference {
                    url: validation_result.url.clone(),
                    range: arg_range,
                });
            }
        }
        
        Some(UrlFunctionNode {
            url_string,
            argument_node: arg_node,
        })
    }
    
    /// Get the extracted URL string
    pub fn url(&self) -> &str {
        &self.url_string
    }
    
    /// Check if the URL is empty
    pub fn is_empty(&self) -> bool {
        self.url_string.is_empty()
    }
}
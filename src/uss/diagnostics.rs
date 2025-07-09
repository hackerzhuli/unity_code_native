//! USS Diagnostics
//!
//! Provides validation and error reporting for USS files.
//! Validates syntax, properties, values, and USS-specific rules.

use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Tree};
use crate::uss::definitions::UssDefinitions;

/// USS diagnostic analyzer
pub struct UssDiagnostics {
    /// USS language definitions
    definitions: UssDefinitions,
}

impl UssDiagnostics {
    /// Create a new diagnostics analyzer
    pub fn new() -> Self {
        Self {
            definitions: UssDefinitions::new(),
        }
    }
    
    /// Analyze USS syntax tree and generate diagnostics
    pub fn analyze(&self, tree: &Tree, content: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let root_node = tree.root_node();
        
        self.walk_node(root_node, content, &mut diagnostics);
        
        diagnostics
    }
    
    /// Recursively walk the syntax tree and validate nodes
    fn walk_node(&self, node: Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        // Check for syntax errors
        if node.has_error() {
            self.add_syntax_error(node, content, diagnostics);
        }
        
        match node.kind() {
            "rule_set" => self.validate_rule_set(node, content, diagnostics),
            "declaration" => self.validate_declaration(node, content, diagnostics),
            "pseudo_class_selector" => self.validate_pseudo_class(node, content, diagnostics),
            "at_rule" => self.validate_at_rule(node, content, diagnostics),
            "call_expression" => self.validate_function_arguments_wrapper(node, content, diagnostics),
            "color_value" => self.validate_color_value(node, content, diagnostics),
            _ => {}
        }
        
        // Recursively check children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.walk_node(child, content, diagnostics);
            }
        }
    }
    
    /// Add syntax error diagnostic
    fn add_syntax_error(&self, node: Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        let range = self.node_to_range(node, content);
        let text = node.utf8_text(content.as_bytes()).unwrap_or("<invalid>");
        
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("syntax-error".to_string())),
            source: Some("uss".to_string()),
            message: format!("Syntax error: {}", text),
            ..Default::default()
        });
    }
    
    /// Validate rule set for nested rules (not allowed in USS)
    fn validate_rule_set(&self, node: Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        // Check if this rule set is nested inside another rule set
        let mut parent = node.parent();
        while let Some(p) = parent {
            if p.kind() == "block" {
                if let Some(grandparent) = p.parent() {
                    if grandparent.kind() == "rule_set" {
                        let range = self.node_to_range(node, content);
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("nested-rules".to_string())),
                            source: Some("uss".to_string()),
                            message: "Nested rules are not supported in USS".to_string(),
                            ..Default::default()
                        });
                        break;
                    }
                }
            }
            parent = p.parent();
        }
    }
    
    /// Validate declaration (property-value pair)
    fn validate_declaration(&self, node: Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        if let Some(property_node) = node.child(0) {
            if property_node.kind() == "property_name" {
                let property_name = property_node.utf8_text(content.as_bytes()).unwrap_or("");
                
                // Check if property is valid
                if !self.definitions.is_valid_property(property_name) {
                    let range = self.node_to_range(property_node, content);
                    diagnostics.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("unknown-property".to_string())),
                        source: Some("uss".to_string()),
                        message: format!("Unknown property: {}", property_name),
                        ..Default::default()
                    });
                }
                
                // Validate property value
                if let Some(value_node) = node.child(2) { // Skip colon
                    self.validate_property_value(property_name, value_node, content, diagnostics);
                }
            }
        }
    }
    
    /// Validate pseudo-class selector
    fn validate_pseudo_class(&self, node: Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        let pseudo_class = node.utf8_text(content.as_bytes()).unwrap_or("");
        
        if !self.definitions.is_valid_pseudo_class(pseudo_class) {
            let range = self.node_to_range(node, content);
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("unknown-pseudo-class".to_string())),
                source: Some("uss".to_string()),
                message: format!("Unknown pseudo-class: {}", pseudo_class),
                ..Default::default()
            });
        }
    }
    
    /// Validate at-rule (only @import is supported)
    fn validate_at_rule(&self, node: Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        let at_rule_text = node.utf8_text(content.as_bytes()).unwrap_or("");
        
        // Extract the at-rule name (e.g., "@import" from "@import url(...)")
        let at_rule_name = if let Some(space_pos) = at_rule_text.find(' ') {
            &at_rule_text[..space_pos]
        } else {
            at_rule_text
        };
        
        if !self.definitions.is_valid_at_rule(at_rule_name) {
            let range = self.node_to_range(node, content);
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("unsupported-at-rule".to_string())),
                source: Some("uss".to_string()),
                message: format!("Unsupported at-rule '{}'. Only @import is supported in USS", at_rule_name),
                ..Default::default()
            });
        }
    }
    
    /// Validate function calls wrapper
    fn validate_function_arguments_wrapper(&self, node: Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        if let Some(function_name_node) = node.child(0) {
            let function_name = function_name_node.utf8_text(content.as_bytes()).unwrap_or("");
            
            if self.definitions.is_valid_function(function_name) {
                // Valid USS function
                self.validate_function_arguments(function_name, node, content, diagnostics);
            } else {
                let range = self.node_to_range(function_name_node, content);
                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("invalid-function".to_string())),
                    source: Some("uss".to_string()),
                    message: format!("Unsupported function: {}", function_name),
                    ..Default::default()
                });
            }
        }
    }
    
    /// Validate color values
    fn validate_color_value(&self, node: Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        let color_text = node.utf8_text(content.as_bytes()).unwrap_or("");
        
        if color_text.starts_with('#') {
            // Validate hex color format
            let hex_part = &color_text[1..];
            if hex_part.len() != 3 && hex_part.len() != 6 && hex_part.len() != 8 {
                let range = self.node_to_range(node, content);
                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("invalid-color".to_string())),
                    source: Some("uss".to_string()),
                    message: format!("Invalid hex color format: {}", color_text),
                    ..Default::default()
                });
            } else if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
                let range = self.node_to_range(node, content);
                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("invalid-color".to_string())),
                    source: Some("uss".to_string()),
                    message: format!("Invalid hex color characters: {}", color_text),
                    ..Default::default()
                });
            }
        }
    }
    
    /// Validate property value based on property type
    fn validate_property_value(&self, property_name: &str, value_node: Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        let value_text = value_node.utf8_text(content.as_bytes()).unwrap_or("");
        
        // Check if the value is valid for this specific property
        if let Some(valid_values) = self.definitions.get_valid_values_for_property(property_name) {
            if !self.definitions.is_valid_value_for_property(property_name, value_text) {
                let expected = valid_values.join(", ");
                self.add_invalid_value_diagnostic(value_node, content, property_name, &format!("Expected: {}", expected), diagnostics);
            }
        } else {
            // For color properties, validate color values
            if property_name.contains("color") && value_node.kind() == "plain_value" {
                if !self.definitions.is_valid_color_keyword(value_text) && !value_text.starts_with('#') {
                    self.add_invalid_value_diagnostic(value_node, content, property_name, "Expected: valid color keyword, hex color, or color function", diagnostics);
                }
            }
        }
    }
    
    /// Validate function arguments
    fn validate_function_arguments(&self, function_name: &str, node: Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        if let Some(args_node) = node.child(1) {
            match function_name {
                "resource" | "url" => {
                    // Should have exactly one string argument
                    let arg_count = args_node.child_count();
                    if arg_count == 0 {
                        let range = self.node_to_range(args_node, content);
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("missing-argument".to_string())),
                            source: Some("uss".to_string()),
                            message: format!("{}() function requires a string argument", function_name),
                            ..Default::default()
                        });
                    }
                }
                "var" => {
                    // Should have at least one argument (CSS variable name)
                    if args_node.child_count() == 0 {
                        let range = self.node_to_range(args_node, content);
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("missing-argument".to_string())),
                            source: Some("uss".to_string()),
                            message: "var() function requires a CSS variable name".to_string(),
                            ..Default::default()
                        });
                    }
                }
                "rgb" => {
                    // Should have exactly 3 arguments
                    let non_comma_children: Vec<_> = (0..args_node.child_count())
                        .filter_map(|i| args_node.child(i))
                        .filter(|child| child.kind() != ",")
                        .collect();
                    
                    if non_comma_children.len() != 3 {
                        let range = self.node_to_range(args_node, content);
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("invalid-argument-count".to_string())),
                            source: Some("uss".to_string()),
                            message: "rgb() function requires exactly 3 arguments (red, green, blue)".to_string(),
                            ..Default::default()
                        });
                    }
                }
                "rgba" => {
                    // Should have exactly 4 arguments
                    let non_comma_children: Vec<_> = (0..args_node.child_count())
                        .filter_map(|i| args_node.child(i))
                        .filter(|child| child.kind() != ",")
                        .collect();
                    
                    if non_comma_children.len() != 4 {
                        let range = self.node_to_range(args_node, content);
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("invalid-argument-count".to_string())),
                            source: Some("uss".to_string()),
                            message: "rgba() function requires exactly 4 arguments (red, green, blue, alpha)".to_string(),
                            ..Default::default()
                        });
                    }
                }
                _ => {}
            }
        }
    }
    
    /// Add invalid value diagnostic
    fn add_invalid_value_diagnostic(&self, value_node: Node, content: &str, property_name: &str, expected: &str, diagnostics: &mut Vec<Diagnostic>) {
        let range = self.node_to_range(value_node, content);
        let value_text = value_node.utf8_text(content.as_bytes()).unwrap_or("<invalid>");
        
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("invalid-value".to_string())),
            source: Some("uss".to_string()),
            message: format!("Invalid value '{}' for property '{}'. {}", value_text, property_name, expected),
            ..Default::default()
        });
    }
    
    /// Convert tree-sitter node to LSP range
    fn node_to_range(&self, node: Node, content: &str) -> Range {
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        
        let start_position = self.byte_to_position(start_byte, content);
        let end_position = self.byte_to_position(end_byte, content);
        
        Range {
            start: start_position,
            end: end_position,
        }
    }
    
    /// Convert byte offset to LSP position
    fn byte_to_position(&self, byte_offset: usize, content: &str) -> Position {
        let mut line = 0;
        let mut character = 0;
        
        for (i, ch) in content.char_indices() {
            if i >= byte_offset {
                break;
            }
            
            if ch == '\n' {
                line += 1;
                character = 0;
            } else {
                character += 1;
            }
        }
        
        Position {
            line: line as u32,
            character: character as u32,
        }
    }
}

impl Default for UssDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}
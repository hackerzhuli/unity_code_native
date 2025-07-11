//! USS Diagnostics
//!
//! Provides validation and error reporting for USS files.
//! Validates syntax, properties, values, and USS-specific rules.

use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Tree};
use crate::uss::definitions::UssDefinitions;
use crate::uss::value::UssValue;
use crate::uss::tree_printer;

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
        self.analyze_with_source_url(tree, content, None)
    }
    
    /// Analyze USS syntax tree and generate diagnostics with optional source URL
    pub fn analyze_with_source_url(&self, tree: &Tree, content: &str, source_url: Option<&str>) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let root_node = tree.root_node();
        
        self.walk_node_with_source_url(root_node, content, source_url, &mut diagnostics);
        
        diagnostics
    }
    
    /// Debug helper: Print the complete syntax tree to stdout
    /// Useful for understanding tree structure during development
    #[allow(dead_code)]
    pub fn debug_print_tree(&self, tree: &Tree, content: &str) {
        let root_node = tree.root_node();
        tree_printer::print_tree_to_stdout(root_node, content);
    }
    
    /// Recursively walk the syntax tree and validate nodes
    fn walk_node(&self, node: Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        self.walk_node_with_source_url(node, content, None, diagnostics);
    }
    
    /// Recursively walk the syntax tree and validate nodes with source URL
    fn walk_node_with_source_url(&self, node: Node, content: &str, source_url: Option<&str>, diagnostics: &mut Vec<Diagnostic>) {
        // Track the number of diagnostics before processing children
        let initial_diagnostic_count = diagnostics.len();
        
        // Check for syntax errors - only report for ERROR nodes directly, not for nodes that contain errors
        if node.kind() == "ERROR" {
            self.add_syntax_error(node, content, diagnostics);
        }
        
        // Recursively check children first to detect any child errors
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.walk_node_with_source_url(child, content, source_url, diagnostics);
            }
        }
        
        // Check if any diagnostics were added by children
        let child_diagnostics_added = diagnostics.len() > initial_diagnostic_count;
        
        match node.kind() {
            "rule_set" => self.validate_rule_set(node, content, diagnostics),
            "declaration" => {
                // Only validate declaration if no child diagnostics were generated
                // This prevents redundant error messages when child nodes (like invalid tokens,
                // syntax errors, or malformed values) have already reported issues.
                // For example, if a property value contains a syntax error, we don't want to
                // also report that the property itself is invalid - the child error is sufficient.
                if !child_diagnostics_added {
                    self.validate_declaration(node, content, diagnostics, source_url);
                }
            },
            "pseudo_class_selector" => self.validate_pseudo_class(node, content, diagnostics),
            "at_rule" => self.validate_at_rule(node, content, diagnostics),
            "call_expression" => {
                // Only validate function if no child diagnostics were generated
                if !child_diagnostics_added {
                    // First validate the function name itself
                    if let Some(function_name_node) = node.child_by_field_name("function") {
                        let function_name = function_name_node.utf8_text(content.as_bytes()).unwrap_or("");
                        
                        // Check if this is a supported USS function
                        if !self.definitions.is_valid_function(function_name) {
                            let range = self.node_to_range(function_name_node, content);
                            diagnostics.push(Diagnostic {
                                range,
                                severity: Some(DiagnosticSeverity::ERROR),
                                code: Some(NumberOrString::String("unsupported-function".to_string())),
                                source: Some("uss".to_string()),
                                message: format!("Function '{}' is not supported in USS. USS does not support calc() and other CSS functions.", function_name),
                                ..Default::default()
                            });
                        }
                    }
                    
                    // Then validate function arguments
                    self.validate_function_arguments_wrapper(node, content, diagnostics);
                }
            }
            "color_value" => {
                if !child_diagnostics_added {
                    self.validate_color_value(node, content, diagnostics);
                }
            },
            "integer_value" | "float_value" => {
                // Numeric values are handled when we encounter their unit children
            },
            "unit" => {
                // Check if this is a supported unit
                if let Some(parent) = node.parent() {
                    if parent.kind() == "integer_value" || parent.kind() == "float_value" {
                        let unit_text = node.utf8_text(content.as_bytes()).unwrap_or("");
                        
                        // Check against supported USS units
                        let valid_units = ["px", "%", "deg", "grad", "rad", "turn"];
                        if !valid_units.contains(&unit_text) {
                            self.add_invalid_unit_diagnostic(node, content, "", unit_text, "px, %, deg, grad, rad, turn", diagnostics);
                        }
                    }
                }
            },
            "plain_value" => {
                // Plain values will be validated by their expected type in validate_property_value
            },
            _ => {
                // Other node types don't need special handling
            }
        }
    }
    
    /// Add syntax error diagnostic
    fn add_syntax_error(&self, node: Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        // Try to find the most specific error location within the node
        let range = self.get_precise_error_range(node, content);
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
    
    /// Get a more precise error range for syntax errors
    fn get_precise_error_range(&self, node: Node, content: &str) -> Range {
        // If the node has errors, find the most specific error location
        if node.has_error() {
            // Look for ERROR nodes within this node
            let error_nodes = self.find_error_nodes(node);
            
            if !error_nodes.is_empty() {
                // Use the first ERROR node found (they should be small and specific)
                let error_node = error_nodes[0];
                let error_range = self.node_to_range(error_node, content);
                
                // Limit to single line if it spans multiple lines
                if error_range.end.line > error_range.start.line {
                    let line_end_position = self.find_line_end_position(error_range.start.line, content);
                    return Range {
                        start: error_range.start,
                        end: line_end_position,
                    };
                }
                return error_range;
            }
            
            // Look for MISSING nodes
            let missing_nodes = self.find_missing_nodes(node);
            if !missing_nodes.is_empty() {
                let missing_node = missing_nodes[0];
                return self.node_to_range(missing_node, content);
            }
        }
        
        // If this is an ERROR node itself, use its range directly
        if node.kind() == "ERROR" {
            return self.node_to_range(node, content);
        }
        
        // Fallback: limit the node range to a single line
        let node_range = self.node_to_range(node, content);
        let start_line = node_range.start.line;
        let end_line = node_range.end.line;
        
        // If the error spans multiple lines, limit it to the first line
        if end_line > start_line {
            let line_end_position = self.find_line_end_position(start_line, content);
            Range {
                start: node_range.start,
                end: line_end_position,
            }
        } else {
            node_range
        }
    }
    
    /// Find all ERROR nodes within a given node
    fn find_error_nodes<'a>(&self, node: Node<'a>) -> Vec<Node<'a>> {
        let mut error_nodes = Vec::new();
        let mut cursor = node.walk();
        
        if cursor.goto_first_child() {
            loop {
                let current_node = cursor.node();
                
                if current_node.kind() == "ERROR" {
                    error_nodes.push(current_node);
                }
                
                // Recursively search children
                if current_node.child_count() > 0 {
                    error_nodes.extend(self.find_error_nodes(current_node));
                }
                
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        
        error_nodes
    }
    
    /// Find all MISSING nodes within a given node
    fn find_missing_nodes<'a>(&self, node: Node<'a>) -> Vec<Node<'a>> {
        let mut missing_nodes = Vec::new();
        let mut cursor = node.walk();
        
        if cursor.goto_first_child() {
            loop {
                let current_node = cursor.node();
                
                if current_node.is_missing() {
                    missing_nodes.push(current_node);
                }
                
                // Recursively search children
                if current_node.child_count() > 0 {
                    missing_nodes.extend(self.find_missing_nodes(current_node));
                }
                
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        
        missing_nodes
    }
    

    
    /// Find the end position of a given line
    fn find_line_end_position(&self, line_number: u32, content: &str) -> Position {
        let mut current_line = 0;
        let mut character = 0;
        
        for ch in content.chars() {
            if current_line == line_number {
                if ch == '\n' {
                    break;
                }
                character += 1;
            } else if ch == '\n' {
                current_line += 1;
                character = 0;
            } else if current_line < line_number {
                character += 1;
            }
        }
        
        Position {
            line: line_number,
            character: character as u32,
        }
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
    fn validate_declaration(&self, node: Node, content: &str, diagnostics: &mut Vec<Diagnostic>, source_url: Option<&str>) {
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
                    return; // Don't validate values for unknown properties
                }
                
                // Check for missing semicolon by detecting colon within plain_value
                // This happens when parser treats "background-color: red\n    border-radius:10px" as one declaration
                let plain_values: Vec<_> = (0..node.child_count())
                    .filter_map(|i| node.child(i))
                    .filter(|child| child.kind() == "plain_value")
                    .collect();
                
                for plain_value in &plain_values {
                    let value_text = plain_value.utf8_text(content.as_bytes()).unwrap_or("");
                    
                    // Look for a colon in the plain_value text, which indicates a new property started
                    // without a semicolon ending the previous one
                    if let Some(colon_pos) = value_text.find(':') {
                        // Extract the part before the colon - this should be a property name
                        let potential_property = value_text[..colon_pos].trim();
                        
                        // Check if this looks like a valid CSS property name
                        if potential_property.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') 
                           && potential_property.len() > 2 
                           && !potential_property.chars().next().unwrap_or(' ').is_ascii_digit() {
                            
                            // This is likely a new property declaration, meaning we're missing a semicolon
                            // Find the position just before this property starts
                            let node_start = plain_value.start_byte();
                            let property_start_in_value = value_text[..colon_pos].rfind(potential_property).unwrap_or(0);
                            let error_byte_pos = node_start + property_start_in_value;
                            
                            // Create a range just before the new property
                            let error_position = self.byte_to_position(error_byte_pos, content);
                            let range = Range {
                                start: Position {
                                    line: error_position.line,
                                    character: if error_position.character > 0 { error_position.character - 1 } else { 0 },
                                },
                                end: error_position,
                            };
                            
                            diagnostics.push(Diagnostic {
                                range,
                                severity: Some(DiagnosticSeverity::ERROR),
                                code: Some(NumberOrString::String("missing-semicolon".to_string())),
                                source: Some("uss".to_string()),
                                message: format!("Missing semicolon before property '{}'", potential_property),
                                ..Default::default()
                            });
                            
                            return; // Stop validation if semicolon is missing
                        }
                    }
                }
                
                // Parse values into UssValue objects
                let mut uss_values = Vec::new();
                let mut parsing_failed = false;
                
                // Collect value nodes (everything after the colon, skipping semicolons)
                for i in 2..node.child_count() {
                    if let Some(child) = node.child(i) {
                        // Skip semicolons and whitespace
                        if child.kind() != ";" && !child.kind().is_empty() {
                            // Try to parse the node as a UssValue
                            // Convert source_url string to Url if provided
                            let source_url_parsed = source_url.and_then(|url_str| url::Url::parse(url_str).ok());
                            match UssValue::from_node(child, content, &self.definitions, source_url_parsed.as_ref()) {
                                Ok(value) => uss_values.push(value),
                                Err(error) => {
                                    // Report parsing error and stop
                                    let range = self.node_to_range(child, content);
                                    diagnostics.push(Diagnostic {
                                        range,
                                        severity: Some(DiagnosticSeverity::ERROR),
                                        code: Some(NumberOrString::String("invalid-value".to_string())),
                                        source: Some("uss".to_string()),
                                        message: format!("Invalid value: {}", error.message),
                                        ..Default::default()
                                    });
                                    parsing_failed = true;
                                    break;
                                }
                            }
                        }
                    }
                }
                
                // If parsing failed, don't proceed with validation
                if parsing_failed {
                    return;
                }
                
                // Validate the parsed values against the property's ValueSpec
                if let Some(property_info) = self.definitions.get_property_info(property_name) {
                    // Check if any of the property's value formats match
                    let mut any_format_matches = false;
                    
                    for value_format in &property_info.value_spec.formats {
                        if value_format.is_match(&uss_values, &self.definitions) {
                            any_format_matches = true;
                            break;
                        }
                    }
                    
                    if !any_format_matches {
                        // Find the range covering all values
                        let values_range = if let (Some(first_value_node), Some(last_value_node)) = 
                            (node.child(2), node.child(node.child_count().saturating_sub(2))) {
                            let start_pos = self.byte_to_position(first_value_node.start_byte(), content);
                            let end_pos = self.byte_to_position(last_value_node.end_byte(), content);
                            Range { start: start_pos, end: end_pos }
                        } else {
                            self.node_to_range(node, content)
                        };
                        
                        diagnostics.push(Diagnostic {
                            range: values_range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("invalid-property-value".to_string())),
                            source: Some("uss".to_string()),
                            message: format!("Invalid value for property '{}'", property_name),
                            ..Default::default()
                        });
                    }
                }
                // If property info is not found, we already reported "unknown-property" error above
            }
        }
    }
    
    /// Validate pseudo-class selector
    fn validate_pseudo_class(&self, node: Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        let pseudo_class = node.utf8_text(content.as_bytes()).unwrap_or("");
        
        // Check if this "pseudo-class" is actually a missing semicolon case
        // Pattern: property-name:value (e.g., "border-radius:10px")
        if let Some(colon_pos) = pseudo_class.find(':') {
            let property_part = &pseudo_class[..colon_pos];
            let value_part = &pseudo_class[colon_pos + 1..];
            
            // Check if the part before colon looks like a CSS property name
            if self.is_likely_css_property(property_part) && !value_part.is_empty() {
                // This is likely a missing semicolon, not a pseudo-class
                let range = self.node_to_range(node, content);
                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("missing-semicolon".to_string())),
                    source: Some("uss".to_string()),
                    message: format!("Missing semicolon after property '{}'", property_part),
                    ..Default::default()
                });
                return;
            }
        }
        
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
    
    /// Check if a string looks like a CSS property name
    fn is_likely_css_property(&self, text: &str) -> bool {
        // CSS property names:
        // - contain only lowercase letters, digits, and hyphens
        // - don't start with a digit
        // - are reasonable length (2-30 characters)
        // - contain at least one letter
        if text.len() < 2 || text.len() > 30 {
            return false;
        }
        
        if text.starts_with(char::is_numeric) {
            return false;
        }
        
        let has_letter = text.chars().any(|c| c.is_alphabetic());
        let valid_chars = text.chars().all(|c| c.is_alphanumeric() || c == '-');
        
        has_letter && valid_chars
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
    


    /// Add invalid unit diagnostic
    fn add_invalid_unit_diagnostic(&self, unit_node: Node, content: &str, property_name: &str, unit: &str, expected: &str, diagnostics: &mut Vec<Diagnostic>) {
        let range = self.node_to_range(unit_node, content);
        
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("invalid-unit".to_string())),
            source: Some("uss".to_string()),
            message: format!("Invalid unit '{}' for property '{}'. Expected: {}", unit, property_name, expected),
            ..Default::default()
        });
    }

    /// Validate function arguments
    fn validate_function_arguments(&self, function_name: &str, node: Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        if let Some(args_node) = node.child(1) {
            match function_name {
                "resource" | "url" => {
                    // Should have exactly one string argument
                    let actual_args: Vec<_> = (0..args_node.child_count())
                        .filter_map(|i| args_node.child(i))
                        .filter(|child| child.kind() != "," && child.kind() != "(" && child.kind() != ")")
                        .collect();
                    
                    if actual_args.is_empty() {
                        let range = self.node_to_range(args_node, content);
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("missing-argument".to_string())),
                            source: Some("uss".to_string()),
                            message: format!("{}() function requires a string argument", function_name),
                            ..Default::default()
                        });
                    } else if actual_args.len() != 1 {
                        let range = self.node_to_range(args_node, content);
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("invalid-argument-count".to_string())),
                            source: Some("uss".to_string()),
                            message: format!("{}() function requires exactly 1 argument, found {}", function_name, actual_args.len()),
                            ..Default::default()
                        });
                    } else {
                        // Validate that the argument is a string
                        let arg = actual_args[0];
                        let arg_text = arg.utf8_text(content.as_bytes()).unwrap_or("");
                        
                        // Check if it's a quoted string or looks like a valid path
                        let is_quoted_string = (arg_text.starts_with('"') && arg_text.ends_with('"')) ||
                                             (arg_text.starts_with('\'') && arg_text.ends_with('\''));
                        
                        if !is_quoted_string && arg.kind() != "string_value" {
                            let range = self.node_to_range(arg, content);
                            diagnostics.push(Diagnostic {
                                range,
                                severity: Some(DiagnosticSeverity::ERROR),
                                code: Some(NumberOrString::String("invalid-argument-type".to_string())),
                                source: Some("uss".to_string()),
                                message: format!("{}() function requires a string argument, found: {}", function_name, arg_text),
                                ..Default::default()
                            });
                        }
                        
                        // Additional validation for resource() function
                        if function_name == "resource" && is_quoted_string {
                            let path = if arg_text.len() >= 2 {
                                &arg_text[1..arg_text.len()-1] // Remove quotes
                            } else {
                                ""
                            };
                            
                            // Check for common resource path patterns
                            if path.is_empty() {
                                let range = self.node_to_range(arg, content);
                                diagnostics.push(Diagnostic {
                                    range,
                                    severity: Some(DiagnosticSeverity::WARNING),
                                    code: Some(NumberOrString::String("empty-resource-path".to_string())),
                                    source: Some("uss".to_string()),
                                    message: "Empty resource path in resource() function".to_string(),
                                    ..Default::default()
                                });
                            }
                        }
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
                    let actual_args: Vec<_> = (0..args_node.child_count())
                        .filter_map(|i| args_node.child(i))
                        .filter(|child| child.kind() != "," && child.kind() != "(" && child.kind() != ")")
                        .collect();
                    
                    if actual_args.len() != 3 {
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
                    let actual_args: Vec<_> = (0..args_node.child_count())
                        .filter_map(|i| args_node.child(i))
                        .filter(|child| child.kind() != "," && child.kind() != "(" && child.kind() != ")")
                        .collect();
                    
                    if actual_args.len() != 4 {
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
                "hsl" => {
                    // Should have exactly 3 arguments
                    let actual_args: Vec<_> = (0..args_node.child_count())
                        .filter_map(|i| args_node.child(i))
                        .filter(|child| child.kind() != "," && child.kind() != "(" && child.kind() != ")")
                        .collect();
                    
                    if actual_args.len() != 3 {
                        let range = self.node_to_range(args_node, content);
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("invalid-argument-count".to_string())),
                            source: Some("uss".to_string()),
                            message: "hsl() function requires exactly 3 arguments (hue, saturation, lightness)".to_string(),
                            ..Default::default()
                        });
                    }
                }
                "hsla" => {
                    // Should have exactly 4 arguments
                    let actual_args: Vec<_> = (0..args_node.child_count())
                        .filter_map(|i| args_node.child(i))
                        .filter(|child| child.kind() != "," && child.kind() != "(" && child.kind() != ")")
                        .collect();
                    
                    if actual_args.len() != 4 {
                        let range = self.node_to_range(args_node, content);
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("invalid-argument-count".to_string())),
                            source: Some("uss".to_string()),
                            message: "hsla() function requires exactly 4 arguments (hue, saturation, lightness, alpha)".to_string(),
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
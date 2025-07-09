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
        // Check for syntax errors - only report for ERROR nodes directly, not for nodes that contain errors
        if node.kind() == "ERROR" {
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
                            
                            break; // Only report the first missing semicolon in this declaration
                        }
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uss::parser::UssParser;

    #[test]
    fn test_precise_syntax_error_range() {
        let diagnostics = UssDiagnostics::new();
        let mut parser = UssParser::new().unwrap();
        
        // Test case with syntax error - a simple invalid token on line 4
        let content = ".valid-rule {\n    background-color: red;\n}\n\na;\n\n.another-valid-rule {\n    color: blue;\n}";
        
        let tree = parser.parse(content, None).unwrap();
        let results = diagnostics.analyze(&tree, content);
        
        // Debug: Check diagnostic details
        // Uncomment the following lines for debugging:
        // for (i, diagnostic) in results.iter().enumerate() {
        //     println!("  {}: Line {}:{}-{}:{} - {} - {}", 
        //         i, 
        //         diagnostic.range.start.line, diagnostic.range.start.character,
        //         diagnostic.range.end.line, diagnostic.range.end.character,
        //         diagnostic.code.as_ref().map(|c| match c {
        //             tower_lsp::lsp_types::NumberOrString::String(s) => s.as_str(),
        //             tower_lsp::lsp_types::NumberOrString::Number(_) => "number",
        //         }).unwrap_or("no-code"),
        //         diagnostic.message
        //     );
        // }
        
        // Should have at least one syntax error
        let syntax_errors: Vec<_> = results.iter()
            .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("syntax-error".to_string())))
            .collect();
        
        assert!(!syntax_errors.is_empty(), "Should detect syntax error");
        
        // Verify that we have syntax errors to test
        
        // Test that errors don't span the entire file (which was the original problem)
        // Each error should be limited to a reasonable range
        for error in &syntax_errors {
            let line_span = error.range.end.line - error.range.start.line;
            let char_span = if error.range.start.line == error.range.end.line {
                error.range.end.character - error.range.start.character
            } else {
                100 // If it spans multiple lines, we'll check line span instead
            };
            
            // Error should not span more than 1 line or more than 50 characters on a single line
            assert!(line_span <= 1 && char_span <= 50, 
                "Error range too large: spans {} lines and {} chars. Range: {}:{}-{}:{}",
                line_span, char_span, 
                error.range.start.line, error.range.start.character,
                error.range.end.line, error.range.end.character);
        }
        
        // Check that at least one error is on line 4 (where "a;" is located)
        let has_error_on_line_4 = syntax_errors.iter().any(|error| {
            error.range.start.line == 4 // Line 4 is where "a;" is located
        });
        
        assert!(has_error_on_line_4, "Should have at least one error on line 4 where 'a;' is located");
        
        // At least one error should be small and precise
        let has_small_error = syntax_errors.iter().any(|error| {
            let line_span = error.range.end.line - error.range.start.line;
            let char_span = if error.range.start.line == error.range.end.line {
                error.range.end.character - error.range.start.character
            } else {
                100
            };
            line_span == 0 && char_span <= 10 // Small, precise error
        });
        
        assert!(has_small_error, "Should have at least one small, precise error range");
    }
    
    #[test]
    fn test_missing_semicolon_detection() {
        let diagnostics = UssDiagnostics::new();
        let mut parser = UssParser::new().unwrap();
        
        // Test case with missing semicolon after background-color: red
        let content = r#"@import url("a.css");

a {
    background-color: red
    border-radius:10px;
}"#;
        
        let tree = parser.parse(content, None).unwrap();
        let results = diagnostics.analyze(&tree, content);
        
        // Check if missing semicolon is detected
        let missing_semicolon_errors: Vec<_> = results.iter()
            .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("missing-semicolon".to_string())))
            .collect();
        
        assert!(!missing_semicolon_errors.is_empty(), "Should detect missing semicolon");
        
        // Verify the error is reported at the correct location (before 'border-radius')
        let has_border_radius_error = missing_semicolon_errors.iter().any(|error| {
            error.message.contains("border-radius")
        });
        
        assert!(has_border_radius_error, "Should detect missing semicolon before 'border-radius' property");
    }

    #[test]
    fn test_nested_rule_missing_semicolon() {
        let diagnostics = UssDiagnostics::new();
        let mut parser = UssParser::new().unwrap();
        
        // Test case with missing semicolon before nested rule
        let content = r#"@import url("a.css");

a {
    background-color: red;
    border-radius:10px
    c{
        
    }
}"#;
        
        let tree = parser.parse(content, None).unwrap();
        let results = diagnostics.analyze(&tree, content);
        

        
        // Should detect missing semicolon before nested rule, not pseudo-class error
        let missing_semicolon_errors: Vec<_> = results.iter()
            .filter(|e| e.message.contains("Missing semicolon after property"))
            .collect();
        
        assert!(!missing_semicolon_errors.is_empty(), "Should detect missing semicolon before nested rule");
        
        // Verify the specific error message
        let semicolon_error = &missing_semicolon_errors[0];
        assert!(semicolon_error.message.contains("border-radius"), "Should identify the correct property name");
        assert_eq!(semicolon_error.code, Some(NumberOrString::String("missing-semicolon".to_string())));
    }
    
    #[test]
    fn test_multiline_error_limitation() {
        let diagnostics = UssDiagnostics::new();
        let mut parser = UssParser::new().unwrap();
        
        // Test case with a syntax error that might span multiple lines
        let content = r#"
.rule {
    background-color: red
    /* missing semicolon */
    color: blue;
}
"#;
        
        let tree = parser.parse(content, None);
        if let Some(tree) = tree {
            let results = diagnostics.analyze(&tree, content);
            
            // Check that any syntax errors don't span too many lines
            for diagnostic in results {
                if diagnostic.code == Some(tower_lsp::lsp_types::NumberOrString::String("syntax-error".to_string())) {
                    let line_span = diagnostic.range.end.line - diagnostic.range.start.line;
                    assert!(line_span <= 1, "Syntax error should not span more than 1 line, but spans {} lines", line_span);
                }
            }
        }
    }
}
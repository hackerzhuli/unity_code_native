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
        // If the node is an ERROR node, try to find the actual problematic token
        if node.kind() == "ERROR" {
            // First, try to find the deepest ERROR node or MISSING node
            let mut deepest_error = None;
            let mut cursor = node.walk();
            
            // Walk through all descendants to find the most specific error
            if cursor.goto_first_child() {
                loop {
                    let current_node = cursor.node();
                    
                    // Prioritize MISSING nodes as they indicate exactly what's wrong
                    if current_node.is_missing() {
                        deepest_error = Some(current_node);
                        break;
                    }
                    
                    // Also consider ERROR nodes, but MISSING takes priority
                    if current_node.kind() == "ERROR" && deepest_error.is_none() {
                        deepest_error = Some(current_node);
                    }
                    
                    // Continue traversing
                    if !cursor.goto_next_sibling() {
                        if !cursor.goto_parent() {
                            break;
                        }
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                }
            }
            
            // If we found a specific error node, use it
            if let Some(error_node) = deepest_error {
                let error_range = self.node_to_range(error_node, content);
                // Still limit to single line if it spans multiple lines
                if error_range.end.line > error_range.start.line {
                    let line_end_position = self.find_line_end_position(error_range.start.line, content);
                    return Range {
                        start: error_range.start,
                        end: line_end_position,
                    };
                }
                return error_range;
            }
            
            // Fallback: look for the last non-whitespace token before the error
            for i in (0..node.child_count()).rev() {
                if let Some(child) = node.child(i) {
                    let child_text = child.utf8_text(content.as_bytes()).unwrap_or("");
                    // Find the last meaningful token
                    if !child_text.trim().is_empty() && !child_text.starts_with("/*") && !child_text.starts_with("//") {
                        let child_range = self.node_to_range(child, content);
                        // Create a small range at the end of this token to indicate where the error is
                        return Range {
                            start: child_range.end,
                            end: Position {
                                line: child_range.end.line,
                                character: child_range.end.character + 1,
                            },
                        };
                    }
                }
            }
        }
        
        // If we can't find a more specific location, try to limit to a single line
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
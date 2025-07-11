//! USS Diagnostics
//!
//! Provides validation and error reporting for USS files.
//! Validates syntax, properties, values, and USS-specific rules.

use crate::language::asset_url::validate_url;
use crate::language::tree_utils::{byte_to_position, node_to_range};
use crate::uss::definitions::UssDefinitions;
use crate::uss::tree_printer;
use crate::uss::value::UssValue;
use crate::uss::variable_resolver::{VariableResolver, VariableStatus};
use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Tree};
use url::Url;

/// Represents a URL found in USS code along with its location range
/// Used for future asset validation (file existence checks, etc.)
#[derive(Debug, Clone)]
pub struct UrlReference {
    /// The URL found in the USS code
    pub url: Url,
    /// The LSP range of the URL (for url() functions, this is just the argument range, not including the function name)
    pub range: Range,
}

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
    pub fn analyze_with_source_url(
        &self,
        tree: &Tree,
        content: &str,
        source_url: Option<&Url>,
    ) -> Vec<Diagnostic> {
        let (diagnostics, _) = self.analyze_with_variables(tree, content, source_url, None);
        diagnostics
    }

    /// Analyze USS syntax tree and generate diagnostics with variable resolver support
    ///
    /// **Note**: Variable resolution has limitations:
    /// - Only resolves variables defined within the same document
    /// - Does not support imported variables from other USS files
    /// - When variable resolution is uncertain, warnings are generated instead of errors
    ///
    /// Returns a tuple of (diagnostics, url_references) where url_references contains
    /// URLs found in import statements and url() functions for future asset validation.
    pub fn analyze_with_variables(
        &self,
        tree: &Tree,
        content: &str,
        source_url: Option<&Url>,
        variable_resolver: Option<&VariableResolver>,
    ) -> (Vec<Diagnostic>, Vec<UrlReference>) {
        let mut diagnostics = Vec::new();
        let mut url_references = Vec::new();
        let root_node = tree.root_node();

        // Assert that if a source URL is provided, it must be a project scheme URL
        if let Some(url) = source_url {
            assert_eq!(
                url.scheme(),
                "project",
                "Source URL must use project scheme for Unity compatibility, got: {}",
                url
            );
        }

        self.walk_node_with_variables(
            root_node,
            content,
            source_url,
            variable_resolver,
            &mut diagnostics,
            &mut url_references,
        );

        (diagnostics, url_references)
    }

    /// Debug helper: Print the complete syntax tree to stdout
    /// Useful for understanding tree structure during development
    #[allow(dead_code)]
    pub fn debug_print_tree(&self, tree: &Tree, content: &str) {
        let root_node = tree.root_node();
        tree_printer::print_tree_to_stdout(root_node, content);
    }

    /// Recursively walk the syntax tree and validate nodes with variable resolver support
    fn walk_node_with_variables(
        &self,
        node: Node,
        content: &str,
        source_url: Option<&Url>,
        variable_resolver: Option<&VariableResolver>,
        diagnostics: &mut Vec<Diagnostic>,
        url_references: &mut Vec<UrlReference>,
    ) {
        // Track the number of diagnostics before processing children
        let initial_diagnostic_count = diagnostics.len();

        // Check for syntax errors - only report for ERROR nodes directly, not for nodes that contain errors
        if node.kind() == "ERROR" {
            self.add_syntax_error(node, content, diagnostics);
        }

        // Recursively check children first to detect any child errors
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.walk_node_with_variables(
                    child,
                    content,
                    source_url,
                    variable_resolver,
                    diagnostics,
                    url_references,
                );
            }
        }

        // Check if any error diagnostics were added by children (warnings are fine, we should keep going)
        let child_error_diagnostics_added =
            (initial_diagnostic_count..diagnostics.len()).any(|i| {
                diagnostics[i].severity >= Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR)
            });

        match node.kind() {
            "rule_set" => self.validate_rule_set(node, content, diagnostics),
            "declaration" => {
                // Only validate declaration if no child error diagnostics were generated
                // This prevents redundant error messages when child nodes (like invalid tokens,
                // syntax errors, or malformed values) have already reported issues.
                // For example, if a property value contains a syntax error, we don't want to
                // also report that the property itself is invalid - the child error is sufficient.
                // Warnings from children are fine and we should continue with validation.
                if !child_error_diagnostics_added {
                    self.validate_declaration(
                        node,
                        content,
                        diagnostics,
                        source_url,
                        variable_resolver,
                    );
                }
            }
            "call_expression" => {
                self.validate_function_call(node, content, diagnostics, source_url, url_references)
            }
            "pseudo_class_selector" => self.validate_pseudo_class(node, content, diagnostics),
            "at_rule"
            | "charset_statement"
            | "import_statement"
            | "keyframes_statement"
            | "media_statement"
            | "namespace_statement"
            | "supports_statement" => {
                self.validate_at_rule(node, content, diagnostics, source_url, url_references)
            }
            _ => {}
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
                let error_range = node_to_range(error_node, content);

                // Limit to single line if it spans multiple lines
                if error_range.end.line > error_range.start.line {
                    let line_end_position =
                        self.find_line_end_position(error_range.start.line, content);
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
                return node_to_range(missing_node, content);
            }
        }

        // If this is an ERROR node itself, use its range directly
        if node.kind() == "ERROR" {
            return node_to_range(node, content);
        }

        // Fallback: limit the node range to a single line
        let node_range = node_to_range(node, content);
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
                        let range = node_to_range(node, content);
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
    fn validate_declaration(
        &self,
        node: Node,
        content: &str,
        diagnostics: &mut Vec<Diagnostic>,
        source_url: Option<&Url>,
        variable_resolver: Option<&VariableResolver>,
    ) {
        if let Some(property_node) = node.child(0) {
            if property_node.kind() == "property_name" {
                let property_name = property_node.utf8_text(content.as_bytes()).unwrap_or("");

                // Check if property is valid
                if !self.definitions.is_valid_property(property_name) {
                    let range = node_to_range(property_node, content);
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

                // Parse values into UssValue objects first
                let mut uss_values = Vec::new();
                let mut parsing_failed = false;
                let mut value_nodes = Vec::new(); // Keep track of nodes for error reporting

                // Collect value nodes (everything after the colon, skipping semicolons)
                for i in 2..node.child_count() {
                    if let Some(child) = node.child(i) {
                        // Skip semicolons and whitespace
                        if child.kind() != ";" && !child.kind().is_empty() {
                            value_nodes.push(child);
                        }
                    }
                }

                // Parse each value node
                for child in &value_nodes {
                    // Try to parse the node as a UssValue
                    match UssValue::from_node(*child, content, &self.definitions, source_url) {
                        Ok(value) => uss_values.push(value),
                        Err(error) => {
                            // Report parsing error and stop
                            let range = node_to_range(*child, content);

                            diagnostics.push(Diagnostic {
                                range,
                                severity: Some(error.severity),
                                code: Some(NumberOrString::String("invalid-value".to_string())),
                                source: Some("uss".to_string()),
                                message: format!("Invalid value: {}", error.message),
                                ..Default::default()
                            });

                            if error.severity >= DiagnosticSeverity::ERROR {
                                parsing_failed = true;
                            }
                        }
                    }
                }

                if parsing_failed {
                    return;
                }

                // Check for missing semicolon by detecting identifiers that contain colons
                // This happens when parser treats "background-color: red\n    border-radius:10px" as one declaration
                for (i, value) in uss_values.iter().enumerate() {
                    if let UssValue::Identifier(identifier_text) = value {
                        if let Some(colon_pos) = identifier_text.find(':') {
                            // Extract the part before the colon - this should be a property name
                            let potential_property = identifier_text[..colon_pos].trim();

                            // Check if this looks like a valid CSS property name
                            if self.is_likely_css_property(potential_property) {
                                // This is likely a new property declaration, meaning we're missing a semicolon
                                // Use the corresponding value node for error positioning
                                if let Some(value_node) = value_nodes.get(i) {
                                    let range = node_to_range(*value_node, content);
                                    diagnostics.push(Diagnostic {
                                        range,
                                        severity: Some(DiagnosticSeverity::ERROR),
                                        code: Some(NumberOrString::String(
                                            "missing-semicolon".to_string(),
                                        )),
                                        source: Some("uss".to_string()),
                                        message: format!(
                                            "Missing semicolon before property '{}'",
                                            potential_property
                                        ),
                                        ..Default::default()
                                    });

                                    return; // Stop validation if semicolon is missing
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
                        let values_range = if let (Some(first_value_node), Some(last_value_node)) = (
                            node.child(2),
                            node.child(node.child_count().saturating_sub(2)),
                        ) {
                            let start_pos =
                                byte_to_position(first_value_node.start_byte(), content);
                            let end_pos = byte_to_position(last_value_node.end_byte(), content);
                            Range {
                                start: start_pos,
                                end: end_pos,
                            }
                        } else {
                            node_to_range(node, content)
                        };

                        diagnostics.push(Diagnostic {
                            range: values_range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String(
                                "invalid-property-value".to_string(),
                            )),
                            source: Some("uss".to_string()),
                            message: format!("Invalid value for property '{}'", property_name),
                            ..Default::default()
                        });
                    } else if let Some(resolver) = variable_resolver {
                        // Validation passed without variable resolution, now check with resolved variables for warnings
                        let resolved_values =
                            self.resolve_variables_in_values(&uss_values, resolver);
                        let mut resolved_format_matches = false;

                        // Try validation with resolved values
                        for value_format in &property_info.value_spec.formats {
                            if value_format.is_match(&resolved_values, &self.definitions) {
                                resolved_format_matches = true;
                                break;
                            }
                        }

                        // Only generate warnings if there are unresolved variables and validation would fail with resolved values
                        if !resolved_format_matches {
                            let values_range =
                                if let (Some(first_value_node), Some(last_value_node)) = (
                                    node.child(2),
                                    node.child(node.child_count().saturating_sub(2)),
                                ) {
                                    let start_pos =
                                        byte_to_position(first_value_node.start_byte(), content);
                                    let end_pos =
                                        byte_to_position(last_value_node.end_byte(), content);
                                    Range {
                                        start: start_pos,
                                        end: end_pos,
                                    }
                                } else {
                                    node_to_range(node, content)
                                };

                            // Create a readable string of the resolved property values
                            let resolved_values_str = resolved_values
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<_>>()
                                .join(" ");

                            // Collect information about resolved variables
                            let mut variable_info = Vec::new();
                            for value in &uss_values {
                                if let UssValue::VariableReference(var_name) = value {
                                    if let Some(var_status) = resolver.get_variable(var_name) {
                                        if let VariableStatus::Resolved(resolved_vals) = var_status
                                        {
                                            let resolved_str = resolved_vals
                                                .iter()
                                                .map(|v| v.to_string())
                                                .collect::<Vec<_>>()
                                                .join(" ");
                                            variable_info
                                                .push(format!("--{} = {}", var_name, resolved_str));
                                        }
                                    }
                                }
                            }

                            let message = if variable_info.is_empty() {
                                format!(
                                    "Property '{}' value '{}' is likely invalid",
                                    property_name, resolved_values_str
                                )
                            } else {
                                format!(
                                    "Property '{}' value '{}' is likely invalid. The resolved variables are: {}",
                                    property_name,
                                    resolved_values_str,
                                    variable_info.join(", ")
                                )
                            };

                            diagnostics.push(Diagnostic {
                                range: values_range,
                                severity: Some(DiagnosticSeverity::WARNING),
                                code: Some(NumberOrString::String(
                                    "uncertain-property-value".to_string(),
                                )),
                                source: Some("uss".to_string()),
                                message,
                                ..Default::default()
                            });
                        }
                    }
                }
                // If property info is not found, we already reported "unknown-property" error above
            }
        }
    }

    /// Resolve variables in a list of UssValues using the variable resolver
    fn resolve_variables_in_values(
        &self,
        values: &[UssValue],
        variable_resolver: &VariableResolver,
    ) -> Vec<UssValue> {
        let mut resolved_values = Vec::new();

        for value in values {
            match value {
                UssValue::VariableReference(var_name) => {
                    // Try to resolve the variable
                    if let Some(var_status) = variable_resolver.get_variable(var_name) {
                        match var_status {
                            VariableStatus::Resolved(resolved_vals) => {
                                // Add all resolved values
                                resolved_values.extend(resolved_vals.clone());
                            }
                            _ => {
                                // Variable is unresolved, ambiguous, or has errors - keep the original reference
                                resolved_values.push(value.clone());
                            }
                        }
                    } else {
                        // Variable not found - keep the original reference
                        resolved_values.push(value.clone());
                    }
                }
                _ => {
                    // Non-variable value - keep as-is
                    resolved_values.push(value.clone());
                }
            }
        }

        resolved_values
    }

    /// Validate function call (specifically for URL functions to generate warnings)
    fn validate_function_call(
        &self,
        node: Node,
        content: &str,
        diagnostics: &mut Vec<Diagnostic>,
        source_url: Option<&Url>,
        url_references: &mut Vec<UrlReference>,
    ) {
        // Check if this is a url() function call (not resource())
        if let Some(function_name_node) = node.child(0) {
            let function_name = function_name_node
                .utf8_text(content.as_bytes())
                .unwrap_or("");

            if function_name == "url" {
                // Parse the function call using UssValue
                match UssValue::from_node(node, content, &self.definitions, source_url) {
                    Ok(uss_value) => {
                        //eprintln!("DEBUG: Successfully parsed UssValue: {:?}", uss_value);
                        if let UssValue::Url(url) = uss_value {
                            // Find the argument node (excluding the function name and parentheses)
                            if let Some(args_node) = node.child(1) {
                                // first argument is '(', second is actual argument
                                if let Some(arg_node) = args_node.child(1) {
                                    if arg_node.kind() == "string_value" {
                                        let arg_range = node_to_range(arg_node, content);
                                        url_references.push(UrlReference {
                                            url: url.clone(),
                                            range: arg_range,
                                        });

                                        self.add_url_argument_warning(
                                            arg_node,
                                            content,
                                            diagnostics,
                                            source_url,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        //eprintln!("DEBUG: Failed to parse UssValue: {:?}", e);
                    }
                }
            }
        }
    }

    /// try to find if a url argument node contains warnings
    fn add_url_argument_warning(
        &self,
        node: Node<'_>,
        content: &str,
        diagnostics: &mut Vec<Diagnostic>,
        source_url: Option<&Url>,
    ) {
        if node.kind() != "string_value" {
            return;
        }

        if let Ok(arg_value) = UssValue::from_node(node, content, &self.definitions, source_url) {
            if let UssValue::String(arg_str) = arg_value {
                if let Ok(validation_result) =
                    crate::language::asset_url::validate_url(arg_str.as_str(), source_url)
                {
                    if !validation_result.warnings.is_empty() {
                        for warning in validation_result.warnings {
                            let range = node_to_range(node, content);
                            diagnostics.push(Diagnostic {
                                range,
                                severity: Some(DiagnosticSeverity::WARNING),
                                code: Some(NumberOrString::String("url-warning".to_string())),
                                source: Some("uss".to_string()),
                                message: warning.message,
                                ..Default::default()
                            });
                        }
                    }
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
                let range = node_to_range(node, content);
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
            let range = node_to_range(node, content);
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
    fn validate_at_rule(
        &self,
        node: Node,
        content: &str,
        diagnostics: &mut Vec<Diagnostic>,
        source_url: Option<&Url>,
        url_references: &mut Vec<UrlReference>,
    ) {
        match node.kind() {
            "import_statement" => {
                self.validate_import_statement(
                    node,
                    content,
                    diagnostics,
                    source_url,
                    url_references,
                );
            }
            _ => {
                // Generic at-rule that's not an import - these are not supported
                let range = node_to_range(node, content);
                let mut at_rule_text = "unknown";
                if let Some(name_node) = node.child(0) {
                    at_rule_text = name_node.utf8_text(content.as_bytes()).unwrap_or("unknown");
                }
                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("unsupported-at-rule".to_string())),
                    source: Some("uss".to_string()),
                    message: format!(
                        "Unsupported at-rule '{}'. Only @import is supported in USS",
                        at_rule_text
                    ),
                    ..Default::default()
                });
            }
        }
    }

    /// Validate import statement structure and values
    fn validate_import_statement(
        &self,
        node: Node,
        content: &str,
        diagnostics: &mut Vec<Diagnostic>,
        source_url: Option<&Url>,
        url_references: &mut Vec<UrlReference>,
    ) {
        // Import statement structure: import_statement -> @import + (string_value | call_expression)
        // Find the value child that contains the import path (either string or url() function)
        let mut import_value_node = None;

        // first node must be @import,  second node is url function or a string, third node must be ; to end the statement, and nothing after that
        // first node is already checked so no need to check that
        if node.child_count() > 1 {
            import_value_node = Some(node.child(1).unwrap());
        }

        if let Some(value_node) = import_value_node {
            self.validate_import_value_node(
                content,
                diagnostics,
                source_url,
                url_references,
                value_node,
            );
        }

        // we expect the third child to be a ";"
        if node.child_count() > 2 {
            let semi_node = node.child(2).unwrap();
            if semi_node.kind() != ";" {
                let range = node_to_range(semi_node, content);
                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("missing-semicolon".to_string())),
                    source: Some("uss".to_string()),
                    message: format!(
                        "Import statement is expecting a semicolon, but found {}",
                        semi_node.utf8_text(content.as_bytes()).unwrap_or("None")
                    ),
                    ..Default::default()
                });
            }
        }
    }

    /// validate the value node of import statement, must be a url function or a string
    fn validate_import_value_node(
        &self,
        content: &str,
        diagnostics: &mut Vec<Diagnostic>,
        source_url: Option<&Url>,
        url_references: &mut Vec<UrlReference>,
        value_node: Node<'_>,
    ) {
        match UssValue::from_node(value_node, content, &self.definitions, source_url) {
            Ok(uss_value) => {
                match uss_value {
                    UssValue::String(import_path) => {
                        // Validate URL for string import paths using asset_url validation
                        match validate_url(&import_path, source_url) {
                            Err(validation_error) => {
                                let range = node_to_range(value_node, content);
                                diagnostics.push(Diagnostic {
                                    range,
                                    severity: Some(DiagnosticSeverity::ERROR),
                                    code: Some(NumberOrString::String(
                                        "invalid-import-url".to_string(),
                                    )),
                                    source: Some("uss".to_string()),
                                    message: format!(
                                        "Invalid import path: {}",
                                        validation_error.message
                                    ),
                                    ..Default::default()
                                });
                            }
                            Ok(validation_result) => {
                                let range = node_to_range(value_node, content);
                                url_references.push(UrlReference {
                                    url: validation_result.url.clone(),
                                    range,
                                });

                                // Check for URL validation warnings
                                for warning in &validation_result.warnings {
                                    let range = node_to_range(value_node, content);
                                    diagnostics.push(Diagnostic {
                                        range,
                                        severity: Some(DiagnosticSeverity::WARNING),
                                        code: Some(NumberOrString::String(
                                            "import-url-warning".to_string(),
                                        )),
                                        source: Some("uss".to_string()),
                                        message: warning.message.clone(),
                                        ..Default::default()
                                    });
                                }

                                // Check for .uss extension warning
                                if !validation_result.url.path().ends_with(".uss") {
                                    let range = node_to_range(value_node, content);
                                    diagnostics.push(Diagnostic {
                                        range,
                                        severity: Some(DiagnosticSeverity::WARNING),
                                        code: Some(NumberOrString::String(
                                            "missing-uss-extension".to_string(),
                                        )),
                                        source: Some("uss".to_string()),
                                        message: "Import path should have .uss extension"
                                            .to_string(),
                                        ..Default::default()
                                    });
                                }
                            }
                        }
                    }
                    UssValue::Url(_) => {
                        // since URL is valid, so first child is "(", ignore
                        if let Some(arg) = value_node.child(1) {
                            self.add_url_argument_warning(arg, content, diagnostics, source_url);
                        }
                    }
                    _ => {
                        // Import value is neither a string nor a url function
                        let range = node_to_range(value_node, content);
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("invalid-import-value".to_string())),
                            source: Some("uss".to_string()),
                            message: "Import path must be a string or url() function".to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
            Err(err) => {
                // UssValue validation failed - use the detailed error from UssValue
                let range = node_to_range(value_node, content);
                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("invalid-import-syntax".to_string())),
                    source: Some("uss".to_string()),
                    message: err.message,
                    ..Default::default()
                });
            }
        }
    }
}

impl Default for UssDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

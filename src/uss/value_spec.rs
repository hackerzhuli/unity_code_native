//! USS Value Specification Types
//!
//! Contains types for defining and validating USS property values,
//! including ValueType, ValueEntry, ValueFormat, and ValueSpec.

use tree_sitter::Node;
use crate::uss::definitions::UssDefinitions;

/// Basic value type that a property accepts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueType {
    /// Length values (px, %)
    /// 
    /// note: value 0 can be intepreted as length even without unit
    Length,
    /// Numeric values (unitless numbers)(can be integer or float)
    Number,
    /// Integer values (unitless numbers, must be integer)
    Integer,
    String,
    Time,
    /// Color values (hex, named colors, color functions)
    Color,
    /// Angle values (units are deg, rad, grad, turn)
    Angle,
    /// Keyword values from a predefined list (a USS Keyword)
    Keyword(&'static str),
    /// Asset references (url(), resource())
    Asset,
    /// property names in animation related property
    PropertyName
}

/// one value entry of property
#[derive(Debug, Clone)]
pub struct ValueEntry {
    /// All valid value types for this entry
    pub types: Vec<ValueType>
}

/// Specific value format with exact type and count requirements
#[derive(Debug, Clone)]
pub struct ValueFormat {
    /// this format should have these entries in this order
    pub entries: Vec<ValueEntry>,
}

impl ValueFormat {
    /// Create a ValueFormat for a single value type
    pub fn single(value_type: ValueType) -> Self {
        Self {
            entries: vec![ValueEntry {
                types: vec![value_type],
            }],
        }
    }

    /// Create a ValueFormat that accepts one of multiple value types
    pub fn one_of(value_types: Vec<ValueType>) -> Self {
        Self {
            entries: vec![ValueEntry {
                types: value_types,
            }],
        }
    }

    /// Create a ValueFormat for keywords only
    pub fn keywords(keywords: &[&'static str]) -> Self {
        Self {
            entries: vec![ValueEntry {
                types: keywords.iter().map(|&k| ValueType::Keyword(k)).collect(),
            }],
        }
    }

    /// Create a ValueFormat for a sequence of specific value types
    pub fn sequence(value_types: Vec<ValueType>) -> Self {
        Self {
            entries: value_types.into_iter().map(|vt| ValueEntry {
                types: vec![vt],
            }).collect(),
        }
    }

    /// Check if a declaration node matches this value format
    /// 
    /// The declaration node should have the structure:
    /// - child(0): property_name
    /// - child(1): colon ":"
    /// - child(2..): value nodes to validate against this format
    /// 
    /// Special handling for CSS variables (var(--name)):
    /// - var() calls are treated as wildcards that can match 0-n values
    /// - If any var() is present, we validate non-var values and return true if they could potentially match
    pub fn is_match(&self, declaration_node: Node, content: &str) -> bool {
        // Verify this is a declaration node
        if declaration_node.kind() != "declaration" {
            return false;
        }

        // Check minimum structure: property_name + colon
        if declaration_node.child_count() < 2 {
            return false;
        }

        // Verify first child is property_name
        if let Some(first_child) = declaration_node.child(0) {
            if first_child.kind() != "property_name" {
                return false;
            }
        } else {
            return false;
        }

        // Verify second child is colon
        if let Some(second_child) = declaration_node.child(1) {
            if second_child.kind() != ":" {
                return false;
            }
        } else {
            return false;
        }

        // Collect value nodes (everything after the colon)
        let mut value_nodes = Vec::new();
        for i in 2..declaration_node.child_count() {
            if let Some(child) = declaration_node.child(i) {
                // Skip semicolons and whitespace
                if child.kind() != ";" && !child.kind().is_empty() {
                    value_nodes.push(child);
                }
            }
        }

        // Check for CSS variables (var() calls)
        let has_variables = value_nodes.iter().any(|node| self.is_css_variable(*node, content));
        
        if has_variables {
            // With variables present, we use flexible matching
            // Separate variable and non-variable nodes
            let non_var_nodes: Vec<_> = value_nodes.iter()
                .filter(|node| !self.is_css_variable(**node, content))
                .collect();
            
            // If we have more non-variable nodes than format entries, it's definitely invalid
            if non_var_nodes.len() > self.entries.len() {
                return false;
            }
            
            // Try to match non-variable nodes against format entries
            // We'll be permissive here - if any reasonable assignment could work, return true
            if non_var_nodes.is_empty() {
                // Only variables - always valid since variables can match anything
                return true;
            }
            
            // Check if non-variable nodes can match any subset of our format entries
            return self.can_match_subset(&non_var_nodes, content);
        } else {
            // No variables - use strict matching
            if value_nodes.len() != self.entries.len() {
                return false;
            }

            // Validate each value node against corresponding entry
            for (value_node, entry) in value_nodes.iter().zip(&self.entries) {
                if !self.is_value_node_valid(*value_node, entry, content) {
                    return false;
                }
            }

            true
        }
    }

    /// Check if a value node matches any of the types in a ValueEntry
    fn is_value_node_valid(&self, value_node: Node, entry: &ValueEntry, content: &str) -> bool {
        for value_type in &entry.types {
            if self.is_node_of_type(value_node, *value_type, content) {
                return true;
            }
        }
        false
    }

    /// Check if a node represents a CSS variable (var() call)
    fn is_css_variable(&self, node: Node, content: &str) -> bool {
        if node.kind() == "call_expression" {
            // Check if the function name is "var"
            if let Some(function_name) = node.child(0) {
                let name_text = &content[function_name.start_byte()..function_name.end_byte()];
                return name_text == "var";
            }
        }
        false
    }

    /// Check if non-variable nodes can match any subset of format entries
    /// This is a permissive check for when variables are present
    fn can_match_subset(&self, non_var_nodes: &[&Node], content: &str) -> bool {
        // If we have no format entries, only variables can be valid
        if self.entries.is_empty() {
            return non_var_nodes.is_empty();
        }

        // Try to find a valid assignment of non-variable nodes to format entries
        // For simplicity, we'll check if each non-variable node can match at least one format entry
        for node in non_var_nodes {
            let mut found_match = false;
            for entry in &self.entries {
                if self.is_value_node_valid(**node, entry, content) {
                    found_match = true;
                    break;
                }
            }
            if !found_match {
                return false;
            }
        }
        
        true
    }

    /// Check if a node matches a specific ValueType
    fn is_node_of_type(&self, node: Node, value_type: ValueType, content: &str) -> bool {
        let node_kind = node.kind();
        let node_text = node.utf8_text(content.as_bytes()).unwrap_or("");

        match value_type {
            ValueType::Length => {
                // Length can be: integer_value with px/% unit, or plain "0"
                match node_kind {
                    "integer_value" | "float_value" => {
                        // Check if it has a length unit (px, %) or is unitless 0
                        // First check if there's a unit child
                        if node.child_count() > 1 {
                            if let Some(unit_child) = node.child(1) {
                                if unit_child.kind() == "unit" {
                                    let unit = unit_child.utf8_text(content.as_bytes()).unwrap_or("");
                                    return unit == "px" || unit == "%";
                                }
                            }
                        }
                        // If no unit child, check if the text itself contains the unit
                        if node_text.ends_with("px") || node_text.ends_with("%") {
                            return true;
                        }
                        // Unitless number - only valid if it's 0
                        node_text == "0"
                    }
                    "plain_value" => {
                        // Check for plain "0" or values with length units
                        node_text == "0" || node_text.ends_with("px") || node_text.ends_with("%")
                    }
                    _ => false,
                }
            }
            ValueType::Number => {
                // Any numeric value without unit restrictions
                matches!(node_kind, "integer_value" | "float_value" | "plain_value") &&
                node_text.parse::<f64>().is_ok()
            }
            ValueType::Integer => {
                // Integer values only
                matches!(node_kind, "integer_value" | "plain_value") &&
                node_text.parse::<i32>().is_ok()
            }
            ValueType::String => {
                // String literals
                node_kind == "string_value" || node_kind == "plain_value"
            }
            ValueType::Time => {
                // Time values with s or ms units
                match node_kind {
                    "integer_value" | "float_value" => {
                        if let Some(unit_child) = node.child(1) {
                            let unit = unit_child.utf8_text(content.as_bytes()).unwrap_or("");
                            unit == "s" || unit == "ms"
                        } else {
                            false
                        }
                    }
                    "plain_value" => {
                        node_text.ends_with("s") || node_text.ends_with("ms")
                    }
                    _ => false,
                }
            }
            ValueType::Color => {
                // Color values: hex, named colors, rgb(), rgba(), etc.
                match node_kind {
                    "color_value" => true,
                    "call_expression" => {
                        // Check for color functions like rgb(), rgba()
                        if let Some(func_name) = node.child(0) {
                            let func_text = func_name.utf8_text(content.as_bytes()).unwrap_or("");
                            matches!(func_text, "rgb" | "rgba" | "hsl" | "hsla")
                        } else {
                            false
                        }
                    }
                    "plain_value" => {
                        // Named colors or hex values
                        if node_text.starts_with('#') {
                            // Validate hex color format
                            let hex_part = &node_text[1..];
                            (hex_part.len() == 3 || hex_part.len() == 6) && 
                            hex_part.chars().all(|c| c.is_ascii_hexdigit())
                        } else {
                            // Check against comprehensive color keywords from UssDefinitions
                            let definitions = UssDefinitions::new();
                            definitions.is_valid_color_keyword(node_text)
                        }
                    }
                    _ => false,
                }
            }
            ValueType::Angle => {
                // Angle values with deg, rad, grad, turn units
                match node_kind {
                    "integer_value" | "float_value" => {
                        if let Some(unit_child) = node.child(1) {
                            let unit = unit_child.utf8_text(content.as_bytes()).unwrap_or("");
                            matches!(unit, "deg" | "rad" | "grad" | "turn")
                        } else {
                            false
                        }
                    }
                    "plain_value" => {
                        node_text.ends_with("deg") || node_text.ends_with("rad") ||
                        node_text.ends_with("grad") || node_text.ends_with("turn")
                    }
                    _ => false,
                }
            }
            ValueType::Keyword(expected_keyword) => {
                // Exact keyword match
                node_kind == "plain_value" && node_text == expected_keyword
            }
            ValueType::Asset => {
                // Asset references: url() or resource() functions
                match node_kind {
                    "call_expression" => {
                        if let Some(func_name) = node.child(0) {
                            let func_text = func_name.utf8_text(content.as_bytes()).unwrap_or("");
                            matches!(func_text, "url" | "resource")
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
            ValueType::PropertyName => {
                // Property names for animation properties
                node_kind == "plain_value" && 
                node_text.chars().all(|c| c.is_alphanumeric() || c == '-')
            }
        }
    }
}



/// Complete value specification for a property
#[derive(Debug, Clone)]
pub struct ValueSpec {
    /// All possible value formats for this property
    pub formats: Vec<ValueFormat>
}

impl ValueSpec {
    /// Create a ValueSpec for a single value type
    pub fn single(value_type: ValueType) -> Self {
        Self {
            formats: vec![ValueFormat::single(value_type)],
        }
    }

    /// Create a ValueSpec for color values (hex, keywords, rgb, rgba)
    pub fn color() -> Self {
        Self::single(ValueType::Color)
    }

    /// Create a ValueSpec for shorthand properties (1-4 values of the same type)
    pub fn repeat(value_type: ValueType, min_count: usize, max_count: usize) -> Self {
        let mut formats = Vec::new();
        
        for count in min_count..=max_count {
            let mut entries = Vec::new();
            for _i in 0..count {
                entries.push(ValueEntry {
                    types: vec![value_type],
                });
            }
            formats.push(ValueFormat { entries });
        }
        
        Self { formats }
    }

    /// Create a ValueSpec that accepts one of multiple value types
    pub fn one_of(value_types: Vec<ValueType>) -> Self {
        Self {
            formats: vec![ValueFormat::one_of(value_types)],
        }
    }

    /// Create a ValueSpec for keywords only
    pub fn keywords(keywords: &[&'static str]) -> Self {
        Self {
            formats: vec![ValueFormat::keywords(keywords)],
        }
    }

    /// Create a ValueSpec for a sequence of specific value types
    pub fn sequence(value_types: Vec<ValueType>) -> Self {
        Self {
            formats: vec![ValueFormat::sequence(value_types)],
        }
    }

    /// Create a ValueSpec with multiple possible formats
    pub fn multiple_formats(formats: Vec<ValueFormat>) -> Self {
        Self { formats }
    }
}
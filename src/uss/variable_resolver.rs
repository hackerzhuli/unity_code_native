//! Variable resolution for USS documents.
//!
//! This module provides functionality to extract and resolve CSS custom properties
//! (variables) from USS documents, handling dependencies, circular references, and
//! ambiguous definitions.
//!  
//! supports CSS custom property (variable) resolution with the following limitations:
//! 
//! - **Ambiguous Variables**: When multiple variables with the same name are defined 
//!   (either within this document or from imported documents), the resolution becomes 
//!   ambiguous and cannot be determined. This is different from a variable that doesn't exist.
//!   This does not mean that the uss is incorrect or one variable will override the other, it just means that we don't have a prefect way to resolve the variable's value.
//!   For example, the runtime engine may find no conflicts or override between the same named variables at all, because of the different selectors that they are defined in.
//! 
//! - **Dependency Resolution**: Variables can depend on other variables. The resolver 
//!   attempts to resolve dependencies recursively, but circular dependencies will result 
//!   in unresolved status.
//! 
//! - **Resolution Status**: Variables can be in one of three states:
//!   - `Resolved`: Successfully resolved to concrete values
//!   - `Unresolved`: Exists but cannot be resolved due to missing dependencies or circular references

use std::collections::{HashMap, HashSet};
use tower_lsp::lsp_types::Range;
use tree_sitter::Node;
use crate::uss::definitions::UssDefinitions;

/// A concrete USS value that represents a single valid value in USS
#[derive(Debug, Clone, PartialEq)]
pub enum UssValue {
    /// Length values (e.g., "10px", "50%", "0")
    Length { value: f64, unit: Option<String> },
    /// Numeric values (unitless numbers)
    Number(f64),
    /// Integer values
    Integer(i32),
    /// String literals
    String(String),
    /// Time values (e.g., "1s", "500ms")
    Time { value: f64, unit: String },
    /// Color values (hex, named colors, rgb functions)
    Color(String),
    /// Angle values (e.g., "45deg", "1.5rad")
    Angle { value: f64, unit: String },
    /// Keyword values
    Keyword(String),
    /// Asset references (url(), resource()) - kept as-is
    Asset(String),
    /// Property names for animations
    PropertyName(String),
    /// Variable reference that couldn't be resolved
    Variable(String),
}

impl UssValue {
    /// Parse a USS value from a tree-sitter node
    pub fn from_node(node: Node, content: &str) -> Option<Self> {
        let node_kind = node.kind();
        let node_text = node.utf8_text(content.as_bytes()).ok()?;
        
        match node_kind {
            "integer_value" | "float_value" => {
                // Check if it has a unit
                if node.child_count() > 1 {
                    if let Some(unit_child) = node.child(1) {
                        if unit_child.kind() == "unit" {
                            let unit = unit_child.utf8_text(content.as_bytes()).ok()?.to_string();
                            let value_text = node.child(0)?.utf8_text(content.as_bytes()).ok()?;
                            let value: f64 = value_text.parse().ok()?;
                            
                            return match unit.as_str() {
                                "px" | "%" => Some(UssValue::Length { value, unit: Some(unit) }),
                                "s" | "ms" => Some(UssValue::Time { value, unit }),
                                "deg" | "rad" | "grad" | "turn" => Some(UssValue::Angle { value, unit }),
                                _ => Some(UssValue::Number(value)),
                            };
                        }
                    }
                }
                
                // No unit - parse as number or length (if 0)
                if let Ok(int_val) = node_text.parse::<i32>() {
                    if node_text == "0" {
                        Some(UssValue::Length { value: 0.0, unit: None })
                    } else {
                        Some(UssValue::Integer(int_val))
                    }
                } else if let Ok(float_val) = node_text.parse::<f64>() {
                    Some(UssValue::Number(float_val))
                } else {
                    None
                }
            }
            "plain_value" => {
                // Handle various plain value types
                if node_text == "0" {
                    Some(UssValue::Length { value: 0.0, unit: None })
                } else if node_text.ends_with("px") || node_text.ends_with("%") {
                    let (value_str, unit) = if node_text.ends_with("px") {
                        (&node_text[..node_text.len()-2], "px")
                    } else {
                        (&node_text[..node_text.len()-1], "%")
                    };
                    if let Ok(value) = value_str.parse::<f64>() {
                        Some(UssValue::Length { value, unit: Some(unit.to_string()) })
                    } else {
                        None
                    }
                } else if node_text.ends_with("s") || node_text.ends_with("ms") {
                    let (value_str, unit) = if node_text.ends_with("ms") {
                        (&node_text[..node_text.len()-2], "ms")
                    } else {
                        (&node_text[..node_text.len()-1], "s")
                    };
                    if let Ok(value) = value_str.parse::<f64>() {
                        Some(UssValue::Time { value, unit: unit.to_string() })
                    } else {
                        None
                    }
                } else if node_text.ends_with("deg") || node_text.ends_with("rad") || 
                         node_text.ends_with("grad") || node_text.ends_with("turn") {
                    let unit_len = if node_text.ends_with("grad") || node_text.ends_with("turn") { 4 } else { 3 };
                    let value_str = &node_text[..node_text.len()-unit_len];
                    let unit = &node_text[node_text.len()-unit_len..];
                    if let Ok(value) = value_str.parse::<f64>() {
                        Some(UssValue::Angle { value, unit: unit.to_string() })
                    } else {
                        None
                    }
                } else if node_text.starts_with('#') {
                    // Hex color
                    let hex_part = &node_text[1..];
                    if (hex_part.len() == 3 || hex_part.len() == 6) && 
                       hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
                        Some(UssValue::Color(node_text.to_string()))
                    } else {
                        None
                    }
                } else if let Ok(int_val) = node_text.parse::<i32>() {
                    Some(UssValue::Integer(int_val))
                } else if let Ok(float_val) = node_text.parse::<f64>() {
                    Some(UssValue::Number(float_val))
                } else {
                    // Check if it's a color keyword
                    let definitions = UssDefinitions::new();
                    if definitions.is_valid_color_keyword(node_text) {
                        Some(UssValue::Color(node_text.to_string()))
                    } else if node_text.chars().all(|c| c.is_alphanumeric() || c == '-') {
                        // Could be a keyword or property name
                        Some(UssValue::Keyword(node_text.to_string()))
                    } else {
                        None
                    }
                }
            }
            "string_value" => {
                Some(UssValue::String(node_text.to_string()))
            }
            "color_value" => {
                Some(UssValue::Color(node_text.to_string()))
            }
            "call_expression" => {
                if let Some(func_name) = node.child(0) {
                    let func_text = func_name.utf8_text(content.as_bytes()).ok()?;
                    match func_text {
                        "url" | "resource" => Some(UssValue::Asset(node_text.to_string())),
                        "rgb" | "rgba" | "hsl" | "hsla" => Some(UssValue::Color(node_text.to_string())),
                        "var" => {
                            // Extract variable name from var(--variable-name)
                            if node_text.len() > 6 { // "var()" is 5 chars minimum
                                let var_ref = &node_text[4..node_text.len()-1]; // Remove "var(" and ")"
                                let var_name = if var_ref.starts_with("--") {
                                    &var_ref[2..] // Remove -- prefix
                                } else {
                                    var_ref
                                };
                                Some(UssValue::Variable(var_name.to_string()))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Convert the UssValue back to a string representation
    pub fn to_string(&self) -> String {
        match self {
            UssValue::Length { value, unit } => {
                if let Some(unit) = unit {
                    format!("{}{}", value, unit)
                } else {
                    value.to_string()
                }
            }
            UssValue::Number(n) => n.to_string(),
            UssValue::Integer(i) => i.to_string(),
            UssValue::String(s) => s.clone(),
            UssValue::Time { value, unit } => format!("{}{}", value, unit),
            UssValue::Color(c) => c.clone(),
            UssValue::Angle { value, unit } => format!("{}{}", value, unit),
            UssValue::Keyword(k) => k.clone(),
            UssValue::Asset(a) => a.clone(),
            UssValue::PropertyName(p) => p.clone(),
            UssValue::Variable(v) => format!("var(--{})", v),
        }
    }
}

/// Status of a variable's resolution
#[derive(Debug, Clone, PartialEq)]
pub enum VariableResolutionStatus {
    /// Variable has been successfully resolved to concrete values
    Resolved(Vec<UssValue>),
    /// Variable cannot be resolved (circular dependency, missing dependency, etc.)
    Unresolved,
    /// Multiple definitions exist for this variable name
    Ambiguous,
}

/// Definition of a CSS custom property variable
#[derive(Debug, Clone)]
pub struct VariableDefinition {
    pub name: String,
    pub values: Vec<UssValue>,
    pub range: Range,
    pub status: VariableResolutionStatus,
}

/// Temporary storage for declaration information during extraction
#[derive(Debug, Clone)]
struct DeclarationInfo {
    name: String,
    start_byte: usize,
    end_byte: usize,
    range: Range,
}

/// Handles variable extraction and resolution for USS documents
#[derive(Clone, Debug)]
pub struct VariableResolver {
    variables: HashMap<String, VariableDefinition>,
    resolved: bool,
}

impl VariableResolver {
    /// Create a new variable resolver
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            resolved: false,
        }
    }

    /// Clear all variables and mark as unresolved
    pub fn clear(&mut self) {
        self.variables.clear();
        self.resolved = false;
    }

    /// Extract variables from a syntax tree and resolve them
    pub fn extract_and_resolve(&mut self, root_node: Node, content: &str) {
        self.variables.clear();
        
        // Step 1: Extract declaration nodes into a temporary hashmap
        let mut declarations = HashMap::<String, Vec<DeclarationInfo>>::new();
        Self::extract_variables_from_node(root_node, content, &mut declarations);
        
        // Step 2: Resolve variables and create VariableDefinitions
        self.resolve_variables(declarations, content);
        self.resolved = true;
    }

    /// Get all variables
    pub fn get_variables(&self) -> &HashMap<String, VariableDefinition> {
        &self.variables
    }

    /// Get a specific variable by name
    pub fn get_variable(&self, name: &str) -> Option<&VariableDefinition> {
        self.variables.get(name)
    }

    /// Check if variables have been resolved
    pub fn are_variables_resolved(&self) -> bool {
        self.resolved
    }

    /// Step 1: Recursively extract variable declaration nodes from a syntax tree
    fn extract_variables_from_node(node: Node, content: &str, declarations: &mut HashMap<String, Vec<DeclarationInfo>>) {
        // Look for CSS custom property declarations (--variable-name: value;)
        if node.kind() == "declaration" {
            // Try different ways to find the property name
            let property_text = if let Some(property_node) = node.child_by_field_name("property") {
                Self::node_text(property_node, content)
            } else if let Some(first_child) = node.child(0) {
                // Fallback: use first child if field name doesn't work
                Self::node_text(first_child, content)
            } else {
                String::new()
            };
            
            if property_text.starts_with("--") {
                let variable_name = property_text[2..].to_string(); // Remove -- prefix
                let range = Self::node_to_range(node);
                
                let declaration_info = DeclarationInfo {
                    name: variable_name.clone(),
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    range,
                };
                
                declarations.entry(variable_name)
                    .or_insert_with(Vec::new)
                    .push(declaration_info);
            }
        }
        
        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::extract_variables_from_node(child, content, declarations);
        }
    }



    /// Get text content of a node with content
    fn node_text(node: Node, content: &str) -> String {
        content[node.start_byte()..node.end_byte()].to_string()
    }

    /// Invalidate a variable and all variables that depend on it
    pub fn invalidate_variable(&mut self, var_name: &str) {
        let mut to_invalidate = HashSet::new();
        to_invalidate.insert(var_name.to_string());
        
        // Find all variables that depend on the invalidated variable
        let mut changed = true;
        while changed {
            changed = false;
            for (name, var_def) in &self.variables {
                if to_invalidate.contains(name) {
                    continue;
                }
                
                // Check if this variable depends on any invalidated variable
                for value in &var_def.values {
                    if let UssValue::Variable(dep_name) = value {
                        if to_invalidate.contains(dep_name) {
                            to_invalidate.insert(name.clone());
                            changed = true;
                            break;
                        }
                    }
                }
            }
        }
        
        // Mark all identified variables as unresolved
        for var_name in to_invalidate {
            if let Some(var_def) = self.variables.get_mut(&var_name) {
                var_def.status = VariableResolutionStatus::Unresolved;
            }
        }
    }

    /// Step 2: Resolve variables from declaration nodes and create VariableDefinitions
    fn resolve_variables(&mut self, declarations: HashMap<String, Vec<DeclarationInfo>>, content: &str) {
        for (var_name, declaration_infos) in declarations {
            // If multiple declarations exist, mark as ambiguous
            if declaration_infos.len() > 1 {
                // Use the first declaration for the definition but mark as ambiguous
                let first_decl = &declaration_infos[0];
                let definition = VariableDefinition {
                    name: var_name.clone(),
                    values: Vec::new(),
                    range: first_decl.range,
                    status: VariableResolutionStatus::Ambiguous,
                };
                self.variables.insert(var_name, definition);
            } else if let Some(decl_info) = declaration_infos.first() {
                // Single declaration - extract values and resolve
                let values = self.extract_values_from_declaration_bytes(decl_info.start_byte, decl_info.end_byte, content);
                let definition = VariableDefinition {
                    name: var_name.clone(),
                    values,
                    range: decl_info.range,
                    status: VariableResolutionStatus::Unresolved,
                };
                self.variables.insert(var_name, definition);
            }
        }
        
        // Now resolve all variables
        self.resolve_all_variables();
    }
    
    /// Extract UssValues from a declaration using byte positions
    fn extract_values_from_declaration_bytes(&self, start_byte: usize, end_byte: usize, content: &str) -> Vec<UssValue> {
        let mut values = Vec::new();
        
        // Extract the declaration text
        let decl_text = &content[start_byte..end_byte];
        
        // Find the colon and extract everything after it until semicolon
        if let Some(colon_pos) = decl_text.find(':') {
            let value_part = &decl_text[colon_pos + 1..];
            let value_text = if let Some(semicolon_pos) = value_part.find(';') {
                &value_part[..semicolon_pos]
            } else {
                value_part
            }.trim();
            
            // Parse the value text - this is a simplified approach
            // In a real implementation, you'd want to properly parse this
            if value_text.starts_with("var(") && value_text.ends_with(")") {
                // Extract variable reference
                let var_ref = &value_text[4..value_text.len()-1]; // Remove "var(" and ")"
                let var_name = if var_ref.starts_with("--") {
                    &var_ref[2..] // Remove -- prefix
                } else {
                    var_ref
                };
                values.push(UssValue::Variable(var_name.to_string()));
            } else if value_text.starts_with('#') && value_text.len() > 1 {
                // Color value
                values.push(UssValue::Color(value_text.to_string()));
            } else if value_text.ends_with("px") {
                // Length value
                if let Ok(num) = value_text[..value_text.len()-2].parse::<f64>() {
                    values.push(UssValue::Length { value: num, unit: Some("px".to_string()) });
                }
            } else if let Ok(num) = value_text.parse::<f64>() {
                // Numeric value
                values.push(UssValue::Number(num));
            } else {
                // Keyword or other value
                values.push(UssValue::Keyword(value_text.to_string()));
            }
        }
        
        values
    }

    /// Convert a tree-sitter node to LSP range
    fn node_to_range(node: Node) -> Range {
        Range {
            start: tower_lsp::lsp_types::Position {
                line: node.start_position().row as u32,
                character: node.start_position().column as u32,
            },
            end: tower_lsp::lsp_types::Position {
                line: node.end_position().row as u32,
                character: node.end_position().column as u32,
            },
        }
    }

    /// Resolve all variables, handling dependencies and circular references
    fn resolve_all_variables(&mut self) {
        let mut resolved_vars = HashSet::new();
        let variable_names: Vec<String> = self.variables.keys().cloned().collect();
        
        for var_name in variable_names {
            if !resolved_vars.contains(&var_name) {
                let mut visiting = HashSet::new();
                self.resolve_variable_recursive(&var_name, &mut visiting, &mut resolved_vars);
            }
        }
    }

    /// Recursively resolve a variable, detecting circular dependencies
    fn resolve_variable_recursive(
        &mut self,
        var_name: &str,
        visiting: &mut HashSet<String>,
        resolved: &mut HashSet<String>,
    ) -> Option<Vec<UssValue>> {
        // If already resolved, return the cached result
        if resolved.contains(var_name) {
            if let Some(var_def) = self.variables.get(var_name) {
                if let VariableResolutionStatus::Resolved(values) = &var_def.status {
                    return Some(values.clone());
                }
            }
            return None;
        }
        
        // Check for circular dependency
        if visiting.contains(var_name) {
            // Mark as unresolved due to circular dependency
            if let Some(var_def) = self.variables.get_mut(var_name) {
                var_def.status = VariableResolutionStatus::Unresolved;
            }
            return None;
        }
        
        // Get the variable definition
        let var_def = match self.variables.get(var_name) {
            Some(def) => def.clone(),
            None => return None,
        };
        
        // Skip if already marked as ambiguous
        if matches!(var_def.status, VariableResolutionStatus::Ambiguous) {
            resolved.insert(var_name.to_string());
            return None;
        }
        
        visiting.insert(var_name.to_string());
        
        // Resolve the variable values
        let mut resolved_values = Vec::new();
        let mut has_unresolved_deps = false;
        
        for value in &var_def.values {
            match value {
                UssValue::Variable(dep_name) => {
                    // Recursively resolve the dependency
                    if let Some(dep_values) = self.resolve_variable_recursive(dep_name, visiting, resolved) {
                        resolved_values.extend(dep_values);
                    } else {
                        has_unresolved_deps = true;
                        break;
                    }
                }
                _ => {
                    // Non-variable value, add as-is
                    resolved_values.push(value.clone());
                }
            }
        }
        
        visiting.remove(var_name);
        resolved.insert(var_name.to_string());
        
        // Update the variable status
        if let Some(var_def) = self.variables.get_mut(var_name) {
            if has_unresolved_deps {
                var_def.status = VariableResolutionStatus::Unresolved;
                None
            } else {
                var_def.status = VariableResolutionStatus::Resolved(resolved_values.clone());
                Some(resolved_values)
            }
        } else {
            None
        }
    }
}

impl Default for VariableResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uss::parser::UssParser;

    fn create_test_tree(content: &str) -> Option<tree_sitter::Tree> {
        let mut parser = UssParser::new().unwrap();
        parser.parse(content, None)
    }

    #[test]
    fn test_variable_extraction() {
        let content = r#":root {
    --primary-color: #ff0000;
    --secondary-color: #00ff00;
    --margin: 10px;
}"#;
        
        let tree = create_test_tree(content).unwrap();
        let mut resolver = VariableResolver::new();
        resolver.extract_and_resolve(tree.root_node(), content);
        
        let variables = resolver.get_variables();
        assert!(variables.len() > 0, "Should find at least one variable, found: {}", variables.len());
    }

    #[test]
    fn test_variable_resolution_simple() {
        let content = r#"
            :root {
                --primary-color: #ff0000;
                --text-color: var(--primary-color);
            }
        "#;
        
        let tree = create_test_tree(content).unwrap();
        let mut resolver = VariableResolver::new();
        resolver.extract_and_resolve(tree.root_node(), content);
        
        let primary_var = resolver.get_variable("primary-color").unwrap();
        assert!(matches!(primary_var.status, VariableResolutionStatus::Resolved(_)));
        
        let text_var = resolver.get_variable("text-color").unwrap();
        assert!(matches!(text_var.status, VariableResolutionStatus::Resolved(_)));
        
        // Check that the resolved value is correct
        if let VariableResolutionStatus::Resolved(values) = &text_var.status {
            assert_eq!(values.len(), 1);
            assert!(matches!(values[0], UssValue::Color(_)));
        }
    }

    #[test]
    fn test_variable_resolution_circular() {
        let content = r#"
            :root {
                --color-a: var(--color-b);
                --color-b: var(--color-a);
            }
        "#;
        
        let tree = create_test_tree(content).unwrap();
        let mut resolver = VariableResolver::new();
        resolver.extract_and_resolve(tree.root_node(), content);
        
        let color_a = resolver.get_variable("color-a").unwrap();
        let color_b = resolver.get_variable("color-b").unwrap();
        
        // Both should be unresolved due to circular dependency
        assert!(matches!(color_a.status, VariableResolutionStatus::Unresolved));
        assert!(matches!(color_b.status, VariableResolutionStatus::Unresolved));
    }

    #[test]
    fn test_variable_resolution_ambiguous() {
        let content = r#"
            .class1 {
                --primary-color: #ff0000;
            }
            .class2 {
                --primary-color: #00ff00;
            }
        "#;
        
        let tree = create_test_tree(content).unwrap();
        let mut resolver = VariableResolver::new();
        resolver.extract_and_resolve(tree.root_node(), content);
        
        let primary_var = resolver.get_variable("primary-color").unwrap();
        assert!(matches!(primary_var.status, VariableResolutionStatus::Ambiguous));
    }

    #[test]
    fn test_variable_invalidation() {
        let content = r#"
            :root {
                --primary-color: #ff0000;
                --text-color: var(--primary-color);
            }
        "#;
        
        let tree = create_test_tree(content).unwrap();
        let mut resolver = VariableResolver::new();
        resolver.extract_and_resolve(tree.root_node(), content);
        
        // Initially both should be resolved
        let primary_var = resolver.get_variable("primary-color").unwrap();
        assert!(matches!(primary_var.status, VariableResolutionStatus::Resolved(_)));
        
        let text_var = resolver.get_variable("text-color").unwrap();
        assert!(matches!(text_var.status, VariableResolutionStatus::Resolved(_)));
        
        // Invalidate primary-color
        resolver.invalidate_variable("primary-color");
        
        // primary-color should be unresolved, text-color should also be unresolved
        let primary_var = resolver.get_variable("primary-color").unwrap();
        assert!(matches!(primary_var.status, VariableResolutionStatus::Unresolved));
        
        let text_var = resolver.get_variable("text-color").unwrap();
        assert!(matches!(text_var.status, VariableResolutionStatus::Unresolved));
    }
}
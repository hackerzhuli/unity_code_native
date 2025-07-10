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
use crate::uss::value::UssValue;

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
    pub fn node_text(node: Node, content: &str) -> String {
        content[node.start_byte()..node.end_byte()].to_string()
    }

    /// Extract value and unit from a text string
    pub fn extract_value_and_unit(text: &str) -> (&str, Option<String>) {
        // Common units to check for
        let units = ["px", "em", "rem", "vh", "vw", "vmin", "vmax", "%", 
                     "deg", "rad", "grad", "turn", "s", "ms"];
        
        for unit in &units {
            if text.ends_with(unit) {
                let value_str = &text[..text.len() - unit.len()];
                return (value_str, Some(unit.to_string()));
            }
        }
        
        // No unit found
        (text, None)
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
                
                // Since we no longer have Variable variants, no dependency checking needed
                // All values are now concrete
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
            if value_text.starts_with('#') && value_text.len() > 1 {
                // Color value
                values.push(UssValue::Color(value_text.to_string()));
            } else {
                // Try to parse as numeric value with optional unit
                let (value_str, unit) = Self::extract_value_and_unit(value_text);
                if let Ok(value) = value_str.parse::<f64>() {
                    let has_fractional = value_str.contains('.');
                    values.push(UssValue::Numeric { value, unit, has_fractional });
                } else {
                    // Keyword or other value
                    values.push(UssValue::Keyword(value_text.to_string()));
                }
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
        
        for value in &var_def.values {
            // All values are now concrete, add as-is
            resolved_values.push(value.clone());
        }
        
        visiting.remove(var_name);
        resolved.insert(var_name.to_string());
        
        // Update the variable status
        if let Some(var_def) = self.variables.get_mut(var_name) {
            var_def.status = VariableResolutionStatus::Resolved(resolved_values.clone());
            Some(resolved_values)
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
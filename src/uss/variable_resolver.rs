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
use crate::uss::constants::*;
use crate::uss::definitions::UssDefinitions;

/// Status and value of a CSS custom property variable
#[derive(Debug, Clone, PartialEq)]
pub enum VariableStatus {
    /// Variable has been successfully resolved to concrete values
    Resolved(Vec<UssValue>),
    /// Variable cannot be resolved (circular dependency, missing dependency, etc.)
    Unresolved,
    /// Multiple definitions exist for this variable name
    Ambiguous,
    /// Variable declaration has parsing errors
    Error,
}

/// Handles variable extraction and resolution for USS documents
#[derive(Clone, Debug)]
pub struct VariableResolver {
    variables: HashMap<String, VariableStatus>,
    /// Temporary storage for parsed values during extraction
    parsed_values: HashMap<String, Vec<UssValue>>,
    resolved: bool,
    /// USS language definitions for value parsing
    definitions: UssDefinitions,
}

impl VariableResolver {
    /// Create a new variable resolver
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parsed_values: HashMap::new(),
            resolved: false,
            definitions: UssDefinitions::new(),
        }
    }

    /// Clear all variables and mark as unresolved
    pub fn clear(&mut self) {
        self.variables.clear();
        self.resolved = false;
    }

    /// Extract variables from a syntax tree and resolve them
    pub fn add_variables_from_tree(&mut self, root_node: Node, content: &str) {
        self.add_variables_from_tree_with_source_url(root_node, content, None);
    }
    
    /// Extract variables from a syntax tree and resolve them with source URL
    pub fn add_variables_from_tree_with_source_url(&mut self, root_node: Node, content: &str, source_url: Option<&url::Url>) {
        self.variables.clear();
        
        // Extract variables and their values in a single pass
        self.extract_variables_from_node_with_source_url(root_node, content, source_url);
        
        // Resolve all variable dependencies
        self.resolve_all_variables();
        self.resolved = true;
    }

    /// Get all variables
    pub fn get_variables(&self) -> &HashMap<String, VariableStatus> {
        &self.variables
    }

    /// Get a specific variable by name
    pub fn get_variable(&self, name: &str) -> Option<&VariableStatus> {
        self.variables.get(name)
    }

    /// Check if variables have been resolved
    pub fn are_variables_resolved(&self) -> bool {
        self.resolved
    }

    /// Extract variable declarations and their values from a syntax tree in a single pass with source URL
    fn extract_variables_from_node_with_source_url(&mut self, node: Node, content: &str, source_url: Option<&url::Url>) {
        // Look for CSS custom property declarations (--variable-name: value;)
        if node.kind() == NODE_DECLARATION {
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
                
                // Check if this variable already exists (ambiguous case)
                if self.variables.contains_key(&variable_name) {
                    // Mark existing variable as ambiguous
                    self.variables.insert(variable_name.clone(), VariableStatus::Ambiguous);
                    self.parsed_values.remove(&variable_name); // Remove parsed values for ambiguous variables
                } else {
                    // Extract values immediately during traversal
                    match self.extract_values_from_declaration_node_with_source_url(node, content, source_url) {
                        Ok(values) => {
                            // Store parsed values and mark as unresolved initially
                            self.parsed_values.insert(variable_name.clone(), values);
                            self.variables.insert(variable_name, VariableStatus::Unresolved);
                        }
                        Err(_) => {
                            // If extraction fails, record this variable with error status
                            self.variables.insert(variable_name, VariableStatus::Error);
                        }
                    }
                }
            }
        }
        
        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_variables_from_node_with_source_url(child, content, source_url);
        }
    }

    /// Get text content of a node with content
    pub fn node_text(node: Node, content: &str) -> String {
        content[node.start_byte()..node.end_byte()].to_string()
    }
    
    /// Extract UssValues from a declaration node using proper tree-sitter parsing with source URL
    /// Validates strict CSS declaration structure: property : values ;
    fn extract_values_from_declaration_node_with_source_url(&self, declaration_node: Node, content: &str, source_url: Option<&url::Url>) -> Result<Vec<UssValue>, ()> {
        // Use the stored definitions
        let child_count = declaration_node.child_count();
        
        // Validate minimum structure: property + colon + at least one value
        if child_count < 3 {
            return Err(());
        }
        
        // Second child must be colon
        if let Some(colon_node) = declaration_node.child(1) {
            if colon_node.kind() != NODE_COLON {
                return Err(());
            }
        } else {
            return Err(());
        }
        
        // Check if semicolon exists as last child
        let has_semicolon = declaration_node.child(child_count - 1)
            .map(|n| n.kind()) == Some(NODE_SEMICOLON);
        let value_end_index = if has_semicolon { child_count - 1 } else { child_count };
        
        // Extract values between colon and semicolon (or end)
        let mut values = Vec::new();
        for i in 2..value_end_index {
            if let Some(child) = declaration_node.child(i) {
                // Skip whitespace and other non-value nodes
                if child.kind().is_empty() {
                    continue;
                }
                
                // If any value fails to parse, return the error
                let value = UssValue::from_node(child, content, &self.definitions, source_url).map_err(|_| ())?;
                values.push(value);
            }
        }
        
        Ok(values)
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
            if let Some(VariableStatus::Resolved(values)) = self.variables.get(var_name) {
                return Some(values.clone());
            }
            return None;
        }
        
        // Check for circular dependency
        if visiting.contains(var_name) {
            // Mark as unresolved due to circular dependency
            self.variables.insert(var_name.to_string(), VariableStatus::Unresolved);
            return None;
        }
        
        // Get the variable status
        let var_status = match self.variables.get(var_name) {
            Some(status) => status.clone(),
            None => return None,
        };
        
        // Skip if already marked as ambiguous or has errors
        if matches!(var_status, VariableStatus::Ambiguous | VariableStatus::Error) {
            resolved.insert(var_name.to_string());
            return None;
        }
        
        // Get the parsed values
        let parsed_values = match self.parsed_values.get(var_name) {
            Some(values) => values.clone(),
            None => return None,
        };
        
        visiting.insert(var_name.to_string());
        
        // Resolve the variable values
        let mut resolved_values = Vec::new();
        
        for value in &parsed_values {
            match value {
                UssValue::VariableReference(ref_var_name) => {
                    // Recursively resolve the referenced variable
                    if let Some(ref_values) = self.resolve_variable_recursive(ref_var_name, visiting, resolved) {
                        resolved_values.extend(ref_values);
                    } else {
                        // If we can't resolve the reference, mark as unresolved
                        visiting.remove(var_name);
                        self.variables.insert(var_name.to_string(), VariableStatus::Unresolved);
                        return None;
                    }
                }
                _ => {
                    // Concrete value, add as-is
                    resolved_values.push(value.clone());
                }
            }
        }
        
        visiting.remove(var_name);
        resolved.insert(var_name.to_string());
        
        // Update the variable status
        self.variables.insert(var_name.to_string(), VariableStatus::Resolved(resolved_values.clone()));
        Some(resolved_values)
    }
}

impl Default for VariableResolver {
    fn default() -> Self {
        Self::new()
    }
}

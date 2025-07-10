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

/// Status of a variable's resolution
#[derive(Debug, Clone, PartialEq)]
pub enum VariableResolutionStatus {
    /// Variable has been successfully resolved to a concrete value
    Resolved(String),
    /// Variable cannot be resolved (circular dependency, missing dependency, etc.)
    Unresolved,
    /// Multiple definitions exist for this variable name
    Ambiguous,
}

/// Definition of a CSS custom property variable
#[derive(Debug, Clone)]
pub struct VariableDefinition {
    pub name: String,
    pub value_nodes: Vec<String>,
    pub range: Range,
    pub status: VariableResolutionStatus,
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
        Self::extract_variables_from_node(root_node, content, &mut self.variables);
        self.resolve_variables();
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

    /// Recursively extract variables from a syntax tree node
    fn extract_variables_from_node(node: Node, content: &str, variables: &mut HashMap<String, VariableDefinition>) {
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
                
                // Try different ways to find the value
                let value_tokens = if let Some(value_node) = node.child_by_field_name("value") {
                    Self::extract_value_tokens(value_node, content)
                } else {
                    // Fallback: look for value after colon
                    let mut found_colon = false;
                    let mut tokens = Vec::new();
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        let child_text = Self::node_text(child, content);
                        if found_colon && !child_text.trim().is_empty() && child_text != ";" {
                            tokens.push(child_text);
                        } else if child_text == ":" {
                            found_colon = true;
                        }
                    }
                    tokens
                };
                
                if !value_tokens.is_empty() {
                    let range = Self::node_to_range(node);
                    
                    let definition = VariableDefinition {
                        name: variable_name.clone(),
                        value_nodes: value_tokens,
                        range,
                        status: VariableResolutionStatus::Unresolved,
                    };
                    
                    // Check for ambiguous definitions (multiple variables with same name)
                    if variables.contains_key(&variable_name) {
                        // Mark both as ambiguous
                        if let Some(existing) = variables.get_mut(&variable_name) {
                            existing.status = VariableResolutionStatus::Ambiguous;
                        }
                        variables.insert(variable_name, VariableDefinition {
                            status: VariableResolutionStatus::Ambiguous,
                            ..definition
                        });
                    } else {
                        variables.insert(variable_name, definition);
                    }
                }
            }
        }
        
        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::extract_variables_from_node(child, content, variables);
        }
    }

    /// Extract value tokens from a value node
    fn extract_value_tokens(node: Node, content: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            let text = Self::node_text(child, content);
            if !text.trim().is_empty() && text != ";" {
                tokens.push(text.to_string());
            }
        }
        
        // If no child tokens, use the node itself
        if tokens.is_empty() {
            let text = Self::node_text(node, content);
            if !text.trim().is_empty() && text != ";" {
                tokens.push(text.to_string());
            }
        }
        
        tokens
    }

    /// Get text content of a node
    fn node_text(node: Node, content: &str) -> String {
        content[node.start_byte()..node.end_byte()].to_string()
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
    fn resolve_variables(&mut self) {
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
    ) -> Option<String> {
        // If already resolved, return the cached result
        if resolved.contains(var_name) {
            if let Some(var_def) = self.variables.get(var_name) {
                if let VariableResolutionStatus::Resolved(value) = &var_def.status {
                    return Some(value.clone());
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
        
        // Resolve the variable value
        let mut resolved_value = String::new();
        let mut has_unresolved_deps = false;
        
        for token_text in &var_def.value_nodes {
            if token_text.starts_with("var(") && token_text.ends_with(")") {
                // Extract variable name from var(--variable-name)
                let var_ref = &token_text[4..token_text.len()-1]; // Remove "var(" and ")"
                let dep_name = if var_ref.starts_with("--") {
                    &var_ref[2..] // Remove -- prefix
                } else {
                    var_ref
                };
                
                // Recursively resolve the dependency
                if let Some(dep_value) = self.resolve_variable_recursive(dep_name, visiting, resolved) {
                    resolved_value.push_str(&dep_value);
                } else {
                    has_unresolved_deps = true;
                    break;
                }
            } else {
                resolved_value.push_str(token_text);
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
                var_def.status = VariableResolutionStatus::Resolved(resolved_value.clone());
                Some(resolved_value)
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
    }

    #[test]
    fn test_variable_resolution_circular() {
        let content = r#"
            :root {
                --var-a: var(--var-b);
                --var-b: var(--var-a);
            }
        "#;
        
        let tree = create_test_tree(content).unwrap();
        let mut resolver = VariableResolver::new();
        resolver.extract_and_resolve(tree.root_node(), content);
        
        let var_a = resolver.get_variable("var-a").unwrap();
        assert!(matches!(var_a.status, VariableResolutionStatus::Unresolved));
        
        let var_b = resolver.get_variable("var-b").unwrap();
        assert!(matches!(var_b.status, VariableResolutionStatus::Unresolved));
    }

    #[test]
    fn test_variable_resolution_ambiguous() {
        let content = r#"
            :root {
                --primary-color: #ff0000;
            }
            .class {
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
            }
        "#;
        
        let tree = create_test_tree(content).unwrap();
        let mut resolver = VariableResolver::new();
        resolver.extract_and_resolve(tree.root_node(), content);
        
        assert!(resolver.are_variables_resolved());
        assert_eq!(resolver.get_variables().len(), 1);
        
        // Clear variables
        resolver.clear();
        assert!(!resolver.are_variables_resolved());
        assert_eq!(resolver.get_variables().len(), 0);
    }
}
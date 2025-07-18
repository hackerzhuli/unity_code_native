//! Compilation utilities for C# documentation processing
//!
//! This module contains standalone functions for normalizing C# type and member names
//! when processing syntax trees for documentation compilation.

use tokio::net::windows::named_pipe::NamedPipeClient;
use tree_sitter::Node;
use super::constants::*;

/// Normalize a type name from a tree-sitter node
/// Returns the fully qualified type name including namespace
pub fn normalize_type_name(node: Node, source: &str) -> Option<String> {
    // For a class name node, we need to traverse up to find the full namespace path
    let class_name = node.utf8_text(source.as_bytes()).ok()?.to_string();
    
    // Find the class declaration that contains this name node
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == CLASS_DECLARATION {
            // Found the class, now traverse up to collect namespace hierarchy
            let mut namespaces = Vec::new();
            let mut namespace_parent = parent.parent();
            
            while let Some(ns_parent) = namespace_parent {
                if ns_parent.kind() == NAMESPACE_DECLARATION {
                    if let Some(ns_name_node) = ns_parent.child_by_field_name(NAME_FIELD) {
                        if let Ok(ns_name) = ns_name_node.utf8_text(source.as_bytes()) {
                            namespaces.push(ns_name.to_string());
                        }
                    }
                }
                namespace_parent = ns_parent.parent();
            }
            
            // Reverse to get correct order (outermost to innermost)
            namespaces.reverse();
            
            // Combine namespace and class name
            if namespaces.is_empty() {
                return Some(class_name);
            } else {
                return Some(format!("{}.{}", namespaces.join("."), class_name));
            }
        }
        current = parent.parent();
    }
    
    // Fallback: just return the simple name if we can't find the context
    Some(class_name)
}

/// Normalize a member name from a tree-sitter node
pub fn normalize_member_name(node: Node, source: &str) -> Option<String> {
    match node.kind() {
        METHOD_DECLARATION | CONSTRUCTOR_DECLARATION | DESTRUCTOR_DECLARATION => {
            normalize_method_name(node, source)
        }
        FIELD_DECLARATION => {
            // For field declarations, we need to find the variable_declarator
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == VARIABLE_DECLARATION {
                    let mut var_cursor = child.walk();
                    for var_child in child.children(&mut var_cursor) {
                        if var_child.kind() == VARIABLE_DECLARATOR {
                            // The identifier is a direct child of variable_declarator
                            let mut declarator_cursor = var_child.walk();
                            for declarator_child in var_child.children(&mut declarator_cursor) {
                                if declarator_child.kind() == IDENTIFIER {
                                    return declarator_child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                                }
                            }
                        }
                    }
                }
            }
            None
        }
        _ => {
            // For other types (properties, events, etc.), look for the name field or identifier
            if let Some(name_node) = node.child_by_field_name(NAME_FIELD) {
                name_node.utf8_text(source.as_bytes()).ok().map(|s| s.to_string())
            } else {
                // Fallback: find first identifier
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == IDENTIFIER {
                        return child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                    }
                }
                None
            }
        }
    }
}

/// Normalize a method name with parameters from a tree-sitter node
pub fn normalize_method_name(node: Node, source: &str) -> Option<String> {
    let name_node = node.child_by_field_name(NAME_FIELD)?;
    let base_name = name_node.utf8_text(source.as_bytes()).ok()?;
    
    let mut result = base_name.to_string();
    
    // Add generic parameters if present
    if let Some(type_params) = node.child_by_field_name(TYPE_PARAMETERS_FIELD) {
        let generic_text = type_params.utf8_text(source.as_bytes()).ok()?;
        let normalized_generics = normalize_generic_parameters(generic_text);
        result.push_str(&normalized_generics);
    }
    
    // Add parameters
    if let Some(params_node) = node.child_by_field_name(PARAMETERS_FIELD) {
        let params_text = params_node.utf8_text(source.as_bytes()).ok()?;
        let normalized_params = normalize_parameter_type(params_node, source)?;
        result.push('(');
        result.push_str(&normalized_params);
        result.push(')');
    } else {
        result.push_str("()");
    }
    
    Some(result)
}

/// Normalize parameter types from a tree-sitter node
pub fn normalize_parameter_type(node: Node, source: &str) -> Option<String> {
    let mut params = Vec::new();
    let mut cursor = node.walk();
    
    for child in node.children(&mut cursor) {
        if child.kind() == PARAMETER {
            let mut param_parts = Vec::new();
            
            // Check for parameter modifiers (ref, in, out)
            let mut param_cursor = child.walk();
            for param_child in child.children(&mut param_cursor) {
                if param_child.kind() == PARAMETER_MODIFIER {
                    if let Ok(modifier_text) = param_child.utf8_text(source.as_bytes()) {
                        param_parts.push(modifier_text.to_string());
                    }
                }
            }
            
            // Get the type
            if let Some(type_node) = child.child_by_field_name(TYPE_FIELD) {
                let type_text = type_node.utf8_text(source.as_bytes()).ok()?;
                let simple_type = get_simple_type_name(type_text);
                param_parts.push(simple_type);
            }
            
            if !param_parts.is_empty() {
                params.push(param_parts.join(" "));
            }
        }
    }
    
    Some(params.join(", "))
}

/// Get simple type name from a potentially qualified type
pub fn get_simple_type_name(type_name: &str) -> String {
    // Handle generic types
    if let Some(generic_start) = type_name.find('<') {
        let base_type = &type_name[..generic_start];
        let generic_part = &type_name[generic_start..];
        let simple_base = get_simple_name(base_type);
        let normalized_generics = normalize_generic_parameters(generic_part);
        format!("{}{}", simple_base, normalized_generics)
    } else {
        get_simple_name(type_name)
    }
}

/// Get simple name from a potentially qualified name
fn get_simple_name(name: &str) -> String {
    // Handle C# primitive types with exact equality checks
    match name {
        SYSTEM_INT32_TYPE => INT_TYPE.to_string(),
        SYSTEM_STRING_TYPE => STRING_TYPE.to_string(),
        SYSTEM_BOOLEAN_TYPE => BOOL_TYPE.to_string(),
        SYSTEM_DOUBLE_TYPE => DOUBLE_TYPE.to_string(),
        SYSTEM_SINGLE_TYPE => FLOAT_TYPE.to_string(),
        SYSTEM_INT64_TYPE => LONG_TYPE.to_string(),
        SYSTEM_INT16_TYPE => SHORT_TYPE.to_string(),
        SYSTEM_BYTE_TYPE => BYTE_TYPE.to_string(),
        SYSTEM_OBJECT_TYPE => OBJECT_TYPE.to_string(),
        _ => name.split('.').last().unwrap_or(name).to_string(),
    }
}

/// Normalize generic parameters in a type
pub fn normalize_generic_parameters(generic_text: &str) -> String {
    if !generic_text.starts_with('<') || !generic_text.ends_with('>') {
        return generic_text.to_string();
    }
    
    let inner = &generic_text[1..generic_text.len()-1];
    let params = split_parameters(inner);
    let normalized: Vec<String> = params.iter()
        .map(|p| get_simple_type_name(p.trim()))
        .collect();
    
    format!("<{}>", normalized.join(", "))
}

/// Split parameter string handling nested generics
pub fn split_parameters(params: &str) -> Vec<String> {
    if params.trim().is_empty() {
        return Vec::new();
    }
    
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    
    for ch in params.chars() {
        match ch {
            '<' => {
                depth += 1;
                current.push(ch);
            }
            '>' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                result.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }
    
    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }
    
    result
}

/// Normalize parameter type from string (for backward compatibility)
pub fn normalize_parameter_type_string(param_type: &str) -> String {
    get_simple_type_name(param_type)
}

/// Normalize a symbol name(can be a method name) from string
pub fn normalize_symbol_name(named: &str) -> String {
    let mut result = named.to_string();
    
    // Normalize spaces around parentheses and commas
    result = result.replace(" (", "(");
    result = result.replace("( ", "(");
    result = result.replace(" )", ")");
    result = result.replace(") ", ")");
    
    // Normalize spaces around commas in parameter lists
    let mut normalized = String::new();
    let mut in_params = false;
    let mut chars = result.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '(' => {
                in_params = true;
                normalized.push(ch);
            }
            ')' => {
                in_params = false;
                normalized.push(ch);
            }
            ',' if in_params => {
                normalized.push(ch);
                // Skip any following whitespace and add exactly one space
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
                normalized.push(' ');
            }
            ' ' if in_params => {
                // Skip extra spaces in parameter lists, they'll be normalized by comma handling
                if chars.peek() != Some(&',') && !normalized.ends_with(' ') {
                    normalized.push(ch);
                }
            }
            _ => {
                normalized.push(ch);
            }
        }
    }
    
    result = normalized;
    
    // Primitive type normalization is now handled in get_simple_name() for better precision
    
    // Replace {} with <> for generics
    result = result.replace('{', "<");
    result = result.replace('}', ">");
    
    // Normalize parameter types by removing namespace prefixes
     if let Some(paren_start) = result.find('(') {
         if let Some(paren_end) = result.rfind(')') {
             let method_name = result[..paren_start].to_string();
             let params_str = result[paren_start + 1..paren_end].to_string();
             let suffix = result[paren_end..].to_string();
             
             if !params_str.trim().is_empty() {
                 let params: Vec<String> = split_parameters(&params_str)
                     .into_iter()
                     .map(|param| {
                         let trimmed = param.trim();
                         // Handle ref, in, out modifiers
                         if trimmed.starts_with(REF_MODIFIER_WITH_SPACE) {
                             format!("ref {}", get_simple_type_name(&trimmed[4..]))
                         } else if trimmed.starts_with(IN_MODIFIER_WITH_SPACE) {
                             format!("in {}", get_simple_type_name(&trimmed[3..]))
                         } else if trimmed.starts_with(OUT_MODIFIER_WITH_SPACE) {
                             format!("out {}", get_simple_type_name(&trimmed[4..]))
                         } else {
                             get_simple_type_name(trimmed)
                         }
                     })
                     .collect();
                 
                 result = format!("{}{}{}", method_name, "(", params.join(", "));
                 result.push_str(&suffix);
             }
         }
     }
    
    result
}

#[cfg(test)]
#[path="compile_utils_tests.rs"]
mod tests;
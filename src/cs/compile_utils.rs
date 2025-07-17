//! Compilation utilities for C# documentation processing
//!
//! This module contains standalone functions for normalizing C# type and member names
//! when processing syntax trees for documentation compilation.

use tokio::net::windows::named_pipe::NamedPipeClient;
use tree_sitter::Node;

/// Normalize a type name from a tree-sitter node
/// Returns the fully qualified type name including namespace
pub fn normalize_type_name(node: Node, source: &str) -> Option<String> {
    // For a class name node, we need to traverse up to find the full namespace path
    let class_name = node.utf8_text(source.as_bytes()).ok()?.to_string();
    
    // Find the class declaration that contains this name node
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "class_declaration" {
            // Found the class, now traverse up to collect namespace hierarchy
            let mut namespaces = Vec::new();
            let mut namespace_parent = parent.parent();
            
            while let Some(ns_parent) = namespace_parent {
                if ns_parent.kind() == "namespace_declaration" {
                    if let Some(ns_name_node) = ns_parent.child_by_field_name("name") {
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
        "method_declaration" | "constructor_declaration" | "destructor_declaration" => {
            normalize_method_name(node, source)
        }
        "field_declaration" => {
            // For field declarations, we need to find the variable_declarator
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "variable_declaration" {
                    let mut var_cursor = child.walk();
                    for var_child in child.children(&mut var_cursor) {
                        if var_child.kind() == "variable_declarator" {
                            // The identifier is a direct child of variable_declarator
                            let mut declarator_cursor = var_child.walk();
                            for declarator_child in var_child.children(&mut declarator_cursor) {
                                if declarator_child.kind() == "identifier" {
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
            if let Some(name_node) = node.child_by_field_name("name") {
                name_node.utf8_text(source.as_bytes()).ok().map(|s| s.to_string())
            } else {
                // Fallback: find first identifier
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
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
    let name_node = node.child_by_field_name("name")?;
    let base_name = name_node.utf8_text(source.as_bytes()).ok()?;
    
    let mut result = base_name.to_string();
    
    // Add generic parameters if present
    if let Some(type_params) = node.child_by_field_name("type_parameters") {
        let generic_text = type_params.utf8_text(source.as_bytes()).ok()?;
        let normalized_generics = normalize_generic_parameters(generic_text);
        result.push_str(&normalized_generics);
    }
    
    // Add parameters
    if let Some(params_node) = node.child_by_field_name("parameters") {
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
        if child.kind() == "parameter" {
            let mut param_parts = Vec::new();
            
            // Check for parameter modifiers (ref, in, out)
            let mut param_cursor = child.walk();
            for param_child in child.children(&mut param_cursor) {
                if param_child.kind() == "parameter_modifier" {
                    if let Ok(modifier_text) = param_child.utf8_text(source.as_bytes()) {
                        param_parts.push(modifier_text.to_string());
                    }
                }
            }
            
            // Get the type
            if let Some(type_node) = child.child_by_field_name("type") {
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
        "System.Int32" => "int".to_string(),
        "System.String" => "string".to_string(),
        "System.Boolean" => "bool".to_string(),
        "System.Double" => "double".to_string(),
        "System.Single" => "float".to_string(),
        "System.Int64" => "long".to_string(),
        "System.Int16" => "short".to_string(),
        "System.Byte" => "byte".to_string(),
        "System.Object" => "object".to_string(),
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
                         if trimmed.starts_with("ref ") {
                             format!("ref {}", get_simple_type_name(&trimmed[4..]))
                         } else if trimmed.starts_with("in ") {
                             format!("in {}", get_simple_type_name(&trimmed[3..]))
                         } else if trimmed.starts_with("out ") {
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
mod tests {
    use super::*;
    use tree_sitter::{Parser, Language, Node};
    use crate::language::tree_printer::print_tree_to_stdout;
    
    /// Helper function to find a class declaration node by name in a Tree-sitter AST
    fn find_class_by_name<'a>(root: Node<'a>, class_name: &str, source: &str) -> Option<Node<'a>> {
        fn search_node<'a>(node: Node<'a>, class_name: &str, source: &str) -> Option<Node<'a>> {
            // Check if this is a class_declaration with the target name
            if node.kind() == "class_declaration" {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Ok(name_text) = name_node.utf8_text(source.as_bytes()) {
                        if name_text == class_name {
                            return Some(node);
                        }
                    }
                }
            }
            
            // Recursively search children
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if let Some(found) = search_node(child, class_name, source) {
                    return Some(found);
                }
            }
            
            None
        }
        
        search_node(root, class_name, source)
    }

    #[test]
    fn test_get_simple_type_name() {
        assert_eq!(get_simple_type_name("int"), "int");
        assert_eq!(get_simple_type_name("System.String"), "string");
        assert_eq!(get_simple_type_name("List<int>"), "List<int>");
        assert_eq!(get_simple_type_name("Dictionary<string, int>"), "Dictionary<string, int>");
        assert_eq!(get_simple_type_name("System.Collections.Generic.List<System.String>"), "List<string>");
    }

    #[test]
    fn test_split_parameters() {
        assert_eq!(split_parameters(""), Vec::<String>::new());
        assert_eq!(split_parameters("int"), vec!["int"]);
        assert_eq!(split_parameters("int, string"), vec!["int", "string"]);
        assert_eq!(split_parameters("List<int>, Dictionary<string, int>"), vec!["List<int>", "Dictionary<string, int>"]);
        assert_eq!(split_parameters("Dictionary<string, List<int>>"), vec!["Dictionary<string, List<int>>"]);
    }

    #[test]
    fn test_normalize_generic_parameters() {
        assert_eq!(normalize_generic_parameters("<int>"), "<int>");
        assert_eq!(normalize_generic_parameters("<System.String>"), "<string>");
        assert_eq!(normalize_generic_parameters("<int, string>"), "<int, string>");
        assert_eq!(normalize_generic_parameters("<System.Collections.Generic.List<System.String>>"), "<List<string>>");
    }

    #[test]
    fn test_normalize_functions_with_tree_sitter() {
        let mut parser = Parser::new();
        let language = tree_sitter_c_sharp::language();
        parser.set_language(language).unwrap();
        
        // Test normalize_type_name with a simple class in a compilation unit
        let source: &'static str = "namespace Test { public class TestClass { } }";
        let tree = parser.parse(source, None).unwrap();
        println!("\n=== Type Name Test Tree ===");
        print_tree_to_stdout(tree.root_node(), source);
        
        // Test normalize_member_name with a field in a class context
        let field_source = "namespace Test { public class TestClass { public int PublicField; } }";
        let field_tree = parser.parse(field_source, None).unwrap();
        println!("\n=== Field Test Tree ===");
        print_tree_to_stdout(field_tree.root_node(), field_source);
        
        // Test normalize_member_name with a method
        let method_source = "namespace Test { public class TestClass { public void TestMethod(int param) { } } }";
        let method_tree = parser.parse(method_source, None).unwrap();
        println!("\n=== Method Test Tree ===");
        print_tree_to_stdout(method_tree.root_node(), method_source);
        
        // Test normalize_member_name with a property
        let property_source = "namespace Test { public class TestClass { public int TestProperty { get; set; } } }";
        let property_tree = parser.parse(property_source, None).unwrap();
        println!("\n=== Property Test Tree ===");
        print_tree_to_stdout(property_tree.root_node(), property_source);
    }
    
    #[test]
    fn test_method_with_ref_in_out_modifiers() {
        let mut parser = Parser::new();
        let language = tree_sitter_c_sharp::language();
        parser.set_language(language).unwrap();
        
        // Test method with ref, in, out modifiers
        let method_source = r#"namespace Test {
    public class TestClass {
        public void ProcessData(ref int refParam, in string inParam, out bool outParam) {
            outParam = true;
        }
    }
}"#;
        
        let tree = parser.parse(method_source, None).unwrap();
        let class_node = find_class_by_name(tree.root_node(), "TestClass", method_source).unwrap();
        
        // Find the method declaration
        let mut cursor = class_node.walk();
        let mut method_node = None;
        for child in class_node.children(&mut cursor) {
            if child.kind() == "method_declaration" {
                method_node = Some(child);
                break;
            } else if child.kind() == "declaration_list" {
                // Methods might be inside a declaration_list
                let mut decl_cursor = child.walk();
                for decl_child in child.children(&mut decl_cursor) {
                    if decl_child.kind() == "method_declaration" {
                        method_node = Some(decl_child);
                        break;
                    }
                }
                if method_node.is_some() {
                    break;
                }
            }
        }
        
        let method_node = method_node.unwrap();
        let normalized_name = normalize_method_name(method_node, method_source).unwrap();
        
        // Should preserve ref, in, out modifiers
        assert_eq!(normalized_name, "ProcessData(ref int, in string, out bool)");
    }
    
    #[test]
    fn test_normalize_method_name_string() {
        // Test basic method normalization
        assert_eq!(normalize_symbol_name("A.B.Method()"), "A.B.Method()");
        assert_eq!(normalize_symbol_name("Method(int)"), "Method(int)");
        assert_eq!(normalize_symbol_name("Method(int, string)"), "Method(int, string)");
        
        // Test space normalization
        assert_eq!(normalize_symbol_name("C.D.Method ( int , string )"), "C.D.Method(int, string)");
        assert_eq!(normalize_symbol_name("Method(  int  ,  string  )"), "Method(int, string)");
        assert_eq!(normalize_symbol_name("Method( int,string )"), "Method(int, string)");
        
        // Test C# primitive type normalization
        assert_eq!(normalize_symbol_name("Method(System.Int32)"), "Method(int)");
        assert_eq!(normalize_symbol_name("Method(System.String, System.Boolean)"), "Method(string, bool)");
        assert_eq!(normalize_symbol_name("Method(System.Double, System.Single)"), "Method(double, float)");
        assert_eq!(normalize_symbol_name("Method(System.Int64, System.Int16, System.Byte)"), "Method(long, short, byte)");
        assert_eq!(normalize_symbol_name("Method(System.Object)"), "Method(object)");
        
        // Test generic bracket normalization
        assert_eq!(normalize_symbol_name("GenericMethod{T}(T)"), "GenericMethod<T>(T)");
        assert_eq!(normalize_symbol_name("Method(List{int})"), "Method(List<int>)");
        assert_eq!(normalize_symbol_name("Method(Dictionary{string, int})"), "Method(Dictionary<string, int>)");
        
        // Test ref, in, out modifiers
        assert_eq!(normalize_symbol_name("Method(ref int)"), "Method(ref int)");
        assert_eq!(normalize_symbol_name("Method(in string)"), "Method(in string)");
        assert_eq!(normalize_symbol_name("Method(out bool)"), "Method(out bool)");
        assert_eq!(normalize_symbol_name("Method(ref System.Int32, in System.String, out System.Boolean)"), "Method(ref int, in string, out bool)");
        
        // Test namespace stripping for parameter types
        assert_eq!(normalize_symbol_name("Method(System.Collections.Generic.List<T>)"), "Method(List<T>)");
        assert_eq!(normalize_symbol_name("Method(System.Collections.Generic.Dictionary<string, int>)"), "Method(Dictionary<string, int>)");
        assert_eq!(normalize_symbol_name("Method(UnityEngine.GameObject, UnityEngine.Transform)"), "Method(GameObject, Transform)");
        
        // Test complex combinations
        assert_eq!(
            normalize_symbol_name("ProcessData( ref System.Collections.Generic.List{System.String} , in UnityEngine.GameObject,out System.Boolean )"),
            "ProcessData(ref List<string>, in GameObject, out bool)"
        );
        
        // Test generic methods with complex parameters
        assert_eq!(
            normalize_symbol_name("GenericMethod{T, U}(System.Collections.Generic.Dictionary{T, System.Collections.Generic.List{U}}, System.Int32)"),
            "GenericMethod<T, U>(Dictionary<T, List<U>>, int)"
        );
        
        // Test methods without parameters
        assert_eq!(normalize_symbol_name("GetValue()"), "GetValue()");
        assert_eq!(normalize_symbol_name("GetValue( )"), "GetValue()");
        
        // Test edge cases
        assert_eq!(normalize_symbol_name("Method"), "Method"); // No parentheses
        assert_eq!(normalize_symbol_name("Method()"), "Method()"); // Empty parameters
    }
    
    #[test]
    fn test_method_with_attributes_and_comments() {
        let mut parser = Parser::new();
        let language = tree_sitter_c_sharp::language();
        parser.set_language(language).unwrap();
        
        // Test method with attributes and comments in parameters
        let method_source = r#"namespace Test {
    public class TestClass {
        public void Log(
            [CallerLineNumber] int line = -1, // this is some comments
            [CallerFilePath] string path = null, /*some other comment*/
            [CallerMemberName] string name = null
        )
        {
            Console.WriteLine((line < 0) ? "No line" : "Line "+ line);
            Console.WriteLine((path == null) ? "No file path" : path);
            Console.WriteLine((name == null) ? "No member name" : name);
        }
    }
}"#;
        let method_tree = parser.parse(method_source, None).unwrap();
        println!("\n=== Method with Attributes and Comments Test Tree ===");
        print_tree_to_stdout(method_tree.root_node(), method_source);
        
        // Find the Log method using the helper method
        let class_node = find_class_by_name(method_tree.root_node(), "TestClass", method_source)
            .expect("Could not find TestClass");
        
        let class_body = class_node.child_by_field_name("body").unwrap();
        
        // Find the method in the class body
        let mut method_node = None;
        let mut cursor = class_body.walk();
        for child in class_body.children(&mut cursor) {
            if child.kind() == "method_declaration" {
                method_node = Some(child);
                break;
            }
        }
        
        if let Some(method_node) = method_node {
            let method_name = normalize_member_name(method_node, method_source).unwrap();
            // Should extract parameter types correctly, ignoring attributes and comments
            assert_eq!(method_name, "Log(int, string, string)");
        } else {
            panic!("Could not find method node");
        }
    }

    #[test]
    fn test_nested_namespace_normalization() {
        let mut parser = Parser::new();
        let language = tree_sitter_c_sharp::language();
        parser.set_language(language).unwrap();
        
        // Test nested namespaces with complex hierarchy
        let nested_source = "namespace Namespace.Hello { namespace World.How.Are.You { public class HelloWorld { public void Method() { } } } }";
        let nested_tree = parser.parse(nested_source, None).unwrap();
        println!("\n=== Nested Namespace Test Tree ===");
        print_tree_to_stdout(nested_tree.root_node(), nested_source);
        
        // Find the HelloWorld class using the helper method
        let class_node = find_class_by_name(nested_tree.root_node(), "HelloWorld", nested_source)
            .expect("Could not find HelloWorld class in nested namespace");
        
        // Test class name normalization
        let class_name_node = class_node.child_by_field_name("name").or_else(|| {
            // Fallback: find identifier child directly
            let mut cursor = class_node.walk();
            for child in class_node.children(&mut cursor) {
                if child.kind() == "identifier" {
                    return Some(child);
                }
            }
            None
        });
        
        if let Some(name_node) = class_name_node {
            let class_name = normalize_type_name(name_node, nested_source).unwrap();
            assert_eq!(class_name, "Namespace.Hello.World.How.Are.You.HelloWorld");
        } else {
            panic!("Could not find class name node");
        }
        
        // Test method name normalization
        let class_body = class_node.child_by_field_name("body").unwrap();
        
        // Find the method in the class body
        let mut method_node = None;
        let mut cursor = class_body.walk();
        for child in class_body.children(&mut cursor) {
            if child.kind() == "method_declaration" {
                method_node = Some(child);
                break;
            }
        }
        
        if let Some(method_node) = method_node {
            let method_name = normalize_member_name(method_node, nested_source).unwrap();
            assert_eq!(method_name, "Method()");
        } else {
            panic!("Could not find method node");
        }
    }
}
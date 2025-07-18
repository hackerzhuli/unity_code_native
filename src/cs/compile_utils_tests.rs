use crate::{cs::compile_utils::*, cs::constants::*, language::tree_printer::print_tree_to_stdout};
use tree_sitter::{Language, Node, Parser};
use crate::cs::constants::*;

/// Helper function to find a class declaration node by name in a Tree-sitter AST
fn find_class_by_name<'a>(root: Node<'a>, class_name: &str, source: &str) -> Option<Node<'a>> {
    fn search_node<'a>(node: Node<'a>, class_name: &str, source: &str) -> Option<Node<'a>> {
        // Check if this is a class_declaration with the target name
        if node.kind() == CLASS_DECLARATION {
            if let Some(name_node) = node.child_by_field_name(NAME_FIELD) {
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
    assert_eq!(get_simple_type_name(SYSTEM_STRING_TYPE), STRING_TYPE);
    assert_eq!(get_simple_type_name("List<int>"), "List<int>");
    assert_eq!(
        get_simple_type_name("Dictionary<string, int>"),
        "Dictionary<string, int>"
    );
    assert_eq!(
        get_simple_type_name("System.Collections.Generic.List<System.String>"),
        "List<string>"
    );
}

#[test]
fn test_split_parameters() {
    assert_eq!(split_parameters(""), Vec::<String>::new());
    assert_eq!(split_parameters("int"), vec!["int"]);
    assert_eq!(split_parameters("int, string"), vec!["int", "string"]);
    assert_eq!(
        split_parameters("List<int>, Dictionary<string, int>"),
        vec!["List<int>", "Dictionary<string, int>"]
    );
    assert_eq!(
        split_parameters("Dictionary<string, List<int>>"),
        vec!["Dictionary<string, List<int>>"]
    );
}

#[test]
fn test_normalize_generic_parameters() {
    assert_eq!(normalize_generic_parameters("<int>"), "<int>");
    assert_eq!(normalize_generic_parameters("<System.String>"), "<string>");
    assert_eq!(
        normalize_generic_parameters("<int, string>"),
        "<int, string>"
    );
    assert_eq!(
        normalize_generic_parameters("<System.Collections.Generic.List<System.String>>"),
        "<List<string>>"
    );
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
    let method_source =
        "namespace Test { public class TestClass { public void TestMethod(int param) { } } }";
    let method_tree = parser.parse(method_source, None).unwrap();
    println!("\n=== Method Test Tree ===");
    print_tree_to_stdout(method_tree.root_node(), method_source);

    // Test normalize_member_name with a property
    let property_source =
        "namespace Test { public class TestClass { public int TestProperty { get; set; } } }";
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
        if child.kind() == METHOD_DECLARATION {
            method_node = Some(child);
            break;
        } else if child.kind() == DECLARATION_LIST {
            // Methods might be inside a declaration_list
            let mut decl_cursor = child.walk();
            for decl_child in child.children(&mut decl_cursor) {
                if decl_child.kind() == METHOD_DECLARATION {
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
    assert_eq!(
        normalize_symbol_name("Method(int, string)"),
        "Method(int, string)"
    );

    // Test space normalization
    assert_eq!(
        normalize_symbol_name("C.D.Method ( int , string )"),
        "C.D.Method(int, string)"
    );
    assert_eq!(
        normalize_symbol_name("Method(  int  ,  string  )"),
        "Method(int, string)"
    );
    assert_eq!(
        normalize_symbol_name("Method( int,string )"),
        "Method(int, string)"
    );

    // Test C# primitive type normalization
    assert_eq!(normalize_symbol_name("Method(System.Int32)"), "Method(int)");
    assert_eq!(
        normalize_symbol_name("Method(System.String, System.Boolean)"),
        "Method(string, bool)"
    );
    assert_eq!(
        normalize_symbol_name("Method(System.Double, System.Single)"),
        "Method(double, float)"
    );
    assert_eq!(
        normalize_symbol_name("Method(System.Int64, System.Int16, System.Byte)"),
        "Method(long, short, byte)"
    );
    assert_eq!(
        normalize_symbol_name("Method(System.Object)"),
        "Method(object)"
    );

    // Test generic bracket normalization
    assert_eq!(
        normalize_symbol_name("GenericMethod{T}(T)"),
        "GenericMethod<T>(T)"
    );
    assert_eq!(
        normalize_symbol_name("Method(List{int})"),
        "Method(List<int>)"
    );
    assert_eq!(
        normalize_symbol_name("Method(Dictionary{string, int})"),
        "Method(Dictionary<string, int>)"
    );

    // Test ref, in, out modifiers
    assert_eq!(normalize_symbol_name("Method(ref int)"), "Method(ref int)");
    assert_eq!(
        normalize_symbol_name("Method(in string)"),
        "Method(in string)"
    );
    assert_eq!(
        normalize_symbol_name("Method(out bool)"),
        "Method(out bool)"
    );
    assert_eq!(
        normalize_symbol_name("Method(ref System.Int32, in System.String, out System.Boolean)"),
        "Method(ref int, in string, out bool)"
    );

    // Test namespace stripping for parameter types
    assert_eq!(
        normalize_symbol_name("Method(System.Collections.Generic.List<T>)"),
        "Method(List<T>)"
    );
    assert_eq!(
        normalize_symbol_name("Method(System.Collections.Generic.Dictionary<string, int>)"),
        "Method(Dictionary<string, int>)"
    );
    assert_eq!(
        normalize_symbol_name("Method(UnityEngine.GameObject, UnityEngine.Transform)"),
        "Method(GameObject, Transform)"
    );

    // Test complex combinations
    assert_eq!(
        normalize_symbol_name(
            "ProcessData( ref System.Collections.Generic.List{System.String} , in UnityEngine.GameObject,out System.Boolean )"
        ),
        "ProcessData(ref List<string>, in GameObject, out bool)"
    );

    // Test generic methods with complex parameters
    assert_eq!(
        normalize_symbol_name(
            "GenericMethod{T, U}(System.Collections.Generic.Dictionary{T, System.Collections.Generic.List{U}}, System.Int32)"
        ),
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

    let class_body = class_node.child_by_field_name(BODY_FIELD).unwrap();

    // Find the method in the class body
    let mut method_node = None;
    let mut cursor = class_body.walk();
    for child in class_body.children(&mut cursor) {
        if child.kind() == METHOD_DECLARATION {
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
    let class_name_node = class_node.child_by_field_name(NAME_FIELD).or_else(|| {
        // Fallback: find identifier child directly
        let mut cursor = class_node.walk();
        for child in class_node.children(&mut cursor) {
            if child.kind() == IDENTIFIER {
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
    let class_body = class_node.child_by_field_name(BODY_FIELD).unwrap();

    // Find the method in the class body
    let mut method_node = None;
    let mut cursor = class_body.walk();
    for child in class_body.children(&mut cursor) {
        if child.kind() == METHOD_DECLARATION {
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

use tower_lsp::lsp_types::NumberOrString;

use crate::language::tree_utils::find_node_by_type;
use crate::uss::constants::NODE_CALL_EXPRESSION;
use crate::uss::function_node::FunctionNode;
use crate::uss::parser::UssParser;

#[test]
fn test_url_function_single_argument() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "background-image: url(\"image.png\");";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let result = FunctionNode::from_node(call_node, source, None);

        assert!(result.is_some(), "Expected valid FunctionNode");

        if let Some(func) = result {
            assert_eq!(func.function_name, "url");
            assert_eq!(func.argument_count(), 1);
            assert!(func.is_function("url"));

            let arg_text = func.get_argument_text(0, source).unwrap();
            assert_eq!(arg_text, "\"image.png\"");
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_rgb_function_multiple_arguments() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "color: rgb(255, 128, 0);";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let result = FunctionNode::from_node(call_node, source, None);

        assert!(result.is_some(), "Expected valid FunctionNode");

        if let Some(func) = result {
            assert_eq!(func.function_name, "rgb");
            assert_eq!(func.argument_count(), 3);
            assert!(func.is_function("rgb"));

            assert_eq!(func.get_argument_text(0, source).unwrap(), "255");
            assert_eq!(func.get_argument_text(1, source).unwrap(), "128");
            assert_eq!(func.get_argument_text(2, source).unwrap(), "0");
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_function_no_arguments() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "background-image: url();";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let result = FunctionNode::from_node(call_node, source, None);

        assert!(result.is_some(), "Expected valid FunctionNode");

        if let Some(func) = result {
            assert_eq!(func.function_name, "url");
            assert_eq!(func.argument_count(), 0);
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_non_call_expression_returns_none() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = ".test { color: red; }";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    // Try to create FunctionNode from a rule_set node
    if let Some(rule_node) = find_node_by_type(root, "rule_set") {
        let result = FunctionNode::from_node(rule_node, source, None);

        assert!(
            result.is_none(),
            "Expected None for non-call-expression node"
        );
    }
}

#[test]
fn test_function_with_syntax_errors_returns_none() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    // Malformed CSS that will have error nodes
    let source = "background-image: url(\"image.png\" {{{;";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    // Check if there are any error nodes in the tree
    fn has_error_nodes(node: tree_sitter::Node) -> bool {
        if node.is_error() || node.kind() == "ERROR" {
            return true;
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if has_error_nodes(child) {
                    return true;
                }
            }
        }
        false
    }

    if has_error_nodes(root) {
        if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
            let result = FunctionNode::from_node(call_node, source, None);

            assert!(
                result.is_none(),
                "Expected None when syntax tree has errors"
            );
        }
    }
}

#[test]
fn test_calc_function_complex_arguments() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "width: calc(100% - 20px);";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let result = FunctionNode::from_node(call_node, source, None);

        assert!(result.is_some(), "Expected valid FunctionNode");

        if let Some(func) = result {
            assert_eq!(func.function_name, "calc");
            // calc typically has one complex expression argument
            assert!(func.argument_count() >= 1);
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_nested_function_calls_rejected() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    // Test nested url() inside calc()
    let source = "width: calc(url(\"image.png\") + 10px);";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = FunctionNode::from_node(call_node, source, Some(&mut diagnostics));

        assert!(result.is_none(), "Expected None for nested function calls");
        assert!(
            !diagnostics.is_empty(),
            "Expected diagnostic for nested functions"
        );

        let diagnostic = &diagnostics[0];
        assert_eq!(
            diagnostic.code,
            Some(NumberOrString::String(
                "nested-functions-not-supported".to_string()
            ))
        );
        assert_eq!(
            diagnostic.message,
            "Nested function calls are not supported in USS"
        );
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_multiple_nested_functions_rejected() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    // Test multiple nested functions
    let source = "background: linear-gradient(rgb(255, 0, 0), url(\"bg.png\"));";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = FunctionNode::from_node(call_node, source, Some(&mut diagnostics));

        assert!(result.is_none(), "Expected None for nested function calls");
        assert!(
            !diagnostics.is_empty(),
            "Expected diagnostic for nested functions"
        );
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_deeply_nested_functions_rejected() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    // Test deeply nested functions: calc(rgb(url("test")))
    let source = "color: calc(rgb(url(\"test.png\")));";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = FunctionNode::from_node(call_node, source, Some(&mut diagnostics));

        assert!(
            result.is_none(),
            "Expected None for deeply nested function calls"
        );
        assert!(
            !diagnostics.is_empty(),
            "Expected diagnostic for nested functions"
        );
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_function_with_mixed_arguments() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    // Test function with string, number, and identifier arguments (no nesting)
    // Using simple values instead of var() to avoid nested function detection
    let source = "transform: translate3d(10px, 20%, 5em);";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let result = FunctionNode::from_node(call_node, source, None);

        assert!(
            result.is_some(),
            "Expected valid FunctionNode for mixed arguments"
        );

        if let Some(func) = result {
            assert_eq!(func.function_name, "translate3d");
            assert_eq!(func.argument_count(), 3);
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_css_var_function_rejected() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    // Test that var() functions are also rejected as nested functions
    let source = "transform: translate3d(10px, 20%, var(--offset));";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = FunctionNode::from_node(call_node, source, Some(&mut diagnostics));

        // This should be rejected because var() is a nested function call
        assert!(
            result.is_none(),
            "Expected None for function with var() argument"
        );
        assert!(
            !diagnostics.is_empty(),
            "Expected diagnostic for nested var() function"
        );
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_function_with_complex_expressions() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    // Test function with complex mathematical expressions (no function nesting)
    let source = "width: calc(100vw - 2 * 20px + 5%);";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let result = FunctionNode::from_node(call_node, source, None);

        assert!(
            result.is_some(),
            "Expected valid FunctionNode for complex expressions"
        );

        if let Some(func) = result {
            assert_eq!(func.function_name, "calc");
            assert!(func.argument_count() >= 1);
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_function_with_parentheses_in_strings() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    // Test function with parentheses inside string arguments (should not be confused with function calls)
    let source = "background-image: url(\"path/to/file(1).png\");";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let result = FunctionNode::from_node(call_node, source, None);

        assert!(
            result.is_some(),
            "Expected valid FunctionNode for string with parentheses"
        );

        if let Some(func) = result {
            assert_eq!(func.function_name, "url");
            assert_eq!(func.argument_count(), 1);
            let arg_text = func.get_argument_text(0, source).unwrap();
            assert_eq!(arg_text, "\"path/to/file(1).png\"");
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_function_with_empty_string_argument() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "background-image: url(\"\");";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let result = FunctionNode::from_node(call_node, source, None);

        assert!(
            result.is_some(),
            "Expected valid FunctionNode for empty string"
        );

        if let Some(func) = result {
            assert_eq!(func.function_name, "url");
            assert_eq!(func.argument_count(), 1);
            let arg_text = func.get_argument_text(0, source).unwrap();
            assert_eq!(arg_text, "\"\"");
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_function_with_single_quotes() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "background-image: url('image.png');";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let result = FunctionNode::from_node(call_node, source, None);

        assert!(
            result.is_some(),
            "Expected valid FunctionNode for single quotes"
        );

        if let Some(func) = result {
            assert_eq!(func.function_name, "url");
            assert_eq!(func.argument_count(), 1);
            let arg_text = func.get_argument_text(0, source).unwrap();
            assert_eq!(arg_text, "'image.png'");
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

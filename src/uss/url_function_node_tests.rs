
use crate::language::tree_utils::find_node_by_type;
use crate::uss::constants::NODE_CALL_EXPRESSION;
use crate::uss::parser::UssParser;
use crate::uss::url_function_node::UrlFunctionNode;

#[test]
fn test_valid_url_function() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "background-image: url(\"image.png\");";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics), None, None, false);

        assert!(result.is_some(), "Expected valid UrlFunctionNode");

        if let Some(url_func) = result {
            assert_eq!(url_func.url(), "image.png");
            assert!(!url_func.is_empty());
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_url_function_with_single_quotes() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "background-image: url('image.png');";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics), None, None, false);


        assert!(
            result.is_some(),
            "Expected valid UrlFunctionNode with single quotes"
        );

        if let Some(url_func) = result {
            assert_eq!(url_func.url(), "image.png");
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_url_function_with_escapes() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = r#"background-image: url("path\\to\\file.png");"#;
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics), None, None, false);

        assert!(
            result.is_some(),
            "Expected valid UrlFunctionNode with escapes"
        );

        if let Some(url_func) = result {
            assert_eq!(url_func.url(), "path\\to\\file.png");
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_url_function_empty_string() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "background-image: url(\"\");";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics), None, None, false);


        assert!(
            result.is_some(),
            "Expected valid UrlFunctionNode with empty string"
        );

        if let Some(url_func) = result {
            assert_eq!(url_func.url(), "");
            assert!(url_func.is_empty());
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_non_url_function_rejected() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "color: rgb(255, 0, 0);";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics), None, None, false);


        assert!(result.is_none(), "Expected None for rgb() function");
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_url_function_no_arguments() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "background-image: url();";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics), None, None, false);


        assert!(
            result.is_none(),
            "Expected None for url() with no arguments"
        );
        assert!(!diagnostics.is_empty());
        assert!(
            diagnostics[0]
                .message
                .contains("expects exactly 1 argument")
        );
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_url_function_too_many_arguments() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "background-image: url(\"image.png\", \"fallback.png\");";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics), None, None, false);


        assert!(
            result.is_none(),
            "Expected None for url() with too many arguments"
        );
        assert!(!diagnostics.is_empty());
        assert!(
            diagnostics[0]
                .message
                .contains("expects exactly 1 argument")
        );
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_url_function_non_string_argument() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "background-image: url(123);";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics), None, None, false);


        assert!(
            result.is_none(),
            "Expected None for url() with non-string/non-identifier argument"
        );
        assert!(!diagnostics.is_empty());
        assert!(
            diagnostics[0]
                .message
                .contains("expects a string or identifier argument")
        );
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_url_function_with_hex_escapes() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    // Test hex escape: \26 = & (ampersand)
    let source = r#"background-image: url("test\26 file.png");"#;
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics), None, None, false);


        assert!(
            result.is_some(),
            "Expected valid UrlFunctionNode with hex escapes"
        );

        if let Some(url_func) = result {
            assert_eq!(url_func.url(), "test&file.png");
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_url_function_with_plain_value() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "background-image: url(image.png);";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics), None, None, false);


        assert!(
            result.is_some(),
            "Expected valid UrlFunctionNode with plain value"
        );

        if let Some(url_func) = result {
            assert_eq!(url_func.url(), "image.png");
            assert!(!url_func.is_empty());
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_url_function_with_plain_value_path() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "background-image: url(path/to/image.png);";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics), None, None, false);


        assert!(
            result.is_some(),
            "Expected valid UrlFunctionNode with plain value path"
        );

        if let Some(url_func) = result {
            assert_eq!(url_func.url(), "path/to/image.png");
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_url_function_with_plain_value_escapes() {
    // Test shows that plain values with escape sequences are parsed as multiple tokens
    // This is expected behavior - CSS escape sequences in unquoted values split on whitespace
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = r".test { background-image: url(image\ with\ space.jpg); }";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics), None, None, false);


        // This should fail because the parser treats escaped spaces as separate tokens
        assert!(
            result.is_none(),
            "Expected None for url() with escaped spaces in plain value (parsed as multiple tokens)"
        );
        assert!(
            !diagnostics.is_empty(),
            "Expected diagnostic for invalid argument count"
        );

        // Verify the diagnostic message
        if let Some(diagnostic) = diagnostics.first() {
            assert!(
                diagnostic
                    .message
                    .contains("expects exactly 1 argument, found 3")
            );
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_url_function_with_quoted_string_escapes() {
    // Test how quoted strings with escape sequences are handled
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = r#".test { background-image: url("image\ with\ space.jpg"); }"#;
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics), None, None, false);


        assert!(
            result.is_some(),
            "Expected valid UrlFunctionNode with quoted string escapes"
        );

        if let Some(url_func) = result {
            // Quoted strings properly process escape sequences
            assert_eq!(
                url_func.url(),
                "image with space.jpg",
                "Expected escape sequences to be processed in quoted strings"
            );
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_url_function_with_plain_value_simple_escapes() {
    // Test simple escape sequences in plain values that don't break the parser
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = r".test { background-image: url(i\mage.jpg); }";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics), None, None, false);


        assert!(
            result.is_some(),
            "Expected valid UrlFunctionNode with simple escape in plain value"
        );

        if let Some(url_func) = result {
            // Plain values now process escapes as per documentation
            assert_eq!(
                url_func.url(),
                "image.jpg",
                "Expected escape sequence to be processed in plain values"
            );
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

#[test]
fn test_url_function_with_plain_value_dot_escapes() {
    // Test escaped dots in plain values - they work and preserve the escape sequence
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = r".test { background-image: url(image\.jpg); }";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(call_node) = find_node_by_type(root, NODE_CALL_EXPRESSION) {
        let mut diagnostics = Vec::new();
        let result = UrlFunctionNode::from_node(call_node, source, Some(&mut diagnostics), None, None, false);

        assert!(
            result.is_some(),
            "Expected valid UrlFunctionNode with escaped dot in plain value"
        );

        if let Some(url_func) = result {
            // Plain values now process escapes as per documentation
            assert_eq!(
                url_func.url(),
                "image.jpg",
                "Expected escape sequence to be processed in plain values"
            );
        }
    } else {
        panic!("Expected to find call_expression node");
    }
}

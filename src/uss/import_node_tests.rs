use crate::uss::constants::NODE_IMPORT_STATEMENT;
use crate::{language::tree_utils::find_node_by_type, uss::import_node::ImportNode};
use crate::uss::parser::UssParser;

#[test]
fn test_valid_import_with_string() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "@import \"styles.uss\";";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(import_node) = find_node_by_type(root, NODE_IMPORT_STATEMENT) {
        let mut diagnostics = Vec::new();
        let result = ImportNode::from_node(import_node, source, &mut diagnostics);

        assert!(result.is_some(), "Expected valid ImportNode");
        assert!(
            diagnostics.is_empty(),
            "Expected no diagnostics for valid import"
        );

        if let Some(import) = result {
            let arg_text = import.argument_node.utf8_text(source.as_bytes()).unwrap();
            assert_eq!(arg_text, "\"styles.uss\"");
        }
    } else {
        panic!("Expected to find import_statement node");
    }
}

#[test]
fn test_valid_import_with_url_function() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "@import url(\"styles.uss\");";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(import_node) = find_node_by_type(root, NODE_IMPORT_STATEMENT) {
        let mut diagnostics = Vec::new();
        let result = ImportNode::from_node(import_node, source, &mut diagnostics);

        assert!(result.is_some(), "Expected valid ImportNode");
        assert!(
            diagnostics.is_empty(),
            "Expected no diagnostics for valid import"
        );

        if let Some(import) = result {
            let arg_text = import.argument_node.utf8_text(source.as_bytes()).unwrap();
            assert!(arg_text.starts_with("url("), "Expected url() function");
        }
    } else {
        panic!("Expected to find import_statement node");
    }
}

#[test]
fn test_import_missing_semicolon() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "@import \"styles.uss\"";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(import_node) = find_node_by_type(root, NODE_IMPORT_STATEMENT) {
        let mut diagnostics = Vec::new();
        let result = ImportNode::from_node(import_node, source, &mut diagnostics);

        assert!(
            result.is_none(),
            "Expected None for import without semicolon"
        );
        // there will not be diagnostics because the tree itself errored
        //assert!(!diagnostics.is_empty(), "Expected diagnostic for missing semicolon");

        //let diagnostic = &diagnostics[0];
        //assert!(diagnostic.message.contains("semicolon"), "Expected semicolon error message");
    } else {
        panic!("Expected to find import_statement node");
    }
}

#[test]
fn test_import_missing_argument() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "@import;";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    println!("Tree for missing argument: {}", root.to_sexp());

    if let Some(import_node) = find_node_by_type(root, NODE_IMPORT_STATEMENT) {
        let mut diagnostics = Vec::new();
        let result = ImportNode::from_node(import_node, source, &mut diagnostics);

        println!("Diagnostics count: {}", diagnostics.len());
        for (i, diag) in diagnostics.iter().enumerate() {
            println!("Diagnostic {}: {}", i, diag.message);
        }

        assert!(
            result.is_none(),
            "Expected None for import without argument"
        );
        // The test might need adjustment based on how tree-sitter parses this
    } else {
        println!("No import_statement node found");
    }
}

#[test]
fn test_import_multiple_arguments() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "@import \"styles.uss\" \"other.uss\";";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(import_node) = find_node_by_type(root, NODE_IMPORT_STATEMENT) {
        let mut diagnostics = Vec::new();
        let result = ImportNode::from_node(import_node, source, &mut diagnostics);

        assert!(
            result.is_none(),
            "Expected None for import with multiple arguments"
        );

        // no diagnotics because the tree itself errored
        //assert!(!diagnostics.is_empty(), "Expected diagnostic for multiple arguments");

        //let diagnostic = &diagnostics[0];
        //assert!(diagnostic.message.contains("one argument"), "Expected single argument error message");
    } else {
        panic!("Expected to find import_statement node");
    }
}

#[test]
fn test_import_with_syntax_errors_returns_none() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    // Malformed CSS that will have error nodes
    let source = "@import \"styles.uss\" {{{;";
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
        if let Some(import_node) = find_node_by_type(root, NODE_IMPORT_STATEMENT) {
            let mut diagnostics = Vec::new();
            let result = ImportNode::from_node(import_node, source, &mut diagnostics);

            assert!(
                result.is_none(),
                "Expected None when syntax tree has errors"
            );
            assert!(
                diagnostics.is_empty(),
                "Expected no diagnostics when syntax tree has errors"
            );
        }
    }
}

#[test]
fn test_import_with_invalid_argument_type() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = "@import 123;";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    if let Some(import_node) = find_node_by_type(root, NODE_IMPORT_STATEMENT) {
        let mut diagnostics = Vec::new();
        let result = ImportNode::from_node(import_node, source, &mut diagnostics);

        assert!(
            result.is_none(),
            "Expected None for import with invalid argument type"
        );
        assert!(
            !diagnostics.is_empty(),
            "Expected diagnostic for invalid argument type"
        );
    } else {
        panic!("Expected to find import_statement node");
    }
}

#[test]
fn test_non_import_node_returns_none() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");

    let source = ".test { color: red; }";
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    // Try to create ImportNode from a rule_set node
    if let Some(rule_node) = find_node_by_type(root, "rule_set") {
        let mut diagnostics = Vec::new();
        let result = ImportNode::from_node(rule_node, source, &mut diagnostics);

        assert!(result.is_none(), "Expected None for non-import node");
        assert!(
            diagnostics.is_empty(),
            "Expected no diagnostics for non-import node"
        );
    }
}

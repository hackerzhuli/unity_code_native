//! Tests for USS refactoring functionality
//!
//! These tests help understand the tree structure and validate refactoring operations.

use crate::language::tree_printer::print_tree_to_stdout;
use crate::uss::refactor::{UssRefactorProvider, SelectorType};
use tree_sitter::Parser;
use url::Url;

/// Test USS content with various selector types for tree structure analysis
const TEST_USS_CONTENT: &str = r#"
.my-class {
    color: red;
    background-color: blue;
}

#my-id {
    width: 100px;
    height: 50px;
}

.another-class,
.multiple-class {
    margin: 10px;
}

#another-id {
    padding: 5px;
}

.nested .child-class {
    font-size: 14px;
}

.parent > .direct-child {
    border: 1px solid black;
}

.hover:hover {
    opacity: 0.8;
}
"#;

/// Test content with complex selectors
const COMPLEX_USS_CONTENT: &str = r#"
.container .item:first-child {
    color: green;
}

#main-content .sidebar .widget {
    background: white;
}

.btn.primary,
.btn.secondary {
    padding: 8px 16px;
}

.form-group > .input-field {
    width: 100%;
}
"#;

fn create_parser() -> Parser {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_css::LANGUAGE.into())
        .expect("Error loading CSS language");
    parser
}

#[test]
fn test_print_basic_selector_tree() {
    println!("\n=== BASIC SELECTOR TREE ANALYSIS ===");
    let mut parser = create_parser();
    let tree = parser
        .parse(TEST_USS_CONTENT, None)
        .expect("Error parsing USS");
    
    print_tree_to_stdout(tree.root_node(), TEST_USS_CONTENT);
}

#[test]
fn test_print_complex_selector_tree() {
    println!("\n=== COMPLEX SELECTOR TREE ANALYSIS ===");
    let mut parser = create_parser();
    let tree = parser
        .parse(COMPLEX_USS_CONTENT, None)
        .expect("Error parsing USS");
    
    print_tree_to_stdout(tree.root_node(), COMPLEX_USS_CONTENT);
}

#[test]
fn test_print_single_class_selector() {
    println!("\n=== SINGLE CLASS SELECTOR ANALYSIS ===");
    let content = ".test-class { color: red; }";
    let mut parser = create_parser();
    let tree = parser.parse(content, None).expect("Error parsing USS");
    
    print_tree_to_stdout(tree.root_node(), content);
}

#[test]
fn test_print_single_id_selector() {
    println!("\n=== SINGLE ID SELECTOR ANALYSIS ===");
    let content = "#test-id { width: 100px; }";
    let mut parser = create_parser();
    let tree = parser.parse(content, None).expect("Error parsing USS");
    
    print_tree_to_stdout(tree.root_node(), content);
}

#[test]
fn test_print_multiple_selectors() {
    println!("\n=== MULTIPLE SELECTORS ANALYSIS ===");
    let content = ".class1, .class2, #id1 { margin: 0; }";
    let mut parser = create_parser();
    let tree = parser.parse(content, None).expect("Error parsing USS");
    
    print_tree_to_stdout(tree.root_node(), content);
}

#[test]
fn test_print_nested_selectors() {
    println!("\n=== NESTED SELECTORS ANALYSIS ===");
    let content = ".parent .child { font-size: 12px; }";
    let mut parser = create_parser();
    let tree = parser.parse(content, None).expect("Error parsing USS");
    
    print_tree_to_stdout(tree.root_node(), content);
}

#[test]
fn test_print_pseudo_class_selectors() {
    println!("\n=== PSEUDO CLASS SELECTORS ANALYSIS ===");
    let content = ".button:hover, .link:active { color: blue; }";
    let mut parser = create_parser();
    let tree = parser.parse(content, None).expect("Error parsing USS");
    
    print_tree_to_stdout(tree.root_node(), content);
}

#[test]
fn test_refactor_provider_creation() {
    let provider = UssRefactorProvider::new();
    // Just test that we can create the provider
    assert!(true); // Placeholder assertion
}

#[test]
fn test_find_class_selector_references() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let tree = parser
        .parse(TEST_USS_CONTENT, None)
        .expect("Error parsing USS");
    
    let references = provider.find_selector_references(
        &tree,
        TEST_USS_CONTENT,
        "my-class",
        SelectorType::Class,
    );
    
    println!("Found {} references to .my-class", references.len());
    assert_eq!(references.len(), 1, "Should find exactly one reference to .my-class");
    
    // Test another class that appears multiple times
    let references = provider.find_selector_references(
        &tree,
        TEST_USS_CONTENT,
        "another-class",
        SelectorType::Class,
    );
    
    println!("Found {} references to .another-class", references.len());
    assert_eq!(references.len(), 1, "Should find exactly one reference to .another-class");
}

#[test]
fn test_find_id_selector_references() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let tree = parser
        .parse(TEST_USS_CONTENT, None)
        .expect("Error parsing USS");
    
    let references = provider.find_selector_references(
        &tree,
        TEST_USS_CONTENT,
        "my-id",
        SelectorType::Id,
    );
    
    println!("Found {} references to #my-id", references.len());
    assert_eq!(references.len(), 1, "Should find exactly one reference to #my-id");
    
    // Test another ID
    let references = provider.find_selector_references(
        &tree,
        TEST_USS_CONTENT,
        "another-id",
        SelectorType::Id,
    );
    
    println!("Found {} references to #another-id", references.len());
    assert_eq!(references.len(), 1, "Should find exactly one reference to #another-id");
}

#[test]
fn test_rename_class_selector() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let content = ".old-class { color: red; } .old-class:hover { color: blue; }";
    let tree = parser.parse(content, None).expect("Error parsing USS");
    let uri = Url::parse("file:///test.uss").unwrap();
    
    let workspace_edit = provider.rename_selector(
        &tree,
        content,
        &uri,
        "old-class",
        "new-class",
        SelectorType::Class,
    );
    
    assert!(workspace_edit.is_some(), "Should generate workspace edit");
    let edit = workspace_edit.unwrap();
    
    if let Some(changes) = edit.changes {
        let file_changes = changes.get(&uri).expect("Should have changes for the file");
        assert_eq!(file_changes.len(), 2, "Should have 2 text edits for 2 occurrences");
        
        for text_edit in file_changes {
            assert_eq!(text_edit.new_text, "new-class", "Should replace with new class name");
        }
    } else {
        panic!("Should have changes in workspace edit");
    }
}

#[test]
fn test_rename_id_selector() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let content = "#old-id { width: 100px; }";
    let tree = parser.parse(content, None).expect("Error parsing USS");
    let uri = Url::parse("file:///test.uss").unwrap();
    
    let workspace_edit = provider.rename_selector(
        &tree,
        content,
        &uri,
        "old-id",
        "new-id",
        SelectorType::Id,
    );
    
    assert!(workspace_edit.is_some(), "Should generate workspace edit");
    let edit = workspace_edit.unwrap();
    
    if let Some(changes) = edit.changes {
        let file_changes = changes.get(&uri).expect("Should have changes for the file");
        assert_eq!(file_changes.len(), 1, "Should have 1 text edit");
        
        let text_edit = &file_changes[0];
        assert_eq!(text_edit.new_text, "new-id", "Should replace with new ID name");
    } else {
        panic!("Should have changes in workspace edit");
    }
}

#[test]
fn test_rename_nonexistent_selector() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let content = ".existing-class { color: red; }";
    let tree = parser.parse(content, None).expect("Error parsing USS");
    let uri = Url::parse("file:///test.uss").unwrap();
    
    let workspace_edit = provider.rename_selector(
        &tree,
        content,
        &uri,
        "nonexistent-class",
        "new-class",
        SelectorType::Class,
    );
    
    assert!(workspace_edit.is_none(), "Should not generate workspace edit for nonexistent selector");
}

#[test]
fn test_multiple_selectors_in_one_rule() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let content = ".class1, .class2, .class1 { margin: 0; }";
    let tree = parser.parse(content, None).expect("Error parsing USS");
    
    let references = provider.find_selector_references(
        &tree,
        content,
        "class1",
        SelectorType::Class,
    );
    
    println!("Found {} references to .class1 in multiple selector rule", references.len());
    assert_eq!(references.len(), 2, "Should find both occurrences of .class1");
}

#[test]
fn test_nested_selectors() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let content = ".parent .child { color: red; } .child { color: blue; }";
    let tree = parser.parse(content, None).expect("Error parsing USS");
    
    let references = provider.find_selector_references(
        &tree,
        content,
        "child",
        SelectorType::Class,
    );
    
    println!("Found {} references to .child in nested selectors", references.len());
    assert_eq!(references.len(), 2, "Should find both occurrences of .child");
}
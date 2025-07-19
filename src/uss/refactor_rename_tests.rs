//! Tests for USS refactoring functionality
//!
//! These tests help understand the tree structure and validate refactoring operations.

use std::sync::Arc;

use crate::{language::tree_utils::find_node_at_position, uss::definitions::UssDefinitions};
use crate::uss::parser::UssParser;
use crate::uss::refactor::*;
use tower_lsp::lsp_types::{Position, PrepareRenameResponse, Range};
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
fn test_find_class_selector_references() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let tree = parser
        .parse(TEST_USS_CONTENT, None)
        .expect("Error parsing USS");
    
    let references = provider.find_selector_references(
        tree.root_node(),
        TEST_USS_CONTENT,
        "my-class",
        SelectorType::Class,
    );
    
    assert_eq!(references.len(), 1, "Should find exactly one reference to .my-class");
    
    // Test another class that appears multiple times
    let references = provider.find_selector_references(
        tree.root_node(),
        TEST_USS_CONTENT,
        "another-class",
        SelectorType::Class,
    );
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
        tree.root_node(),
        TEST_USS_CONTENT,
        "my-id",
        SelectorType::Id,
    );
    
    assert_eq!(references.len(), 1, "Should find exactly one reference to #my-id");
    
    // Test another ID
    let references = provider.find_selector_references(
        tree.root_node(),
        TEST_USS_CONTENT,
        "another-id",
        SelectorType::Id,
    );
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
        tree.root_node(),
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
        tree.root_node(),
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
        tree.root_node(),
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
        tree.root_node(),
        content,
        "class1",
        SelectorType::Class,
    );
    

    assert_eq!(references.len(), 2, "Should find both occurrences of .class1");
}

#[test]
fn test_nested_selectors() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let content = ".parent .child { color: red; } .child { color: blue; }";
    let tree = parser.parse(content, None).expect("Error parsing USS");
    
    let references = provider.find_selector_references(
        tree.root_node(),
        content,
        "child",
        SelectorType::Class,
    );
    

    assert_eq!(references.len(), 2, "Should find both occurrences of .child");
}

/// Test content with complex chained selectors
const CHAINED_SELECTORS_CONTENT: &str = r#"
.c #a.class#b.my-class.class2:hover:active {
    color: red;
    background: blue;
}

.container .item.active:first-child:hover {
    transform: scale(1.1);
}

#main-nav .menu-item.selected.highlighted:focus:not(.disabled) {
    border: 2px solid gold;
}

.form .input-group .field.required:valid:focus {
    border-color: green;
}

.sidebar .widget.collapsible.expanded > .content {
    display: block;
}

.grid .row .col.sm-6.md-4.lg-3:nth-child(odd) {
    background: #f5f5f5;
}

.btn.primary.large:hover:not(:disabled) {
    box-shadow: 0 4px 8px rgba(0,0,0,0.2);
}

.modal.open .overlay.dark + .dialog.centered {
    opacity: 1;
}
"#;

#[test]
fn test_find_class_in_complex_chained_selector() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let tree = parser
        .parse(CHAINED_SELECTORS_CONTENT, None)
        .expect("Error parsing USS");
    
    // Test finding 'my-class' in the complex selector .c #a.class#b.my-class.class2:hover:active
    let references = provider.find_selector_references(
        tree.root_node(),
        CHAINED_SELECTORS_CONTENT,
        "my-class",
        SelectorType::Class,
    );
    

    assert_eq!(references.len(), 1, "Should find .my-class in the complex chained selector");
    
    // Test finding 'class2' in the same selector
    let references = provider.find_selector_references(
        tree.root_node(),
        CHAINED_SELECTORS_CONTENT,
        "class2",
        SelectorType::Class,
    );
    

    assert_eq!(references.len(), 1, "Should find .class2 in the complex chained selector");
}

#[test]
fn test_find_id_in_complex_chained_selector() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let tree = parser
        .parse(CHAINED_SELECTORS_CONTENT, None)
        .expect("Error parsing USS");
    
    // Test finding 'a' in the complex selector .c #a.class#b.my-class.class2:hover:active
    let references = provider.find_selector_references(
        tree.root_node(),
        CHAINED_SELECTORS_CONTENT,
        "a",
        SelectorType::Id,
    );
    

    assert_eq!(references.len(), 1, "Should find #a in the complex chained selector");
    
    // Test finding 'b' in the same selector
    let references = provider.find_selector_references(
        tree.root_node(),
        CHAINED_SELECTORS_CONTENT,
        "b",
        SelectorType::Id,
    );
    

    assert_eq!(references.len(), 1, "Should find #b in the complex chained selector");
}

#[test]
fn test_rename_class_in_chained_selector() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let content = ".c #a.class#b.my-class.class2:hover:active { color: red; }";
    let tree = parser.parse(content, None).expect("Error parsing USS");
    let uri = Url::parse("file:///test.uss").unwrap();
    
    let workspace_edit = provider.rename_selector(
        tree.root_node(),
        content,
        &uri,
        "my-class",
        "new-class",
        SelectorType::Class,
    );
    
    assert!(workspace_edit.is_some(), "Should generate workspace edit for chained selector");
    let edit = workspace_edit.unwrap();
    
    if let Some(changes) = edit.changes {
        let file_changes = changes.get(&uri).expect("Should have changes for the file");
        assert_eq!(file_changes.len(), 1, "Should have 1 text edit for .my-class");
        
        let text_edit = &file_changes[0];
        assert_eq!(text_edit.new_text, "new-class", "Should replace with new class name");
        
        // Verify the range is correct (should only replace 'my-class', not the dot)
        let replaced_text = &content[text_edit.range.start.character as usize..text_edit.range.end.character as usize];
        assert_eq!(replaced_text, "my-class", "Should only replace the class name without the dot");
    } else {
        panic!("Should have changes in workspace edit");
    }
}

#[test]
fn test_rename_id_in_chained_selector() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let content = ".c #a.class#b.my-class.class2:hover:active { color: red; }";
    let tree = parser.parse(content, None).expect("Error parsing USS");
    let uri = Url::parse("file:///test.uss").unwrap();
    
    let workspace_edit = provider.rename_selector(
        tree.root_node(),
        content,
        &uri,
        "b",
        "new-id",
        SelectorType::Id,
    );
    
    assert!(workspace_edit.is_some(), "Should generate workspace edit for chained selector");
    let edit = workspace_edit.unwrap();
    
    if let Some(changes) = edit.changes {
        let file_changes = changes.get(&uri).expect("Should have changes for the file");
        assert_eq!(file_changes.len(), 1, "Should have 1 text edit for #b");
        
        let text_edit = &file_changes[0];
        assert_eq!(text_edit.new_text, "new-id", "Should replace with new ID name");
        
        // Verify the range is correct (should only replace 'b', not the hash)
        let replaced_text = &content[text_edit.range.start.character as usize..text_edit.range.end.character as usize];
        assert_eq!(replaced_text, "b", "Should only replace the ID name without the hash");
    } else {
        panic!("Should have changes in workspace edit");
    }
}

#[test]
fn test_multiple_classes_in_complex_selectors() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let content = r#"
.container .item.active:first-child:hover { color: red; }
.sidebar .item.inactive { color: gray; }
.item.special { font-weight: bold; }
"#;
    let tree = parser.parse(content, None).expect("Error parsing USS");
    
    // Test finding 'item' which appears in multiple complex selectors
    let references = provider.find_selector_references(
        tree.root_node(),
        content,
        "item",
        SelectorType::Class,
    );
    

    assert_eq!(references.len(), 3, "Should find all 3 occurrences of .item");
}

#[test]
fn test_pseudo_classes_with_chained_selectors() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let content = ".btn.primary.large:hover:not(:disabled) { box-shadow: 0 4px 8px rgba(0,0,0,0.2); }";
    let tree = parser.parse(content, None).expect("Error parsing USS");
    
    // Test finding classes in selector with pseudo-classes
    let references = provider.find_selector_references(
        tree.root_node(),
        content,
        "primary",
        SelectorType::Class,
    );
    

    assert_eq!(references.len(), 1, "Should find .primary in selector with pseudo-classes");
    
    let references = provider.find_selector_references(
        tree.root_node(),
        content,
        "large",
        SelectorType::Class,
    );
    

    assert_eq!(references.len(), 1, "Should find .large in selector with pseudo-classes");
}

#[test]
fn test_descendant_and_child_combinators() {
    let provider = UssRefactorProvider::new();
    let mut parser = create_parser();
    let content = r#"
.sidebar .widget.collapsible.expanded > .content { display: block; }
.modal.open .overlay.dark + .dialog.centered { opacity: 1; }
"#;
    let tree = parser.parse(content, None).expect("Error parsing USS");
    
    // Test finding 'content' in child combinator
    let references = provider.find_selector_references(
        tree.root_node(),
        content,
        "content",
        SelectorType::Class,
    );
    

    assert_eq!(references.len(), 1, "Should find .content with child combinator");
    
    // Test finding 'dialog' in adjacent sibling combinator
    let references = provider.find_selector_references(
        tree.root_node(),
        content,
        "dialog",
        SelectorType::Class,
    );
    

    assert_eq!(references.len(), 1, "Should find .dialog with adjacent sibling combinator");
}

#[test]
fn test_handle_rename_chained_selector() {
    use crate::uss::document::UssDocument;
    use tower_lsp::lsp_types::{Position, TextEdit, Range};
    use url::Url;
    
    let provider = UssRefactorProvider::new();
    let content = ".class1.class2 { color: red; }\n.class1 { margin: 10px; }\n.class2 { padding: 5px; }";
    let uri = Url::parse("file:///test.uss").unwrap();
    let mut document = UssDocument::new(uri.clone(), content.to_string(), 1, Arc::new(UssDefinitions::new()));
    
    // Parse the document to create the syntax tree
    let mut parser = crate::uss::parser::UssParser::new().expect("Failed to create USS parser");
    document.parse(&mut parser);
    
    // Ensure we have a valid tree
    assert!(document.tree().is_some(), "Document should have a valid syntax tree");
    let tree = document.tree().unwrap();
    
    // Test renaming class1 to newclass1
    let position = Position::new(0, 1); // Position at start of class1
    let new_name = "newclass1";
    
    let workspace_edit = provider.handle_rename(tree.root_node(), document.content(), &uri, position, new_name);
    assert!(workspace_edit.is_some(), "handle_rename should return a WorkspaceEdit");
    
    let edit = workspace_edit.unwrap();
    assert!(edit.changes.is_some(), "WorkspaceEdit should contain changes");
    
    let changes = edit.changes.unwrap();
    assert!(changes.contains_key(&uri), "Changes should contain edits for the test file");
    
    let text_edits = &changes[&uri];
    
    // We expect at least 2 edits: one for .class1.class2 and one for standalone .class1
    assert!(text_edits.len() >= 2, "Should have at least 2 text edits, got {}", text_edits.len());
    
    // Check that the edits contain the expected replacements
    let mut found_chained = false;
    let mut found_standalone = false;
    
    for edit in text_edits {
        let edit_text = &content[edit.range.start.character as usize..edit.range.end.character as usize];
        
        if edit_text == "class1" {
            assert_eq!(edit.new_text, new_name, "Replacement text should be '{}'", new_name);
            
            // Check if this is the chained selector or standalone
            if edit.range.start.line == 0 {
                found_chained = true;
            } else if edit.range.start.line == 1 {
                found_standalone = true;
            }
        }
    }
    
    assert!(found_chained, "Should find and rename class1 in chained selector .class1.class2");
    assert!(found_standalone, "Should find and rename standalone .class1 selector");
    

}

#[test]
fn test_prepare_rename_chained_selector() {
    let provider = UssRefactorProvider::new();
    let content = "#name.class2.class3:hover { color: red; } .class1.class2#id1 { margin: 10px; }";
    let uri = Url::parse("file:///test.uss").unwrap();
    
    // Parse the document to create the syntax tree
    let mut parser = UssParser::new().expect("Failed to create USS parser");
    let tree = parser.parse(&content, None);
    
    // Ensure we have a valid tree
    assert!(tree.is_some(), "Document should have a valid syntax tree");
    let tree = tree.unwrap();

    let position = Position::new(0, 9); // in .class2
    let result = provider.prepare_rename(tree.root_node(), content, position);
    
    assert!(result.is_some(), 
                "prepare_rename should succeed");
            
    let expectedRange = Range::new(
        Position::new(0, 6),
        Position::new(0, 12),
    );
    let response = result.unwrap();
    match response {
        PrepareRenameResponse::Range(range) => assert_eq!(range, expectedRange),
        PrepareRenameResponse::RangeWithPlaceholder { range, placeholder } => assert_eq!(range, expectedRange),
        PrepareRenameResponse::DefaultBehavior { default_behavior } => {},
    }
}

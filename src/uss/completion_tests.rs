use tower_lsp::lsp_types::Position;
use std::path::PathBuf;

use crate::uss::{completion::{CompletionType, UssCompletionProvider}, parser::UssParser};
use crate::unity_project_manager::UnityProjectManager;

#[test]
fn test_completion_provider_creation() {
    let provider = UssCompletionProvider::new();
    assert!(provider.definitions.is_valid_property("color"));
}

#[test]
fn test_pseudo_class_completion() {
    let provider = UssCompletionProvider::new();
    let completions = provider.complete_pseudo_classes();
    
    // Should include common pseudo-classes
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(labels.contains(&"hover".to_string()));
    assert!(labels.contains(&"active".to_string()));
    assert!(labels.contains(&"focus".to_string()));
}

#[test]
fn test_simple_completion_after_colon() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();
    
    // Test case: cursor right after colon
    let content = ".some { \n    color: \n}";
    let length = content.len();
    let tree = parser.parse(content, None).unwrap();
    
    // Position right at the colon (line 1, character 9 - at the ':')
    let position = Position {
        line: 1,
        character: 10,
    };
    
    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
    );

    // Should have completions for color property
    assert!(!completions.is_empty(), "Should have completions after colon");
    
    // Verify we have some expected color completions
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(labels.contains(&"red".to_string()), "Should include 'red' color");
    assert!(labels.contains(&"blue".to_string()), "Should include 'blue' color");
    assert!(labels.contains(&"transparent".to_string()), "Should include 'transparent' keyword");
    
    // Should have a reasonable number of completions (color has many options)
    assert!(completions.len() > 50, "Should have many color completion options");
}
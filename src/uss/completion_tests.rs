use tower_lsp::lsp_types::Position;
use std::path::PathBuf;

use crate::uss::{completion::UssCompletionProvider, parser::UssParser};
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

#[test]
fn test_property_name_completion() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();
    
    // Test case: typing property name
    let content = ".some { \n    col\n}";
    let tree = parser.parse(content, None).unwrap();
    
    // Position at the end of "col" (line 1, character 7)
    let position = Position {
        line: 1,
        character: 7,
    };
    
    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
    );

    // Should have completions for properties starting with "col"
    assert!(!completions.is_empty(), "Should have property name completions");
    
    // Verify we have expected property completions
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(labels.contains(&"color".to_string()), "Should include 'color' property");
    
    // Check that completions include colon and space in insert text
    let color_completion = completions.iter().find(|c| c.label == "color").unwrap();
    assert_eq!(color_completion.insert_text, Some("color: ".to_string()), "Should include colon and space");
    
    // Should be property kind
    assert_eq!(color_completion.kind, Some(tower_lsp::lsp_types::CompletionItemKind::PROPERTY));
}

#[test]
fn test_property_name_completion_empty() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();
    
    // Test case: empty property name (just after opening brace)
    let content = ".some { \n    \n}";
    let tree = parser.parse(content, None).unwrap();
    
    // Position at the beginning of line 1 (where property would start)
    let position = Position {
        line: 1,
        character: 4,
    };
    
    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
    );

    // Should not provide completions when no text has been typed
    // This is the expected behavior - user must type at least one character
    assert!(completions.is_empty(), "Should not provide completions for empty property name");
}

#[test]
fn test_property_name_completion_partial_match() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();
    
    // Test case: partial property name that should match multiple properties
    let content = ".some { \n    back\n}";
    let tree = parser.parse(content, None).unwrap();
    
    // Position at the end of "back"
    let position = Position {
        line: 1,
        character: 8,
    };
    
    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
    );

    // Should have completions for properties starting with "back"
    assert!(!completions.is_empty(), "Should have property name completions");
    
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(labels.contains(&"background-color".to_string()), "Should include 'background-color' property");
    assert!(labels.contains(&"background-image".to_string()), "Should include 'background-image' property");
    
    // Should not include properties that don't start with "back"
    assert!(!labels.contains(&"color".to_string()), "Should not include 'color' property");
}



#[test]
fn test_property_name_completion_case_insensitive() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();
    
    // Test case: uppercase partial property name
    let content = ".some { \n    COL\n}";
    let tree = parser.parse(content, None).unwrap();
    
    // Position at the end of "COL"
    let position = Position {
        line: 1,
        character: 7,
    };
    
    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
    );

    // Should have completions for properties starting with "col" (case insensitive)
    assert!(!completions.is_empty(), "Should have property name completions");
    
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(labels.contains(&"color".to_string()), "Should include 'color' property");
}
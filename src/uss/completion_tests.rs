use tower_lsp::lsp_types::Position;
use std::path::PathBuf;

use crate::uss::{completion::{CompletionContext, UssCompletionProvider}, parser::UssParser};
use crate::unity_project_manager::UnityProjectManager;

#[test]
fn test_completion_provider_creation() {
    let provider = UssCompletionProvider::new();
    assert!(provider.definitions.is_valid_property("color"));
}

#[test]
fn test_property_value_completion() {
    let provider = UssCompletionProvider::new();
    
    // Test 1: Right after colon (empty partial value) should provide completions
    let completions_empty = provider.complete_property_value("color", "");
    let labels_empty: Vec<String> = completions_empty.iter().map(|c| c.label.clone()).collect();
    assert!(labels_empty.contains(&"red".to_string()), "Should include 'red' when partial value is empty");
    
    // Test 2: With partial value for non-keyword-only property should return empty
    let completions_partial = provider.complete_property_value("color", "r");
    assert!(completions_partial.is_empty(), "Should not provide completions for color with partial value 'r'");
    
    // Test 3: With partial value for keyword-only property should provide filtered results
    let completions_display = provider.complete_property_value("display", "f");
    let labels_display: Vec<String> = completions_display.iter().map(|c| c.label.clone()).collect();
    assert!(labels_display.contains(&"flex".to_string()), "Should include 'flex' for display with partial value 'f'");
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
fn test_completion_context_detection() {
    let provider = UssCompletionProvider::new();
    let mut parser = UssParser::new().expect("Failed to create parser");
    
    // Test property value context
    let content = "Button { color: r";
    let tree = parser.parse(content, None).expect("Failed to parse");
    let position = Position { line: 0, character: 16 }; // At 'r'
    
    let context = provider.get_completion_context(&tree, content, position);
        
        // Verify that we correctly detect PropertyValue context
        match context {
            CompletionContext::PropertyValue { property_name, partial_value } => {
                assert_eq!(property_name, "color");
                assert_eq!(partial_value, "r"); // Should extract the partial value 'r'
            }
            _ => {
                panic!("Expected PropertyValue context, got: {:?}", context);
            }
        }
}
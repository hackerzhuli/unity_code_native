use tower_lsp::lsp_types::Position;

use crate::uss::{completion::{CompletionContext, UssCompletionProvider}, parser::UssParser};

#[test]
fn test_completion_provider_creation() {
    let provider = UssCompletionProvider::new();
    assert!(provider.definitions.is_valid_property("color"));
}

#[test]
fn test_property_value_completion() {
    let provider = UssCompletionProvider::new();
    let completions = provider.complete_property_value("color", "r");
    
    // Should include color values starting with 'r'
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(labels.contains(&"red".to_string()));
    assert!(labels.contains(&"rgb(255, 255, 255)".to_string()));
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
                assert_eq!(partial_value, ""); // Empty for now, will be enhanced later
            }
            _ => {
                panic!("Expected PropertyValue context, got: {:?}", context);
            }
        }
}
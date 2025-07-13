use std::path::PathBuf;
use tower_lsp::lsp_types::{CompletionItemKind, Position};
use tree_sitter::Node;

use crate::test_utils::get_project_root;
use crate::unity_project_manager::UnityProjectManager;
use crate::uss::{completion::UssCompletionProvider, parser::UssParser};

// Helper function to print tree structure for debugging
fn print_tree_recursive(node: Node, content: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    let text = node.utf8_text(content.as_bytes()).unwrap_or("<invalid>");
    println!("{}{}[{}]: '{}'", indent, node.kind(), node.id(), text);

    for child in node.children(&mut node.walk()) {
        print_tree_recursive(child, content, depth + 1);
    }
}

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
fn test_pseudo_class_completion_after_colon() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: cursor right after colon in selector
    let content = ".button: ";
    let tree = parser.parse(content, None).unwrap();

    // Position right after the colon
    let position = Position {
        line: 0,
        character: 8,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should have pseudo-class completions
    assert!(
        !completions.is_empty(),
        "Should have pseudo-class completions after colon"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"hover".to_string()),
        "Should include 'hover' pseudo-class"
    );
    assert!(
        labels.contains(&"active".to_string()),
        "Should include 'active' pseudo-class"
    );
    assert!(
        labels.contains(&"focus".to_string()),
        "Should include 'focus' pseudo-class"
    );
}

#[test]
fn test_pseudo_class_completion_partial_match() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: partial pseudo-class name
    let content = ".button:h";
    let tree = parser.parse(content, None).unwrap();

    // Position after 'h'
    let position = Position {
        line: 0,
        character: 9,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should have filtered pseudo-class completions
    assert!(
        !completions.is_empty(),
        "Should have filtered pseudo-class completions"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"hover".to_string()),
        "Should include 'hover' pseudo-class"
    );
    assert!(
        !labels.contains(&"active".to_string()),
        "Should not include 'active' pseudo-class"
    );
    assert!(
        !labels.contains(&"focus".to_string()),
        "Should not include 'focus' pseudo-class"
    );
}

#[test]
fn test_pseudo_class_completion_case_insensitive() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: uppercase partial pseudo-class name
    let content = ".button:H";
    let tree = parser.parse(content, None).unwrap();

    // Position after 'H'
    let position = Position {
        line: 0,
        character: 9,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should have case-insensitive filtered completions
    assert!(
        !completions.is_empty(),
        "Should have case-insensitive pseudo-class completions"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"hover".to_string()),
        "Should include 'hover' pseudo-class (case insensitive)"
    );
}

#[test]
fn test_pseudo_class_completion_multiple_selectors() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: pseudo-class after class selector in complex selector
    let content = ".button.primary:";
    let tree = parser.parse(content, None).unwrap();

    // Position right after the colon
    let position = Position {
        line: 0,
        character: 16,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should have pseudo-class completions
    assert!(
        !completions.is_empty(),
        "Should have pseudo-class completions in complex selector"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"hover".to_string()),
        "Should include 'hover' pseudo-class"
    );
}

#[test]
fn test_no_pseudo_class_completion_in_property_value() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: colon in property declaration should not trigger pseudo-class completion
    let content = ".button { color: ";
    let tree = parser.parse(content, None).unwrap();

    // Position right after the colon in property declaration
    let position = Position {
        line: 0,
        character: 16,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should not have pseudo-class completions (should have color value completions instead)
    let pseudo_class_completions: Vec<_> = completions
        .iter()
        .filter(|c| c.kind == Some(CompletionItemKind::KEYWORD) && c.detail == Some("Pseudo-class".to_string()))
        .collect();
    
    assert!(
        pseudo_class_completions.is_empty(),
        "Should not provide pseudo-class completions in property value context"
    );
}

#[test]
fn test_property_value_simple_completion_after_colon() {
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
        None,
    );

    // Should have completions for color property
    assert!(
        !completions.is_empty(),
        "Should have completions after colon"
    );

    // Verify we have some expected color completions
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"red".to_string()),
        "Should include 'red' color"
    );
    assert!(
        labels.contains(&"blue".to_string()),
        "Should include 'blue' color"
    );
    assert!(
        labels.contains(&"transparent".to_string()),
        "Should include 'transparent' keyword"
    );

    // Should have a reasonable number of completions (color has many options)
    assert!(
        completions.len() > 50,
        "Should have many color completion options"
    );
}

#[test]
fn test_property_value_completion_after_typing() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: cursor right after colon
    let content = ".some { \n    color: ro \n}";
    let length = content.len();
    let tree = parser.parse(content, None).unwrap();

    // Position right after ro
    let position = Position {
        line: 1,
        character: 13,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should have completions for color property
    assert!(
        !completions.is_empty(),
        "Should have completions after colon"
    );

    // Verify we have some expected color completions
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"rosybrown".to_string()),
        "Should include 'rosybrown' color"
    );
    assert!(
        labels.contains(&"royalblue".to_string()),
        "Should include 'royalblue' color"
    );
}

#[test]
fn test_property_value_completion_after_typing_keyword() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: cursor right after ro
    let content = ".some { \n    flex-direction: ro \n}";
    let length = content.len();
    let tree = parser.parse(content, None).unwrap();

    // Position right after ro
    let position = Position {
        line: 1,
        character: 22,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    assert!(
        !completions.is_empty(),
        "Should have completions after colon"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"row".to_string()),
        "Should include 'row' keyword"
    );
    assert!(
        labels.contains(&"row-reverse".to_string()),
        "Should include 'row-reverse' keyword"
    );
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
        None,
    );

    // Should have completions for properties starting with "col"
    assert!(
        !completions.is_empty(),
        "Should have property name completions"
    );

    // Verify we have expected property completions
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"color".to_string()),
        "Should include 'color' property"
    );

    // Check that completions include colon and space in insert text
    let color_completion = completions.iter().find(|c| c.label == "color").unwrap();
    assert_eq!(
        color_completion.insert_text,
        Some("color: ".to_string()),
        "Should include colon and space"
    );

    // Should be property kind
    assert_eq!(
        color_completion.kind,
        Some(tower_lsp::lsp_types::CompletionItemKind::PROPERTY)
    );
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
        None,
    );

    // Should not provide completions when no text has been typed
    // This is the expected behavior - user must type at least one character
    assert!(
        completions.is_empty(),
        "Should not provide completions for empty property name"
    );
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
        None,
    );

    // Should have completions for properties starting with "back"
    assert!(
        !completions.is_empty(),
        "Should have property name completions"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"background-color".to_string()),
        "Should include 'background-color' property"
    );
    assert!(
        labels.contains(&"background-image".to_string()),
        "Should include 'background-image' property"
    );

    // Should not include properties that don't start with "back"
    assert!(
        !labels.contains(&"color".to_string()),
        "Should not include 'color' property"
    );
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
        None,
    );

    // Should have completions for properties starting with "col" (case insensitive)
    assert!(
        !completions.is_empty(),
        "Should have property name completions"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"color".to_string()),
        "Should include 'color' property"
    );
}

#[test]
fn test_class_selector_completion_after_dot() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: completion after typing '.'
    let content = ".my-class { color: red; }\n.another-class { margin: 10px; }\n.";
    let tree = parser.parse(content, None).unwrap();

    // Position right after the last '.'
    let position = Position {
        line: 2,
        character: 1,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should have completions for existing class selectors
    assert!(
        !completions.is_empty(),
        "Should have class selector completions"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"my-class".to_string()),
        "Should include 'my-class'"
    );
    assert!(
        labels.contains(&"another-class".to_string()),
        "Should include 'another-class'"
    );

    // Verify completion item properties
    let my_class_completion = completions.iter().find(|c| c.label == "my-class").unwrap();
    //assert_eq!(my_class_completion.kind, Some(CompletionItemKind::CLASS));
    assert_eq!(
        my_class_completion.detail,
        Some("Class selector".to_string())
    );
    assert_eq!(
        my_class_completion.insert_text,
        Some("my-class".to_string())
    );
}

#[test]
fn test_class_selector_partial_completion() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: partial class name completion
    let content =
        ".my-class { color: red; }\n.my-other { margin: 10px; }\n.another { padding: 5px; }\n.my";
    let tree = parser.parse(content, None).unwrap();

    // Position after '.my'
    let position = Position {
        line: 3,
        character: 3,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should have completions for classes starting with 'my'
    assert!(
        !completions.is_empty(),
        "Should have partial class completions"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"my-class".to_string()),
        "Should include 'my-class'"
    );
    assert!(
        labels.contains(&"my-other".to_string()),
        "Should include 'my-other'"
    );
    assert!(
        !labels.contains(&"another".to_string()),
        "Should not include 'another'"
    );
}

#[test]
fn test_id_selector_completion_after_hash() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: completion after typing '#'
    let content =
        "#my-id { color: red; }\n#another-id { margin: 10px; }\n.some-class { padding: 5px; }\n#";
    let tree = parser.parse(content, None).unwrap();

    // Position right after the last '#'
    let position = Position {
        line: 3,
        character: 1,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should have completions for existing ID selectors only
    assert!(
        !completions.is_empty(),
        "Should have ID selector completions"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"my-id".to_string()),
        "Should include 'my-id'"
    );
    assert!(
        labels.contains(&"another-id".to_string()),
        "Should include 'another-id'"
    );
    assert!(
        !labels.contains(&"some-class".to_string()),
        "Should not include class selectors"
    );

    // Verify completion item properties
    let my_id_completion = completions.iter().find(|c| c.label == "my-id").unwrap();
    assert_eq!(my_id_completion.kind, Some(CompletionItemKind::CONSTANT));
    assert_eq!(my_id_completion.detail, Some("ID selector".to_string()));
    assert_eq!(my_id_completion.insert_text, Some("my-id".to_string()));
}

#[test]
fn test_id_selector_partial_completion() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: partial ID name completion
    let content =
        "#my-id { color: red; }\n#my-other-id { margin: 10px; }\n#different { padding: 5px; }\n#my";
    let tree = parser.parse(content, None).unwrap();

    // Position after '#my'
    let position = Position {
        line: 3,
        character: 3,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should have completions for IDs starting with 'my'
    assert!(
        !completions.is_empty(),
        "Should have partial ID completions"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"my-id".to_string()),
        "Should include 'my-id'"
    );
    assert!(
        labels.contains(&"my-other-id".to_string()),
        "Should include 'my-other-id'"
    );
    assert!(
        !labels.contains(&"different".to_string()),
        "Should not include 'different'"
    );
}

#[test]
fn test_selector_completion_case_insensitive() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: case insensitive selector completion
    let content = ".MyClass { color: red; }\n.ANOTHER { margin: 10px; }\n.my";
    let tree = parser.parse(content, None).unwrap();

    // Position after '.my'
    let position = Position {
        line: 2,
        character: 3,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should have case-insensitive completions
    assert!(
        !completions.is_empty(),
        "Should have case-insensitive completions"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"MyClass".to_string()),
        "Should include 'MyClass' (case preserved)"
    );
}

#[test]
fn test_no_selector_completion_in_declaration_block() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: should not provide selector completion inside declaration blocks
    let content = ".my-class {\n    color: .\n}";
    let tree = parser.parse(content, None).unwrap();

    // Position after the '.' inside the declaration block
    let position = Position {
        line: 1,
        character: 12,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should not provide selector completions inside declaration blocks
    // (This should either be empty or provide property value completions)
    let class_completions: Vec<_> = completions
        .iter()
        .filter(|c| c.kind == Some(CompletionItemKind::CLASS))
        .collect();
    assert!(
        class_completions.is_empty(),
        "Should not provide class selector completions inside declaration blocks"
    );
}

#[test]
fn test_tag_selector_completion() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: completion for partial tag name
    let content = "Button { color: red; }\nLabel { margin: 10px; }\nBu";
    let tree = parser.parse(content, None).unwrap();

    // Position after 'Bu'
    let position = Position {
        line: 2,
        character: 2,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should have completions for tags starting with 'Bu'
    assert!(!completions.is_empty(), "Should have tag completions");

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"Button".to_string()),
        "Should include 'Button' tag"
    );
    assert!(
        !labels.contains(&"Label".to_string()),
        "Should not include 'Label' tag"
    );
    assert!(
        !labels.contains(&"Slider".to_string()),
        "Should not include 'Slider' tag"
    );

    // Verify completion item properties
    let button_completion = completions.iter().find(|c| c.label == "Button").unwrap();
    assert_eq!(button_completion.kind, Some(CompletionItemKind::CLASS));
    assert_eq!(
        button_completion.detail,
        Some("Unity UI element (fallback)".to_string())
    );
    assert_eq!(button_completion.insert_text, Some("Button".to_string()));
}

#[test]
fn test_tag_selector_completion_partial_match() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: completion for 'S' should match 'Slider'
    let content = "Button { color: red; }\nS {color:blue}";
    let tree = parser.parse(content, None).unwrap();

    // Position after 'S'
    let position = Position {
        line: 1,
        character: 1,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should have completions for tags starting with 'S'
    assert!(!completions.is_empty(), "Should have tag completions");

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"Slider".to_string()),
        "Should include 'Slider' tag"
    );
    assert!(
        !labels.contains(&"Button".to_string()),
        "Should not include 'Button' tag"
    );
    assert!(
        !labels.contains(&"Label".to_string()),
        "Should not include 'Label' tag"
    );
}

#[test]
fn test_tag_selector_completion_case_insensitive() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: lowercase 'b' should match 'Button'
    let content = "b";
    let tree = parser.parse(content, None).unwrap();

    // Position after 'b'
    let position = Position {
        line: 0,
        character: 1,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should have case-insensitive completions
    assert!(
        !completions.is_empty(),
        "Should have case-insensitive tag completions"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"Button".to_string()),
        "Should include 'Button' tag (case insensitive)"
    );
}

#[test]
fn test_tag_selector_completion_no_empty_input() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: no completions when no characters are typed
    let content = "";
    let tree = parser.parse(content, None).unwrap();

    // Position at the beginning
    let position = Position {
        line: 0,
        character: 0,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should not have any tag completions for empty input
    let tag_completions: Vec<_> = completions
        .iter()
        .filter(|c| c.detail == Some("Unity UI element (fallback)".to_string()))
        .collect();
    assert!(
        tag_completions.is_empty(),
        "Should not provide tag completions for empty input"
    );
}

#[test]
fn test_class_selector_excludes_self() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: when typing ".my" and there's a class ".my", it should not suggest itself
    let content = ".my { color: red; }\n.my-class { margin: 10px; }\n.my";
    let tree = parser.parse(content, None).unwrap();

    // Position after '.my' (the incomplete selector)
    let position = Position {
        line: 2,
        character: 3,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should have completions but not include the exact match "my"
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        !labels.contains(&"my".to_string()),
        "Should not include the exact match 'my' that user is typing"
    );
    assert!(
        labels.contains(&"my-class".to_string()),
        "Should include 'my-class' which starts with 'my'"
    );
}

#[test]
fn test_id_selector_excludes_self() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: when typing "#my" and there's an ID "#my", it should not suggest itself
    let content = "#my { color: red; }\n#my-id { margin: 10px; }\n#my";
    let tree = parser.parse(content, None).unwrap();

    // Position after '#my' (the incomplete selector)
    let position = Position {
        line: 2,
        character: 3,
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(PathBuf::from("test")),
        None,
        None,
    );

    // Should have completions but not include the exact match "my"
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        !labels.contains(&"my".to_string()),
        "Should not include the exact match 'my' that user is typing"
    );
    assert!(
        labels.contains(&"my-id".to_string()),
        "Should include 'my-id' which starts with 'my'"
    );
}

#[test]
fn test_url_completion() {
    let mut parser = UssParser::new().unwrap();
    let project_root = get_project_root();
    let provider = UssCompletionProvider::new_with_project_root(&project_root);

    // Test case: cursor inside url() function pointing to Assets directory
    let content = ".some { \n    background-image: url(\"project:/Assets/\"); \n}";
    let tree = parser.parse(content, None).unwrap();

    // Position inside the URL string at the end of "Assets/"
    let position = Position {
        line: 1,
        character: 43, // At the end of "Assets/" but before closing quote
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(project_root.clone()),
        None,
        None,
    );

    // Should provide completions for directories in Assets
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();

    assert!(
        labels.contains(&"Resources".to_string()),
        "Should include Resources directory"
    );
    assert!(
        labels.contains(&"UI".to_string()),
        "Should include UI directory"
    );
    assert!(
        labels.contains(&"examples".to_string()),
        "Should include examples directory"
    );
}

#[test]
fn test_url_completion_resources_directory() {
    let mut parser = UssParser::new().unwrap();
    let project_root = get_project_root();
    let provider = UssCompletionProvider::new_with_project_root(&project_root);

    // Test case: cursor inside url() function pointing to Resources directory
    let content = ".icon { \n    background-image: url(\"project:/Assets/Resources/\"); \n}";
    let tree = parser.parse(content, None).unwrap();

    // Position inside the URL string after "Resources/"
    let position = Position {
        line: 1,
        character: 53, // At the end of "Resources/" but before closing quote
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(project_root.clone()),
        None,
        None,
    );

    // Should provide completions for subdirectories in Resources
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"Icons".to_string()),
        "Should include Icons directory"
    );
    assert!(
        labels.contains(&"Textures".to_string()),
        "Should include Textures directory"
    );
}

#[test]
fn test_url_completion_specific_files() {
    let mut parser = UssParser::new().unwrap();
    let project_root = get_project_root();
    let provider = UssCompletionProvider::new_with_project_root(&project_root);

    // Test case: cursor inside url() function pointing to Icons directory
    let content = ".icon { \n    background-image: url(\"project:/Assets/Resources/Icons/\"); \n}";
    let tree = parser.parse(content, None).unwrap();

    // Position inside the URL string after "Icons/"
    let position = Position {
        line: 1,
        character: 59, // At the end of "Icons/" but before closing quote
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(project_root.clone()),
        None,
        None,
    );

    // Should provide completions for files in Icons directory
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"icon.png".to_string()),
        "Should include icon.png file"
    );
}

#[test]
fn test_url_completion_in_import_statement() {
    let mut parser = UssParser::new().unwrap();
    let project_root = get_project_root();
    let provider = UssCompletionProvider::new_with_project_root(&project_root);

    // Test case: cursor inside import statement string pointing to UI directory
    let content = "@import \"project:/Assets/UI/\";";
    let tree = parser.parse(content, None).unwrap();

    // Position inside the import string after "UI/"
    let position = Position {
        line: 0,
        character: 28, // At the end of "UI/" but before closing quote
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(project_root.clone()),
        None,
        None,
    );

    // Should provide completions for items in UI directory
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"Components".to_string()),
        "Should include Components directory"
    );
    assert!(
        labels.contains(&"Styles".to_string()),
        "Should include Styles directory"
    );
    assert!(
        labels.contains(&"MainWindow.uxml".to_string()),
        "Should include MainWindow.uxml file"
    );
}


#[test]
fn test_url_completion_in_import_statement_with_url_function() {
    let mut parser = UssParser::new().unwrap();
    let project_root = get_project_root();
    let provider = UssCompletionProvider::new_with_project_root(&project_root);

    // Test case: cursor inside import statement string pointing to UI directory
    let content = "@import url(\"project:/Assets/UI/\");";
    let tree = parser.parse(content, None).unwrap();

    // Position inside the import string after "UI/"
    let position = Position {
        line: 0,
        character: 32, // At the end of "UI/" but before closing quote
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(project_root.clone()),
        None,
        None,
    );

    // Should provide completions for items in UI directory
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"Components".to_string()),
        "Should include Components directory"
    );
    assert!(
        labels.contains(&"Styles".to_string()),
        "Should include Styles directory"
    );
    assert!(
        labels.contains(&"MainWindow.uxml".to_string()),
        "Should include MainWindow.uxml file"
    );
}


#[test]
fn test_url_completion_uss_files() {
    let mut parser = UssParser::new().unwrap();
    let project_root = get_project_root();
    let provider = UssCompletionProvider::new_with_project_root(&project_root);

    // Test case: cursor inside url() function pointing to UI/Styles directory
    let content = "@import url(\"project:/Assets/UI/Styles/\");";
    let tree = parser.parse(content, None).unwrap();

    // Position inside the URL string after "Styles/"
    let position = Position {
        line: 0,
        character: 39, // At the end of "Styles/" but before closing quote
    };

    let completions = provider.complete(
        &tree,
        content,
        position,
        &UnityProjectManager::new(project_root.clone()),
        None,
        None,
    );

    // Should provide completions for USS files in Styles directory
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"main.uss".to_string()),
        "Should include main.uss file"
    );
}

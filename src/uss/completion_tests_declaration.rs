use crate::uss::{completion::UssCompletionProvider, parser::UssParser};
use tower_lsp::lsp_types::Position;
use crate::language::tree_printer::print_tree_to_stdout;

#[test]
fn test_property_value_simple_completion_after_colon() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: cursor right after colon
    let content = ".some { \n    color: \n}";
    let _length = content.len();
    let tree = parser.parse(content, None).unwrap();

    // Position right at the colon (line 1, character 9 - at the ':')
    let position = Position {
        line: 1,
        character: 10,
    };

    let completions = provider.complete(&tree, content, position, None, None, None);

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
    let tree = parser.parse(content, None).unwrap();

    // Position right after ro
    let position = Position {
        line: 1,
        character: 13,
    };

    let completions = provider.complete(&tree, content, position, None, None, None);

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
    let _length = content.len();
    let tree = parser.parse(content, None).unwrap();

    // Position right after ro
    let position = Position {
        line: 1,
        character: 22,
    };

    let completions = provider.complete(&tree, content, position, None, None, None);

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
    let content = ".some { \n    col\n} .other{width:10px;}";
    let tree = parser.parse(content, None).unwrap();

    print_tree_to_stdout(tree.root_node(), content);

    // Position at the end of "col" (line 1, character 7)
    let position = Position {
        line: 1,
        character: 7,
    };

    let completions = provider.complete(&tree, content, position, None, None, None);

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
        Some("color".to_string()),
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

    let completions = provider.complete(&tree, content, position, None, None, None);

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

    let completions = provider.complete(&tree, content, position, None, None, None);

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
fn test_property_name_completion_partial_match_before_another() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: partial property name that should match multiple properties
    let content = ".some { \n    back\n      color:red;}";
    let tree = parser.parse(content, None).unwrap();

    // Position at the end of "back"
    let position = Position {
        line: 1,
        character: 8,
    };

    let completions = provider.complete(&tree, content, position, None, None, None);

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

    // Should not include properties that don't start with "back"
    assert!(
        !labels.contains(&"color".to_string()),
        "Should not include 'color' property"
    );
}

#[test]
fn test_property_name_completion_partial_match_between_others() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: partial property name that should match multiple properties
    let content = ".some { width:10px;\n    back\n    color:red;}";
    let tree = parser.parse(content, None).unwrap();

    // Position at the end of "back"
    let position = Position {
        line: 1,
        character: 8,
    };

    let completions = provider.complete(&tree, content, position, None, None, None);

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

    // Should not include properties that don't start with "back"
    assert!(
        !labels.contains(&"color".to_string()),
        "Should not include 'color' property"
    );
}

#[test]
fn test_property_name_completion_partial_match_before_2() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: partial property name that should match multiple properties
    let content = r#".anim{
    tr
    translate: 200px 300px;
    color: azure;
    flex-direction: column;
    transition-property: translate, rotate, scale;
    transition-duration: 1s;
} "#;

    let tree = parser.parse(content, None).unwrap();

    print_tree_to_stdout(tree.root_node(), content);

    // Position at the end of "back"
    let position = Position {
        line: 1,
        character: 6,
    };

    let completions = provider.complete(&tree, content, position, None, None, None);

    // Should have completions for properties starting with "back"
    assert!(
        !completions.is_empty(),
        "Should have property name completions"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"translate".to_string()),
        "Should include 'translate' property"
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

    let completions = provider.complete(&tree, content, position, None, None, None);

    // Should have completions for properties starting with "col" (case in-sensitive)
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
fn test_comma_separated_values_completion() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: completion after comma in comma-separated values
    let content = ".some { \n    transition-property: opacity, \n}";
    let tree = parser.parse(content, None).unwrap();

    // Position right after comma and space
    let position = Position {
        line: 1,
        character: 33,
    };

    let completions = provider.complete(&tree, content, position, None, None, None);

    // Should have completions after comma
    assert!(
        !completions.is_empty(),
        "Should have completions after comma"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"color".to_string()),
        "Should include 'color' property after comma"
    );
}

#[test]
fn test_comma_separated_values_completion_partial() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: completion after comma in comma-separated values
    let content = ".some { \n    transition-property: opacity, tr\n}";
    let tree = parser.parse(content, None).unwrap();

    // Position right after comma and space
    let position = Position {
        line: 1,
        character: 36,
    };

    let completions = provider.complete(&tree, content, position, None, None, None);

    // Should have completions after comma
    assert!(
        !completions.is_empty(),
        "Should have completions"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"translate".to_string()),
        "Should include 'translate' property"
    );
}

#[test]
fn test_comma_separated_values_completion_with_semicolon() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: completion after comma in comma-separated values
    let content = ".some { \n    transition-property: opacity,    ;\n}";
    let tree = parser.parse(content, None).unwrap();

    // Position right after comma and space
    let position = Position {
        line: 1,
        character: 33,
    };

    let completions = provider.complete(&tree, content, position, None, None, None);

    // Should have completions after comma
    assert!(
        !completions.is_empty(),
        "Should have completions after comma"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"color".to_string()),
        "Should include 'color' property after comma"
    );
}

#[test]
fn test_comma_separated_values_completion_partial_with_semicolon() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: completion after comma in comma-separated values
    let content = ".some { \n    transition-property: opacity, tr  ;\n}";
    let tree = parser.parse(content, None).unwrap();

    // Position right after comma and space
    let position = Position {
        line: 1,
        character: 36,
    };

    let completions = provider.complete(&tree, content, position, None, None, None);

    // Should have completions after comma
    assert!(
        !completions.is_empty(),
        "Should have completions"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"translate".to_string()),
        "Should include 'translate' property"
    );
}

#[test]
fn test_non_keyword_property_no_completion() {
    let mut parser = UssParser::new().unwrap();
    let provider = UssCompletionProvider::new();

    // Test case: property that doesn't have single keyword values
    let content = ".some { \n    width: \n}";
    let tree = parser.parse(content, None).unwrap();

    // Position right after colon
    let position = Position {
        line: 1,
        character: 11,
    };

    let completions = provider.complete(&tree, content, position, None, None, None);

    // Should have limited or no completions for width (mainly length values)
    // Width accepts length/percentage values, not many single keywords
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();

    // Should include 'auto' if it's a valid keyword for width
    if !labels.is_empty() {
        // If there are completions, they should be valid keywords only
        for label in &labels {
            // Common keywords that might be valid
            assert!(
                ["auto", "initial", "inherit", "unset"].contains(&label.as_str()),
                "Unexpected completion '{}' for width property",
                label
            );
        }
    }
}

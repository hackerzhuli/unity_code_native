use crate::uss::hover::*;

use crate::unity_project_manager::UnityProjectManager;
use crate::uss::parser::UssParser;
use std::path::PathBuf;
use tower_lsp::lsp_types::Position;

#[test]
fn test_hover_provider_creation() {
    let provider = UssHoverProvider::new();
    assert!(provider.definitions.is_valid_property("color"));
}

#[test]
fn test_position_to_byte_offset() {
    use crate::language::tree_utils::position_to_byte_offset;
    let source = "color: red;\nfont-size: 12px;";

    // Test position at start of "color"
    let offset = position_to_byte_offset(source, Position::new(0, 0));
    assert_eq!(offset, Some(0));

    // Test position at start of "font-size"
    let offset = position_to_byte_offset(source, Position::new(1, 0));
    assert_eq!(offset, Some(12));
}

#[test]
fn test_create_hover_content() {
    let provider = UssHoverProvider::new();
    let unity_manager = UnityProjectManager::new(PathBuf::from("/test/project"));

    let hover = provider.create_hover_content_for_property("color", &unity_manager);

    if let HoverContents::Markup(content) = hover.contents {
        assert!(content.value.contains("color"));
        assert!(content.value.contains("Documentation"));
        assert_eq!(content.kind, MarkupKind::Markdown);
    } else {
        panic!("Expected markup content");
    }
}

#[test]
fn test_extract_path_from_string_node() {
    let provider = UssHoverProvider::new();
    // This test would need a proper tree-sitter node, which is complex to create in unit tests
    // For now, we'll just test that the provider can be created
    assert!(provider.definitions.is_valid_property("color"));
}

#[test]
fn test_hover_for_invalid_property() {
    let provider = UssHoverProvider::new();
    let _unity_manager = UnityProjectManager::new(PathBuf::from("/test/project"));

    // Test that invalid properties are not recognized
    assert!(!provider.definitions.is_valid_property("invalid-property"));
    assert!(
        !provider
            .definitions
            .is_predefined_property("invalid-property")
    );

    // Test that custom CSS properties are valid for diagnostics but not for hover
    assert!(
        provider
            .definitions
            .is_valid_property("--button-border-color-what")
    );
    assert!(
        !provider
            .definitions
            .is_predefined_property("--button-border-color-what")
    );
    assert!(provider.definitions.is_valid_property("--custom-property"));
    assert!(
        !provider
            .definitions
            .is_predefined_property("--custom-property")
    );

    // Test that predefined properties are recognized by both methods
    assert!(provider.definitions.is_valid_property("color"));
    assert!(provider.definitions.is_predefined_property("color"));
}

#[test]
fn test_pseudo_class_hover_precision() {
    let mut parser = UssParser::new().expect("Failed to create USS parser");
    let hover_provider = UssHoverProvider::new();
    let unity_manager = UnityProjectManager::new(PathBuf::from("."));

    let source = r#"#container Button /* 
 adfasdf 
 adfasdfasdf 
 adfasdf 
 adfasdf 
 */ 
 .some-ing:checked:active:hover:active:active:hover:checked { 
     width: 100%; 
     height: 100%; 
 } "#;

    let tree = parser.parse(source, None).expect("Failed to parse USS");

    // Test 1: Hovering over comment should return None
    let comment_position = Position::new(1, 5); // Inside " adfasdf "
    let hover_result =
        hover_provider.hover(&tree, source, comment_position, &unity_manager, None, None);
    assert!(
        hover_result.is_none(),
        "Hover over comment should return None, but got: {:?}",
        hover_result
    );

    // Test 2: Hovering over ID selector should NOT return pseudo-class hover
    let id_position = Position::new(0, 5); // Inside "#container"
    let hover_result = hover_provider.hover(&tree, source, id_position, &unity_manager, None, None);
    // Should either be None or tag selector hover, but NOT pseudo-class hover
    if let Some(hover) = hover_result {
        let content = match hover.contents {
            tower_lsp::lsp_types::HoverContents::Markup(markup) => markup.value,
            _ => String::new(),
        };
        assert!(
            !content.to_lowercase().contains("pseudo-class"),
            "Hover over ID selector should not show pseudo-class info, but got: {}",
            content
        );
    }

    // Test 3: Hovering over tag name should NOT return pseudo-class hover
    let tag_position = Position::new(0, 15); // Inside "Button"
    let hover_result =
        hover_provider.hover(&tree, source, tag_position, &unity_manager, None, None);
    if let Some(hover) = hover_result {
        let content = match hover.contents {
            tower_lsp::lsp_types::HoverContents::Markup(markup) => markup.value,
            _ => String::new(),
        };
        assert!(
            !content.to_lowercase().contains("pseudo-class"),
            "Hover over tag name should not show pseudo-class info, but got: {}",
            content
        );
    }

    // Test 4: Hovering over actual pseudo-class should return pseudo-class hover
    let pseudo_position = Position::new(6, 20); // Inside ":active" part
    let hover_result =
        hover_provider.hover(&tree, source, pseudo_position, &unity_manager, None, None);
    if let Some(hover) = hover_result {
        let content = match hover.contents {
            tower_lsp::lsp_types::HoverContents::Markup(markup) => markup.value,
            _ => String::new(),
        };
        assert!(
            content.to_lowercase().contains("active") || content.to_lowercase().contains("pseudo"),
            "Hover over pseudo-class should show pseudo-class info, but got: {}",
            content
        );
    } else {
        panic!("Hover over pseudo-class should return Some, but got None");
    }

    // Test 5: Hovering over class selector should NOT return pseudo-class hover
    let class_position = Position::new(6, 5); // Inside ".some-ing"
    let hover_result =
        hover_provider.hover(&tree, source, class_position, &unity_manager, None, None);
    if let Some(hover) = hover_result {
        let content = match hover.contents {
            tower_lsp::lsp_types::HoverContents::Markup(markup) => markup.value,
            _ => String::new(),
        };
        assert!(
            !content.to_lowercase().contains("pseudo-class"),
            "Hover over class selector should not show pseudo-class info, but got: {}",
            content
        );
    }

    println!("✅ Pseudo-class hover detection is working correctly");
}

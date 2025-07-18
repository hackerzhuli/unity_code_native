use crate::uss::hover::*;

use crate::unity_project_manager::UnityProjectManager;
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

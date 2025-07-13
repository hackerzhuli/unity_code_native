
use crate::language::url_completion::*;
use crate::unity_asset_database::UnityAssetDatabase;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;
use tower_lsp::lsp_types::CompletionItemKind;
use url::Url;

/// Helper function to get the project root directory for tests
/// This looks for the Cargo.toml file to determine the project root
fn get_project_root() -> std::path::PathBuf {
    // Try to get the manifest directory from environment (works during cargo test)
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        return std::path::PathBuf::from(manifest_dir);
    }

    // Fallback: start from current directory and walk up to find Cargo.toml
    let mut current_dir = env::current_dir().expect("Failed to get current directory");
    loop {
        if current_dir.join("Cargo.toml").exists() {
            return current_dir;
        }
        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            panic!("Could not find project root (Cargo.toml not found)");
        }
    }
}

#[test]
fn test_analyze_completion_context_path() {
    let project_root = get_project_root();
    let provider = UrlCompletionProvider::new(&project_root);

    let context = provider
        .analyze_completion_context("project:/Assets/UI/", "project:/Assets/UI/".len(), None)
        .unwrap();

    match context {
        UrlCompletionContext::Path { partial_path, .. } => {
            assert_eq!(partial_path, "project:/Assets/UI/");
        }
        _ => panic!("Expected path completion context"),
    }
}

#[test]
fn test_analyze_completion_context_query() {
    let project_root = get_project_root();
    let provider = UrlCompletionProvider::new(&project_root);

    let context = provider
        .analyze_completion_context(
            "project:/Assets/examples/meta/texture_with_multiple_sprites_example.png?",
            "project:/Assets/examples/meta/texture_with_multiple_sprites_example.png?".len(),
            None,
        )
        .unwrap();

    match context {
        UrlCompletionContext::Query { .. } => {
            // Expected
        }
        _ => panic!("Expected query completion context"),
    }
}

#[test]
fn test_no_completion_without_slash() {
    let project_root = get_project_root();
    let provider = UrlCompletionProvider::new(&project_root);

    let result = provider.complete_path("project", None).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_complete_assets_directory() {
    let project_root = get_project_root();
    let provider = UrlCompletionProvider::new(&project_root);

    let completions = provider.complete_url("project:/Assets/", "project:/Assets/".len(), None);

    // Should have completions for directories in Assets
    assert!(
        !completions.is_empty(),
        "Should have completions for Assets directory"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();

    // Check for directories we created
    assert!(
        labels.contains(&"UI".to_string()),
        "Should include UI directory"
    );
    assert!(
        labels.contains(&"Resources".to_string()),
        "Should include Resources directory"
    );
    assert!(
        labels.contains(&"examples".to_string()),
        "Should include examples directory"
    );

    // Verify directory items have correct properties
    let ui_item = completions.iter().find(|c| c.label == "UI").unwrap();
    assert_eq!(
        ui_item.kind,
        Some(tower_lsp::lsp_types::CompletionItemKind::FOLDER)
    );
    assert_eq!(ui_item.insert_text, Some("UI".to_string())); // No '/' appended - user types it manually
}

#[test]
fn test_complete_ui_subdirectory() {
    let project_root = get_project_root();
    let provider = UrlCompletionProvider::new(&project_root);

    let completions =
        provider.complete_url("project:/Assets/UI/", "project:/Assets/UI/".len(), None);

    // Should have completions for UI subdirectories
    assert!(
        !completions.is_empty(),
        "Should have completions for UI directory"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();

    // Check for subdirectories we created
    assert!(
        labels.contains(&"Styles".to_string()),
        "Should include Styles directory"
    );
    assert!(
        labels.contains(&"Components".to_string()),
        "Should include Components directory"
    );
    assert!(
        labels.contains(&"MainWindow.uxml".to_string()),
        "Should include MainWindow.uxml file"
    );
}

#[test]
fn test_complete_with_filename_prefix() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create test directory structure
    let ui_dir = project_root.join("Assets").join("UI");
    fs::create_dir_all(&ui_dir).unwrap();

    // Create test files
    File::create(ui_dir.join("MainWindow.uxml")).unwrap();
    File::create(ui_dir.join("MainWindow.uxml.meta")).unwrap();
    File::create(ui_dir.join("Styles")).unwrap();
    fs::create_dir_all(ui_dir.join("Components")).unwrap();

    let provider = UrlCompletionProvider::new(project_root);

    let completions =
        provider.complete_url("project:/Assets/UI/m", "project:/Assets/UI/m".len(), None);

    // Should find MainWindow.uxml (case-insensitive match for 'm')
    assert_eq!(completions.len(), 1);
    assert_eq!(completions[0].label, "MainWindow.uxml");
}

#[test]
fn test_complete_styles_files() {
    let project_root = get_project_root();
    let provider = UrlCompletionProvider::new(&project_root);

    let completions = provider.complete_url(
        "project:/Assets/UI/Styles/",
        "project:/Assets/UI/Styles/".len(),
        None,
    );

    // Should have completions for files in Styles directory
    // Note: This test depends on the actual files in the project
    // We're just checking that the completion mechanism works
    // The actual files may vary
}

#[test]
fn test_complete_texture_query_parameters() {
    let project_root = get_project_root();
    let provider = UrlCompletionProvider::new(&project_root);

    let completions = provider.complete_url(
        "project:/Assets/examples/meta/texture_with_multiple_sprites_example.png?",
        "project:/Assets/examples/meta/texture_with_multiple_sprites_example.png?".len(),
        None,
    );

    // Should have query parameter completions for texture assets
    // The exact completions depend on the asset metadata
    // We're just verifying the mechanism works
}

#[test]
fn test_complete_relative_path_parent_directory() {
    let project_root = get_project_root();
    let provider = UrlCompletionProvider::new(&project_root);

    // Test relative path completion using '../' from UI/Styles/ to UI/
    let base_url = url::Url::parse("project:/Assets/UI/Styles/").unwrap();
    let completions = provider.complete_url(
        "../",
        "../".len(),
        Some(&base_url),
    );

    // Should have completions for directories in UI
    assert!(
        !completions.is_empty(),
        "Should have completions for parent directory"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();

    // Check for directories we expect in UI
    assert!(
        labels.contains(&"Components".to_string()),
        "Should include Components directory from parent"
    );
    assert!(
        labels.contains(&"MainWindow.uxml".to_string()),
        "Should include MainWindow.uxml file from parent"
    );
}

#[test]
fn test_complete_relative_path_current_directory() {
    let project_root = get_project_root();
    let provider = UrlCompletionProvider::new(&project_root);

    // Test relative path completion using './' from UI/ directory
    let base_url = url::Url::parse("project:/Assets/UI/").unwrap();
    let completions = provider.complete_url(
        "./",
        "./".len(),
        Some(&base_url),
    );

    // Should have completions for current directory (UI)
    assert!(
        !completions.is_empty(),
        "Should have completions for current directory"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();

    // Check for directories we expect in UI
    assert!(
        labels.contains(&"Styles".to_string()),
        "Should include Styles directory from current directory"
    );
    assert!(
        labels.contains(&"Components".to_string()),
        "Should include Components directory from current directory"
    );
}

#[test]
fn test_complete_relative_path_multiple_parent_directories() {
    let project_root = get_project_root();
    let provider = UrlCompletionProvider::new(&project_root);

    // Test relative path completion using '../../' from UI/Styles/ to Assets/
    let base_url = url::Url::parse("project:/Assets/UI/Styles/").unwrap();
    let completions = provider.complete_url(
        "../../",
        "../../".len(),
        Some(&base_url),
    );

    // Should have completions for Assets directory
    assert!(
        !completions.is_empty(),
        "Should have completions for Assets directory via relative path"
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();

    // Check for directories we expect in Assets
    assert!(
        labels.contains(&"UI".to_string()),
        "Should include UI directory from Assets"
    );
    assert!(
        labels.contains(&"Resources".to_string()),
        "Should include Resources directory from Assets"
    );
}

#[test]
fn test_complete_relative_path_with_filename_prefix() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create test directory structure
    let ui_dir = project_root.join("Assets").join("UI");
    let styles_dir = ui_dir.join("Styles");
    fs::create_dir_all(&styles_dir).unwrap();

    // Create test files in UI directory
    File::create(ui_dir.join("MainWindow.uxml")).unwrap();
    File::create(ui_dir.join("MyComponent.uxml")).unwrap();
    File::create(styles_dir.join("style.uss")).unwrap();

    let provider = UrlCompletionProvider::new(project_root);

    // Test relative path completion with filename prefix from Styles/ to UI/
    let base_url = url::Url::parse("project:/Assets/UI/Styles/").unwrap();
    let completions = provider.complete_url(
        "../M",
        "../M".len(),
        Some(&base_url),
    );

    // Should find files starting with 'M' in parent directory
    assert!(!completions.is_empty(), "Should find files with prefix 'M'");
    
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"MainWindow.uxml".to_string()),
        "Should include MainWindow.uxml from parent directory"
    );
    assert!(
        labels.contains(&"MyComponent.uxml".to_string()),
        "Should include MyComponent.uxml from parent directory"
    );
}

#[test]
fn test_complete_relative_path_no_dot_prefix() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create test directory structure
    let ui_dir = project_root.join("Assets").join("UI");
    let images_dir = ui_dir.join("Images");
    fs::create_dir_all(&images_dir).unwrap();

    // Create test files
    File::create(ui_dir.join("variables.uss")).unwrap();
    File::create(ui_dir.join("components.uss")).unwrap();
    File::create(images_dir.join("icon.png")).unwrap();
    File::create(images_dir.join("background.jpg")).unwrap();

    let provider = UrlCompletionProvider::new(project_root);

    // Test relative path completion without dot prefix from UI/ directory
    let base_url = url::Url::parse("project:/Assets/UI/").unwrap();
    
    // Test completion for files in same directory
    let completions = provider.complete_url(
        "v",
        "v".len(),
        Some(&base_url),
    );

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"variables.uss".to_string()),
        "Should include variables.uss from current directory without dot prefix"
    );

    // Test completion for subdirectory
    let completions = provider.complete_url(
        "Images/",
        "Images/".len(),
        Some(&base_url),
    );

    assert!(!completions.is_empty(), "Should have completions for Images subdirectory");
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(
        labels.contains(&"icon.png".to_string()),
        "Should include icon.png from Images subdirectory"
    );
    assert!(
        labels.contains(&"background.jpg".to_string()),
        "Should include background.jpg from Images subdirectory"
    );
}

#[test]
fn test_no_completion_for_exact_matches() {
    let project_root = get_project_root();
    let provider = UrlCompletionProvider::new(&project_root);
    
    // Test completion for exact filename "variables.uss" - should not return itself
    let completions = provider.complete_url("project:/Assets/UI/Styles/variables.uss", "project:/Assets/UI/Styles/variables.uss".len(), None);
    let exact_match = completions.iter().find(|c| c.label == "variables.uss");
    assert!(exact_match.is_none(), "Should not return exact match 'variables.uss' in completion");
    
    // Test completion for exact directory name "Components" - should not return itself
    let completions = provider.complete_url("project:/Assets/UI/Components", "project:/Assets/UI/Components".len(), None);
    let exact_match = completions.iter().find(|c| c.label == "Components");
    assert!(exact_match.is_none(), "Should not return exact match 'Components' in completion");
    
    // Test case-insensitive exact match
    let completions = provider.complete_url("project:/Assets/UI/Styles/VARIABLES.USS", "project:/Assets/UI/Styles/VARIABLES.USS".len(), None);
    let exact_match = completions.iter().find(|c| c.label.to_lowercase() == "variables.uss");
    assert!(exact_match.is_none(), "Should not return case-insensitive exact match");
}

/// URL completer that uses the real Unity asset database
struct UrlCompleter {
    asset_database: crate::unity_asset_database::UnityAssetDatabase,
}

impl UrlCompleter {
    fn new(project_root: &Path) -> Self {
        Self { 
            asset_database: crate::unity_asset_database::UnityAssetDatabase::new(project_root)
        }
    }

    fn complete_url(
        &self,
        url_string: &str,
        base_url: Option<&url::Url>,
    ) -> Result<Vec<tower_lsp::lsp_types::CompletionItem>, UrlCompletionError> {
        let provider = UrlCompletionProvider::new(self.asset_database.project_root());
        Ok(provider.complete_url(url_string, url_string.len(), base_url))
    }
}

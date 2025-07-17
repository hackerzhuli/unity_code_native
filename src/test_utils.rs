//! Test utilities shared across the codebase

use std::env;
use std::path::PathBuf;

/// Helper function to get the project root directory for tests
/// This looks for the Cargo.toml file to determine the project root
pub fn get_project_root() -> PathBuf {
    // Try to get the manifest directory from environment (works during cargo test)
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        return PathBuf::from(manifest_dir);
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

/// Helper function to get the Unity project root directory path
pub fn get_unity_project_root() -> PathBuf {
    get_project_root().join("UnityProject")
}

/// Helper function to get the UIElementsSchema directory path
pub fn get_ui_elements_schema_dir() -> PathBuf {
    get_unity_project_root().join("UIElementsSchema")
}

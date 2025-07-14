//! Unity project management and version detection.
//!
//! This module provides functionality to manage Unity projects and detect
//! their versions and configurations.

use std::path::PathBuf;
use std::fs;
use serde::Deserialize;
use url::Url;
use std::io;

use crate::language::asset_url::create_project_url_with_normalization;

/// Represents the structure of Unity's ProjectVersion.txt file
#[derive(Debug, Deserialize)]
struct ProjectVersion {
    #[serde(rename = "m_EditorVersion")]
    editor_version: String
}

/// Result type for Unity project operations
type Result<T> = std::result::Result<T, UnityProjectError>;

/// Errors that can occur when working with Unity projects
#[derive(Debug, thiserror::Error)]
pub enum UnityProjectError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Not a valid Unity project: missing ProjectSettings/ProjectVersion.txt")]
    NotUnityProject,
    #[error("Invalid Unity version format: {0}")]
    InvalidVersionFormat(String),
}

/// Manages Unity project information and provides version detection capabilities.
#[derive(Debug, Clone)]
pub struct UnityProjectManager {
    project_path: PathBuf,
}

impl UnityProjectManager {
    /// Creates a new UnityProjectManager for the specified project path.
    ///
    /// # Arguments
    ///
    /// * `project_path` - The path to the Unity project directory
    ///
    /// # Examples
    ///
    /// ```
    /// use unity_code_native::unity_project_manager::UnityProjectManager;
    /// use std::path::PathBuf;
    ///
    /// let manager = UnityProjectManager::new(PathBuf::from("/path/to/unity/project"));
    /// ```
    pub fn new(project_path: PathBuf) -> Self {
        Self { project_path }
    }

    /// Checks if the project path is a valid Unity project.
    ///
    /// A valid Unity project must have a ProjectSettings/ProjectVersion.txt file
    /// that can be successfully parsed.
    ///
    /// # Returns
    ///
    /// `true` if this is a valid Unity project, `false` otherwise
    pub fn is_valid_unity_project(&self) -> bool {
        self.detect_unity_version().is_ok()
    }

    /// Detects the Unity version of the project.
    ///
    /// Reads and parses the ProjectSettings/ProjectVersion.txt file to extract
    /// the Unity editor version.
    ///
    /// # Returns
    ///
    /// A `Result` containing the Unity version string (e.g., "6000.0.51f1") on success,
    /// or a `UnityProjectError` if the project is invalid or the version cannot be determined.
    ///
    /// # Errors
    ///
    /// - `UnityProjectError::NotUnityProject` if ProjectVersion.txt doesn't exist
    /// - `UnityProjectError::Io` if there's an error reading the file
    /// - `UnityProjectError::Yaml` if the YAML cannot be parsed
    /// - `UnityProjectError::InvalidVersionFormat` if the version format is unexpected
    pub fn detect_unity_version(&self) -> Result<String> {
        let project_version_path = self.project_path
            .join("ProjectSettings")
            .join("ProjectVersion.txt");

        if !project_version_path.exists() {
            return Err(UnityProjectError::NotUnityProject);
        }

        let content = fs::read_to_string(&project_version_path)?;
        let project_version: ProjectVersion = serde_yaml::from_str(&content)?;
        
        // Validate that the version string is not empty
        if project_version.editor_version.trim().is_empty() {
            return Err(UnityProjectError::InvalidVersionFormat(
                "Empty version string".to_string()
            ));
        }

        Ok(project_version.editor_version)
    }

    /// Gets the Unity version as a string, returning None if the project is invalid.
    ///
    /// This is a convenience method that wraps `detect_unity_version()` and
    /// returns `None` instead of an error for easier use in contexts where
    /// error handling is not needed.
    ///
    /// # Returns
    ///
    /// `Some(version)` if the project is valid, `None` otherwise
    pub fn get_unity_version(&self) -> Option<String> {
        self.detect_unity_version().ok()
    }

    /// Gets the Unity version in documentation format (major.minor).
    ///
    /// This method extracts the major and minor version numbers from the full
    /// Unity version string for use in documentation URLs.
    ///
    /// # Returns
    ///
    /// `Some(version)` in major.minor format (e.g., "6000.0") if the project is valid,
    /// `None` if the project is invalid or the version cannot be determined.
    ///
    /// # Examples
    ///
    /// ```
    /// use unity_code_native::unity_project_manager::UnityProjectManager;
    /// use std::path::PathBuf;
    ///
    /// let manager = UnityProjectManager::new(PathBuf::from("/path/to/unity/project"));
    /// if let Some(doc_version) = manager.get_unity_version_for_docs() {
    ///     assert_eq!(doc_version, "6000.0");
    /// }
    /// ```
    pub fn get_unity_version_for_docs(&self) -> Option<String> {
        let full_version = self.detect_unity_version().ok()?;
        
        // Extract major.minor from version string like "6000.0.51f1"
        let parts: Vec<&str> = full_version.split('.').collect();
        if parts.len() >= 2 {
            Some(format!("{}.{}", parts[0], parts[1]))
        } else {
            // Fallback to full version if parsing fails
            Some(full_version)
        }
    }

    /// Returns the project path.
    pub fn project_path(&self) -> &PathBuf {
        &self.project_path
    }

    /// Converts a file system URL to a project scheme URL
    ///
    /// ## Arguments
    ///
    /// * `url` - The URL to convert (typically a file:// URL)
    ///
    /// ## Returns
    ///
    /// `Some(Url)` if the conversion is successful, `None` if the URL cannot be converted
    /// 
    /// ## Note
    /// The file must exist on file system, otherwise the conversion will fail.
    pub fn convert_to_project_url(&self, url: &Url) -> Option<Url> {        
        if url.scheme() == "file" {
            if let Ok(file_path) = url.to_file_path() {
                create_project_url_with_normalization(
                    &file_path,
                    &self.project_path,
                )
                .ok()
            } else {
                None
            }
        } else if url.scheme() == "project" {
            // If it's already a project:// URL, use as-is
            Some(url.clone())
        }else{
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_unity_project(version: &str) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let project_settings_dir = temp_dir.path().join("ProjectSettings");
        fs::create_dir_all(&project_settings_dir).unwrap();
        
        let project_version_path = project_settings_dir.join("ProjectVersion.txt");
        let mut file = fs::File::create(&project_version_path).unwrap();
        writeln!(file, "m_EditorVersion: {}", version).unwrap();
        writeln!(file, "m_EditorVersionWithRevision: {} (01c3ff5872c5)", version).unwrap();
        
        temp_dir
    }

    #[test]
    fn test_unity_project_manager_creation() {
        let path = PathBuf::from("/test/project");
        let manager = UnityProjectManager::new(path.clone());
        assert_eq!(manager.project_path(), &path);
    }

    #[test]
    fn test_detect_unity_version_valid_project() {
        let temp_dir = create_test_unity_project("6000.0.51f1");
        let manager = UnityProjectManager::new(temp_dir.path().to_path_buf());
        
        let version = manager.detect_unity_version().unwrap();
        assert_eq!(version, "6000.0.51f1");
        
        assert!(manager.is_valid_unity_project());
        assert_eq!(manager.get_unity_version(), Some("6000.0.51f1".to_string()));
    }

    #[test]
    fn test_detect_unity_version_invalid_project() {
        let temp_dir = TempDir::new().unwrap();
        let manager = UnityProjectManager::new(temp_dir.path().to_path_buf());
        
        let result = manager.detect_unity_version();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UnityProjectError::NotUnityProject));
        
        assert!(!manager.is_valid_unity_project());
        assert_eq!(manager.get_unity_version(), None);
    }

    #[test]
    fn test_get_unity_version_for_docs() {
        let temp_dir = create_test_unity_project("6000.0.51f1");
        let manager = UnityProjectManager::new(temp_dir.path().to_path_buf());
        
        let doc_version = manager.get_unity_version_for_docs().unwrap();
        assert_eq!(doc_version, "6000.0");
    }

    #[test]
    fn test_get_unity_version_for_docs_different_versions() {
        // Test Unity 2023 version
        let temp_dir = create_test_unity_project("2023.3.15f1");
        let manager = UnityProjectManager::new(temp_dir.path().to_path_buf());
        assert_eq!(manager.get_unity_version_for_docs().unwrap(), "2023.3");
        
        // Test Unity 2022 version
        let temp_dir = create_test_unity_project("2022.3.42f1");
        let manager = UnityProjectManager::new(temp_dir.path().to_path_buf());
        assert_eq!(manager.get_unity_version_for_docs().unwrap(), "2022.3");
    }

    #[test]
    fn test_get_unity_version_for_docs_invalid_project() {
        let temp_dir = TempDir::new().unwrap();
        let manager = UnityProjectManager::new(temp_dir.path().to_path_buf());
        
        let doc_version = manager.get_unity_version_for_docs();
        assert_eq!(doc_version, None);
    }

    #[test]
    fn test_malformed_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let project_settings_dir = temp_dir.path().join("ProjectSettings");
        fs::create_dir_all(&project_settings_dir).unwrap();
        
        let project_version_path = project_settings_dir.join("ProjectVersion.txt");
        let mut file = fs::File::create(&project_version_path).unwrap();
        writeln!(file, "invalid yaml content: [[[[").unwrap();
        
        let manager = UnityProjectManager::new(temp_dir.path().to_path_buf());
        let result = manager.detect_unity_version();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UnityProjectError::Yaml(_)));
    }

    #[test]
    fn test_empty_version() {
        let temp_dir = TempDir::new().unwrap();
        let project_settings_dir = temp_dir.path().join("ProjectSettings");
        fs::create_dir_all(&project_settings_dir).unwrap();
        
        let project_version_path = project_settings_dir.join("ProjectVersion.txt");
        let mut file = fs::File::create(&project_version_path).unwrap();
        writeln!(file, "m_EditorVersion: ").unwrap();
        
        let manager = UnityProjectManager::new(temp_dir.path().to_path_buf());
        let result = manager.detect_unity_version();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UnityProjectError::InvalidVersionFormat(_)));
    }
}
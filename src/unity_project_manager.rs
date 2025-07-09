//! Unity project management and version detection.
//!
//! This module provides functionality to manage Unity projects and detect
//! their versions and configurations.

use std::path::PathBuf;

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

    /// Detects the Unity version of the project.
    ///
    /// Currently returns a hardcoded version string. This will be implemented
    /// to actually detect the Unity version from project files in the future.
    ///
    /// # Returns
    ///
    /// A string representing the Unity version (e.g., "6000.0.51f1")
    pub fn detect_unity_version(&self) -> String {
        // TODO: Implement actual Unity version detection by reading:
        // - ProjectSettings/ProjectVersion.txt
        // - Library/PackageCache/com.unity.*/package.json
        // - Other Unity project files
        "6000.0.51f1".to_string()
    }

    /// Gets the Unity version in documentation format (major.minor).
    ///
    /// This method extracts the major and minor version numbers from the full
    /// Unity version string for use in documentation URLs.
    ///
    /// # Returns
    ///
    /// A string representing the Unity version in major.minor format (e.g., "6000.0")
    ///
    /// # Examples
    ///
    /// ```
    /// use unity_code_native::unity_project_manager::UnityProjectManager;
    /// use std::path::PathBuf;
    ///
    /// let manager = UnityProjectManager::new(PathBuf::from("/path/to/unity/project"));
    /// let doc_version = manager.get_unity_version_for_docs();
    /// assert_eq!(doc_version, "6000.0");
    /// ```
    pub fn get_unity_version_for_docs(&self) -> String {
        let full_version = self.detect_unity_version();
        
        // Extract major.minor from version string like "6000.0.51f1"
        let parts: Vec<&str> = full_version.split('.').collect();
        if parts.len() >= 2 {
            format!("{}.{}", parts[0], parts[1])
        } else {
            // Fallback to full version if parsing fails
            full_version
        }
    }

    /// Returns the project path.
    pub fn project_path(&self) -> &PathBuf {
        &self.project_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_unity_project_manager_creation() {
        let path = PathBuf::from("/test/project");
        let manager = UnityProjectManager::new(path.clone());
        assert_eq!(manager.project_path(), &path);
    }

    #[test]
    fn test_detect_unity_version() {
        let manager = UnityProjectManager::new(PathBuf::from("/test/project"));
        let version = manager.detect_unity_version();
        assert_eq!(version, "6000.0.51f1");
    }

    #[test]
    fn test_get_unity_version_for_docs() {
        let manager = UnityProjectManager::new(PathBuf::from("/test/project"));
        let doc_version = manager.get_unity_version_for_docs();
        assert_eq!(doc_version, "6000.0");
    }
}
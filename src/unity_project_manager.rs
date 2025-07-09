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
}
//! CS Documentation module
//! 
//! This module provides functionality to extract XML documentation from C# source files
//! and create documentation assemblies for Unity projects.
//!
//! It handles two types of source locations:
//! 1. User code: Found in .csproj files in the Unity project root
//! 2. Package cache code: Found in .asmdef files within Library/PackageCache

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

pub mod assembly_finder;
pub mod source_finder;

/// Represents information about a C# assembly and its source files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyInfo {
    /// The name of the assembly (e.g., "Assembly-CSharp")
    pub name: String,
    /// List of source file paths relative to Unity project root
    pub source_files: Vec<PathBuf>,
    /// Whether this is user code (true) or package code (false)
    pub is_user_code: bool,
    /// The location where this assembly info was found (csproj or asmdef path)
    pub source_location: PathBuf,
}

/// Main CS documentation manager
#[derive(Debug)]
pub struct CsDocsManager {
    unity_project_root: PathBuf,
    assemblies: HashMap<String, AssemblyInfo>,
}

impl CsDocsManager {
    /// Create a new CS documentation manager for the given Unity project
    pub fn new(unity_project_root: PathBuf) -> Self {
        Self {
            unity_project_root,
            assemblies: HashMap::new(),
        }
    }

    /// Find all assemblies and their source files
    pub async fn discover_assemblies(&mut self) -> Result<()> {
        // Clear existing assemblies
        self.assemblies.clear();

        // Find user code assemblies from .csproj files
        let user_assemblies = self.find_user_assemblies().await
            .context("Failed to find user assemblies")?;
        
        for assembly in user_assemblies {
            self.assemblies.insert(assembly.name.clone(), assembly);
        }

        // Find package assemblies from .asmdef files
        let package_assemblies = self.find_package_assemblies().await
            .context("Failed to find package assemblies")?;
        
        for assembly in package_assemblies {
            self.assemblies.insert(assembly.name.clone(), assembly);
        }

        Ok(())
    }

    /// Get all discovered assemblies
    pub fn get_assemblies(&self) -> &HashMap<String, AssemblyInfo> {
        &self.assemblies
    }

    /// Get a specific assembly by name
    pub fn get_assembly(&self, name: &str) -> Option<&AssemblyInfo> {
        self.assemblies.get(name)
    }

    /// Find user code assemblies from .csproj files in the project root
    async fn find_user_assemblies(&self) -> Result<Vec<AssemblyInfo>> {
        source_finder::find_user_assemblies(&self.unity_project_root).await
    }

    /// Find package assemblies from .asmdef files in Library/PackageCache
    async fn find_package_assemblies(&self) -> Result<Vec<AssemblyInfo>> {
        source_finder::find_package_assemblies(&self.unity_project_root).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_unity_project_root;

    #[tokio::test]
    async fn test_discover_assemblies() {
        let unity_root = get_unity_project_root();
        let mut manager = CsDocsManager::new(unity_root);
        
        let result = manager.discover_assemblies().await;
        assert!(result.is_ok(), "Failed to discover assemblies: {:?}", result.err());
        
        let assemblies = manager.get_assemblies();
        assert!(!assemblies.is_empty(), "No assemblies found");
        
        // Should find at least Assembly-CSharp
        assert!(assemblies.contains_key("Assembly-CSharp"), "Assembly-CSharp not found");
        
        // Print discovered assemblies for debugging
        for (name, info) in assemblies {
            println!("Assembly: {}", name);
            println!("  User code: {}", info.is_user_code);
            println!("  Source files: {:?}", info.source_files);
            println!("  Source location: {:?}", info.source_location);
            println!();
        }
    }
}
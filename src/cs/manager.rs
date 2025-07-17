//! CS Documentation Manager
//!
//! This module provides the main CsDocsManager that coordinates assembly discovery
//! and manages the overall C# documentation functionality.

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

use super::{source_assembly::SourceAssembly, source_finder, UnityPackageManager};

/// Main CS documentation manager
#[derive(Debug)]
pub struct CsDocsManager {
    unity_project_root: PathBuf,
    assemblies: HashMap<String, SourceAssembly>,
    package_manager: UnityPackageManager,
}

impl CsDocsManager {
    /// Create a new CS documentation manager for the given Unity project
    pub fn new(unity_project_root: PathBuf) -> Self {
        let package_manager = UnityPackageManager::new(unity_project_root.clone());
        Self {
            unity_project_root,
            assemblies: HashMap::new(),
            package_manager,
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

        // Find package assemblies using the package manager
        self.package_manager.update().await
            .context("Failed to find package assemblies")?;
        
        for package in self.package_manager.get_packages() {
            for assembly in package.assemblies {
                self.assemblies.insert(assembly.name.clone(), assembly);
            }
        }

        Ok(())
    }

    /// Get all discovered assemblies
    pub fn get_assemblies(&self) -> &HashMap<String, SourceAssembly> {
        &self.assemblies
    }

    /// Get a specific assembly by name
    pub fn get_assembly(&self, name: &str) -> Option<&SourceAssembly> {
        self.assemblies.get(name)
    }

    /// Find user code assemblies from .csproj files in the project root
    async fn find_user_assemblies(&self) -> Result<Vec<SourceAssembly>> {
        source_finder::find_user_assemblies(&self.unity_project_root).await
    }

    /// Get source files for a specific assembly on-demand
    pub async fn get_assembly_source_files(&self, assembly_name: &str) -> Result<Vec<PathBuf>> {
        if let Some(assembly) = self.assemblies.get(assembly_name) {
            source_finder::get_assembly_source_files(assembly, &self.unity_project_root).await
        } else {
            Ok(Vec::new())
        }
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
        
        // Print discovered user assemblies for debugging
        for (name, info) in assemblies {
            if !info.is_user_code {
                continue;
            }

            println!("Assembly: {}", name);
            println!("  User code: {}", info.is_user_code);
            println!("  Source location: {:?}", info.source_location);
            
            // Get source files on-demand
            let source_files = manager.get_assembly_source_files(name).await.unwrap_or_default();
            println!("  Source files: {:?}", source_files);
            println!();
        }
        
        // Test cache functionality by running discovery again
        let first_count = assemblies.len();
        let result2 = manager.discover_assemblies().await;
        assert!(result2.is_ok(), "Second discovery should also succeed");
        
        let assemblies2 = manager.get_assemblies();
        assert_eq!(first_count, assemblies2.len(), "Cache should return consistent results");
        
        println!("Cache test passed - consistent results across multiple discoveries");
        
        // Test on-demand source file retrieval for package assemblies
        let package_assembly_name = assemblies2.iter()
            .find(|(_, info)| !info.is_user_code)
            .map(|(name, _)| name.clone());
        
        if let Some(assembly_name) = package_assembly_name {
            let source_files = manager.get_assembly_source_files(&assembly_name).await;
            assert!(source_files.is_ok(), "Should be able to get source files for package assembly");
            println!("Successfully tested on-demand source file retrieval for: {} ({} files)", assembly_name, source_files.unwrap().len());
        }
    }
}
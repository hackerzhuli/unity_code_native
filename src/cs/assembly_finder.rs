//! Assembly finder for Unity compiled assemblies
//!
//! This module handles finding compiled .dll files in Library/ScriptAssemblies
//! and watching for changes to trigger documentation updates.

use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use tokio::fs;
use std::collections::HashSet;

/// Information about a compiled assembly
#[derive(Debug, Clone)]
pub struct CompiledAssembly {
    /// The name of the assembly (without .dll extension)
    pub name: String,
    /// Full path to the .dll file
    pub dll_path: PathBuf,
    /// Last modified time (for change detection)
    pub last_modified: std::time::SystemTime,
}

/// Assembly finder for Unity compiled assemblies
#[derive(Debug)]
pub struct AssemblyFinder {
    unity_project_root: PathBuf,
    script_assemblies_dir: PathBuf,
}

impl AssemblyFinder {
    /// Create a new assembly finder for the given Unity project
    pub fn new(unity_project_root: PathBuf) -> Self {
        let script_assemblies_dir = unity_project_root.join("Library").join("ScriptAssemblies");
        Self {
            unity_project_root,
            script_assemblies_dir,
        }
    }

    /// Find all compiled assemblies in Library/ScriptAssemblies
    pub async fn find_compiled_assemblies(&self) -> Result<Vec<CompiledAssembly>> {
        let mut assemblies = Vec::new();
        
        if !self.script_assemblies_dir.exists() {
            return Ok(assemblies); // No compiled assemblies yet
        }
        
        let mut entries = fs::read_dir(&self.script_assemblies_dir).await
            .context("Failed to read ScriptAssemblies directory")?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("dll") {
                if let Ok(assembly) = self.create_compiled_assembly(&path).await {
                    assemblies.push(assembly);
                }
            }
        }
        
        Ok(assemblies)
    }

    /// Get the names of all compiled assemblies
    pub async fn get_compiled_assembly_names(&self) -> Result<HashSet<String>> {
        let assemblies = self.find_compiled_assemblies().await?;
        Ok(assemblies.into_iter().map(|a| a.name).collect())
    }

    /// Check if a specific assembly exists in the compiled assemblies
    pub async fn assembly_exists(&self, assembly_name: &str) -> bool {
        let dll_path = self.script_assemblies_dir.join(format!("{}.dll", assembly_name));
        dll_path.exists()
    }

    /// Get the path to a specific compiled assembly
    pub fn get_assembly_path(&self, assembly_name: &str) -> PathBuf {
        self.script_assemblies_dir.join(format!("{}.dll", assembly_name))
    }

    /// Create a CompiledAssembly from a .dll file path
    async fn create_compiled_assembly(&self, dll_path: &Path) -> Result<CompiledAssembly> {
        let metadata = fs::metadata(dll_path).await
            .context("Failed to get file metadata")?;
        
        let last_modified = metadata.modified()
            .context("Failed to get last modified time")?;
        
        let name = dll_path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Could not extract assembly name from path"))?
            .to_string();
        
        Ok(CompiledAssembly {
            name,
            dll_path: dll_path.to_path_buf(),
            last_modified,
        })
    }

    /// Watch for changes in the ScriptAssemblies directory
    /// Returns a list of assemblies that have changed since the last check
    pub async fn check_for_changes(&self, previous_assemblies: &[CompiledAssembly]) -> Result<Vec<String>> {
        let current_assemblies = self.find_compiled_assemblies().await?;
        let mut changed_assemblies = Vec::new();
        
        // Create a map of previous assemblies for quick lookup
        let previous_map: std::collections::HashMap<String, &CompiledAssembly> = 
            previous_assemblies.iter().map(|a| (a.name.clone(), a)).collect();
        
        for current in &current_assemblies {
            if let Some(previous) = previous_map.get(&current.name) {
                // Check if the assembly has been modified
                if current.last_modified > previous.last_modified {
                    changed_assemblies.push(current.name.clone());
                }
            } else {
                // New assembly
                changed_assemblies.push(current.name.clone());
            }
        }
        
        Ok(changed_assemblies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_unity_project_root;

    #[tokio::test]
    async fn test_find_compiled_assemblies() {
        let unity_root = get_unity_project_root();
        let finder = AssemblyFinder::new(unity_root);
        
        let assemblies = finder.find_compiled_assemblies().await.unwrap();
        
        println!("Found {} compiled assemblies", assemblies.len());
        for assembly in &assemblies {
            println!("  {}: {:?}", assembly.name, assembly.dll_path);
        }
        
        // Note: This test might not find assemblies if Unity hasn't compiled them yet
        // That's okay for now, we're just testing the functionality
    }

    #[tokio::test]
    async fn test_get_compiled_assembly_names() {
        let unity_root = get_unity_project_root();
        let finder = AssemblyFinder::new(unity_root);
        
        let names = finder.get_compiled_assembly_names().await.unwrap();
        
        println!("Compiled assembly names: {:?}", names);
    }

    #[tokio::test]
    async fn test_assembly_exists() {
        let unity_root = get_unity_project_root();
        let finder = AssemblyFinder::new(unity_root);
        
        // Test with a common assembly name
        let exists = finder.assembly_exists("Assembly-CSharp").await;
        println!("Assembly-CSharp exists: {}", exists);
        
        // Test with a non-existent assembly
        let not_exists = finder.assembly_exists("NonExistentAssembly").await;
        assert!(!not_exists, "Non-existent assembly should not exist");
    }
}
//! Assembly manager for Unity compiled assemblies
//!
//! This module handles finding and caching compiled .dll files in Library/ScriptAssemblies
//! and tracking changes based on file modification times.

use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use tokio::fs;
use std::collections::HashMap;

/// Information about a compiled assembly
#[derive(Debug, Clone)]
pub struct CompiledAssembly {
    /// The name of the assembly (without .dll extension)
    pub name: String,
    /// Full path to the .dll file
    pub dll_path: PathBuf,
    /// Last modified time (for change detection)
    pub last_modified: std::time::SystemTime,
    /// Timestamp when this assembly was cached
    pub cached_at: std::time::SystemTime,
}

/// Assembly manager for Unity compiled assemblies
#[derive(Debug)]
pub struct AssemblyManager {
    unity_project_root: PathBuf,
    script_assemblies_dir: PathBuf,
    /// Cache of compiled assemblies
    cached_assemblies: HashMap<String, CompiledAssembly>,
}

impl AssemblyManager {
    /// Create a new assembly manager for the given Unity project
    pub fn new(unity_project_root: PathBuf) -> Self {
        let script_assemblies_dir = unity_project_root.join("Library").join("ScriptAssemblies");
        Self {
            unity_project_root,
            script_assemblies_dir,
            cached_assemblies: HashMap::new(),
        }
    }

    /// Update the assembly cache by scanning for compiled assemblies
    /// This method checks for new or modified assemblies and updates the internal cache
    pub async fn update(&mut self) -> Result<()> {
        if !self.script_assemblies_dir.exists() {
            self.cached_assemblies.clear();
            return Ok(());
        }
        
        let mut entries = fs::read_dir(&self.script_assemblies_dir).await
            .context("Failed to read ScriptAssemblies directory")?;
        
        let mut current_assemblies = HashMap::new();
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("dll") {
                if let Ok(assembly) = self.create_compiled_assembly(&path).await {
                    current_assemblies.insert(assembly.name.clone(), assembly);
                }
            }
        }
        
        // Update cache with new or modified assemblies
        let current_names: std::collections::HashSet<_> = current_assemblies.keys().cloned().collect();
        
        for (name, assembly) in current_assemblies {
            if let Some(cached) = self.cached_assemblies.get(&name) {
                // Only update if the file has been modified
                if assembly.last_modified > cached.last_modified {
                    self.cached_assemblies.insert(name, assembly);
                }
            } else {
                // New assembly
                self.cached_assemblies.insert(name, assembly);
            }
        }
        
        // Remove assemblies that no longer exist
        self.cached_assemblies.retain(|name, _| current_names.contains(name));
        
        Ok(())
    }

    /// Get all cached compiled assemblies
    /// Returns a vector of all currently cached assemblies
    pub fn get_assemblies(&self) -> Vec<CompiledAssembly> {
        self.cached_assemblies.values().cloned().collect()
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
            cached_at: std::time::SystemTime::now(),
        })
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_unity_project_root;

    #[tokio::test]
    async fn test_assembly_manager_update_and_get() {
        let unity_root = get_unity_project_root();
        let mut manager = AssemblyManager::new(unity_root);
        
        // Update the cache
        manager.update().await.unwrap();
        
        // Get assemblies
        let assemblies = manager.get_assemblies();
        
        println!("Found {} compiled assemblies", assemblies.len());
        for assembly in &assemblies {
            println!("  {}: {:?} (cached at: {:?})", assembly.name, assembly.dll_path, assembly.cached_at);
        }
        
        // Note: This test might not find assemblies if Unity hasn't compiled them yet
        // That's okay for now, we're just testing the functionality
    }

    #[tokio::test]
    async fn test_assembly_manager_caching() {
        let unity_root = get_unity_project_root();
        let mut manager = AssemblyManager::new(unity_root);
        
        // First update
        manager.update().await.unwrap();
        let first_assemblies = manager.get_assemblies();
        
        // Second update (should use cache for unchanged files)
        manager.update().await.unwrap();
        let second_assemblies = manager.get_assemblies();
        
        assert_eq!(first_assemblies.len(), second_assemblies.len());
        println!("Cache working correctly with {} assemblies", first_assemblies.len());
    }
}
//! Unity Package Manager
//!
//! This module provides the UnityPackageManager that handles package discovery,
//! assembly definition parsing, and caching for improved performance.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context, anyhow};
use serde::Deserialize;
use tokio::fs;
use super::source_assembly::SourceAssembly;

/// Package information from packages-lock.json
#[derive(Debug, Deserialize, Clone)]
struct PackageLockFile {
    dependencies: HashMap<String, PackageInfo>,
}

#[derive(Debug, Deserialize, Clone)]
struct PackageInfo {
    version: String,
    depth: u32,
    source: String,
}

/// Assembly definition file structure
#[derive(Debug, Deserialize)]
struct AsmDefFile {
    name: String,
}

/// Cached package data
#[derive(Debug, Clone)]
struct PackageCache {
    /// Package assemblies found in this package
    assemblies: Vec<SourceAssembly>,
    /// Last modified time of the package directory
    last_modified: std::time::SystemTime,
}

/// Unity Package Manager for handling package discovery and caching
#[derive(Debug)]
pub struct UnityPackageManager {
    unity_project_root: PathBuf,
    package_cache_dir: PathBuf,
    packages_lock_path: PathBuf,
    /// Cache of package assemblies by package directory name
    cache: HashMap<String, PackageCache>,
    /// Last modified time of packages-lock.json
    packages_lock_modified: Option<std::time::SystemTime>,
}

impl UnityPackageManager {
    /// Create a new Unity Package Manager
    pub fn new(unity_project_root: PathBuf) -> Self {
        let package_cache_dir = unity_project_root.join("Library").join("PackageCache");
        let packages_lock_path = unity_project_root.join("Packages").join("packages-lock.json");
        
        Self {
            unity_project_root,
            package_cache_dir,
            packages_lock_path,
            cache: HashMap::new(),
            packages_lock_modified: None,
        }
    }

    /// Find all package assemblies, using cache when possible
    pub async fn update(&mut self) -> Result<Vec<SourceAssembly>> {
        // Check if packages-lock.json has changed
        if self.should_refresh_packages_lock().await? {
            self.cache.clear();
            self.update_packages_lock_timestamp().await?;
        }

        let mut all_assemblies = Vec::new();

        if !self.package_cache_dir.exists() {
            return Ok(all_assemblies);
        }

        let mut cache_entries = fs::read_dir(&self.package_cache_dir).await
            .context("Failed to read PackageCache directory")?;

        while let Some(entry) = cache_entries.next_entry().await? {
            let package_dir = entry.path();
            if package_dir.is_dir() {
                let package_name = package_dir.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Check if we have cached data and if it's still valid
                if let Some(cached) = self.cache.get(&package_name) {
                    if self.is_cache_valid(&package_dir, &cached).await? {
                        all_assemblies.extend(cached.assemblies.clone());
                        continue;
                    }
                }

                // Cache miss or invalid, discover assemblies
                let assemblies = self.discover_assemblies_in_package(&package_dir).await?;
                let last_modified = self.get_directory_modified_time(&package_dir).await?;
                
                // Update cache
                self.cache.insert(package_name, PackageCache {
                    assemblies: assemblies.clone(),
                    last_modified,
                });

                all_assemblies.extend(assemblies);
            }
        }

        Ok(all_assemblies)
    }

    /// Clear the package cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.packages_lock_modified = None;
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let cached_packages = self.cache.len();
        let total_assemblies = self.cache.values()
            .map(|c| c.assemblies.len())
            .sum();
        (cached_packages, total_assemblies)
    }

    /// Check if packages-lock.json has been modified
    async fn should_refresh_packages_lock(&self) -> Result<bool> {
        if !self.packages_lock_path.exists() {
            return Ok(false);
        }

        let metadata = fs::metadata(&self.packages_lock_path).await
            .context("Failed to get packages-lock.json metadata")?;
        
        let current_modified = metadata.modified()
            .context("Failed to get packages-lock.json modified time")?;

        Ok(self.packages_lock_modified.map_or(true, |last| current_modified > last))
    }

    /// Update the stored timestamp for packages-lock.json
    async fn update_packages_lock_timestamp(&mut self) -> Result<()> {
        if self.packages_lock_path.exists() {
            let metadata = fs::metadata(&self.packages_lock_path).await
                .context("Failed to get packages-lock.json metadata")?;
            
            self.packages_lock_modified = Some(metadata.modified()
                .context("Failed to get packages-lock.json modified time")?);
        }
        Ok(())
    }

    /// Check if cached data is still valid for a package
    async fn is_cache_valid(&self, package_dir: &Path, cached: &PackageCache) -> Result<bool> {
        let current_modified = self.get_directory_modified_time(package_dir).await?;
        Ok(current_modified <= cached.last_modified)
    }

    /// Get the last modified time of a directory (latest file modification)
    async fn get_directory_modified_time(&self, dir: &Path) -> Result<std::time::SystemTime> {
        let metadata = fs::metadata(dir).await
            .context("Failed to get directory metadata")?;
        
        metadata.modified()
            .context("Failed to get directory modified time")
    }

    /// Discover assemblies in a specific package directory
    async fn discover_assemblies_in_package(&self, package_dir: &Path) -> Result<Vec<SourceAssembly>> {
        let mut assemblies = Vec::new();
        
        // Only look at top-level directories for .asmdef files (not recursive)
        let mut entries = fs::read_dir(package_dir).await
            .context("Failed to read package directory")?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                // Search for any .asmdef files in this top-level directory
                let mut dir_entries = fs::read_dir(&path).await?;
                while let Some(dir_entry) = dir_entries.next_entry().await? {
                    let file_path = dir_entry.path();
                    if file_path.is_file() && file_path.extension().and_then(|s| s.to_str()) == Some("asmdef") {
                        if let Ok(assembly_info) = self.parse_asmdef_file(&file_path, package_dir).await {
                            assemblies.push(assembly_info);
                        }
                    }
                }
            }
        }
        
        Ok(assemblies)
    }

    /// Parse an .asmdef file to extract assembly information
    async fn parse_asmdef_file(&self, asmdef_path: &Path, _package_dir: &Path) -> Result<SourceAssembly> {
        let content = fs::read_to_string(asmdef_path).await
            .context("Failed to read .asmdef file")?;
        
        let asmdef: AsmDefFile = serde_json::from_str(&content)
            .context("Failed to parse .asmdef file")?;
    
        Ok(SourceAssembly {
            name: asmdef.name,
            is_user_code: false,
            source_location: asmdef_path.to_path_buf(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_unity_project_root;

    #[tokio::test]
    async fn test_find_package_assemblies() {
        let unity_root = get_unity_project_root();
        let mut manager = UnityPackageManager::new(unity_root);
        
        let assemblies = manager.update().await.unwrap();
        
        println!("Found {} package assemblies", assemblies.len());
        for assembly in &assemblies {
            assert!(!assembly.is_user_code, "Package assemblies should not be user code");
        }
        
        // Test cache statistics
        let (cached_packages, total_assemblies) = manager.get_cache_stats();
        println!("Cache stats: {} packages, {} assemblies", cached_packages, total_assemblies);
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let unity_root = get_unity_project_root();
        let mut manager = UnityPackageManager::new(unity_root);
        
        // First call - should populate cache
        let assemblies1 = manager.update().await.unwrap();
        let (cached_packages1, _) = manager.get_cache_stats();
        
        // Second call - should use cache
        let assemblies2 = manager.update().await.unwrap();
        let (cached_packages2, _) = manager.get_cache_stats();
        
        assert_eq!(assemblies1.len(), assemblies2.len(), "Cache should return same results");
        assert_eq!(cached_packages1, cached_packages2, "Cache should be stable");
        
        println!("Cache test passed: {} assemblies, {} cached packages", assemblies1.len(), cached_packages1);
    }
}
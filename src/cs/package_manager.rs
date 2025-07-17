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
    dependencies: HashMap<String, PackageInfoInPackageLock>,
}

#[derive(Debug, Deserialize, Clone)]
struct PackageInfoInPackageLock {
    version: String,
    depth: u32,
    source: String,
}

/// Package information from package.json
#[derive(Debug, Deserialize)]
struct PackageJson {
    name: String,
    version: String,
}

/// Assembly definition file structure
#[derive(Debug, Deserialize)]
struct AsmDefFile {
    name: String,
}

/// Cached package data
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    /// Assemblies found in this package
    pub assemblies: Vec<SourceAssembly>,
}

/// Unity Package Manager for handling package discovery and caching, note this only includes packages in `Library/PackageCache`
#[derive(Debug)]
pub struct UnityPackageManager {
    unity_project_root: PathBuf,
    package_cache_dir: PathBuf,
    packages_lock_path: PathBuf,
    /// Cache of package assemblies by package name
    cache: HashMap<String, PackageInfo>,
    /// Last modified time of packages-lock.json
    packages_lock_modified: Option<std::time::SystemTime>,
    /// Set of directory names in `Library/PackageCache` that have been processed (never need to process again)
    processed_dirs: std::collections::HashSet<String>,
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
            processed_dirs: std::collections::HashSet::new(),
        }
    }

    pub fn get_packages(&self) -> Vec<PackageInfo> {
        self.cache.values().cloned().collect()
    }

    /// Find all package assemblies, using cache when possible
    pub async fn update(&mut self) -> Result<()> {
        // Load packages-lock.json to validate packages
        let packages_lock = self.load_packages_lock().await?;
        
        // Check if packages-lock.json has changed and invalidate cache selectively
        if self.should_refresh_packages_lock().await? {
            self.invalidate_removed_packages(&packages_lock).await?;
            self.update_packages_lock_timestamp().await?;
        }

        if !self.package_cache_dir.exists() {
            return Ok(());
        }

        let mut cache_entries = fs::read_dir(&self.package_cache_dir).await
            .context("Failed to read PackageCache directory")?;

        while let Some(entry) = cache_entries.next_entry().await? {
            let package_dir = entry.path();
            if package_dir.is_dir() {
                let dir_name = package_dir.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");
                
                // Skip if we've already processed this directory (unique names guarantee no duplicates)
                if self.processed_dirs.contains(dir_name) {
                    continue;
                }
                
                self.processed_dirs.insert(dir_name.to_string());
                
                self.process_package_directory(&package_dir, &packages_lock).await;
            }
        }

        Ok(())
    }

    /// Clear the package cache
    fn clear_cache(&mut self) {
        self.cache.clear();
        self.packages_lock_modified = None;
        self.processed_dirs.clear();
    }

    /// Get cache statistics
    fn get_cache_stats(&self) -> (usize, usize) {
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

    /// Load packages-lock.json file
    async fn load_packages_lock(&self) -> Result<PackageLockFile> {
        if !self.packages_lock_path.exists() {
            return Ok(PackageLockFile {
                dependencies: HashMap::new(),
            });
        }

        let content = fs::read_to_string(&self.packages_lock_path).await
            .context("Failed to read packages-lock.json")?;
        
        serde_json::from_str(&content)
            .context("Failed to parse packages-lock.json")
    }

    /// Invalidate cache entries for packages that are no longer in packages-lock.json
    async fn invalidate_removed_packages(&mut self, packages_lock: &PackageLockFile) -> Result<()> {
        let mut packages_to_remove = Vec::new();
        
        // Find cached packages that are no longer in packages-lock.json
        for package_name in self.cache.keys() {
            if !packages_lock.dependencies.contains_key(package_name) {
                packages_to_remove.push(package_name.clone());
            }
        }
        
        // Remove invalidated packages from cache
        for package_name in packages_to_remove {
            self.cache.remove(&package_name);
        }
        
        Ok(())
    }

    /// Process a single package directory
    async fn process_package_directory(
        &mut self,
        package_dir: &Path,
        packages_lock: &PackageLockFile,
    ) -> Result<()> {
        // Read package.json to get the actual package name and version
        let package_json_path = package_dir.join("package.json");
        if !package_json_path.exists() {
            return Ok(());
        }

        let package_json_content = fs::read_to_string(&package_json_path).await
            .context("Failed to read package.json")?;
        
        let package_json: PackageJson = serde_json::from_str(&package_json_content)
            .context("Failed to parse package.json")?;

        // Check if this package is included in packages-lock.json
        if !packages_lock.dependencies.contains_key(&package_json.name) {
            return Ok(());
        }

        // Check if we have cached data for this package
        if let Some(cached) = self.cache.get(&package_json.name) {
            return Ok(());
        }

        // Cache miss, discover assemblies
        let assemblies = self.discover_assemblies_in_package(package_dir).await?;
        
        // Update cache using package name as key
        self.cache.insert(package_json.name.clone(), PackageInfo {
            name: package_json.name,
            version: package_json.version,
            assemblies: assemblies,
        });

        Ok(())
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
    async fn test_update() {
        let unity_root = get_unity_project_root();
        let mut manager = UnityPackageManager::new(unity_root);
        
        // First call - should populate cache
        manager.update().await;

        let packages = manager.get_packages();
        
        for package in packages {
            println!("Package: {:?}", package);
        }
    }
}
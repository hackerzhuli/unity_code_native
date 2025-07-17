//! CS Documentation Manager
//!
//! This module provides the main CsDocsManager that coordinates assembly discovery
//! and manages the overall C# documentation functionality.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context, anyhow};
use tokio::fs;

use crate::cs::{
    assembly_manager::AssemblyManager, 
    package_manager::UnityPackageManager, 
    source_assembly::SourceAssembly, 
    source_finder::{find_user_assemblies, get_assembly_source_files},
    docs_compiler::{DocsCompiler, DocsAssembly}
};

/// Cached documentation assembly with timestamp
#[derive(Debug, Clone)]
struct CachedDocsAssembly {
    docs: DocsAssembly,
    cached_at: SystemTime,
}

/// Main CS documentation manager
#[derive(Debug)]
pub struct CsDocsManager {
    unity_project_root: PathBuf,
    assemblies: HashMap<String, SourceAssembly>,
    package_manager: UnityPackageManager,
    assembly_manager: AssemblyManager,
    docs_compiler: DocsCompiler,
    /// In-memory cache of compiled documentation assemblies
    docs_cache: HashMap<String, CachedDocsAssembly>,
    /// Directory where JSON documentation files are stored
    docs_assemblies_dir: PathBuf,
}

impl CsDocsManager {
    /// Create a new CS documentation manager for the given Unity project
    pub fn new(unity_project_root: PathBuf) -> Result<Self> {
        let package_manager = UnityPackageManager::new(unity_project_root.clone());
        let assembly_manager = AssemblyManager::new(unity_project_root.clone());
        let docs_compiler = DocsCompiler::new()?;
        let docs_assemblies_dir = unity_project_root.join("Library").join("UnityCode").join("DocAssemblies");
        
        Ok(Self {
            unity_project_root,
            assemblies: HashMap::new(),
            package_manager,
            assembly_manager,
            docs_compiler,
            docs_cache: HashMap::new(),
            docs_assemblies_dir,
        })
    }

    /// Get documentation for a symbol
    /// 
    /// # Arguments
    /// * `symbol_name` - Full symbol name including namespace and type (methods include parameter types)
    /// * `assembly_name` - Optional assembly name to search in
    /// * `source_file_path` - Optional source file path (must be from user code)
    /// 
    /// Returns the XML documentation string for the symbol
    pub async fn get_docs_for_symbol(
        &mut self,
        symbol_name: &str,
        assembly_name: Option<&str>,
        source_file_path: Option<&Path>,
    ) -> Result<String> {
        // Determine which assembly to search
        let target_assembly_name = if let Some(name) = assembly_name {
            name.to_string()
        } else if let Some(source_path) = source_file_path {
            // Find assembly containing this source file
            if let Some(assembly_name) = self.find_assembly_for_source_file(source_path).await? {
                assembly_name
            } else {
                return Err(anyhow!("Source file not found in any assembly: {:?}", source_path)); // Source file not found in any assembly
            }
        } else {
            return Err(anyhow!("No assembly or source file specified")); // No assembly or source file specified
        };
        
        // Get documentation for the assembly
        let docs_assembly = self.get_docs_for_assembly(&target_assembly_name).await?;
        
        // Search for the symbol in the documentation
        if let Some(docs) = docs_assembly {
            self.find_symbol_docs(&docs, symbol_name)
                .ok_or_else(|| anyhow!("Symbol '{}' not found in assembly '{}'", symbol_name, target_assembly_name))
        } else {
            Err(anyhow!("No documentation available for assembly '{}'", target_assembly_name))
        }
    }
    
    /// Find assembly name that contains the given source file path
    async fn find_assembly_for_source_file(&mut self, source_file_path: &Path) -> Result<Option<String>> {
        // Ensure assemblies are discovered
        self.discover_assemblies().await?;
        
        // Only check user code assemblies for source file paths
        for (assembly_name, assembly) in &self.assemblies {
            if assembly.is_user_code {
                let source_files = get_assembly_source_files(assembly, &self.unity_project_root).await?;
                for source_file in source_files {
                    let full_source_path = self.unity_project_root.join(&source_file);
                    if full_source_path == source_file_path {
                        return Ok(Some(assembly_name.clone()));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Get documentation for a specific assembly, using cache when possible
    async fn get_docs_for_assembly(&mut self, assembly_name: &str) -> Result<Option<DocsAssembly>> {
        // Check if we need to update assemblies
        self.discover_assemblies().await?;
        self.assembly_manager.update().await?;
        
        // Check if assembly exists
        let source_assembly = match self.assemblies.get(assembly_name) {
            Some(assembly) => assembly.clone(),
            None => return Ok(None),
        };
        
        // Check if we can use cached documentation
        if let Some(cached_docs) = self.get_cached_docs(assembly_name).await? {
            return Ok(Some(cached_docs));
        }
        
        // Need to compile documentation
        let include_non_public = source_assembly.is_user_code;
        let docs_assembly = self.docs_compiler.compile_assembly(
            &source_assembly, 
            &self.unity_project_root, 
            include_non_public
        ).await?;
        
        // Cache the compiled documentation
        self.cache_docs(&docs_assembly).await?;
        
        Ok(Some(docs_assembly))
    }
    
    /// Check if cached documentation is available and up-to-date
    async fn get_cached_docs(&mut self, assembly_name: &str) -> Result<Option<DocsAssembly>> {
        // First check in-memory cache
        let cached_time = if let Some(cached) = self.docs_cache.get(assembly_name) {
            Some((cached.docs.clone(), cached.cached_at))
        } else {
            None
        };
        
        if let Some((docs, cached_at)) = cached_time {
            if self.is_cache_valid(assembly_name, cached_at).await? {
                return Ok(Some(docs));
            } else {
                // Remove invalid cache entry
                self.docs_cache.remove(assembly_name);
            }
        }
        
        // Check JSON file cache
        let json_path = self.get_docs_json_path(assembly_name);
        if json_path.exists() {
            let json_metadata = fs::metadata(&json_path).await?;
            let json_modified = json_metadata.modified()?;
            
            if self.is_cache_valid(assembly_name, json_modified).await? {
                // Load from JSON file
                let content = fs::read_to_string(&json_path).await?;
                let docs_assembly: DocsAssembly = serde_json::from_str(&content)
                    .context("Failed to deserialize docs assembly")?;
                
                // Cache in memory
                self.docs_cache.insert(assembly_name.to_string(), CachedDocsAssembly {
                    docs: docs_assembly.clone(),
                    cached_at: json_modified,
                });
                
                return Ok(Some(docs_assembly));
            }
        }
        
        Ok(None)
    }
    
    /// Check if cache is valid by comparing with compiled assembly modification time
    async fn is_cache_valid(&mut self, assembly_name: &str, cache_time: SystemTime) -> Result<bool> {
        // Find the compiled assembly
        let compiled_assemblies = self.assembly_manager.get_assemblies();
        if let Some(compiled_assembly) = compiled_assemblies.iter().find(|a| a.name == assembly_name) {
            Ok(cache_time >= compiled_assembly.last_modified)
        } else {
            // If compiled assembly doesn't exist, cache is invalid
            Ok(false)
        }
    }
    
    /// Cache documentation assembly to both memory and JSON file
    async fn cache_docs(&mut self, docs_assembly: &DocsAssembly) -> Result<()> {
        let now = SystemTime::now();
        
        // Cache in memory
        self.docs_cache.insert(docs_assembly.assembly_name.clone(), CachedDocsAssembly {
            docs: docs_assembly.clone(),
            cached_at: now,
        });
        
        // Ensure docs directory exists
        fs::create_dir_all(&self.docs_assemblies_dir).await?;
        
        // Save to JSON file
        let json_path = self.get_docs_json_path(&docs_assembly.assembly_name);
        let json_content = serde_json::to_string_pretty(docs_assembly)
            .context("Failed to serialize docs assembly")?;
        fs::write(&json_path, json_content).await
            .context("Failed to write docs assembly JSON")?;
        
        Ok(())
    }
    
    /// Get the JSON file path for a documentation assembly
    fn get_docs_json_path(&self, assembly_name: &str) -> PathBuf {
        self.docs_assemblies_dir.join(format!("{}.json", assembly_name))
    }
    
    /// Find documentation for a specific symbol in a documentation assembly
    fn find_symbol_docs(&self, docs_assembly: &DocsAssembly, symbol_name: &str) -> Option<String> {
        // Try to find as a type first
        if let Some(type_doc) = docs_assembly.types.get(symbol_name) {
            if !type_doc.xml_doc.trim().is_empty() {
                return Some(type_doc.xml_doc.clone());
            }
        }
        
        // Try to find as a member in any type
        for type_doc in docs_assembly.types.values() {
            // Try direct member name lookup
            if let Some(member_doc) = type_doc.members.get(symbol_name) {
                if !member_doc.xml_doc.trim().is_empty() {
                    return Some(member_doc.xml_doc.clone());
                }
            }
            
            // Try full member name lookup (Type.Member)
            let full_member_name = format!("{}.{}", type_doc.name, symbol_name);
            if let Some(member_doc) = type_doc.members.get(&full_member_name) {
                if !member_doc.xml_doc.trim().is_empty() {
                    return Some(member_doc.xml_doc.clone());
                }
            }
            
            // Try to find by checking if symbol_name matches full member name
            for (member_name, member_doc) in &type_doc.members {
                let full_member_name = format!("{}.{}", type_doc.name, member_name);
                if full_member_name == symbol_name {
                    if !member_doc.xml_doc.trim().is_empty() {
                        return Some(member_doc.xml_doc.clone());
                    }
                }
            }
        }
        
        None
    }
    
    /// Find all assemblies and their source files
    async fn discover_assemblies(&mut self) -> Result<()> {
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
    
    /// Find user code assemblies from .csproj files in the project root
    async fn find_user_assemblies(&self) -> Result<Vec<SourceAssembly>> {
        find_user_assemblies(&self.unity_project_root).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_unity_project_root;

    #[tokio::test]
    async fn test_get_docs_for_symbol() {
        let unity_root = get_unity_project_root();
        let mut manager = CsDocsManager::new(unity_root).expect("Failed to create manager");
        
        // Test with assembly name
        let result = manager.get_docs_for_symbol(
            "TestClass", 
            Some("Assembly-CSharp"), 
            None
        ).await;
        
        match result {
            Ok(docs) => {
                println!("Found documentation: {}", docs);
            },
            Err(e) => println!("Error getting docs: {:?}", e),
        }
    }
    
    #[tokio::test]
    async fn test_unity_mathematics_docs() {
        let unity_root = get_unity_project_root();
        let mut manager = CsDocsManager::new(unity_root).expect("Failed to create manager");
        
        // Test getting documentation for Unity.Mathematics.math.asin(float4) as requested
        println!("Testing Unity.Mathematics.math.asin(float4) documentation retrieval:");
        
        match manager.get_docs_for_symbol(
            "Unity.Mathematics.math.asin(float4)", 
            Some("Unity.Mathematics"), 
            None
        ).await {
            Ok(docs) => {
                println!("✓ Successfully retrieved documentation for Unity.Mathematics.math.asin:");
                println!("{}", docs);
                assert!(!docs.trim().is_empty(), "Documentation should not be empty");
                assert!(docs.contains("arcsine"), "Documentation should mention arcsine");
            },
            Err(e) => {
                println!("✗ Failed to get Unity.Mathematics.math.asin documentation: {:?}", e);
                
                // Let's see what's available in Unity.Mathematics
                manager.discover_assemblies().await.expect("Failed to discover assemblies");
                if let Ok(Some(docs)) = manager.get_docs_for_assembly("Unity.Mathematics").await {
                    println!("Unity.Mathematics assembly has {} types documented", docs.types.len());
                    
                    // Look for math type specifically
                    if let Some(math_type) = docs.types.values().find(|t| t.name.contains("math")) {
                        println!("Found math type: {} with {} members", math_type.name, math_type.members.len());
                        
                        // Show available asin methods
                        let asin_methods: Vec<_> = math_type.members.values()
                            .filter(|m| m.name.contains("asin"))
                            .collect();
                        
                        if !asin_methods.is_empty() {
                            println!("Available asin methods:");
                            for method in asin_methods {
                                println!("  - {}", method.name);
                            }
                        }
                    }
                }
            }
        }
        
        // Also test with a working example from Assembly-CSharp
        println!("\nTesting with Assembly-CSharp for comparison:");
        if let Ok(Some(docs)) = manager.get_docs_for_assembly("Assembly-CSharp").await {
            if let Some(first_type) = docs.types.values().next() {
                match manager.get_docs_for_symbol(&first_type.name, Some("Assembly-CSharp"), None).await {
                    Ok(docs) => println!("✓ Successfully retrieved docs for {}: {}", first_type.name, docs.trim()),
                    Err(e) => println!("✗ Failed to get docs for {}: {:?}", first_type.name, e),
                }
            }
        }
    }
    
    #[tokio::test]
    async fn test_caching_behavior() {
        let unity_root = get_unity_project_root();
        let mut manager = CsDocsManager::new(unity_root).expect("Failed to create manager");
        
        // First call should compile and cache
        let start_time = std::time::Instant::now();
        let result1 = manager.get_docs_for_symbol(
            "TestClass", 
            Some("Assembly-CSharp"), 
            None
        ).await;
        let first_duration = start_time.elapsed();
        
        // Second call should use cache and be faster
        let start_time = std::time::Instant::now();
        let result2 = manager.get_docs_for_symbol(
            "TestClass", 
            Some("Assembly-CSharp"), 
            None
        ).await;
        let second_duration = start_time.elapsed();
        
        println!("First call took: {:?}", first_duration);
        println!("Second call took: {:?}", second_duration);
        
        // Both should return the same result
        match (result1, result2) {
            (Ok(docs1), Ok(docs2)) => {
                assert_eq!(docs1, docs2, "Cache should return consistent results");
                println!("Cache test passed - consistent results");
            },
            _ => println!("One or both calls failed"),
        }
    }
}
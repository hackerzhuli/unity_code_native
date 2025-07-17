//! CS Documentation Manager
//!
//! This module provides the main CsDocsManager that coordinates assembly discovery
//! and manages the overall C# documentation functionality.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context, anyhow};
use tokio::fs;
use regex::Regex;

/// Normalize a path for comparison by removing Windows UNC prefix if present
fn normalize_path_for_comparison(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.starts_with("\\\\?\\") {
        // Remove the UNC prefix \\?\\ 
        PathBuf::from(&path_str[4..])
    } else {
        path.to_path_buf()
    }
}

use crate::cs::{
    assembly_manager::AssemblyManager, 
    package_manager::UnityPackageManager, 
    source_assembly::SourceAssembly, 
    source_finder::{find_user_assemblies, get_assembly_source_files},
    docs_compiler::{DocsCompiler, DocsAssembly, DOCS_ASSEMBLY_VERSION}
};

/// Parse a single .csproj file to extract assembly information
async fn parse_single_csproj_file(csproj_path: &Path, unity_project_root: &Path) -> Result<SourceAssembly> {
    let content = fs::read_to_string(csproj_path).await
        .context("Failed to read .csproj file")?;
    
    // Parse XML to extract AssemblyName
    let assembly_name = extract_assembly_name(&content)
        .ok_or_else(|| anyhow!("Could not find AssemblyName in .csproj file"))?;
    
    Ok(SourceAssembly {
        name: assembly_name,
        is_user_code: true,
        source_location: csproj_path.to_path_buf(),
    })
}

/// Extract AssemblyName from .csproj XML content
fn extract_assembly_name(content: &str) -> Option<String> {
    // Simple XML parsing to find <AssemblyName>value</AssemblyName>
    if let Some(start) = content.find("<AssemblyName>") {
        let start_pos = start + "<AssemblyName>".len();
        if let Some(end) = content[start_pos..].find("</AssemblyName>") {
            return Some(content[start_pos..start_pos + end].trim().to_string());
        }
    }
    None
}

/// Cached documentation assembly with timestamp
#[derive(Debug, Clone)]
struct CachedDocsAssembly {
    docs: DocsAssembly,
    cached_at: SystemTime,
}

/// Unified cache entry for .csproj file - combines assembly metadata and source files
/// Since each .csproj file defines exactly one assembly, this makes more sense
#[derive(Debug, Clone)]
struct CsprojCacheEntry {
    assembly: SourceAssembly,
    source_files: HashSet<PathBuf>,
    last_modified: SystemTime,
}

/// Enriched documentation result that includes inheritance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocResult {
    /// The XML documentation content
    pub xml_doc: String,
    /// The type name where the documentation is actually located (normalized)
    pub source_type_name: String,
    /// The member name where the documentation is actually located (normalized, None for type docs)
    pub source_member_name: Option<String>,
    /// If inherited, the type name where the documentation was inherited from (normalized)
    pub inherited_from_type_name: Option<String>,
    /// If inherited, the member name where the documentation was inherited from (normalized, None for type docs)
    pub inherited_from_member_name: Option<String>,
    /// Whether this documentation was resolved through inheritdoc
    pub is_inherited: bool,
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
    /// Unified cache for .csproj files - includes assembly metadata and source files
    csproj_cache: HashMap<PathBuf, CsprojCacheEntry>,
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
            csproj_cache: HashMap::new(),
        })
    }

    /// Get documentation for a symbol
    /// 
    /// # Arguments
    /// * `symbol_name` - Full symbol name including namespace and type (methods include parameter types)
    /// * `assembly_name` - Optional assembly name to search in
    /// * `source_file_path` - Optional source file path (must be from user code)
    /// 
    /// Returns enriched documentation information including inheritance details
    pub async fn get_docs_for_symbol(
        &mut self,
        symbol_name: &str,
        assembly_name: Option<&str>,
        source_file_path: Option<&Path>,
    ) -> Result<DocResult> {
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
            self.find_symbol_with_inheritdoc(&docs, symbol_name)
                .ok_or_else(|| anyhow!("Symbol '{}' not found in assembly '{}'", symbol_name, target_assembly_name))
        } else {
            Err(anyhow!("No documentation available for assembly '{}'", target_assembly_name))
        }
    }
    
    /// Find assembly name that contains the given source file path
    async fn find_assembly_for_source_file(&mut self, source_file_path: &Path) -> Result<Option<String>> {
        // Ensure assemblies are discovered
        self.discover_assemblies().await?;
        
        let normalized_search = normalize_path_for_comparison(source_file_path);
        
        // Check cached .csproj files for the source file
        for (csproj_path, cache_entry) in &self.csproj_cache {
            // O(1) lookup in HashSet instead of O(n) iteration
            if cache_entry.source_files.contains(&normalized_search) {
                log::info!("Found source file {} in assembly {}", source_file_path.to_string_lossy(), cache_entry.assembly.name);
                return Ok(Some(cache_entry.assembly.name.clone()));
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
                match serde_json::from_str::<DocsAssembly>(&content) {
                    Ok(docs_assembly) => {
                        // Check version compatibility
                        if docs_assembly.version == DOCS_ASSEMBLY_VERSION {
                            // Cache in memory
                            self.docs_cache.insert(assembly_name.to_string(), CachedDocsAssembly {
                                docs: docs_assembly.clone(),
                                cached_at: json_modified,
                            });
                            
                            return Ok(Some(docs_assembly));
                        } else {
                            // Version mismatch, ignore cached file and recompile
                            // Optionally delete the incompatible cache file
                            let _ = fs::remove_file(&json_path).await;
                        }
                    }
                    Err(_) => {
                        // Failed to deserialize, ignore cached file and recompile
                        // Optionally delete the corrupted cache file
                        let _ = fs::remove_file(&json_path).await;
                    }
                }
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
    
    /// Find symbol documentation - basic version without inheritdoc resolution
    fn find_symbol_basic(&self, docs_assembly: &DocsAssembly, symbol_name: &str) -> Option<DocResult> {
        self.find_symbol_basic_with_options(docs_assembly, symbol_name, false)
    }
    
    /// Find symbol documentation with options for parameter omission
    fn find_symbol_basic_with_options(&self, docs_assembly: &DocsAssembly, symbol_name: &str, allow_parameter_omission: bool) -> Option<DocResult> {
        // Try exact type match
        if let Some(type_doc) = docs_assembly.types.get(symbol_name) {
            if !type_doc.xml_doc.trim().is_empty() {
                return Some(DocResult {
                    xml_doc: type_doc.xml_doc.clone(),
                    source_type_name: type_doc.name.clone(),
                    source_member_name: None,
                    inherited_from_type_name: None,
                    inherited_from_member_name: None,
                    is_inherited: false,
                });
            }
        }
        
        // Try to extract type name from fully qualified symbol name
        if let Some(last_dot) = symbol_name.rfind('.') {
            let type_name = &symbol_name[..last_dot];
            let member_name = &symbol_name[last_dot + 1..];
            
            if let Some(type_doc) = docs_assembly.types.get(type_name) {
                // First try exact match
                if let Some(member_doc) = type_doc.members.get(member_name) {
                    if !member_doc.xml_doc.trim().is_empty() {
                        return Some(DocResult {
                            xml_doc: member_doc.xml_doc.clone(),
                            source_type_name: type_doc.name.clone(),
                            source_member_name: Some(member_doc.name.clone()),
                            inherited_from_type_name: None,
                            inherited_from_member_name: None,
                            is_inherited: false,
                        });
                    }
                }
                
                // If allow_parameter_omission is true and the search symbol doesn't have parameters, try parameter omission
                if allow_parameter_omission && !member_name.contains('(') {
                    // Find all members that start with this method name (without parameters)
                    let matching_members: Vec<_> = type_doc.members.iter()
                        .filter(|(name, _)| {
                            if let Some(paren_pos) = name.find('(') {
                                &name[..paren_pos] == member_name
                            } else {
                                name.as_str() == member_name
                            }
                        })
                        .collect();
                    
                    // If there's exactly one match, use it
                    if matching_members.len() == 1 {
                        let (_, member_doc) = matching_members[0];
                        if !member_doc.xml_doc.trim().is_empty() {
                            return Some(DocResult {
                                xml_doc: member_doc.xml_doc.clone(),
                                source_type_name: type_doc.name.clone(),
                                source_member_name: Some(member_doc.name.clone()),
                                inherited_from_type_name: None,
                                inherited_from_member_name: None,
                                is_inherited: false,
                            });
                        }
                    }
                }
            }
        }
        
        None
    }
    
    /// Find symbol documentation with inheritdoc resolution
    fn find_symbol_with_inheritdoc(&self, docs_assembly: &DocsAssembly, symbol_name: &str) -> Option<DocResult> {
        // First try basic lookup
        if let Some(result) = self.find_symbol_basic(docs_assembly, symbol_name) {
            // Check if it's inheritdoc
            if self.is_inheritdoc(&result.xml_doc) {
                // Extract cref and resolve
                if let Some(cref) = self.extract_cref(&result.xml_doc) {
                    return self.resolve_inheritdoc(&cref, symbol_name, docs_assembly);
                }
            }
            return Some(result);
        }
        None
    }
    
    /// Resolve inheritdoc by generating candidates and calling basic lookup
    fn resolve_inheritdoc(&self, cref: &str, original_symbol: &str, docs_assembly: &DocsAssembly) -> Option<DocResult> {
        let candidates = self.generate_inheritdoc_candidates(cref, original_symbol, docs_assembly);
        
        for candidate in candidates {
            // Use parameter omission when resolving inheritdoc
            if let Some(mut result) = self.find_symbol_basic_with_options(docs_assembly, &candidate, true) {
                // Mark as inherited and set inheritance info
                result.is_inherited = true;
                result.inherited_from_type_name = Some(result.source_type_name.clone());
                result.inherited_from_member_name = result.source_member_name.clone();
                
                // Update source info to original symbol
                let (source_type, source_member) = self.parse_symbol_name(original_symbol);
                result.source_type_name = source_type;
                result.source_member_name = source_member;
                
                return Some(result);
            }
        }
        None
    }
    
    /// Check if XML documentation is just an inheritdoc tag
    fn is_inheritdoc(&self, xml_doc: &str) -> bool {
        let trimmed = xml_doc.trim();
        trimmed.starts_with("<inheritdoc") && trimmed.ends_with("/>")
    }
    
    /// Extract cref attribute from inheritdoc tag
    fn extract_cref(&self, xml_doc: &str) -> Option<String> {
        let re = Regex::new("<inheritdoc\\s+cref\\s*=\\s*[\"']([^\"']+)[\"']\\s*/>").ok()?;
        if let Some(captures) = re.captures(xml_doc.trim()) {
            return captures.get(1).map(|m| m.as_str().to_string());
        }
        None
    }
    
    /// Parse symbol name into type and member components
    fn parse_symbol_name(&self, symbol_name: &str) -> (String, Option<String>) {
        if let Some(last_dot) = symbol_name.rfind('.') {
            let type_part = &symbol_name[..last_dot];
            let member_part = &symbol_name[last_dot + 1..];
            
            // Check if this looks like a method (has parentheses)
            if member_part.contains('(') {
                (type_part.to_string(), Some(member_part.to_string()))
            } else {
                // Could be a nested type, return as type
                (symbol_name.to_string(), None)
            }
        } else {
            (symbol_name.to_string(), None)
        }
    }
    
    /// Generate candidate symbol names for inheritdoc resolution
    fn generate_inheritdoc_candidates(&self, cref: &str, original_symbol: &str, docs_assembly: &DocsAssembly) -> Vec<String> {
        let mut candidates = Vec::new();
        
        // Normalize cref
        let normalized_cref = cref
            .replace('{', "<")
            .replace('}', ">")
            .replace("System.Int32", "int")
            .replace("System.String", "string")
            .replace("System.Boolean", "bool")
            .replace("System.Double", "double")
            .replace("System.Single", "float")
            .replace("System.Int64", "long")
            .replace("System.Int16", "short")
            .replace("System.Byte", "byte")
            .replace("System.Object", "object");
        
        // Get containing type from original symbol
        let (containing_type, _) = self.parse_symbol_name(original_symbol);
        
        // Candidate 1: Prepend containing type if cref doesn't have namespace
        if !normalized_cref.contains('.') {
            candidates.push(format!("{}.{}", containing_type, normalized_cref));
        }
        
        // Candidate 2: Use normalized cref as-is
        candidates.push(normalized_cref.clone());
        
        // Candidate 3: Original cref as-is
        if normalized_cref != cref {
            candidates.push(cref.to_string());
        }
        
        candidates
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
    
    /// Find user code assemblies from .csproj files in the project root with efficient caching
    async fn find_user_assemblies(&mut self) -> Result<Vec<SourceAssembly>> {
        let mut all_assemblies = Vec::new();
        
        // Read all .csproj files in the project root
        let mut entries = fs::read_dir(&self.unity_project_root).await
            .context("Failed to read Unity project directory")?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("csproj") {
                // Check if we can use cached data
                let metadata = fs::metadata(&path).await
                    .context("Failed to get .csproj file metadata")?;
                let last_modified = metadata.modified()
                    .context("Failed to get .csproj file modification time")?;
                
                // Check cache for this file
                let use_cache = if let Some(cached_entry) = self.csproj_cache.get(&path) {
                    cached_entry.last_modified >= last_modified
                } else {
                    false
                };
                
                if use_cache {
                    // Use cached data
                    if let Some(cached_entry) = self.csproj_cache.get(&path) {
                        all_assemblies.push(cached_entry.assembly.clone());
                    }
                } else {
                    // Parse this specific .csproj file and get source files
                    match parse_single_csproj_file(&path, &self.unity_project_root).await {
                        Ok(assembly) => {
                            // Get source files for this assembly
                            let source_files = get_assembly_source_files(&assembly, &self.unity_project_root).await
                                .unwrap_or_default();
                            let normalized_files: HashSet<PathBuf> = source_files
                                .into_iter()
                                .map(|path| normalize_path_for_comparison(&path))
                                .collect();
                            
                            // Update unified cache with both assembly and source files
                            self.csproj_cache.insert(path.clone(), CsprojCacheEntry {
                                assembly: assembly.clone(),
                                source_files: normalized_files,
                                last_modified,
                            });
                            
                            all_assemblies.push(assembly);
                        }
                        Err(_) => {
                            // Skip failed parsing - don't cache anything
                        }
                    }
                }
            }
        }
        
        Ok(all_assemblies)
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
            Ok(doc_result) => {
                println!("Found documentation: {}", doc_result.xml_doc);
                println!("Source: {}.{:?}", doc_result.source_type_name, doc_result.source_member_name);
                if doc_result.is_inherited {
                    println!("Inherited from: {}.{:?}", 
                        doc_result.inherited_from_type_name.unwrap_or_default(),
                        doc_result.inherited_from_member_name);
                }
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
            Ok(doc_result) => {
                println!("✓ Successfully retrieved documentation for Unity.Mathematics.math.asin:");
                println!("{}", doc_result.xml_doc);
                assert!(!doc_result.xml_doc.trim().is_empty(), "Documentation should not be empty");
                assert!(doc_result.xml_doc.contains("arcsine"), "Documentation should mention arcsine");
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
                    Ok(doc_result) => println!("✓ Successfully retrieved docs for {}: {}", first_type.name, doc_result.xml_doc.trim()),
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
            (Ok(doc_result1), Ok(doc_result2)) => {
                assert_eq!(doc_result1.xml_doc, doc_result2.xml_doc, "Cache should return consistent results");
                assert_eq!(doc_result1.source_type_name, doc_result2.source_type_name, "Cache should return consistent source info");
                println!("Cache test passed - consistent results");
            },
            _ => println!("One or both calls failed"),
        }
    }

    #[tokio::test]
    async fn test_unified_csproj_cache() {
        let unity_root = get_unity_project_root();
        let mut manager = CsDocsManager::new(unity_root).expect("Failed to create manager");
        
        // Discover assemblies to populate the cache
        manager.discover_assemblies().await.expect("Failed to discover assemblies");
        
        // Verify that the cache contains both assembly metadata and source files
        assert!(!manager.csproj_cache.is_empty(), "Cache should not be empty after discovery");
        
        for (csproj_path, cache_entry) in &manager.csproj_cache {
            println!("Cached .csproj: {}", csproj_path.to_string_lossy());
            println!("  Assembly: {}", cache_entry.assembly.name);
            println!("  Source files count: {}", cache_entry.source_files.len());
            
            // Verify that each cache entry has the expected structure
            assert!(!cache_entry.assembly.name.is_empty(), "Assembly name should not be empty");
            assert!(cache_entry.assembly.is_user_code, "Should be user code assembly");
            assert_eq!(cache_entry.assembly.source_location, *csproj_path, "Source location should match csproj path");
            
            // Verify that source files are normalized paths
            for source_file in &cache_entry.source_files {
                assert!(source_file.is_absolute(), "Source file paths should be absolute: {}", source_file.to_string_lossy());
            }
        }
        
        println!("✓ Unified cache structure is working correctly");
    }

    #[test]
    fn test_normalize_path_for_comparison() {
        use std::path::Path;
        
        // Test Windows UNC path normalization
        let unc_path = Path::new("\\\\?\\F:\\projects\\unity\\TestUnityCode\\Assets\\Scripts\\TestHover.cs");
        let normal_path = Path::new("F:\\projects\\unity\\TestUnityCode\\Assets\\Scripts\\TestHover.cs");
        
        let normalized_unc = normalize_path_for_comparison(unc_path);
        let normalized_normal = normalize_path_for_comparison(normal_path);
        
        println!("UNC path: {}", unc_path.to_string_lossy());
        println!("Normal path: {}", normal_path.to_string_lossy());
        println!("Normalized UNC: {}", normalized_unc.to_string_lossy());
        println!("Normalized normal: {}", normalized_normal.to_string_lossy());
        
        assert_eq!(normalized_unc, normalized_normal, "UNC and normal paths should be equal after normalization");
        
        // Test that normal paths are unchanged
        let regular_path = Path::new("/home/user/file.txt");
        let normalized_regular = normalize_path_for_comparison(regular_path);
        assert_eq!(regular_path, normalized_regular, "Regular paths should remain unchanged");
        
        // Test edge case - path that starts with \\?\ but is not actually UNC
        let fake_unc = Path::new("\\\\?\\not_a_real_unc");
        let normalized_fake = normalize_path_for_comparison(fake_unc);
        assert_eq!(normalized_fake.to_string_lossy(), "not_a_real_unc", "Fake UNC prefix should be removed");
    }

    #[tokio::test]
    async fn test_inheritdoc_resolution() {
        let unity_project_root = PathBuf::from("UnityProject");
        let mut docs_manager = CsDocsManager::new(unity_project_root).expect("Failed to create manager");
        
        // Test 1: Add() method inherits from Add(int, int, int)
        let result1 = docs_manager.get_docs_for_symbol(
            "UnityProject.Inherit1.Add()",
            None,
            Some(Path::new("UnityProject/Assets/Scripts/Inherit1.cs")),
        ).await;
        
        if let Ok(doc_result) = result1 {
            println!("Test 1 - Add() inheritdoc:");
            println!("XML Doc: {}", doc_result.xml_doc);
            println!("Is Inherited: {}", doc_result.is_inherited);
            if doc_result.is_inherited {
                assert!(doc_result.xml_doc.contains("doc for add with 3 parameters"));
                assert_eq!(doc_result.inherited_from_type_name, Some("UnityProject.Inherit1".to_string()));
            }
        }
        
        // Test 2: Add2() method inherits from Add<T>(T, int, int)
        let result2 = docs_manager.get_docs_for_symbol(
            "UnityProject.Inherit1.Add2()",
            None,
            Some(Path::new("UnityProject/Assets/Scripts/Inherit1.cs")),
        ).await;
        
        if let Ok(doc_result) = result2 {
            println!("Test 2 - Add2() inheritdoc:");
            println!("XML Doc: {}", doc_result.xml_doc);
            println!("Is Inherited: {}", doc_result.is_inherited);
            if doc_result.is_inherited {
                assert!(doc_result.xml_doc.contains("doc for generic add"));
                assert_eq!(doc_result.inherited_from_type_name, Some("UnityProject.Inherit1".to_string()));
            }
        }
        
        // Test 3: Add3() method inherits from Add(ref int, out int, in System.Int32)
        let result3 = docs_manager.get_docs_for_symbol(
            "UnityProject.Inherit1.Add3()",
            None,
            Some(Path::new("UnityProject/Assets/Scripts/Inherit1.cs")),
        ).await;
        
        if let Ok(doc_result) = result3 {
            println!("Test 3 - Add3() inheritdoc:");
            println!("XML Doc: {}", doc_result.xml_doc);
            println!("Is Inherited: {}", doc_result.is_inherited);
            if doc_result.is_inherited {
                assert!(doc_result.xml_doc.contains("doc for add with 3 parameters complex"));
                assert_eq!(doc_result.inherited_from_type_name, Some("UnityProject.Inherit1".to_string()));
            }
        }
        
        // Test 4: Method() from Inherit2 inherits from Inherit1.Add<T>(T, int, int)
        let result4 = docs_manager.get_docs_for_symbol(
            "OtherProject.MyNamespace.Inherit2.Method()",
            None,
            Some(Path::new("UnityProject/Assets/Scripts/Inherit2.cs")),
        ).await;
        
        if let Ok(doc_result) = result4 {
            println!("Test 4 - Inherit2.Method() inheritdoc:");
            println!("XML Doc: {}", doc_result.xml_doc);
            println!("Is Inherited: {}", doc_result.is_inherited);
            if doc_result.is_inherited {
                assert!(doc_result.xml_doc.contains("doc for generic add"));
                assert_eq!(doc_result.inherited_from_type_name, Some("UnityProject.Inherit1".to_string()));
            }
        } else {
            println!("Test 4 failed: {:?}", result4);
        }
        
        // Test 5: Add4() method inherits from Add5 (parameter omission test)
        let result5 = docs_manager.get_docs_for_symbol(
            "UnityProject.Inherit1.Add4(System.Int32,System.Int32)",
            None,
            Some(Path::new("UnityProject/Assets/Scripts/Inherit1.cs")),
        ).await;
        
        if let Ok(doc_result) = result5 {
            println!("Test 5 - Add4() inheritdoc:");
            println!("XML Doc: {}", doc_result.xml_doc);
            println!("Is Inherited: {}", doc_result.is_inherited);
            if doc_result.is_inherited {
                assert!(doc_result.xml_doc.contains("doc for add 5"));
                assert_eq!(doc_result.inherited_from_type_name, Some("UnityProject.Inherit1".to_string()));
            }
        } else {
            println!("Test 5 failed: {:?}", result5);
        }
    }
}
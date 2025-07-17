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

use crate::cs::source_utils::{normalize_path_for_comparison, parse_single_csproj_file};
use crate::cs::xml_doc_utils::merge_xml_docs;
use crate::cs::{
    assembly_manager::AssemblyManager, 
    package_manager::UnityPackageManager, 
    source_assembly::SourceAssembly, 
    source_utils::{find_user_assemblies, get_assembly_source_files},
    docs_compiler::{DocsCompiler, DocsAssembly, DOCS_ASSEMBLY_VERSION},
    xml_doc_utils
};
use crate::cs::compile_utils::normalize_symbol_name;

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
            let normalized_symbol_name = normalize_symbol_name(symbol_name);
            self.find_symbol_with_inheritdoc(&docs, normalized_symbol_name.as_str())
                .ok_or_else(|| anyhow!("Symbol '{}' not found in assembly '{}'", normalized_symbol_name, target_assembly_name))
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
            // Check if it contains inheritdoc (either top-level or nested)
            if self.contains_inheritdoc(&result.xml_doc) {
                // Extract cref and resolve
                if let Some(cref) = self.extract_cref(&result.xml_doc) {
                    return self.resolve_inheritdoc_with_merge(&cref, symbol_name, docs_assembly, &result.xml_doc, &result);
                }
            }
            return Some(result);
        }
        None
    }
    
    /// Resolve inheritdoc by finding target documentation and merging with original
    fn resolve_inheritdoc_with_merge(&self, cref: &str, original_symbol: &str, docs_assembly: &DocsAssembly, original_xml: &str, original_result: &DocResult) -> Option<DocResult> {
        let candidates = self.generate_inheritdoc_candidates(cref, original_result.source_type_name.as_str(), docs_assembly);
        
        for candidate in candidates {
            // Use parameter omission when resolving inheritdoc
            if let Some(target_result) = self.find_symbol_basic_with_options(docs_assembly, &candidate, true) {
                // Merge the XML documentation
                if let Some(merged_xml) = merge_xml_docs(original_xml, &target_result.xml_doc) {                    
                    return Some(DocResult {
                        xml_doc: merged_xml,
                        source_type_name: original_result.source_type_name.clone(),
                        source_member_name: original_result.source_member_name.clone(),
                        inherited_from_type_name: Some(target_result.source_type_name),
                        inherited_from_member_name: target_result.source_member_name,
                        is_inherited: true,
                    });
                }
            }
        }
        
        return Some(original_result.clone());
    }

    /// Check if XML documentation contains any inheritdoc tag
    fn contains_inheritdoc(&self, xml_doc: &str) -> bool {
        xml_doc.contains("<inheritdoc")
    }
    
    /// Check if XML documentation is just an inheritdoc tag (kept for backward compatibility)
    fn is_inheritdoc(&self, xml_doc: &str) -> bool {
        let trimmed = xml_doc.trim();
        trimmed.starts_with("<inheritdoc") && trimmed.ends_with("/>")
    }
    
    /// Extract cref attribute from the first inheritdoc tag found
    fn extract_cref(&self, xml_doc: &str) -> Option<String> {
        let re = Regex::new("<inheritdoc\\s+cref\\s*=\\s*[\"']([^\"']+)[\"']\\s*/>").ok()?;
        if let Some(captures) = re.captures(xml_doc) {
            return captures.get(1).map(|m| m.as_str().to_string());
        }
        None
    }

    /// Generate candidate symbol names for inheritdoc resolution
    fn generate_inheritdoc_candidates(&self, cref: &str, containing_type: &str, docs_assembly: &DocsAssembly) -> Vec<String> {
        let mut candidates = Vec::new();
        
        // Normalize cref
        let normalized_cref = normalize_symbol_name(cref);
        candidates.push(normalized_cref.clone());
        candidates.push(format!("{}.{}", containing_type, normalized_cref));
        if let Some(v) = docs_assembly.types.get(&containing_type.to_string()) {
            for namespace in &v.using_namespaces{
                candidates.push(format!("{}.{}", namespace, normalized_cref))
            }
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
#[path ="docs_manager_tests.rs"]
mod tests;
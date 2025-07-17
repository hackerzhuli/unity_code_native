//! Documentation compiler for C# assemblies
//!
//! This module handles compiling XML documentation from C# source files into JSON format.
//! It uses tree-sitter to parse C# source files and extract XML documentation comments.

use std::path::{Path, PathBuf};
use anyhow::{Result, Context, anyhow};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tree_sitter::{Parser, Language, Node, Tree};
use super::source_assembly::SourceAssembly;
use super::compile_utils::{normalize_type_name, normalize_member_name};

/// Current version of the DocsAssembly data structure
pub const DOCS_ASSEMBLY_VERSION: u32 = 1;

/// Represents XML documentation for a C# member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberDoc {
    /// The name of the member (for methods, includes parameter types)
    pub name: String,
    /// The XML documentation content
    pub xml_doc: String,
    /// Whether this member is public
    pub is_public: bool,
}

/// Represents XML documentation for a C# type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDoc {
    /// Fully qualified name of the type
    pub name: String,
    /// The XML documentation content for the type itself
    pub xml_doc: String,
    /// Whether this type is public
    pub is_public: bool,
    /// Documentation for all members of this type (key: member name)
    pub members: std::collections::HashMap<String, MemberDoc>,
}

/// Represents the complete documentation assembly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsAssembly {
    /// Version of the data structure for compatibility checking
    pub version: u32,
    /// Name of the assembly
    pub assembly_name: String,
    /// Whether this is user code or package code
    pub is_user_code: bool,
    /// All type documentation in this assembly (key: fully qualified type name)
    pub types: std::collections::HashMap<String, TypeDoc>,
}

/// Documentation compiler for C# assemblies
pub struct DocsCompiler {
    parser: Parser,
}

impl std::fmt::Debug for DocsCompiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocsCompiler")
            .field("parser", &"<Parser>")
            .finish()
    }
}

impl DocsCompiler {
    /// Create a new documentation compiler
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_c_sharp::language();
        parser.set_language(language)
            .map_err(|e| anyhow!("Failed to set C# language: {}", e))?;
        
        Ok(Self { parser })
    }
    
    /// Compile documentation for a source assembly
    pub async fn compile_assembly(&mut self, assembly: &SourceAssembly, unity_project_root: &Path, include_non_public: bool) -> Result<DocsAssembly> {
        let source_files = self.get_assembly_source_files(assembly, unity_project_root).await?;
        let mut types = Vec::new();
        
        for source_file in source_files {
            let full_path = unity_project_root.join(&source_file);
            let file_types = self.extract_docs_from_file(&full_path, include_non_public).await?;
            types.extend(file_types);
        }
        
        // Merge partial classes
        let merged_types = self.merge_partial_classes(types);
        
        Ok(DocsAssembly {
            version: DOCS_ASSEMBLY_VERSION,
            assembly_name: assembly.name.clone(),
            is_user_code: assembly.is_user_code,
            types: merged_types,
        })
    }
    
    /// Merge partial classes with the same name
    fn merge_partial_classes(&self, types: Vec<TypeDoc>) -> std::collections::HashMap<String, TypeDoc> {
        let mut merged_types: std::collections::HashMap<String, TypeDoc> = std::collections::HashMap::new();
        
        for type_doc in types {
            if let Some(existing_type) = merged_types.get_mut(&type_doc.name) {
                // Merge members from this partial class into the existing one
                existing_type.members.extend(type_doc.members);
                
                // Merge XML documentation (combine if both exist)
                if !type_doc.xml_doc.is_empty() {
                    if existing_type.xml_doc.is_empty() {
                        existing_type.xml_doc = type_doc.xml_doc;
                    } else {
                        // Combine XML docs with a separator
                        existing_type.xml_doc = format!("{} {}", existing_type.xml_doc, type_doc.xml_doc);
                    }
                }
                
                // Keep the most permissive visibility (if any part is public, the whole type is public)
                existing_type.is_public = existing_type.is_public || type_doc.is_public;
            } else {
                // First occurrence of this type
                merged_types.insert(type_doc.name.clone(), type_doc);
            }
        }
        
        merged_types
    }
    
    /// Extract documentation from a single C# source file
    async fn extract_docs_from_file(&mut self, file_path: &Path, include_non_public: bool) -> Result<Vec<TypeDoc>> {
        let content = fs::read_to_string(file_path).await
            .context("Failed to read source file")?;
        
        let tree = self.parser.parse(&content, None)
            .ok_or_else(|| anyhow!("Failed to parse C# file: {:?}", file_path))?;
        
        let mut types = Vec::new();
        self.extract_types_from_node(tree.root_node(), &content, include_non_public, &mut types, String::new())?;
        
        Ok(types)
    }
    
    /// Recursively extract type documentation from tree-sitter nodes
    fn extract_types_from_node(
        &self,
        node: Node,
        source: &str,
        include_non_public: bool,
        types: &mut Vec<TypeDoc>,
        namespace_prefix: String,
    ) -> Result<()> {
        match node.kind() {
            "namespace_declaration" => {
                // Extract namespace name and recurse into it
                if let Some(name_node) = node.child_by_field_name("name") {
                    let namespace_name = name_node.utf8_text(source.as_bytes())?;
                    let new_prefix = if namespace_prefix.is_empty() {
                        namespace_name.to_string()
                    } else {
                        format!("{}.{}", namespace_prefix, namespace_name)
                    };
                    
                    // Recurse into namespace body
                    if let Some(body) = node.child_by_field_name("body") {
                        for child in body.children(&mut body.walk()) {
                            self.extract_types_from_node(child, source, include_non_public, types, new_prefix.clone())?;
                        }
                    }
                }
            },
            "class_declaration" | "interface_declaration" | "struct_declaration" | "enum_declaration" => {
                // Extract type documentation
                if let Some(type_doc) = self.extract_type_doc(node, source, include_non_public, &namespace_prefix)? {
                    types.push(type_doc);
                }
            },
            _ => {
                // Recurse into child nodes
                for child in node.children(&mut node.walk()) {
                    self.extract_types_from_node(child, source, include_non_public, types, namespace_prefix.clone())?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract documentation for a specific type
    fn extract_type_doc(
        &self,
        node: Node,
        source: &str,
        include_non_public: bool,
        namespace_prefix: &str,
    ) -> Result<Option<TypeDoc>> {
        // Get type name
        let name_node = node.child_by_field_name("name")
            .ok_or_else(|| anyhow!("Type declaration missing name"))?;
        let type_name = name_node.utf8_text(source.as_bytes())?;
        
        // Check if type is public
        let is_public = self.is_public_declaration(node, source)?;
        
        // Skip non-public types if not including them
        if !include_non_public && !is_public {
            return Ok(None);
        }
        
        // Build fully qualified name and normalize it
        let full_name = if namespace_prefix.is_empty() {
            normalize_type_name(name_node, source).unwrap_or_else(|| type_name.to_string())
        } else {
            format!("{}.{}", namespace_prefix, type_name)
        };
        
        // Extract XML documentation comment
        let xml_doc = self.extract_xml_doc_comment(node, source)?;
        
        // Extract member documentation
        let mut members = std::collections::HashMap::new();
        if let Some(body) = node.child_by_field_name("body") {
            for child in body.children(&mut body.walk()) {
                if let Some(member_doc) = self.extract_member_doc(child, source, include_non_public)? {
                    members.insert(member_doc.name.clone(), member_doc);
                }
            }
        }
        
        Ok(Some(TypeDoc {
            name: full_name,
            xml_doc,
            is_public,
            members,
        }))
    }
    
    /// Extract documentation for a type member
    fn extract_member_doc(
        &self,
        node: Node,
        source: &str,
        include_non_public: bool,
    ) -> Result<Option<MemberDoc>> {
        match node.kind() {
            "method_declaration" | "property_declaration" | "field_declaration" | "event_declaration" => {
                let is_public = self.is_public_declaration(node, source)?;
                
                // Skip non-public members if not including them
                if !include_non_public && !is_public {
                    return Ok(None);
                }
                
                // Get member name and normalize it
                let name = normalize_member_name(node, source)
                    .ok_or_else(|| anyhow!("Failed to normalize member name for node at {}", node.start_position().row))?;
                let xml_doc = self.extract_xml_doc_comment(node, source)?;
                
                // Skip members without XML documentation
                if xml_doc.trim().is_empty() {
                    return Ok(None);
                }
                
                Ok(Some(MemberDoc {
                    name,
                    xml_doc,
                    is_public,
                }))
            },
            _ => Ok(None),
        }
    }
    
    /// Check if a declaration is public
    fn is_public_declaration(&self, node: Node, source: &str) -> Result<bool> {
        // Look for modifier children (Tree-sitter C# uses "modifier" not "modifiers")
        for child in node.children(&mut node.walk()) {
            if child.kind() == "modifier" {
                let modifier_text = child.utf8_text(source.as_bytes())?;
                if modifier_text.trim() == "public" {
                    return Ok(true);
                }
            }
        }
        
        // Default to false if no public modifier found
        Ok(false)
    }
    
    /// Get the name of a member, including parameter types for methods
    fn get_member_name(&self, node: Node, source: &str) -> Result<String> {
        // Handle field declarations specially
        if node.kind() == "field_declaration" {
            // For field declarations, look for variable_declaration -> variable_declarator -> identifier
            for child in node.children(&mut node.walk()) {
                if child.kind() == "variable_declaration" {
                    for var_child in child.children(&mut child.walk()) {
                        if var_child.kind() == "variable_declarator" {
                            for declarator_child in var_child.children(&mut var_child.walk()) {
                                if declarator_child.kind() == "identifier" {
                                    return Ok(declarator_child.utf8_text(source.as_bytes())?.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Try to find name field first
        if let Some(name_node) = node.child_by_field_name("name") {
            let base_name = name_node.utf8_text(source.as_bytes())?;
            
            // For methods, append generic parameters and parameter types
            if node.kind() == "method_declaration" {
                // Check for generic type parameters
                let mut method_name_with_generics = base_name.to_string();
                if let Some(type_params_node) = node.child_by_field_name("type_parameters") {
                    if let Ok(type_params_text) = type_params_node.utf8_text(source.as_bytes()) {
                        method_name_with_generics = format!("{}{}", base_name, type_params_text);
                    }
                }
                
                if let Some(params_node) = node.child_by_field_name("parameters") {
                    let mut param_types = Vec::new();
                    for child in params_node.children(&mut params_node.walk()) {
                        if child.kind() == "parameter" {
                            if let Some(type_node) = child.child_by_field_name("type") {
                                if let Ok(type_text) = type_node.utf8_text(source.as_bytes()) {
                                    param_types.push(type_text);
                                }
                            }
                        }
                    }
                    return Ok(format!("{}({})", method_name_with_generics, param_types.join(", ")));
                }
            }
            
            Ok(base_name.to_string())
        } else {
            // Fallback: try to find any identifier node
            for child in node.children(&mut node.walk()) {
                if child.kind() == "identifier" {
                    if let Ok(name) = child.utf8_text(source.as_bytes()) {
                        return Ok(name.to_string());
                    }
                }
            }
            
            // Last resort: use the node kind as name
            Ok(format!("<{}_at_{}>", node.kind(), node.start_position().row))
        }
    }
    
    /// Extract XML documentation comment preceding a node
    fn extract_xml_doc_comment(&self, node: Node, source: &str) -> Result<String> {
        let mut xml_doc = String::new();
        
        // Look for preceding comment nodes
        let mut current = node;
        while let Some(prev) = current.prev_sibling() {
            if prev.kind() == "comment" {
                let comment_text = prev.utf8_text(source.as_bytes())?;
                if comment_text.trim_start().starts_with("///") {
                    // This is an XML doc comment
                    let doc_line = comment_text.trim_start().strip_prefix("///")
                        .unwrap_or(comment_text)
                        .trim();
                    if !doc_line.is_empty() {
                        xml_doc = format!("{}{}", doc_line, if xml_doc.is_empty() { "" } else { "\n" }) + &xml_doc;
                    }
                } else {
                    // Not an XML doc comment, stop looking
                    break;
                }
            } else if prev.kind() != "whitespace" {
                // Non-comment, non-whitespace node, stop looking
                break;
            }
            current = prev;
        }
        
        Ok(xml_doc)
    }
    


    /// Get source files for an assembly on-demand based on its source_location
    pub async fn get_assembly_source_files(&self, assembly: &SourceAssembly, unity_project_root: &Path) -> Result<Vec<PathBuf>> {
        let source_location = &assembly.source_location;
        
        if let Some(extension) = source_location.extension().and_then(|s| s.to_str()) {
            match extension {
                "csproj" => {
                    // Read source files from .csproj file
                    let content = fs::read_to_string(source_location).await
                        .context("Failed to read .csproj file")?;
                    self.extract_compile_items(&content, unity_project_root)
                        .context("Failed to extract compile items from .csproj")
                },
                "asmdef" => {
                    // Find .cs files in the directory containing the .asmdef file
                    if let Some(asmdef_dir) = source_location.parent() {
                        self.find_cs_files_in_dir(asmdef_dir, unity_project_root).await
                    } else {
                        Ok(Vec::new())
                    }
                },
                _ => Ok(Vec::new()),
            }
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Recursively find all .cs files in a directory and return paths relative to Unity project root
    pub fn find_cs_files_in_dir<'a>(&'a self, dir: &'a Path, unity_project_root: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<PathBuf>>> + 'a>> {
        Box::pin(async move {
            let mut cs_files = Vec::new();
            
            let mut entries = fs::read_dir(dir).await
                .context("Failed to read directory")?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("cs") {
                    // Convert to relative path from Unity project root
                    if let Ok(relative_path) = path.strip_prefix(unity_project_root) {
                        cs_files.push(relative_path.to_path_buf());
                    }
                } else if path.is_dir() {
                    // Recursively search subdirectories
                    let mut sub_files = self.find_cs_files_in_dir(&path, unity_project_root).await?;
                    cs_files.append(&mut sub_files);
                }
            }
            
            Ok(cs_files)
        })
    }
    
    /// Extract Compile items from .csproj XML content
    fn extract_compile_items(&self, content: &str, unity_project_root: &Path) -> Result<Vec<PathBuf>> {
        let mut source_files = Vec::new();
        
        // Find all <Compile Include="path" /> items
        let mut search_pos = 0;
        while let Some(compile_start) = content[search_pos..].find("<Compile Include=\"") {
            let absolute_start = search_pos + compile_start + "<Compile Include=\"".len();
            if let Some(quote_end) = content[absolute_start..].find('"') {
                let file_path = &content[absolute_start..absolute_start + quote_end];
                
                // Convert to PathBuf and make it relative to unity project root
                let path_buf = PathBuf::from(file_path);
                
                // Ensure the file exists and is a .cs file
                let full_path = unity_project_root.join(&path_buf);
                if full_path.exists() && path_buf.extension().and_then(|s| s.to_str()) == Some("cs") {
                    source_files.push(path_buf);
                }
                
                search_pos = absolute_start + quote_end;
            } else {
                break;
            }
        }
        
        Ok(source_files)
    }
    
    /// Find user assemblies from .csproj files in the Unity project root
    pub async fn find_user_assemblies(&self, unity_project_root: &Path) -> Result<Vec<SourceAssembly>> {
        let mut assemblies = Vec::new();
        
        // Read all .csproj files in the project root
        let mut entries = fs::read_dir(unity_project_root).await
            .context("Failed to read Unity project directory")?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("csproj") {
                if let Ok(assembly_info) = self.parse_csproj_file(&path, unity_project_root).await {
                    assemblies.push(assembly_info);
                }
            }
        }
        
        Ok(assemblies)
    }
    
    /// Parse a .csproj file to extract assembly information
    async fn parse_csproj_file(&self, csproj_path: &Path, unity_project_root: &Path) -> Result<SourceAssembly> {
        let content = fs::read_to_string(csproj_path).await
            .context("Failed to read .csproj file")?;
        
        // Parse XML to extract AssemblyName and Compile items
        let assembly_name = self.extract_assembly_name(&content)
            .ok_or_else(|| anyhow!("Could not find AssemblyName in .csproj file"))?;
        
        Ok(SourceAssembly {
            name: assembly_name,
            is_user_code: true,
            source_location: csproj_path.to_path_buf(),
        })
    }
    
    /// Extract AssemblyName from .csproj XML content
    fn extract_assembly_name(&self, content: &str) -> Option<String> {
        // Simple XML parsing to find <AssemblyName>value</AssemblyName>
        if let Some(start) = content.find("<AssemblyName>") {
            let start_pos = start + "<AssemblyName>".len();
            if let Some(end) = content[start_pos..].find("</AssemblyName>") {
                return Some(content[start_pos..start_pos + end].trim().to_string());
            }
        }
        None
    }
}

#[cfg(test)]
#[path ="docs_compiler_tests.rs"]
mod tests;
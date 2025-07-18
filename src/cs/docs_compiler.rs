//! Documentation compiler for C# assemblies
//!
//! This module handles compiling XML documentation from C# source files into JSON format.
//! It uses tree-sitter to parse C# source files and extract XML documentation comments.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tree_sitter::{Parser, Node};
use crate::cs::source_utils::{extract_compile_items, find_cs_files_in_dir};
use crate::language::tree_utils::has_error_nodes;

use super::source_assembly::SourceAssembly;
use super::compile_utils::{normalize_type_name, normalize_member_name};
use super::constants::*;
use super::error::{CsResult, CsError, IoContext};

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
    /// Using namespaces from the source file containing this type
    pub using_namespaces: Vec<String>,
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
    pub fn new() -> CsResult<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_c_sharp::language();
        parser.set_language(language)
            .map_err(|e| CsError::Parse {
                file: PathBuf::from("<unknown>"),
                message: format!("Failed to set C# language: {}", e),
            })?;
        
        Ok(Self { parser })
    }
    
    /// Compile documentation for a source assembly
    pub async fn compile_assembly(&mut self, assembly: &SourceAssembly, unity_project_root: &Path, include_non_public: bool) -> CsResult<DocsAssembly> {
        let source_files = self.get_assembly_source_files(assembly, unity_project_root).await?;
        let mut merged_types: std::collections::HashMap<String, TypeDoc> = std::collections::HashMap::new();
        
        for source_file in source_files {
            let full_path = unity_project_root.join(&source_file);
            // If any error occured in a file, we can ignore that
            // We're compiling docs, not an executable, so it doesn't matter, just extract the correct stuff
            if let Ok(file_types) = self.extract_docs_from_file(&full_path, include_non_public).await{
                // Incrementally merge types from this file
                for type_doc in file_types {
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
                        
                        // Merge using namespaces (combine and deduplicate)
                        for using_ns in type_doc.using_namespaces {
                            if !existing_type.using_namespaces.contains(&using_ns) {
                                existing_type.using_namespaces.push(using_ns);
                            }
                        }
                    } else {
                        // First occurrence of this type
                        merged_types.insert(type_doc.name.clone(), type_doc);
                    }
                }
            }
        }
        
        Ok(DocsAssembly {
            version: DOCS_ASSEMBLY_VERSION,
            assembly_name: assembly.name.clone(),
            is_user_code: assembly.is_user_code,
            types: merged_types,
        })
    }
    
    /// Extract documentation from a single C# source file
    async fn extract_docs_from_file(&mut self, file_path: &Path, include_non_public: bool) -> CsResult<Vec<TypeDoc>> {
        let content = fs::read_to_string(file_path).await
            .with_io_context("Failed to read source file")?;
        
        let tree = self.parser.parse(&content, None)
            .ok_or_else(|| CsError::Parse {
                file: file_path.to_path_buf(),
                message: "Failed to parse C# file".to_string(),
            })?;

        if has_error_nodes(tree.root_node()){
            return Err(CsError::Parse {
                file: file_path.to_path_buf(),
                message: "There are syntax errors in C# file".to_string(),
            });
        }
        
        // Extract using directives from the file
        let using_namespaces = self.extract_using_directives(tree.root_node(), &content)?;
        
        let mut types = Vec::new();
        self.extract_types_from_node(tree.root_node(), &content, include_non_public, &mut types, String::new(), &using_namespaces)?;
        
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
        using_namespaces: &[String],
    ) -> CsResult<()> {
        match node.kind() {
            NAMESPACE_DECLARATION => {
                // Extract namespace name and recurse into it
                if let Some(name_node) = node.child_by_field_name(NAME_FIELD) {
                    let namespace_name = name_node.utf8_text(source.as_bytes())
                        .map_err(|e| CsError::Parse {
                            file: PathBuf::from("<unknown>"),
                            message: format!("Failed to extract namespace name: {}", e),
                        })?;
                    let new_prefix = if namespace_prefix.is_empty() {
                        namespace_name.to_string()
                    } else {
                        format!("{}.{}", namespace_prefix, namespace_name)
                    };
                    
                    // Recurse into namespace body
                    if let Some(body) = node.child_by_field_name(BODY_FIELD) {
                        for child in body.children(&mut body.walk()) {
                            self.extract_types_from_node(child, source, include_non_public, types, new_prefix.clone(), using_namespaces)?;
                        }
                    }
                }
            },
            CLASS_DECLARATION | INTERFACE_DECLARATION | STRUCT_DECLARATION | ENUM_DECLARATION => {
                // Extract type documentation
                if let Some(type_doc) = self.extract_type_doc(node, source, include_non_public, &namespace_prefix, using_namespaces)? {
                    types.push(type_doc);
                }
            },
            _ => {
                // Recurse into child nodes
                for child in node.children(&mut node.walk()) {
                    self.extract_types_from_node(child, source, include_non_public, types, namespace_prefix.clone(), using_namespaces)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract documentation for a specific type from a syntax tree parsed from a source file
    fn extract_type_doc(
        &self,
        node: Node,
        source: &str,
        include_non_public: bool,
        namespace_prefix: &str,
        using_namespaces: &[String],
    ) -> CsResult<Option<TypeDoc>> {
        // Get type name
        let name_node = node.child_by_field_name(NAME_FIELD)
            .ok_or_else(|| CsError::Parse {
                file: PathBuf::from("<unknown>"),
                message: "Type declaration missing name".to_string(),
            })?;
        let type_name = name_node.utf8_text(source.as_bytes())
            .map_err(|e| CsError::Parse {
                file: PathBuf::from("<unknown>"),
                message: format!("Failed to extract type name: {}", e),
            })?;
        
        // Check if type is public
        let is_public = self.is_public_declaration(node, source)?;
        
        // Skip non-public types if not including them
        if !include_non_public && !is_public {
            return Ok(None);
        }
        
        // Build fully qualified name and normalize it
        let full_name = normalize_type_name(name_node, source).unwrap_or_else(|| {
            // Fallback: manually build the qualified name
            if namespace_prefix.is_empty() {
                type_name.to_string()
            } else {
                format!("{}.{}", namespace_prefix, type_name)
            }
        });
        
        // Extract XML documentation comment
        let xml_doc = self.extract_xml_doc_comment(node, source)?;
        
        // Extract member documentation
        let mut members = std::collections::HashMap::new();
        if let Some(body) = node.child_by_field_name(BODY_FIELD) {
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
            using_namespaces: using_namespaces.to_vec(),
        }))
    }
    
    /// Extract documentation for a type member
    fn extract_member_doc(
        &self,
        node: Node,
        source: &str,
        include_non_public: bool,
    ) -> CsResult<Option<MemberDoc>> {
        match node.kind() {
            METHOD_DECLARATION | PROPERTY_DECLARATION | FIELD_DECLARATION | EVENT_DECLARATION => {
                let is_public = self.is_public_declaration(node, source)?;
                
                // Skip non-public members if not including them
                if !include_non_public && !is_public {
                    return Ok(None);
                }
                
                // Get member name and normalize it
                let name = normalize_member_name(node, source)
                    .ok_or_else(|| CsError::Parse {
                        file: PathBuf::from("<unknown>"),
                        message: format!("Failed to normalize member name for node at {}", node.start_position().row),
                    })?;
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
    fn is_public_declaration(&self, node: Node, source: &str) -> CsResult<bool> {
        // Look for modifier children (Tree-sitter C# uses "modifier" not "modifiers")
        for child in node.children(&mut node.walk()) {
            if child.kind() == MODIFIER {
                let modifier_text = child.utf8_text(source.as_bytes())
                    .map_err(|e| CsError::Parse {
                        file: PathBuf::from("<unknown>"),
                        message: format!("Failed to extract modifier text: {}", e),
                    })?;
                if modifier_text.trim() == PUBLIC_MODIFIER {
                    return Ok(true);
                }
            }
        }
        
        // Default to false if no public modifier found
        Ok(false)
    }
    
    /// Extract using directives from the compilation unit
    fn extract_using_directives(&self, root_node: Node, source: &str) -> CsResult<Vec<String>> {
        let mut using_namespaces = Vec::new();
        
        // Walk through all children of the root node to find using directives
        for child in root_node.children(&mut root_node.walk()) {
            if child.kind() == USING_DIRECTIVE {
                // Extract the namespace from the using directive
                // The tree-sitter C# grammar uses child nodes like 'identifier' and 'qualified_name'
                for grandchild in child.children(&mut child.walk()) {
                    if grandchild.kind() == QUALIFIED_NAME || grandchild.kind() == IDENTIFIER {
                        let namespace_name = grandchild.utf8_text(source.as_bytes())
                            .map_err(|e| CsError::Parse {
                                file: PathBuf::from("<unknown>"),
                                message: format!("Failed to extract using directive: {}", e),
                            })?;
                        using_namespaces.push(namespace_name.to_string());
                        break;
                    }
                }
            }
        }
        
        Ok(using_namespaces)
    }
    
    /// Extract XML documentation comment preceding a node
    fn extract_xml_doc_comment(&self, node: Node, source: &str) -> CsResult<String> {
        let mut xml_doc = String::new();
        
        // Look for preceding comment nodes
        let mut current = node;
        while let Some(prev) = current.prev_sibling() {
            if prev.kind() == COMMENT {
                let comment_text = prev.utf8_text(source.as_bytes())
                    .map_err(|e| CsError::Parse {
                        file: PathBuf::from("<unknown>"),
                        message: format!("Failed to extract comment text: {}", e),
                    })?;
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
            } else if prev.kind() != WHITESPACE {
                // Non-comment, non-whitespace node, stop looking
                break;
            }
            current = prev;
        }
        
        Ok(xml_doc)
    }
    
    /// Get source files for an assembly on-demand based on its source_location
    pub async fn get_assembly_source_files(&self, assembly: &SourceAssembly, unity_project_root: &Path) -> CsResult<Vec<PathBuf>> {
        let source_location = &assembly.source_location;
        
        if let Some(extension) = source_location.extension().and_then(|s| s.to_str()) {
            match extension {
                "csproj" => {
                    // Read source files from .csproj file
                    let content = fs::read_to_string(source_location).await
                        .with_io_context("Failed to read .csproj file")?;
                    extract_compile_items(&content, unity_project_root)
                        .map_err(|e| CsError::Parse {
                            file: source_location.to_path_buf(),
                            message: format!("Failed to extract compile items from .csproj: {}", e),
                        })
                },
                "asmdef" => {
                    // Find .cs files in the directory containing the .asmdef file
                    if let Some(asmdef_dir) = source_location.parent() {
                        find_cs_files_in_dir(asmdef_dir, unity_project_root).await
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
}

#[cfg(test)]
#[path ="docs_compiler_tests.rs"]
mod tests;
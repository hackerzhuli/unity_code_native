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
    /// Documentation for all members of this type
    pub members: Vec<MemberDoc>,
}

/// Represents the complete documentation assembly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsAssembly {
    /// Name of the assembly
    pub assembly_name: String,
    /// Whether this is user code or package code
    pub is_user_code: bool,
    /// All type documentation in this assembly
    pub types: Vec<TypeDoc>,
}

/// Documentation compiler for C# assemblies
pub struct DocsCompiler {
    parser: Parser,
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
            assembly_name: assembly.name.clone(),
            is_user_code: assembly.is_user_code,
            types: merged_types,
        })
    }
    
    /// Merge partial classes with the same name
    fn merge_partial_classes(&self, types: Vec<TypeDoc>) -> Vec<TypeDoc> {
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
        
        merged_types.into_values().collect()
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
        
        // Build fully qualified name
        let full_name = if namespace_prefix.is_empty() {
            type_name.to_string()
        } else {
            format!("{}.{}", namespace_prefix, type_name)
        };
        
        // Extract XML documentation comment
        let xml_doc = self.extract_xml_doc_comment(node, source)?;
        
        // Extract member documentation
        let mut members = Vec::new();
        if let Some(body) = node.child_by_field_name("body") {
            for child in body.children(&mut body.walk()) {
                if let Some(member_doc) = self.extract_member_doc(child, source, include_non_public)? {
                    members.push(member_doc);
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
                
                // Get member name
                let name = self.get_member_name(node, source)?;
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
            
            // For methods, append parameter types
            if node.kind() == "method_declaration" {
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
                    return Ok(format!("{}({})", base_name, param_types.join(", ")));
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
mod tests {
    use super::*;
    use crate::test_utils::get_unity_project_root;

    #[tokio::test]
    async fn test_compile_assembly_csharp() {
        let mut compiler = DocsCompiler::new().unwrap();
        let unity_root = get_unity_project_root();
        
        // Find Assembly-CSharp
        let assemblies = compiler.find_user_assemblies(&unity_root).await.unwrap();
        let assembly_csharp = assemblies.iter().find(|a| a.name == "Assembly-CSharp")
            .expect("Should find Assembly-CSharp assembly");
        
        // Compile documentation (include non-public for user code)
        let docs_assembly = compiler.compile_assembly(assembly_csharp, &unity_root, true).await.unwrap();
        
        assert_eq!(docs_assembly.assembly_name, "Assembly-CSharp");
        assert!(docs_assembly.is_user_code);
        
        // Should have at least some types
        assert!(!docs_assembly.types.is_empty(), "Should find at least one type");
        
        // Find the UnityProject.TestClass type
        let test_type = docs_assembly.types.iter().find(|t| t.name == "UnityProject.TestClass")
            .expect("Should find UnityProject.TestClass type");
        
        // Verify TestClass documentation
        assert!(test_type.is_public, "TestClass should be public");
        assert!(test_type.xml_doc.contains("A simple test class for documentation extraction"), 
                "TestClass should have XML doc mentioning 'simple test class'");
        assert!(test_type.xml_doc.contains("Contains various member types for testing"),
                "TestClass should mention various member types");
        
        // Verify we have the expected public members (should not include private ones)
        let public_members: Vec<_> = test_type.members.iter().filter(|m| m.is_public).collect();
        assert!(public_members.len() >= 3, "Should have at least 3 public members");
        
        // Check for specific public members
        let add_method = test_type.members.iter().find(|m| m.name == "Add(int, int)")
            .expect("Should find Add(int, int) method");
        assert!(add_method.is_public, "Add method should be public");
        assert!(add_method.xml_doc.contains("Adds two numbers together"),
                "Add method should mention adding numbers");
        assert!(add_method.xml_doc.contains("param name=\"a\""),
                "Add method should reference parameter 'a'");
        
        let public_field = test_type.members.iter().find(|m| m.name == "PublicField")
            .expect("Should find PublicField");
        assert!(public_field.is_public, "PublicField should be public");
        assert!(public_field.xml_doc.contains("A public field for testing"),
                "PublicField should mention testing");
        
        let test_property = test_type.members.iter().find(|m| m.name == "TestProperty")
            .expect("Should find TestProperty");
        assert!(test_property.is_public, "TestProperty should be public");
        assert!(test_property.xml_doc.contains("Gets or sets the test property"),
                "TestProperty should mention gets or sets");
        
        // Verify private members are not included for user code (they should be included)
        let private_method = test_type.members.iter().find(|m| m.name.contains("ProcessPrivately"));
        assert!(private_method.is_some(), "Private method should be included for user code");
        if let Some(pm) = private_method {
            assert!(!pm.is_public, "ProcessPrivately should not be public");
        }
        
        // Note: Nested classes are not supported by the doc compiler as documented
        
        // Verify private class is included for user code
        let private_class = docs_assembly.types.iter().find(|t| t.name == "UnityProject.PrivateClass");
        assert!(private_class.is_some(), "PrivateClass should be included for user code");
        if let Some(pc) = private_class {
            assert!(!pc.is_public, "PrivateClass should not be public");
            assert!(pc.xml_doc.contains("A private class with public methods"),
                    "PrivateClass should mention private class");
        }
        
        // Verify that undocumented members are excluded from the results
        let undocumented_method = test_type.members.iter().find(|m| m.name == "UndocumentedMethod()");
        assert!(undocumented_method.is_none(), "UndocumentedMethod should be excluded (no XML docs)");
        
        let undocumented_method_with_params = test_type.members.iter().find(|m| m.name.contains("UndocumentedMethodWithParams"));
        assert!(undocumented_method_with_params.is_none(), "UndocumentedMethodWithParams should be excluded (no XML docs)");
        
        let undocumented_property = test_type.members.iter().find(|m| m.name == "UndocumentedProperty");
        assert!(undocumented_property.is_none(), "UndocumentedProperty should be excluded (no XML docs)");
        
        let undocumented_field = test_type.members.iter().find(|m| m.name == "UndocumentedField");
        assert!(undocumented_field.is_none(), "UndocumentedField should be excluded (no XML docs)");
        
        let undocumented_private_method = test_type.members.iter().find(|m| m.name.contains("UndocumentedPrivateMethod"));
        assert!(undocumented_private_method.is_none(), "UndocumentedPrivateMethod should be excluded (no XML docs)");
        
        // Verify that only documented members are included
        let documented_members: Vec<_> = test_type.members.iter().filter(|m| !m.xml_doc.trim().is_empty()).collect();
        assert_eq!(test_type.members.len(), documented_members.len(), 
                   "All included members should have XML documentation");
        
        println!("Verified that {} undocumented members were excluded from compilation", 5);
        
        // Serialize to JSON
        let json = serde_json::to_string_pretty(&docs_assembly).unwrap();
        println!("Compiled docs assembly JSON:\n{}", json);
        
        // Write to file for inspection
        let output_path = unity_root.join("Library").join("UnityCode").join("DocAssemblies");
        fs::create_dir_all(&output_path).await.unwrap();
        let json_file = output_path.join("Assembly-CSharp.json");
        fs::write(&json_file, &json).await.unwrap();
        
        println!("Documentation written to: {:?}", json_file);
        println!("Successfully verified UnityProject.TestClass with {} members", test_type.members.len());
    }

    #[tokio::test]
    async fn test_partial_class_merging() {
        let mut compiler = DocsCompiler::new().unwrap();
        let unity_root = get_unity_project_root();
        
        // Find Assembly-CSharp
        let assemblies = compiler.find_user_assemblies(&unity_root).await.unwrap();
        let assembly_csharp = assemblies.iter().find(|a| a.name == "Assembly-CSharp")
            .expect("Should find Assembly-CSharp assembly");
        
        // Compile documentation (include non-public for user code)
        let docs_assembly = compiler.compile_assembly(assembly_csharp, &unity_root, true).await.unwrap();
        
        // Find the UnityProject.PartialTestClass type
        let partial_type = docs_assembly.types.iter().find(|t| t.name == "UnityProject.PartialTestClass")
            .expect("Should find UnityProject.PartialTestClass type");
        
        // Verify PartialTestClass documentation
        assert!(partial_type.is_public, "PartialTestClass should be public");
        
        // The XML doc should contain content from one of the partial definitions
        // (Tree-sitter will pick up the first one it encounters)
        assert!(!partial_type.xml_doc.is_empty(), "PartialTestClass should have XML documentation");
        
        // Verify we have members from both partial class files merged together
        let public_members: Vec<_> = partial_type.members.iter().filter(|m| m.is_public).collect();
        assert!(public_members.len() >= 5, "Should have at least 5 public members from both partial files");
        
        // Check for members from the first partial file (PartialTest1.cs)
        let first_part_field = partial_type.members.iter().find(|m| m.name == "FirstPartField")
            .expect("Should find FirstPartField from first partial file");
        assert!(first_part_field.is_public, "FirstPartField should be public");
        assert!(first_part_field.xml_doc.contains("A public field from the first partial file"),
                "FirstPartField should have correct documentation");
        
        let first_part_method = partial_type.members.iter().find(|m| m.name == "ProcessFromFirstPart(int)")
            .expect("Should find ProcessFromFirstPart method from first partial file");
        assert!(first_part_method.is_public, "ProcessFromFirstPart should be public");
        assert!(first_part_method.xml_doc.contains("A method from the first partial class file"),
                "ProcessFromFirstPart should have correct documentation");
        
        // Check for members from the second partial file (PartialTest2.cs)
        let second_part_field = partial_type.members.iter().find(|m| m.name == "SecondPartField")
            .expect("Should find SecondPartField from second partial file");
        assert!(second_part_field.is_public, "SecondPartField should be public");
        assert!(second_part_field.xml_doc.contains("A public field from the second partial file"),
                "SecondPartField should have correct documentation");
        
        let combined_property = partial_type.members.iter().find(|m| m.name == "CombinedProperty")
            .expect("Should find CombinedProperty from second partial file");
        assert!(combined_property.is_public, "CombinedProperty should be public");
        assert!(combined_property.xml_doc.contains("A property from the second partial class file"),
                "CombinedProperty should have correct documentation");
        
        let second_part_method = partial_type.members.iter().find(|m| m.name == "ProcessFromSecondPart(string)")
            .expect("Should find ProcessFromSecondPart method from second partial file");
        assert!(second_part_method.is_public, "ProcessFromSecondPart should be public");
        assert!(second_part_method.xml_doc.contains("A method from the second partial class file"),
                "ProcessFromSecondPart should have correct documentation");
        
        let combine_method = partial_type.members.iter().find(|m| m.name == "CombineFromBothParts()")
            .expect("Should find CombineFromBothParts method from second partial file");
        assert!(combine_method.is_public, "CombineFromBothParts should be public");
        assert!(combine_method.xml_doc.contains("Another public method that combines data from both parts"),
                "CombineFromBothParts should have correct documentation");
        
        // Verify private members from both files are included (for user code)
        let private_members: Vec<_> = partial_type.members.iter().filter(|m| !m.is_public).collect();
        assert!(private_members.len() >= 4, "Should have at least 4 private members from both partial files");
        
        // Check for private members from both files
        let first_private_field = partial_type.members.iter().find(|m| m.name == "firstPrivateField");
        assert!(first_private_field.is_some(), "Should find firstPrivateField from first partial file");
        
        let second_private_field = partial_type.members.iter().find(|m| m.name == "secondPrivateField");
        assert!(second_private_field.is_some(), "Should find secondPrivateField from second partial file");
        
        let first_private_method = partial_type.members.iter().find(|m| m.name.contains("ProcessPrivatelyFromFirst"));
        assert!(first_private_method.is_some(), "Should find ProcessPrivatelyFromFirst from first partial file");
        
        let second_private_method = partial_type.members.iter().find(|m| m.name.contains("ProcessPrivatelyFromSecond"));
        assert!(second_private_method.is_some(), "Should find ProcessPrivatelyFromSecond from second partial file");
        
        println!("Successfully verified partial class merging for UnityProject.PartialTestClass");
        println!("Total members found: {} (public: {}, private: {})", 
                 partial_type.members.len(), public_members.len(), private_members.len());
        
        // Print all member names for debugging
        println!("All members:");
        for member in &partial_type.members {
            println!("  - {} ({})", member.name, if member.is_public { "public" } else { "private" });
        }
    }

    #[tokio::test]
    async fn test_exclude_non_public_types_and_members() {
        let mut compiler = DocsCompiler::new().unwrap();
        let unity_root = get_unity_project_root();
        
        // Find Assembly-CSharp
        let assemblies = compiler.find_user_assemblies(&unity_root).await.unwrap();
        let assembly_csharp = assemblies.iter().find(|a| a.name == "Assembly-CSharp")
            .expect("Should find Assembly-CSharp assembly");
        
        // Compile documentation excluding non-public types and members
        let docs_assembly = compiler.compile_assembly(assembly_csharp, &unity_root, false).await.unwrap();
        
        assert_eq!(docs_assembly.assembly_name, "Assembly-CSharp");
        assert!(docs_assembly.is_user_code);
        
        // Should have at least some types
        assert!(!docs_assembly.types.is_empty(), "Should find at least one public type");
        
        // Verify that private types are excluded
        let private_class = docs_assembly.types.iter().find(|t| t.name == "UnityProject.PrivateClass");
        assert!(private_class.is_none(), "PrivateClass should be excluded when include_non_public is false");
        
        // Find the UnityProject.TestClass type (should still be present as it's public)
        let test_type = docs_assembly.types.iter().find(|t| t.name == "UnityProject.TestClass")
            .expect("Should find UnityProject.TestClass type as it's public");
        
        // Verify TestClass is public
        assert!(test_type.is_public, "TestClass should be public");
        
        // Verify that only public members are included
        let all_members_public = test_type.members.iter().all(|m| m.is_public);
        assert!(all_members_public, "All members should be public when include_non_public is false");
        
        // Check for specific public members that should be present
        let add_method = test_type.members.iter().find(|m| m.name == "Add(int, int)");
        assert!(add_method.is_some(), "Add method should be present as it's public");
        
        let public_field = test_type.members.iter().find(|m| m.name == "PublicField");
        assert!(public_field.is_some(), "PublicField should be present as it's public");
        
        let test_property = test_type.members.iter().find(|m| m.name == "TestProperty");
        assert!(test_property.is_some(), "TestProperty should be present as it's public");
        
        // Verify that private members are excluded
        let private_method = test_type.members.iter().find(|m| m.name.contains("ProcessPrivately"));
        assert!(private_method.is_none(), "Private methods should be excluded when include_non_public is false");
        
        let private_field = test_type.members.iter().find(|m| m.name == "privateField");
        assert!(private_field.is_none(), "Private fields should be excluded when include_non_public is false");
        
        // Test with partial classes - verify only public members are included
        let partial_type = docs_assembly.types.iter().find(|t| t.name == "UnityProject.PartialTestClass")
            .expect("Should find UnityProject.PartialTestClass type as it's public");
        
        // Verify all members in partial class are public
        let all_partial_members_public = partial_type.members.iter().all(|m| m.is_public);
        assert!(all_partial_members_public, "All partial class members should be public when include_non_public is false");
        
        // Verify specific public members from partial classes are present
        let first_part_field = partial_type.members.iter().find(|m| m.name == "FirstPartField");
        assert!(first_part_field.is_some(), "FirstPartField should be present as it's public");
        
        let second_part_field = partial_type.members.iter().find(|m| m.name == "SecondPartField");
        assert!(second_part_field.is_some(), "SecondPartField should be present as it's public");
        
        // Verify private members from partial classes are excluded
        let first_private_field = partial_type.members.iter().find(|m| m.name == "firstPrivateField");
        assert!(first_private_field.is_none(), "firstPrivateField should be excluded when include_non_public is false");
        
        let second_private_field = partial_type.members.iter().find(|m| m.name == "secondPrivateField");
        assert!(second_private_field.is_none(), "secondPrivateField should be excluded when include_non_public is false");
        
        println!("Successfully verified exclusion of non-public types and members");
        println!("TestClass members (all public): {}", test_type.members.len());
        println!("PartialTestClass members (all public): {}", partial_type.members.len());
        
        // Print all types to verify only public ones are included
        println!("All types found (should be only public):");
        for type_doc in &docs_assembly.types {
            println!("  - {} ({})", type_doc.name, if type_doc.is_public { "public" } else { "private" });
        }
    }

    #[tokio::test]
    async fn test_compile_unity_mathematics_package() {
        let mut compiler = DocsCompiler::new().unwrap();
        let unity_root = get_unity_project_root();
        
        // Initialize package manager and update packages
        let mut package_manager = crate::cs::UnityPackageManager::new(unity_root.clone());
        package_manager.update().await.unwrap();
        
        // Find Unity.Mathematics package
        let packages = package_manager.get_packages();
        let math_package = packages.iter()
            .flat_map(|p| &p.assemblies)
            .find(|a| a.name == "Unity.Mathematics")
            .expect("Should find Unity.Mathematics assembly in packages");
        
        println!("Found Unity.Mathematics assembly at: {:?}", math_package.source_location);
        
        // Compile documentation for Unity.Mathematics (include non-public for comprehensive testing)
        let docs_assembly = compiler.compile_assembly(math_package, &unity_root, true).await.unwrap();
        
        assert_eq!(docs_assembly.assembly_name, "Unity.Mathematics");
        assert!(!docs_assembly.is_user_code, "Unity.Mathematics should not be user code");
        
        // Should have at least some types
        assert!(!docs_assembly.types.is_empty(), "Should find at least one type in Unity.Mathematics");
        
        // Look for the Unity.Mathematics.math type
        let math_type = docs_assembly.types.iter()
            .find(|t| t.name == "Unity.Mathematics.math")
            .expect("Should find Unity.Mathematics.math type");
        
        println!("Found Unity.Mathematics.math type with {} members", math_type.members.len());
        println!("Math type is public: {}", math_type.is_public);
        
        // Verify the math type has some members (it should have many mathematical functions)
        assert!(!math_type.members.is_empty(), "Unity.Mathematics.math should have members");
        
        // Write the documentation to a JSON file for manual inspection
        let output_path = unity_root.join("Library").join("UnityCode").join("DocAssemblies");
        fs::create_dir_all(&output_path).await.unwrap();
        
        let json_file_path = output_path.join("Unity.Mathematics.json");
        let json_content = serde_json::to_string(&docs_assembly).unwrap();
        fs::write(&json_file_path, json_content).await.unwrap();
        
        println!("Unity.Mathematics documentation written to: {:?}", json_file_path);
        println!("Total types found: {}", docs_assembly.types.len());
        
        // Print some sample types for verification
        println!("Sample types found:");
        for (i, type_doc) in docs_assembly.types.iter().take(5).enumerate() {
            println!("  {}. {} ({} members)", i + 1, type_doc.name, type_doc.members.len());
        }
        
        // Print some sample members from the math type
        println!("Sample members from Unity.Mathematics.math:");
        for (i, member) in math_type.members.iter().take(10).enumerate() {
            println!("  {}. {} ({})", i + 1, member.name, if member.is_public { "public" } else { "private" });
        }
    }
}
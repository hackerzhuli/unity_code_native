//! Source file finder for C# assemblies
//!
//! This module handles finding source files for assemblies from two sources:
//! 1. User code: .csproj files in Unity project root
//! 2. Package code: .asmdef files in Library/PackageCache

use std::path::{Path, PathBuf};
use anyhow::{Result, Context, anyhow};
use tokio::fs;
use super::source_assembly::SourceAssembly;

/// Normalize a path for comparison by removing Windows UNC prefix if present
pub fn normalize_path_for_comparison(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.starts_with("\\\\?\\") {
        // Remove the UNC prefix \\?\\ 
        PathBuf::from(&path_str[4..])
    } else {
        path.to_path_buf()
    }
}

/// Parse a single .csproj file to extract assembly information
pub async fn parse_csproj_file(csproj_path: &Path) -> Result<SourceAssembly> {
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
pub fn extract_assembly_name(content: &str) -> Option<String> {
    // Simple XML parsing to find <AssemblyName>value</AssemblyName>
    if let Some(start) = content.find("<AssemblyName>") {
        let start_pos = start + "<AssemblyName>".len();
        if let Some(end) = content[start_pos..].find("</AssemblyName>") {
            return Some(content[start_pos..start_pos + end].trim().to_string());
        }
    }
    None
}

/// Find user assemblies from .csproj files in the Unity project root
pub async fn find_user_assemblies(unity_project_root: &Path) -> Result<Vec<SourceAssembly>> {
    let mut assemblies = Vec::new();
    
    // Read all .csproj files in the project root
    let mut entries = fs::read_dir(unity_project_root).await
        .context("Failed to read Unity project directory")?;
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("csproj") {
            if let Ok(assembly_info) = parse_csproj_file(&path).await {
                assemblies.push(assembly_info);
            }
        }
    }
    
    Ok(assemblies)
}

/// Get source files for an assembly on-demand based on its source_location
pub async fn get_assembly_source_files(assembly: &SourceAssembly, unity_project_root: &Path) -> Result<Vec<PathBuf>> {
    let source_location = &assembly.source_location;
    
    if let Some(extension) = source_location.extension().and_then(|s| s.to_str()) {
        match extension {
            "csproj" => {
                // Read source files from .csproj file
                let content = fs::read_to_string(source_location).await
                    .context("Failed to read .csproj file")?;
                extract_compile_items(&content, unity_project_root)
                    .context("Failed to extract compile items from .csproj")
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

/// Recursively find all .cs files in a directory and return absolute paths
pub fn find_cs_files_in_dir<'a>(dir: &'a Path, unity_project_root: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<PathBuf>>> + 'a>> {
    Box::pin(async move {
        let mut cs_files = Vec::new();
        
        let mut entries = fs::read_dir(dir).await
            .context("Failed to read directory")?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("cs") {
                // Ensure all paths are absolute for clients
                let absolute_path = if path.is_absolute() {
                    path
                } else {
                    unity_project_root.join(&path)
                };
                cs_files.push(absolute_path);
            } else if path.is_dir() {
                // Recursively search subdirectories
                let mut sub_files = find_cs_files_in_dir(&path, unity_project_root).await?;
                cs_files.append(&mut sub_files);
            }
        }
        
        Ok(cs_files)
    })
}

/// Extract Compile items from .csproj XML content
pub fn extract_compile_items(content: &str, unity_project_root: &Path) -> Result<Vec<PathBuf>> {
    let mut source_files = Vec::new();
    
    // Find all <Compile Include="path" /> items
    let mut search_pos = 0;
    while let Some(compile_start) = content[search_pos..].find("<Compile Include=\"") {
        let absolute_start = search_pos + compile_start + "<Compile Include=\"".len();
        if let Some(quote_end) = content[absolute_start..].find('"') {
            let file_path = &content[absolute_start..absolute_start + quote_end];
            
            // Convert to PathBuf and ensure it's absolute
            let path_buf = PathBuf::from(file_path);
            
            // Ensure the file exists and is a .cs file
            let full_path = unity_project_root.join(&path_buf);
            if full_path.exists() && path_buf.extension().and_then(|s| s.to_str()) == Some("cs") {
                // Return absolute path for clients
                source_files.push(full_path);
            }
            
            search_pos = absolute_start + quote_end;
        } else {
            break;
        }
    }
    
    Ok(source_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_unity_project_root;

    #[tokio::test]
    async fn test_find_user_assemblies() {
        let unity_root = get_unity_project_root();
        let assemblies = find_user_assemblies(&unity_root).await.unwrap();
        
        assert!(!assemblies.is_empty(), "Should find at least one user assembly");
        
        // Should find Assembly-CSharp
        let assembly_csharp = assemblies.iter().find(|a| a.name == "Assembly-CSharp");
        assert!(assembly_csharp.is_some(), "Should find Assembly-CSharp assembly");
        
        let assembly = assembly_csharp.unwrap();
        assert!(assembly.is_user_code, "Assembly-CSharp should be user code");
        
        // Test on-demand source file retrieval
        let source_files = get_assembly_source_files(assembly, &unity_root).await.unwrap();
        assert!(!source_files.is_empty(), "Should have source files");

        for source_file in &source_files {
            println!("Source file: {}", source_file.to_string_lossy());
        }
        
        println!("Found {} user assemblies", assemblies.len());
    }



    #[tokio::test]
    async fn test_extract_assembly_name() {
        use crate::test_utils::get_unity_project_root;
        use tokio::fs;
        
        let unity_root = get_unity_project_root();
        let csproj_path = unity_root.join("Assembly-CSharp.csproj");
        
        // Read the actual Assembly-CSharp.csproj file
        let content = fs::read_to_string(&csproj_path).await
            .expect("Failed to read Assembly-CSharp.csproj");
        
        let name = extract_assembly_name(&content);
        assert_eq!(name, Some("Assembly-CSharp".to_string()));
    }

    #[tokio::test]
    async fn test_extract_compile_items() {
        use crate::test_utils::get_unity_project_root;
        use tokio::fs;
        
        let unity_root = get_unity_project_root();
        let csproj_path = unity_root.join("Assembly-CSharp.csproj");
        
        // Read the actual Assembly-CSharp.csproj file
        let content = fs::read_to_string(&csproj_path).await
            .expect("Failed to read Assembly-CSharp.csproj");
        
        let items = extract_compile_items(&content, &unity_root).unwrap();
        
        // Verify that the actual existing files are extracted
        assert!(!items.is_empty(), "Should extract at least one source file");
        
        // Check that all extracted files actually exist and are absolute paths
        for item in &items {
            assert!(item.exists(), "Extracted file should exist: {:?}", item);
            assert!(item.is_absolute(), "All paths should be absolute: {:?}", item);
            assert_eq!(item.extension().and_then(|s| s.to_str()), Some("cs"), "Should only extract .cs files");
        }
        
        // Should find the Readme.cs file that we know exists (as absolute path)
        let readme_absolute_path = unity_root.join("Assets/TutorialInfo/Scripts/Readme.cs");
        assert!(items.contains(&readme_absolute_path), "Should find Readme.cs in the extracted files");
        
        println!("Successfully extracted {} source files from Assembly-CSharp.csproj", items.len());
    }
}
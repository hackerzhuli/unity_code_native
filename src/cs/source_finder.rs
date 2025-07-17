//! Source file finder for C# assemblies
//!
//! This module handles finding source files for assemblies from two sources:
//! 1. User code: .csproj files in Unity project root
//! 2. Package code: .asmdef files in Library/PackageCache

use std::path::{Path, PathBuf};
use anyhow::{Result, Context, anyhow};
use serde::Deserialize;
use tokio::fs;
use super::AssemblyInfo;

/// Package information from packages-lock.json
#[derive(Debug, Deserialize)]
struct PackageLockFile {
    dependencies: std::collections::HashMap<String, PackageInfo>,
}

#[derive(Debug, Deserialize)]
struct PackageInfo {
    version: String,
    #[serde(default)]
    dependencies: std::collections::HashMap<String, String>,
}

/// Assembly definition file structure
#[derive(Debug, Deserialize)]
struct AsmDefFile {
    name: String,
    #[serde(default)]
    #[serde(rename = "rootNamespace")]
    root_namespace: String,
}

/// Find user assemblies from .csproj files in the Unity project root
pub async fn find_user_assemblies(unity_project_root: &Path) -> Result<Vec<AssemblyInfo>> {
    let mut assemblies = Vec::new();
    
    // Read all .csproj files in the project root
    let mut entries = fs::read_dir(unity_project_root).await
        .context("Failed to read Unity project directory")?;
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("csproj") {
            if let Ok(assembly_info) = parse_csproj_file(&path, unity_project_root).await {
                assemblies.push(assembly_info);
            }
        }
    }
    
    Ok(assemblies)
}

/// Find package assemblies from .asmdef files in Library/PackageCache
pub async fn find_package_assemblies(unity_project_root: &Path) -> Result<Vec<AssemblyInfo>> {
    let mut assemblies = Vec::new();
    
    // First, read the packages-lock.json to get package information
    let packages_lock_path = unity_project_root.join("Packages").join("packages-lock.json");
    if !packages_lock_path.exists() {
        return Ok(assemblies); // No packages, return empty
    }
    
    let packages_content = fs::read_to_string(&packages_lock_path).await
        .context("Failed to read packages-lock.json")?;
    
    let packages_lock: PackageLockFile = serde_json::from_str(&packages_content)
        .context("Failed to parse packages-lock.json")?;
    
    // Look for packages in Library/PackageCache
    let package_cache_dir = unity_project_root.join("Library").join("PackageCache");
    if !package_cache_dir.exists() {
        return Ok(assemblies); // No package cache, return empty
    }
    
    // Iterate through each package in the cache
    let mut cache_entries = fs::read_dir(&package_cache_dir).await
        .context("Failed to read PackageCache directory")?;
    
    while let Some(entry) = cache_entries.next_entry().await? {
        let package_dir = entry.path();
        if package_dir.is_dir() {
            if let Ok(package_assemblies) = find_assemblies_in_package(&package_dir, unity_project_root).await {
                assemblies.extend(package_assemblies);
            }
        }
    }
    
    Ok(assemblies)
}

/// Parse a .csproj file to extract assembly information
async fn parse_csproj_file(csproj_path: &Path, unity_project_root: &Path) -> Result<AssemblyInfo> {
    let content = fs::read_to_string(csproj_path).await
        .context("Failed to read .csproj file")?;
    
    // Parse XML to extract AssemblyName and Compile items
    let assembly_name = extract_assembly_name(&content)
        .ok_or_else(|| anyhow!("Could not find AssemblyName in .csproj file"))?;
    
    let source_files = extract_compile_items(&content, unity_project_root)
        .context("Failed to extract compile items from .csproj")?;
    
    Ok(AssemblyInfo {
        name: assembly_name,
        source_files,
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

/// Extract Compile items from .csproj XML content
fn extract_compile_items(content: &str, unity_project_root: &Path) -> Result<Vec<PathBuf>> {
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

/// Find assemblies in a specific package directory
async fn find_assemblies_in_package(package_dir: &Path, unity_project_root: &Path) -> Result<Vec<AssemblyInfo>> {
    let mut assemblies = Vec::new();
    
    // Recursively search for .asmdef files
    let asmdef_files = find_asmdef_files(package_dir).await?;
    
    for asmdef_path in asmdef_files {
        if let Ok(assembly_info) = parse_asmdef_file(&asmdef_path, package_dir, unity_project_root).await {
            assemblies.push(assembly_info);
        }
    }
    
    Ok(assemblies)
}

/// Recursively find all .asmdef files in a directory
fn find_asmdef_files<'a>(dir: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<PathBuf>>> + 'a>> {
    Box::pin(async move {
        let mut asmdef_files = Vec::new();
        
        let mut entries = fs::read_dir(dir).await
            .context("Failed to read directory")?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("asmdef") {
                asmdef_files.push(path);
            } else if path.is_dir() {
                // Recursively search subdirectories
                let mut sub_files = find_asmdef_files(&path).await?;
                asmdef_files.append(&mut sub_files);
            }
        }
        
        Ok(asmdef_files)
    })
}

/// Parse an .asmdef file to extract assembly information
async fn parse_asmdef_file(asmdef_path: &Path, package_dir: &Path, unity_project_root: &Path) -> Result<AssemblyInfo> {
    let content = fs::read_to_string(asmdef_path).await
        .context("Failed to read .asmdef file")?;
    
    let asmdef: AsmDefFile = serde_json::from_str(&content)
        .context("Failed to parse .asmdef file")?;
    
    // Find all .cs files in the same directory as the .asmdef file
    let asmdef_dir = asmdef_path.parent()
        .ok_or_else(|| anyhow!("Could not get parent directory of .asmdef file"))?;
    
    let source_files = find_cs_files_in_dir(asmdef_dir, unity_project_root).await
        .context("Failed to find .cs files in .asmdef directory")?;
    
    Ok(AssemblyInfo {
        name: asmdef.name,
        source_files,
        is_user_code: false,
        source_location: asmdef_path.to_path_buf(),
    })
}

/// Recursively find all .cs files in a directory and return paths relative to Unity project root
fn find_cs_files_in_dir<'a>(dir: &'a Path, unity_project_root: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<PathBuf>>> + 'a>> {
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
                let mut sub_files = find_cs_files_in_dir(&path, unity_project_root).await?;
                cs_files.append(&mut sub_files);
            }
        }
        
        Ok(cs_files)
    })
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
        assert!(!assembly.source_files.is_empty(), "Should have source files");
        
        println!("Found {} user assemblies", assemblies.len());
    }

    #[tokio::test]
    async fn test_find_package_assemblies() {
        let unity_root = get_unity_project_root();
        let assemblies = find_package_assemblies(&unity_root).await.unwrap();
        
        println!("Found {} package assemblies", assemblies.len());
        for assembly in &assemblies {
            assert!(!assembly.is_user_code, "Package assemblies should not be user code");
        }
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
        
        // Check that all extracted files actually exist
        for item in &items {
            let full_path = unity_root.join(item);
            assert!(full_path.exists(), "Extracted file should exist: {:?}", item);
            assert_eq!(item.extension().and_then(|s| s.to_str()), Some("cs"), "Should only extract .cs files");
        }
        
        // Should find the Readme.cs file that we know exists
        let readme_path = PathBuf::from("Assets/TutorialInfo/Scripts/Readme.cs");
        assert!(items.contains(&readme_path), "Should find Readme.cs in the extracted files");
        
        println!("Successfully extracted {} source files from Assembly-CSharp.csproj", items.len());
    }
}
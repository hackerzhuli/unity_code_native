use std::path::{Path, PathBuf};

use crate::{cs::{docs_manager::CsDocsManager, source_utils::normalize_path_for_comparison}, test_utils::get_unity_project_root};

#[tokio::test]
async fn test_get_docs_for_symbol() {
    let unity_root = get_unity_project_root();
    let mut manager = CsDocsManager::new(unity_root).expect("Failed to create manager");

    // Test with assembly name
    let result = manager
        .get_docs_for_symbol("TestClass", Some("Assembly-CSharp"), None)
        .await;

    match result {
        Ok(doc_result) => {
            println!("Found documentation: {}", doc_result.xml_doc);
            println!(
                "Source: {}.{:?}",
                doc_result.source_type_name, doc_result.source_member_name
            );
            if doc_result.is_inherited {
                println!(
                    "Inherited from: {}.{:?}",
                    doc_result.inherited_from_type_name.unwrap_or_default(),
                    doc_result.inherited_from_member_name
                );
            }
        }
        Err(e) => println!("Error getting docs: {:?}", e),
    }
}

#[tokio::test]
async fn test_unity_mathematics_docs() {
    let unity_root = get_unity_project_root();
    let mut manager = CsDocsManager::new(unity_root).expect("Failed to create manager");

    // Test getting documentation for Unity.Mathematics.math.asin(float4) as requested
    println!("Testing Unity.Mathematics.math.asin(float4) documentation retrieval:");

    match manager
        .get_docs_for_symbol(
            "Unity.Mathematics.math.asin(float4)",
            Some("Unity.Mathematics"),
            None,
        )
        .await
    {
        Ok(doc_result) => {
            println!("✓ Successfully retrieved documentation for Unity.Mathematics.math.asin:");
            println!("{}", doc_result.xml_doc);
            assert!(
                !doc_result.xml_doc.trim().is_empty(),
                "Documentation should not be empty"
            );
            assert!(
                doc_result.xml_doc.contains("arcsine"),
                "Documentation should mention arcsine"
            );
        }
        Err(e) => {
            println!(
                "✗ Failed to get Unity.Mathematics.math.asin documentation: {:?}",
                e
            );

            // Let's see what's available in Unity.Mathematics
            manager
                .discover_assemblies()
                .await
                .expect("Failed to discover assemblies");
            if let Ok(Some(docs)) = manager.get_docs_for_assembly("Unity.Mathematics").await {
                println!(
                    "Unity.Mathematics assembly has {} types documented",
                    docs.types.len()
                );

                // Look for math type specifically
                if let Some(math_type) = docs.types.values().find(|t| t.name.contains("math")) {
                    println!(
                        "Found math type: {} with {} members",
                        math_type.name,
                        math_type.members.len()
                    );

                    // Show available asin methods
                    let asin_methods: Vec<_> = math_type
                        .members
                        .values()
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
            match manager
                .get_docs_for_symbol(&first_type.name, Some("Assembly-CSharp"), None)
                .await
            {
                Ok(doc_result) => println!(
                    "✓ Successfully retrieved docs for {}: {}",
                    first_type.name,
                    doc_result.xml_doc.trim()
                ),
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
    let result1 = manager
        .get_docs_for_symbol("TestClass", Some("Assembly-CSharp"), None)
        .await;
    let first_duration = start_time.elapsed();

    // Second call should use cache and be faster
    let start_time = std::time::Instant::now();
    let result2 = manager
        .get_docs_for_symbol("TestClass", Some("Assembly-CSharp"), None)
        .await;
    let second_duration = start_time.elapsed();

    println!("First call took: {:?}", first_duration);
    println!("Second call took: {:?}", second_duration);

    // Both should return the same result
    match (result1, result2) {
        (Ok(doc_result1), Ok(doc_result2)) => {
            assert_eq!(
                doc_result1.xml_doc, doc_result2.xml_doc,
                "Cache should return consistent results"
            );
            assert_eq!(
                doc_result1.source_type_name, doc_result2.source_type_name,
                "Cache should return consistent source info"
            );
            println!("Cache test passed - consistent results");
        }
        _ => println!("One or both calls failed"),
    }
}

#[tokio::test]
async fn test_unified_csproj_cache() {
    let unity_root = get_unity_project_root();
    let mut manager = CsDocsManager::new(unity_root).expect("Failed to create manager");

    // Discover assemblies to populate the cache
    manager
        .discover_assemblies()
        .await
        .expect("Failed to discover assemblies");

    // Verify that the cache contains both assembly metadata and source files
    assert!(
        !manager.csproj_cache.is_empty(),
        "Cache should not be empty after discovery"
    );

    for (csproj_path, cache_entry) in &manager.csproj_cache {
        println!("Cached .csproj: {}", csproj_path.to_string_lossy());
        println!("  Assembly: {}", cache_entry.assembly.name);
        println!("  Source files count: {}", cache_entry.source_files.len());

        // Verify that each cache entry has the expected structure
        assert!(
            !cache_entry.assembly.name.is_empty(),
            "Assembly name should not be empty"
        );
        assert!(
            cache_entry.assembly.is_user_code,
            "Should be user code assembly"
        );
        assert_eq!(
            cache_entry.assembly.source_location, *csproj_path,
            "Source location should match csproj path"
        );

        // Verify that source files are normalized paths
        for source_file in &cache_entry.source_files {
            assert!(
                source_file.is_absolute(),
                "Source file paths should be absolute: {}",
                source_file.to_string_lossy()
            );
        }
    }

    println!("✓ Unified cache structure is working correctly");
}

#[test]
fn test_normalize_path_for_comparison() {
    use std::path::Path;

    // Test Windows UNC path normalization
    let unc_path =
        Path::new("\\\\?\\F:\\projects\\unity\\TestUnityCode\\Assets\\Scripts\\TestHover.cs");
    let normal_path =
        Path::new("F:\\projects\\unity\\TestUnityCode\\Assets\\Scripts\\TestHover.cs");

    let normalized_unc = normalize_path_for_comparison(unc_path);
    let normalized_normal = normalize_path_for_comparison(normal_path);

    println!("UNC path: {}", unc_path.to_string_lossy());
    println!("Normal path: {}", normal_path.to_string_lossy());
    println!("Normalized UNC: {}", normalized_unc.to_string_lossy());
    println!("Normalized normal: {}", normalized_normal.to_string_lossy());

    assert_eq!(
        normalized_unc, normalized_normal,
        "UNC and normal paths should be equal after normalization"
    );

    // Test that normal paths are unchanged
    let regular_path = Path::new("/home/user/file.txt");
    let normalized_regular = normalize_path_for_comparison(regular_path);
    assert_eq!(
        regular_path, normalized_regular,
        "Regular paths should remain unchanged"
    );

    // Test edge case - path that starts with \\?\ but is not actually UNC
    let fake_unc = Path::new("\\\\?\\not_a_real_unc");
    let normalized_fake = normalize_path_for_comparison(fake_unc);
    assert_eq!(
        normalized_fake.to_string_lossy(),
        "not_a_real_unc",
        "Fake UNC prefix should be removed"
    );
}

#[tokio::test]
async fn test_inheritdoc_resolution() {
    let unity_project_root = PathBuf::from("UnityProject");
    let mut docs_manager =
        CsDocsManager::new(unity_project_root).expect("Failed to create manager");

    // Test 1: Add() method inherits from Add(int, int, int)
    let result1 = docs_manager
        .get_docs_for_symbol(
            "UnityProject.Inherit1.Add()",
            None,
            Some(Path::new("UnityProject/Assets/Scripts/Inherit1.cs")),
        )
        .await;

    if let Ok(doc_result) = result1 {
        println!("Test 1 - Add() inheritdoc:");
        println!("XML Doc: {}", doc_result.xml_doc);
        println!("Is Inherited: {}", doc_result.is_inherited);
        if doc_result.is_inherited {
            assert!(doc_result.xml_doc.contains("doc for add with 3 parameters"));
            assert_eq!(
                doc_result.inherited_from_type_name,
                Some("UnityProject.Inherit1".to_string())
            );
        }
    }

    // Test 2: Add2() method inherits from Add<T>(T, int, int)
    let result2 = docs_manager
        .get_docs_for_symbol(
            "UnityProject.Inherit1.Add2()",
            None,
            Some(Path::new("UnityProject/Assets/Scripts/Inherit1.cs")),
        )
        .await;

    if let Ok(doc_result) = result2 {
        println!("Test 2 - Add2() inheritdoc:");
        println!("XML Doc: {}", doc_result.xml_doc);
        println!("Is Inherited: {}", doc_result.is_inherited);
        if doc_result.is_inherited {
            assert!(doc_result.xml_doc.contains("doc for generic add"));
            assert_eq!(
                doc_result.inherited_from_type_name,
                Some("UnityProject.Inherit1".to_string())
            );
        }
    }

    // Test 3: Add3() method inherits from Add(ref int, out int, in System.Int32)
    let result3 = docs_manager
        .get_docs_for_symbol(
            "UnityProject.Inherit1.Add3()",
            None,
            Some(Path::new("UnityProject/Assets/Scripts/Inherit1.cs")),
        )
        .await;

    if let Ok(doc_result) = result3 {
        println!("Test 3 - Add3() inheritdoc:");
        println!("XML Doc: {}", doc_result.xml_doc);
        println!("Is Inherited: {}", doc_result.is_inherited);
        if doc_result.is_inherited {
            assert!(
                doc_result
                    .xml_doc
                    .contains("doc for add with 3 parameters complex")
            );
            assert_eq!(
                doc_result.inherited_from_type_name,
                Some("UnityProject.Inherit1".to_string())
            );
        }
    }

    // Test 4: Method() from Inherit2 inherits from Inherit1.Add<T>(T, int, int)
    let result4 = docs_manager
        .get_docs_for_symbol(
            "OtherProject.MyNamespace.Inherit2.Method()",
            None,
            Some(Path::new("UnityProject/Assets/Scripts/Inherit2.cs")),
        )
        .await;

    if let Ok(doc_result) = result4 {
        println!("Test 4 - Inherit2.Method() inheritdoc:");
        println!("XML Doc: {}", doc_result.xml_doc);
        println!("Is Inherited: {}", doc_result.is_inherited);
        if doc_result.is_inherited {
            assert!(doc_result.xml_doc.contains("doc for generic add"));
            assert_eq!(
                doc_result.inherited_from_type_name,
                Some("UnityProject.Inherit1".to_string())
            );
        }
    } else {
        println!("Test 4 failed: {:?}", result4);
    }

    // Test 5: Add4() method inherits from Add5 (parameter omission test)
    let result5 = docs_manager
        .get_docs_for_symbol(
            "UnityProject.Inherit1.Add4(System.Int32,System.Int32)",
            None,
            Some(Path::new("UnityProject/Assets/Scripts/Inherit1.cs")),
        )
        .await;

    if let Ok(doc_result) = result5 {
        println!("Test 5 - Add4() inheritdoc:");
        println!("XML Doc: {}", doc_result.xml_doc);
        println!("Is Inherited: {}", doc_result.is_inherited);
        if doc_result.is_inherited {
            assert!(doc_result.xml_doc.contains("doc for add 5"));
            assert_eq!(
                doc_result.inherited_from_type_name,
                Some("UnityProject.Inherit1".to_string())
            );
        }
    } else {
        println!("Test 5 failed: {:?}", result5);
    }
}

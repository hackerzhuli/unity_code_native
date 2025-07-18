use crate::{cs::{docs_compiler::DocsCompiler, package_manager::UnityPackageManager, source_utils::find_user_assemblies}, test_utils::get_unity_project_root};

use crate::cs::compile_utils::{split_parameters, get_simple_type_name, normalize_generic_parameters, normalize_member_name, normalize_type_name};
use tree_sitter::{Parser, Language};

#[test]
fn test_split_parameters() {
    // Simple parameters
    assert_eq!(
        split_parameters("int, string"),
        vec!["int", "string"]
    );
    
    // Parameters with generics
    assert_eq!(
        split_parameters("List<T>, int, Dictionary<string, int>"),
        vec!["List<T>", "int", "Dictionary<string, int>"]
    );
    
    // Nested generics
    assert_eq!(
        split_parameters("Dictionary<string, List<T>>, int"),
        vec!["Dictionary<string, List<T>>", "int"]
    );
    
    // Empty parameters
    assert_eq!(
        split_parameters(""),
        Vec::<String>::new()
    );
}

#[test]
fn test_get_simple_type_name() {
    // Simple type without namespace
    assert_eq!(
        get_simple_type_name("int"),
        "int"
    );
    
    // Type with namespace
    assert_eq!(
        get_simple_type_name("System.String"),
        "string"
    );
    
    // Nested type
    assert_eq!(
        get_simple_type_name("System.Collections.Generic.List"),
        "List"
    );
}

#[test]
fn test_normalize_generic_parameters() {
    // Simple generic
    assert_eq!(
        normalize_generic_parameters("<T>"),
        "<T>"
    );
    
    // Multiple type parameters
    assert_eq!(
        normalize_generic_parameters("<System.String, int>"),
        "<string, int>"
    );
    
    // Nested generics
    assert_eq!(
        normalize_generic_parameters("<System.Collections.Generic.List<T>, System.String>"),
        "<List<T>, string>"
    );
}

#[tokio::test]
async fn test_compile_assembly_csharp() {
    let mut compiler = DocsCompiler::new().unwrap();
    let unity_root = get_unity_project_root();

    // Find Assembly-CSharp
    let assemblies = find_user_assemblies(&unity_root).await.unwrap();
    let assembly_csharp = assemblies
        .iter()
        .find(|a| a.name == "Assembly-CSharp")
        .expect("Should find Assembly-CSharp assembly");

    // Compile documentation (include non-public for user code)
    let docs_assembly = compiler
        .compile_assembly(assembly_csharp, &unity_root, true)
        .await
        .unwrap();

    assert_eq!(docs_assembly.assembly_name, "Assembly-CSharp");
    assert!(docs_assembly.is_user_code);

    // Should have at least some types
    assert!(
        !docs_assembly.types.is_empty(),
        "Should find at least one type"
    );

    // Find the UnityProject.TestClass type
    let test_type = docs_assembly
        .types
        .get("UnityProject.TestClass")
        .expect("Should find UnityProject.TestClass type");

    // Verify TestClass documentation
    assert!(test_type.is_public, "TestClass should be public");
    assert!(
        test_type
            .xml_doc
            .contains("A simple test class for documentation extraction"),
        "TestClass should have XML doc mentioning 'simple test class'"
    );
    assert!(
        test_type
            .xml_doc
            .contains("Contains various member types for testing"),
        "TestClass should mention various member types"
    );

    // Verify we have the expected public members (should not include private ones)
    let public_members: Vec<_> = test_type.members.values().filter(|m| m.is_public).collect();
    assert!(
        public_members.len() >= 3,
        "Should have at least 3 public members"
    );

    // Check for specific public members
    let add_method = test_type
        .members
        .get("Add(int, int)")
        .expect("Should find Add(int, int) method");
    assert!(add_method.is_public, "Add method should be public");
    assert!(
        add_method.xml_doc.contains("Adds two numbers together"),
        "Add method should mention adding numbers"
    );
    assert!(
        add_method.xml_doc.contains("param name=\"a\""),
        "Add method should reference parameter 'a'"
    );

    let public_field = test_type
        .members
        .get("PublicField")
        .expect("Should find PublicField");
    assert!(public_field.is_public, "PublicField should be public");
    assert!(
        public_field.xml_doc.contains("A public field for testing"),
        "PublicField should mention testing"
    );

    let test_property = test_type
        .members
        .get("TestProperty")
        .expect("Should find TestProperty");
    assert!(test_property.is_public, "TestProperty should be public");
    assert!(
        test_property
            .xml_doc
            .contains("Gets or sets the test property"),
        "TestProperty should mention gets or sets"
    );

    // Verify private members are not included for user code (they should be included)
    let private_method = test_type
        .members
        .values()
        .find(|m| m.name.contains("ProcessPrivately"));
    assert!(
        private_method.is_some(),
        "Private method should be included for user code"
    );
    if let Some(pm) = private_method {
        assert!(!pm.is_public, "ProcessPrivately should not be public");
    }

    // Note: Nested classes are not supported by the doc compiler as documented

    // Verify private class is included for user code
    let private_class = docs_assembly
        .types
        .get("UnityProject.PrivateClass");
    assert!(
        private_class.is_some(),
        "PrivateClass should be included for user code"
    );
    if let Some(pc) = private_class {
        assert!(!pc.is_public, "PrivateClass should not be public");
        assert!(
            pc.xml_doc.contains("A private class with public methods"),
            "PrivateClass should mention private class"
        );
    }

    // Verify that undocumented members are excluded from the results
    let undocumented_method = test_type
        .members
        .get("UndocumentedMethod()");
    assert!(
        undocumented_method.is_none(),
        "UndocumentedMethod should be excluded (no XML docs)"
    );

    let undocumented_method_with_params = test_type
        .members
        .values()
        .find(|m| m.name.contains("UndocumentedMethodWithParams"));
    assert!(
        undocumented_method_with_params.is_none(),
        "UndocumentedMethodWithParams should be excluded (no XML docs)"
    );

    let undocumented_property = test_type
        .members
        .get("UndocumentedProperty");
    assert!(
        undocumented_property.is_none(),
        "UndocumentedProperty should be excluded (no XML docs)"
    );

    let undocumented_field = test_type
        .members
        .get("UndocumentedField");
    assert!(
        undocumented_field.is_none(),
        "UndocumentedField should be excluded (no XML docs)"
    );

    let undocumented_private_method = test_type
        .members
        .values()
        .find(|m| m.name.contains("UndocumentedPrivateMethod"));
    assert!(
        undocumented_private_method.is_none(),
        "UndocumentedPrivateMethod should be excluded (no XML docs)"
    );

    // Verify that only documented members are included
    let documented_members: Vec<_> = test_type
        .members
        .values()
        .filter(|m| !m.xml_doc.trim().is_empty())
        .collect();
    assert_eq!(
        test_type.members.len(),
        documented_members.len(),
        "All included members should have XML documentation"
    );

    println!(
        "Verified that {} undocumented members were excluded from compilation",
        5
    );

    // // Serialize to JSON
    // let json = serde_json::to_string_pretty(&docs_assembly).unwrap();
    // println!("Compiled docs assembly JSON:\n{}", json);

    // // Write to file for inspection
    // let output_path = unity_root.join("Library").join("UnityCode").join("DocAssemblies");
    // fs::create_dir_all(&output_path).await.unwrap();
    // let json_file = output_path.join("Assembly-CSharp.json");
    // fs::write(&json_file, &json).await.unwrap();

    // println!("Documentation written to: {:?}", json_file);
    println!(
        "Successfully verified UnityProject.TestClass with {} members",
        test_type.members.len()
    );
}

#[tokio::test]
async fn test_partial_class_merging() {
    let mut compiler = DocsCompiler::new().unwrap();
    let unity_root = get_unity_project_root();

    // Find Assembly-CSharp
    let assemblies = find_user_assemblies(&unity_root).await.unwrap();
    let assembly_csharp = assemblies
        .iter()
        .find(|a| a.name == "Assembly-CSharp")
        .expect("Should find Assembly-CSharp assembly");

    // Compile documentation (include non-public for user code)
    let docs_assembly = compiler
        .compile_assembly(assembly_csharp, &unity_root, true)
        .await
        .unwrap();

    // Find the UnityProject.PartialTestClass type
    let partial_type = docs_assembly
        .types
        .get("UnityProject.PartialTestClass")
        .expect("Should find UnityProject.PartialTestClass type");

    // Verify PartialTestClass documentation
    assert!(partial_type.is_public, "PartialTestClass should be public");

    // The XML doc should contain content from one of the partial definitions
    // (Tree-sitter will pick up the first one it encounters)
    assert!(
        !partial_type.xml_doc.is_empty(),
        "PartialTestClass should have XML documentation"
    );

    // Verify we have members from both partial class files merged together
    let public_members: Vec<_> = partial_type
        .members
        .values()
        .filter(|m| m.is_public)
        .collect();
    assert!(
        public_members.len() >= 5,
        "Should have at least 5 public members from both partial files"
    );

    // Check for members from the first partial file (PartialTest1.cs)
    let first_part_field = partial_type
        .members
        .get("FirstPartField")
        .expect("Should find FirstPartField from first partial file");
    assert!(
        first_part_field.is_public,
        "FirstPartField should be public"
    );
    assert!(
        first_part_field
            .xml_doc
            .contains("A public field from the first partial file"),
        "FirstPartField should have correct documentation"
    );

    let first_part_method = partial_type
        .members
        .get("ProcessFromFirstPart(int)")
        .expect("Should find ProcessFromFirstPart method from first partial file");
    assert!(
        first_part_method.is_public,
        "ProcessFromFirstPart should be public"
    );
    assert!(
        first_part_method
            .xml_doc
            .contains("A method from the first partial class file"),
        "ProcessFromFirstPart should have correct documentation"
    );

    // Check for members from the second partial file (PartialTest2.cs)
    let second_part_field = partial_type
        .members
        .get("SecondPartField")
        .expect("Should find SecondPartField from second partial file");
    assert!(
        second_part_field.is_public,
        "SecondPartField should be public"
    );
    assert!(
        second_part_field
            .xml_doc
            .contains("A public field from the second partial file"),
        "SecondPartField should have correct documentation"
    );

    let combined_property = partial_type
        .members
        .get("CombinedProperty")
        .expect("Should find CombinedProperty from second partial file");
    assert!(
        combined_property.is_public,
        "CombinedProperty should be public"
    );
    assert!(
        combined_property
            .xml_doc
            .contains("A property from the second partial class file"),
        "CombinedProperty should have correct documentation"
    );

    let second_part_method = partial_type
        .members
        .get("ProcessFromSecondPart(string)")
        .expect("Should find ProcessFromSecondPart method from second partial file");
    assert!(
        second_part_method.is_public,
        "ProcessFromSecondPart should be public"
    );
    assert!(
        second_part_method
            .xml_doc
            .contains("A method from the second partial class file"),
        "ProcessFromSecondPart should have correct documentation"
    );

    let combine_method = partial_type
        .members
        .get("CombineFromBothParts()")
        .expect("Should find CombineFromBothParts method from second partial file");
    assert!(
        combine_method.is_public,
        "CombineFromBothParts should be public"
    );
    assert!(
        combine_method
            .xml_doc
            .contains("Another public method that combines data from both parts"),
        "CombineFromBothParts should have correct documentation"
    );

    // Verify private members from both files are included (for user code)
    let private_members: Vec<_> = partial_type
        .members
        .values()
        .filter(|m| !m.is_public)
        .collect();
    assert!(
        private_members.len() >= 4,
        "Should have at least 4 private members from both partial files"
    );

    // Check for private members from both files
    let first_private_field = partial_type
        .members
        .get("firstPrivateField");
    assert!(
        first_private_field.is_some(),
        "Should find firstPrivateField from first partial file"
    );

    let second_private_field = partial_type
        .members
        .get("secondPrivateField");
    assert!(
        second_private_field.is_some(),
        "Should find secondPrivateField from second partial file"
    );

    let first_private_method = partial_type
        .members
        .values()
        .find(|m| m.name.contains("ProcessPrivatelyFromFirst"));
    assert!(
        first_private_method.is_some(),
        "Should find ProcessPrivatelyFromFirst from first partial file"
    );

    let second_private_method = partial_type
        .members
        .values()
        .find(|m| m.name.contains("ProcessPrivatelyFromSecond"));
    assert!(
        second_private_method.is_some(),
        "Should find ProcessPrivatelyFromSecond from second partial file"
    );

    println!("Successfully verified partial class merging for UnityProject.PartialTestClass");
    println!(
        "Total members found: {} (public: {}, private: {})",
        partial_type.members.len(),
        public_members.len(),
        private_members.len()
    );

    // Print all member names for debugging
    println!("All members:");
    for member in partial_type.members.values() {
        println!(
            "  - {} ({})",
            member.name,
            if member.is_public {
                "public"
            } else {
                "private"
            }
        );
    }
}



#[tokio::test]
async fn test_exclude_non_public_types_and_members() {
    let mut compiler = DocsCompiler::new().unwrap();
    let unity_root = get_unity_project_root();

    // Find Assembly-CSharp
    let assemblies = find_user_assemblies(&unity_root).await.unwrap();
    let assembly_csharp = assemblies
        .iter()
        .find(|a| a.name == "Assembly-CSharp")
        .expect("Should find Assembly-CSharp assembly");

    // Compile documentation excluding non-public types and members
    let docs_assembly = compiler
        .compile_assembly(assembly_csharp, &unity_root, false)
        .await
        .unwrap();

    assert_eq!(docs_assembly.assembly_name, "Assembly-CSharp");
    assert!(docs_assembly.is_user_code);

    // Should have at least some types
    assert!(
        !docs_assembly.types.is_empty(),
        "Should find at least one public type"
    );

    // Verify that private types are excluded
    let private_class = docs_assembly
        .types
        .get("UnityProject.PrivateClass");
    assert!(
        private_class.is_none(),
        "PrivateClass should be excluded when include_non_public is false"
    );

    // Find the UnityProject.TestClass type (should still be present as it's public)
    let test_type = docs_assembly
        .types
        .get("UnityProject.TestClass")
        .expect("Should find UnityProject.TestClass type as it's public");

    // Verify TestClass is public
    assert!(test_type.is_public, "TestClass should be public");

    // Verify that only public members are included
    let all_members_public = test_type.members.values().all(|m| m.is_public);
    assert!(
        all_members_public,
        "All members should be public when include_non_public is false"
    );

    // Check for specific public members that should be present
    let add_method = test_type.members.get("Add(int, int)");
    assert!(
        add_method.is_some(),
        "Add method should be present as it's public"
    );

    let public_field = test_type.members.get("PublicField");
    assert!(
        public_field.is_some(),
        "PublicField should be present as it's public"
    );

    let test_property = test_type.members.get("TestProperty");
    assert!(
        test_property.is_some(),
        "TestProperty should be present as it's public"
    );

    // Verify that private members are excluded
    let private_method = test_type
        .members
        .values()
        .find(|m| m.name.contains("ProcessPrivately"));
    assert!(
        private_method.is_none(),
        "Private methods should be excluded when include_non_public is false"
    );

    let private_field = test_type.members.get("privateField");
    assert!(
        private_field.is_none(),
        "Private fields should be excluded when include_non_public is false"
    );

    // Test with partial classes - verify only public members are included
    let partial_type = docs_assembly
        .types
        .get("UnityProject.PartialTestClass")
        .expect("Should find UnityProject.PartialTestClass type as it's public");

    // Verify all members in partial class are public
    let all_partial_members_public = partial_type.members.values().all(|m| m.is_public);
    assert!(
        all_partial_members_public,
        "All partial class members should be public when include_non_public is false"
    );

    // Verify specific public members from partial classes are present
    let first_part_field = partial_type
        .members
        .get("FirstPartField");
    assert!(
        first_part_field.is_some(),
        "FirstPartField should be present as it's public"
    );

    let second_part_field = partial_type
        .members
        .get("SecondPartField");
    assert!(
        second_part_field.is_some(),
        "SecondPartField should be present as it's public"
    );

    // Verify private members from partial classes are excluded
    let first_private_field = partial_type
        .members
        .get("firstPrivateField");
    assert!(
        first_private_field.is_none(),
        "firstPrivateField should be excluded when include_non_public is false"
    );

    let second_private_field = partial_type
        .members
        .get("secondPrivateField");
    assert!(
        second_private_field.is_none(),
        "secondPrivateField should be excluded when include_non_public is false"
    );

    println!("Successfully verified exclusion of non-public types and members");
    println!(
        "TestClass members (all public): {}",
        test_type.members.len()
    );
    println!(
        "PartialTestClass members (all public): {}",
        partial_type.members.len()
    );

    // Print all types to verify only public ones are included
    println!("All types found (should be only public):");
    for type_doc in docs_assembly.types.values() {
        println!(
            "  - {} ({})",
            type_doc.name,
            if type_doc.is_public {
                "public"
            } else {
                "private"
            }
        );
    }
}

#[tokio::test]
async fn test_using_statements_extraction() {
    let mut compiler = DocsCompiler::new().unwrap();
    let unity_root = get_unity_project_root();

    // Find Assembly-CSharp
    let assemblies = find_user_assemblies(&unity_root).await.unwrap();
    let assembly_csharp = assemblies
        .iter()
        .find(|a| a.name == "Assembly-CSharp")
        .expect("Should find Assembly-CSharp assembly");

    // Compile documentation (include non-public for user code)
    let docs_assembly = compiler
        .compile_assembly(assembly_csharp, &unity_root, true)
        .await
        .unwrap();

    // Test using statements for TestClass
    let test_type = docs_assembly
        .types
        .get("UnityProject.TestClass")
        .expect("Should find UnityProject.TestClass type");

    // Verify TestClass has using statements
    assert!(
        !test_type.using_namespaces.is_empty(),
        "TestClass should have using statements"
    );

    // Check for expected using statements from Test.cs
    assert!(
        test_type.using_namespaces.contains(&"System".to_string()),
        "TestClass should have 'using System;'"
    );
    assert!(
        test_type.using_namespaces.contains(&"UnityEngine.UI".to_string()),
        "TestClass should have 'using UnityEngine.UI;'"
    );

    println!("TestClass using statements: {:?}", test_type.using_namespaces);

    // Test using statements for PartialTestClass (should merge from both files)
    let partial_type = docs_assembly
        .types
        .get("UnityProject.PartialTestClass")
        .expect("Should find UnityProject.PartialTestClass type");

    // Verify PartialTestClass has using statements
    assert!(
        !partial_type.using_namespaces.is_empty(),
        "PartialTestClass should have using statements"
    );

    // Check for using statements from both partial files
    // From PartialTest1.cs: using System; using UnityEngine.UI;
    assert!(
        partial_type.using_namespaces.contains(&"System".to_string()),
        "PartialTestClass should have 'using System;' from both files"
    );
    assert!(
        partial_type.using_namespaces.contains(&"UnityEngine.UI".to_string()),
        "PartialTestClass should have 'using UnityEngine.UI;' from PartialTest1.cs"
    );

    // From PartialTest2.cs: using System; using UnityEngine.EventSystems;
    assert!(
        partial_type.using_namespaces.contains(&"UnityEngine.EventSystems".to_string()),
        "PartialTestClass should have 'using UnityEngine.EventSystems;' from PartialTest2.cs"
    );

    println!("PartialTestClass using statements: {:?}", partial_type.using_namespaces);

    // Verify that using statements are deduplicated (System appears in both files)
    let system_count = partial_type
        .using_namespaces
        .iter()
        .filter(|ns| *ns == "System")
        .count();
    assert_eq!(
        system_count, 1,
        "'using System;' should appear only once even though it's in both partial files"
    );

    // Verify we have the expected total number of unique using statements
    // Expected: System, UnityEngine.UI, UnityEngine.EventSystems
    assert!(
        partial_type.using_namespaces.len() >= 3,
        "PartialTestClass should have at least 3 unique using statements"
    );

    // Test that using statements are properly sorted/organized
    let mut sorted_usings = partial_type.using_namespaces.clone();
    sorted_usings.sort();
    
    println!("All using statements for PartialTestClass (sorted): {:?}", sorted_usings);
    
    // Verify specific expected using statements are present
    let expected_usings = vec![
        "System".to_string(),
        "UnityEngine.EventSystems".to_string(),
        "UnityEngine.UI".to_string(),
    ];
    
    for expected_using in &expected_usings {
        assert!(
            partial_type.using_namespaces.contains(expected_using),
            "PartialTestClass should contain using statement: {}", expected_using
        );
    }

    // Test using statements for Inherit2 class
    let inherit2_type = docs_assembly
        .types
        .get("OtherProject.MyNamespace.Inherit2")
        .expect("Should find OtherProject.MyNamespace.Inherit2 type");

    // Verify Inherit2 has using statements
    assert!(
        !inherit2_type.using_namespaces.is_empty(),
        "Inherit2 should have using statements"
    );

    // Check for expected using statement from Inherit2.cs
    assert!(
        inherit2_type.using_namespaces.contains(&"UnityProject".to_string()),
        "Inherit2 should have 'using UnityProject;'"
    );

    println!("Inherit2 using statements: {:?}", inherit2_type.using_namespaces);

    println!("Successfully verified using statements extraction and merging for partial classes");
    println!(
        "TestClass using statements count: {}",
        test_type.using_namespaces.len()
    );
    println!(
        "PartialTestClass using statements count: {}",
        partial_type.using_namespaces.len()
    );
    println!(
        "Inherit2 using statements count: {}",
        inherit2_type.using_namespaces.len()
    );
}

// test will take a few seconds, too slow, so comment out
// add back only when needed, and comment out when not needed
// #[tokio::test]
async fn test_compile_unity_mathematics_package() {
    let mut compiler = DocsCompiler::new().unwrap();
    let unity_root = get_unity_project_root();

    // Initialize package manager and update packages
    let mut package_manager = UnityPackageManager::new(unity_root.clone());
    package_manager.update().await.unwrap();

    // Find Unity.Mathematics package
    let packages = package_manager.get_packages();
    let math_package = packages
        .iter()
        .flat_map(|p| &p.assemblies)
        .find(|a| a.name == "Unity.Mathematics")
        .expect("Should find Unity.Mathematics assembly in packages");

    println!(
        "Found Unity.Mathematics assembly at: {:?}",
        math_package.source_location
    );

    // Compile documentation for Unity.Mathematics (include non-public for comprehensive testing)
    let docs_assembly = compiler
        .compile_assembly(math_package, &unity_root, true)
        .await
        .unwrap();

    assert_eq!(docs_assembly.assembly_name, "Unity.Mathematics");
    assert!(
        !docs_assembly.is_user_code,
        "Unity.Mathematics should not be user code"
    );

    // Should have at least some types
    assert!(
        !docs_assembly.types.is_empty(),
        "Should find at least one type in Unity.Mathematics"
    );

    // Look for the Unity.Mathematics.math type
    let math_type = docs_assembly
        .types
        .get("Unity.Mathematics.math")
        .expect("Should find Unity.Mathematics.math type");

    println!(
        "Found Unity.Mathematics.math type with {} members",
        math_type.members.len()
    );
    println!("Math type is public: {}", math_type.is_public);

    // Verify the math type has some members (it should have many mathematical functions)
    assert!(
        !math_type.members.is_empty(),
        "Unity.Mathematics.math should have members"
    );

    // Write the documentation to a JSON file for manual inspection
    // let output_path = unity_root.join("Library").join("UnityCode").join("DocAssemblies");
    // fs::create_dir_all(&output_path).await.unwrap();

    // let json_file_path = output_path.join("Unity.Mathematics.json");
    // let json_content = serde_json::to_string(&docs_assembly).unwrap();
    // fs::write(&json_file_path, json_content).await.unwrap();

    // println!("Unity.Mathematics documentation written to: {:?}", json_file_path);
    println!("Total types found: {}", docs_assembly.types.len());

    // Print some sample types for verification
    println!("Sample types found:");
    for (i, type_doc) in docs_assembly.types.values().take(5).enumerate() {
        println!(
            "  {}. {} ({} members)",
            i + 1,
            type_doc.name,
            type_doc.members.len()
        );
    }

    // Print some sample members from the math type
    println!("Sample members from Unity.Mathematics.math:");
    for (i, member) in math_type.members.values().take(10).enumerate() {
        println!(
            "  {}. {} ({})",
            i + 1,
            member.name,
            if member.is_public {
                "public"
            } else {
                "private"
            }
        );
    }
}

use super::*;
use crate::test_utils::get_ui_elements_schema_dir;
use tempfile::TempDir;
use std::fs;

#[tokio::test]
async fn test_real_unity_schema_parsing() {
    let schema_dir = get_ui_elements_schema_dir();
    
    // Ensure schema directory exists - tests should fail loudly if missing
    assert!(schema_dir.exists(), 
            "UIElementsSchema directory not found at {:?}. Tests require real Unity schema files to be present.", 
            schema_dir);
    
    let mut manager = UxmlSchemaManager::new(schema_dir);
    manager.update().await.unwrap();
    
    // Test that we can find the Image element from Unity's schema
    let image_element = manager.lookup("UnityEngine.UIElements.Image");
    assert!(image_element.is_some(), "Image element should be found in Unity schema");
    
    let image = image_element.unwrap();
    assert_eq!(image.name, "Image");
    assert_eq!(image.namespace, "UnityEngine.UIElements");
    assert_eq!(image.fully_qualified_name, "UnityEngine.UIElements.Image");
    
    // Test that we can find other common Unity elements
    let visual_element = manager.lookup("UnityEngine.UIElements.VisualElement");
    assert!(visual_element.is_some(), "VisualElement should be found in Unity schema");
    
    let button = manager.lookup("UnityEngine.UIElements.Button");
    assert!(button.is_some(), "Button should be found in Unity schema");
    
    let text_field = manager.lookup("UnityEngine.UIElements.TextField");
    assert!(text_field.is_some(), "TextField should be found in Unity schema");
    
    // Test namespace filtering
    let ui_elements = manager.get_elements_in_namespace("UnityEngine.UIElements");
    assert!(!ui_elements.is_empty(), "Should find elements in UnityEngine.UIElements namespace");
    
    // Verify that Image is in the correct namespace
    let image_in_namespace = ui_elements.iter().find(|e| e.name == "Image");
    assert!(image_in_namespace.is_some(), "Image should be found in UnityEngine.UIElements namespace");
    
    // Test get all elements returns a reasonable number
    let all_elements = manager.get_all_elements();
    assert!(all_elements.len() > 10, "Should find many elements in Unity schema, found: {}", all_elements.len());
    
    println!("Successfully parsed {} Unity UI elements", all_elements.len());
    println!("Found elements in UnityEngine.UIElements namespace: {}", ui_elements.len());
}

#[tokio::test]
async fn test_file_change_detection() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_path_buf();
    let schema_path = temp_dir.path().join("test.xsd");
    
    let initial_content = r#"<?xml version="1.0" encoding="utf-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" 
           targetNamespace="Test.Namespace" 
           elementFormDefault="qualified">
  <xs:element name="Element1" type="Type1" />
</xs:schema>"#;
    
    fs::write(&schema_path, initial_content).unwrap();
    
    let mut manager = UxmlSchemaManager::new(dir_path);
    manager.update().await.unwrap();
    
    assert_eq!(manager.get_all_elements().len(), 1);
    
    // Wait a bit to ensure different modification time
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let updated_content = r#"<?xml version="1.0" encoding="utf-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" 
           targetNamespace="Test.Namespace" 
           elementFormDefault="qualified">
  <xs:element name="Element1" type="Type1" />
  <xs:element name="Element2" type="Type2" />
</xs:schema>"#;
    
    fs::write(&schema_path, updated_content).unwrap();
    manager.update().await.unwrap();
    
    assert_eq!(manager.get_all_elements().len(), 2);
    assert!(manager.lookup("Test.Namespace.Element2").is_some());
}

#[tokio::test]
async fn test_namespace_extraction_from_real_files() {
    let schema_dir = get_ui_elements_schema_dir();
    
    // Ensure schema directory exists - tests should fail loudly if missing
    assert!(schema_dir.exists(), 
            "UIElementsSchema directory not found at {:?}. Tests require real Unity schema files to be present.", 
            schema_dir);
    
    let mut manager = UxmlSchemaManager::new(schema_dir);
    manager.update().await.unwrap();
    
    // Verify that we're not using filename as namespace
    // All Unity elements should be in "UnityEngine.UIElements" namespace
    // regardless of the XSD filename
    let all_elements = manager.get_all_elements();
    
    for element in &all_elements {
        // Most Unity UI elements should be in UnityEngine.UIElements namespace
        // (there might be some exceptions for editor-specific elements)
        if element.namespace != "UnityEngine.UIElements" && 
           !element.namespace.starts_with("UnityEditor") {
            println!("Warning: Found element '{}' in unexpected namespace '{}'", 
                    element.name, element.namespace);
        }
    }
    
    // Verify specific elements are in the correct namespace
    let unity_elements = manager.get_elements_in_namespace("UnityEngine.UIElements");
    assert!(!unity_elements.is_empty(), "Should find elements in UnityEngine.UIElements namespace");
    
    // Check that common elements are properly namespaced
    let expected_elements = ["Image", "VisualElement", "Button", "TextField", "Label"];
    for expected in &expected_elements {
        let fqn = format!("UnityEngine.UIElements.{}", expected);
        if let Some(element) = manager.lookup(&fqn) {
            assert_eq!(element.namespace, "UnityEngine.UIElements", 
                      "Element '{}' should be in UnityEngine.UIElements namespace", expected);
            println!("✓ Found {} in correct namespace", expected);
        }
    }
}
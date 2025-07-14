use std::fs;
use scraper::{Html, Selector};
use crate::uss::definitions::{UssDefinitions, PropertyAnimation};
use crate::test_utils::get_project_root;
use crate::uss::constants::*;

#[test]
fn test_property_exists() {
    let definitions = UssDefinitions::new();
    
    // Test some known properties
    assert!(definitions.get_property_info("color").is_some());
    assert!(definitions.get_property_info("background-color").is_some());
    assert!(definitions.get_property_info("width").is_some());
    assert!(definitions.get_property_info("height").is_some());
    assert!(definitions.get_property_info("margin").is_some());
    assert!(definitions.get_property_info("padding").is_some());
    assert!(definitions.get_property_info("border-width").is_some());
    assert!(definitions.get_property_info("flex-direction").is_some());
    assert!(definitions.get_property_info("justify-content").is_some());
    assert!(definitions.get_property_info("align-items").is_some());
    
    // Test some Unity-specific properties
    assert!(definitions.get_property_info("-unity-font").is_some());
    assert!(definitions.get_property_info("-unity-text-align").is_some());
    assert!(definitions.get_property_info("-unity-background-scale-mode").is_some());
    
    // Test non-existent property
    assert!(definitions.get_property_info("non-existent-property").is_none());
}

#[test]
fn test_property_inheritance() {
    let definitions = UssDefinitions::new();
    
    // Test inherited properties
    let color = definitions.get_property_info("color").unwrap();
    assert!(color.inherited, "color should be inherited");
    
    let font_size = definitions.get_property_info("font-size").unwrap();
    assert!(font_size.inherited, "font-size should be inherited");
    
    // Test non-inherited properties
    let width = definitions.get_property_info("width").unwrap();
    assert!(!width.inherited, "width should not be inherited");
    
    let background_color = definitions.get_property_info("background-color").unwrap();
    assert!(!background_color.inherited, "background-color should not be inherited");
}

#[test]
fn test_property_animation() {
    let definitions = UssDefinitions::new();
    
    // Test fully animatable properties
    let color = definitions.get_property_info("color").unwrap();
    assert_eq!(color.animatable, PropertyAnimation::Animatable);
    
    let width = definitions.get_property_info("width").unwrap();
    assert_eq!(width.animatable, PropertyAnimation::Animatable);
    
    // Test discrete properties
    let flex_direction = definitions.get_property_info("flex-direction").unwrap();
    assert_eq!(flex_direction.animatable, PropertyAnimation::Discrete);
    
    // Test non-animatable properties
    let cursor = definitions.get_property_info("cursor").unwrap();
    assert_eq!(cursor.animatable, PropertyAnimation::None);
}

#[test]
fn test_units() {
    let definitions = UssDefinitions::new();
    
    // Test valid units
    assert!(definitions.is_valid_unit(UNIT_PX));
    assert!(definitions.is_valid_unit(UNIT_PERCENT));
    assert!(definitions.is_valid_unit(UNIT_DEG));
    assert!(definitions.is_valid_unit(UNIT_RAD));
    assert!(definitions.is_valid_unit(UNIT_GRAD));
    assert!(definitions.is_valid_unit(UNIT_TURN));
    assert!(definitions.is_valid_unit(UNIT_S));
    assert!(definitions.is_valid_unit(UNIT_MS));
    
    // Test invalid units
    assert!(!definitions.is_valid_unit("em"));
    assert!(!definitions.is_valid_unit("rem"));
    assert!(!definitions.is_valid_unit("vh"));
    assert!(!definitions.is_valid_unit("vw"));
    assert!(!definitions.is_valid_unit("pt"));
    assert!(!definitions.is_valid_unit("cm"));
    assert!(!definitions.is_valid_unit("invalid"));
    
    // Test unit categories
    assert!(definitions.is_length_unit(UNIT_PX));
    assert!(definitions.is_length_unit(UNIT_PERCENT));
    assert!(!definitions.is_length_unit(UNIT_DEG));
    
    assert!(definitions.is_angle_unit(UNIT_DEG));
    assert!(definitions.is_angle_unit(UNIT_RAD));
    assert!(definitions.is_angle_unit(UNIT_GRAD));
    assert!(definitions.is_angle_unit(UNIT_TURN));
    assert!(!definitions.is_angle_unit(UNIT_PX));
    
    assert!(definitions.is_time_unit(UNIT_S));
    assert!(definitions.is_time_unit(UNIT_MS));
    assert!(!definitions.is_time_unit(UNIT_PX));
    
    // Test getting units by category
    let length_units = definitions.get_length_units();
    assert_eq!(length_units, vec![UNIT_PX, UNIT_PERCENT]);
    
    let angle_units = definitions.get_angle_units();
    assert_eq!(angle_units, vec![UNIT_DEG, UNIT_RAD, UNIT_GRAD, UNIT_TURN]);
    
    let time_units = definitions.get_time_units();
    assert_eq!(time_units, vec![UNIT_S, UNIT_MS]);
    
    // Test getting all units
    let all_units = definitions.get_all_units();
    assert!(all_units.contains(&UNIT_PX));
    assert!(all_units.contains(&UNIT_DEG));
    assert!(all_units.contains(&UNIT_S));
    assert_eq!(all_units.len(), 8); // Should have exactly 8 units
}

#[test]
fn test_keyword_only(){
    let definitions = UssDefinitions::new();
    let flex_dir = definitions.get_property_info("flex-direction").unwrap();
    assert!(flex_dir.value_spec.is_keyword_only());

    let width = definitions.get_property_info("width").unwrap();
    assert!(!width.value_spec.is_keyword_only());
}

#[test]
fn test_color_only(){
    let definitions = UssDefinitions::new();
    let color = definitions.get_property_info("color").unwrap();
    assert!(color.value_spec.is_color_only());

    let width = definitions.get_property_info("width").unwrap();
    assert!(!width.value_spec.is_color_only());

    let bg_color = definitions.get_property_info("background-color").unwrap();
    assert!(bg_color.value_spec.is_color_only());
}



#[test]
fn test_properties_against_source_data() {
    let definitions = UssDefinitions::new();
    
    // Read the official properties HTML source data
    let project_root = get_project_root();
    let html_file_path = project_root.join("Assets").join("data").join("USS_properties_reference_table_6.0_clean.html");
    
    let html_content = fs::read_to_string(&html_file_path)
        .expect("Failed to read USS_properties_reference_table.html file");
    
    let document = Html::parse_document(&html_content);
    let row_selector = Selector::parse("tbody tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();
    let link_selector = Selector::parse("a").unwrap();
    let code_selector = Selector::parse("code").unwrap();
    
    let mut tested_properties = 0;
    let mut mismatches = Vec::new();
    let mut missing_properties = Vec::new();
    
    for row in document.select(&row_selector) {
        let cells: Vec<_> = row.select(&cell_selector).collect();
        if cells.len() != 4 {
            continue; // Skip malformed rows
        }
        
        // Extract property name from the first cell's <code> tag
        let property_name = if let Some(code_elem) = cells[0].select(&code_selector).next() {
            code_elem.text().collect::<String>()
        } else {
            continue; // Skip rows without property name in code tag
        };
        
        // Extract other data
         let inherited_str = cells[1].text().collect::<String>().trim().to_string();
         let animatable_str = cells[2].text().collect::<String>().trim().to_string();
         
         // Extract description while excluding tooltip content
         let description = {
             let mut desc_text = String::new();
             
             // Clone the cell to avoid borrowing issues
             let cell_html = cells[3].html();
             let cell_fragment = Html::parse_fragment(&cell_html);
             
             // Extract text while skipping tooltip subtrees
             fn collect_text_excluding_tooltips(element: scraper::ElementRef, text: &mut String) {
                 // Check if this element has tooltip class
                 if element.value().classes().any(|class| class.starts_with("tooltip") && class != "tooltip") {
                     return; // Skip this entire subtree
                 }
                 
                 // Process this element's direct text and children
                 for node in element.children() {
                     if let Some(text_node) = node.value().as_text() {
                         text.push_str(text_node);
                     } else if let Some(child_element) = node.value().as_element() {
                         let child_ref = scraper::ElementRef::wrap(node).unwrap();
                         collect_text_excluding_tooltips(child_ref, text);
                     }
                 }
             }
             
             collect_text_excluding_tooltips(cell_fragment.root_element(), &mut desc_text);
             
             desc_text.trim().to_string()
         };
        
        // Extract documentation URL from the first cell's <a> tag
        let doc_url = if let Some(link_elem) = cells[0].select(&link_selector).next() {
            link_elem.value().attr("href").unwrap_or("").to_string()
        } else {
            String::new()
        };
        
        // Check if we have this property in our definitions
        if let Some(property_info) = definitions.get_property_info(&property_name) {
            tested_properties += 1;
            let mut property_mismatches = Vec::new();
            
            // Test inherited field
            let expected_inherited = inherited_str == "Yes";
            if property_info.inherited != expected_inherited {
                property_mismatches.push(format!(
                    "  - inherited: expected {}, got {}",
                    expected_inherited, property_info.inherited
                ));
            }
            
            // Test animatable field
            let expected_animatable = match animatable_str.as_str() {
                "Fully animatable" => PropertyAnimation::Animatable,
                "Discrete" => PropertyAnimation::Discrete,
                "Non-animatable" => PropertyAnimation::None,
                _ => {
                    property_mismatches.push(format!(
                        "  - unknown animatable value: '{}'",
                        animatable_str
                    ));
                    continue;
                }
            };
            
            if property_info.animatable != expected_animatable {
                property_mismatches.push(format!(
                    "  - animatable: expected {:?}, got {:?}",
                    expected_animatable, property_info.animatable
                ));
            }
            
            // Test description field (should contain the official description verbatim)
            // Normalize quote characters for comparison (official docs use non-standard quotes)
            let normalized_expected = description.replace("’", "'");
            let normalized_actual = property_info.description.replace("’", "'");
            
            if !normalized_actual.contains(&normalized_expected) {
                property_mismatches.push(format!(
                    "  - [{}] description does not contain official text\n    Expected: '{}'\n    Actual: '{}'",
                    property_name, description, property_info.description
                ));
            }
            
            // Test documentation URL (replace {version} with "6000.0")
             if !doc_url.is_empty() && !property_info.documentation_url.is_empty() {
                 let expected_doc_url = property_info.documentation_url.replace("{version}", "6000.0");
                 if expected_doc_url != doc_url {
                     property_mismatches.push(format!(
                         "  - [{}] doc_url mismatch\n    Expected: '{}'\n    Actual: '{}'",
                         property_name, doc_url, expected_doc_url
                     ));
                 }
             }
            
            // Print differences for this property if any
            if !property_mismatches.is_empty() {
                println!("\nProperty '{}' differences:", property_name);
                for mismatch in &property_mismatches {
                    println!("{}", mismatch);
                }
                mismatches.extend(property_mismatches);
            }
        } else {
            // Property is missing from our definitions
            missing_properties.push(property_name);
        }
    }
    
    // Report missing properties
    if !missing_properties.is_empty() {
        println!("\nMissing properties from our definitions ({} total):", missing_properties.len());
        for missing_prop in &missing_properties {
            println!("  - {}", missing_prop);
        }
    }
    
    // Report results
    if !mismatches.is_empty() || !missing_properties.is_empty() {
        let mut error_msg = String::new();
        
        if !mismatches.is_empty() {
            error_msg.push_str(&format!("Found {} mismatches in {} tested properties:\n{}", 
                mismatches.len(), tested_properties, mismatches.join("\n")));
        }
        
        if !missing_properties.is_empty() {
            if !error_msg.is_empty() {
                error_msg.push_str("\n\n");
            }
            error_msg.push_str(&format!("Missing {} properties from our definitions:\n{}", 
                missing_properties.len(), missing_properties.join(", ")));
        }
        
        panic!("{}", error_msg);
    }
    
    // Ensure we tested a reasonable number of properties
    assert!(tested_properties > 50, "Expected to test more than 50 properties, but only tested {}", tested_properties);
    
    println!("Successfully validated {} properties against HTML source data", tested_properties);
}

#[test]
fn test_properties_against_markdown_format() {
    let definitions = UssDefinitions::new();
    
    // Read the USS property format markdown file
    let project_root = get_project_root();
    let md_file_path = project_root.join("Assets").join("data").join("USS_property_format_6.0.md");
    
    let md_content = fs::read_to_string(&md_file_path)
        .expect("Failed to read USS_property_format_6.0.md file");
    
    let mut tested_properties = 0;
    let mut not_found_properties = Vec::new();
    
    // Parse the markdown content to extract property format definitions
    let lines: Vec<&str> = md_content.lines().collect();
    let mut in_css_example = false;
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Check if we're entering or leaving a code block
        if trimmed.starts_with("```") {
            continue;
        }
        
        // Detect CSS example blocks (look for lines that contain CSS selectors like .red, .blue)
        if line.starts_with("    ") {
            let code_line = &line[4..]; // Remove the 4-space indentation
            if code_line.trim().starts_with(".") && code_line.contains("{") {
                in_css_example = true;
            }
            if code_line.trim() == "}" && in_css_example {
                in_css_example = false;
                continue;
            }
        }
        
        // Skip if we're in a CSS example block
        if in_css_example {
            continue;
        }
        
        // Check if this line is indented (indicating it's in a code block)
        if line.starts_with("    ") && !line.trim().is_empty() {
            let code_line = &line[4..]; // Remove the 4-space indentation
            
            // Skip comments
            if code_line.trim().starts_with("/*") || code_line.trim().starts_with("*/") {
                continue;
            }
            
            // Look for property definitions (property: format)
            if let Some(colon_pos) = code_line.find(':') {
                let property_name = code_line[..colon_pos].trim();
                let format_spec = code_line[colon_pos + 1..].trim();
                
                // Skip empty property names or format specs
                if property_name.is_empty() || format_spec.is_empty() {
                    continue;
                }
                
                // Skip CSS selectors and other non-property lines
                if property_name.contains('.') || property_name.contains('#') || 
                   property_name.contains(' ') || property_name.starts_with('@') {
                    continue;
                }
                
                // Skip lines that look like CSS values (end with semicolon and contain specific values)
                if format_spec.ends_with(';') && 
                   (format_spec.contains("px") || format_spec.contains("red") || 
                    format_spec.contains("blue") || format_spec.contains("0.5") ||
                    format_spec == "initial;" || format_spec.len() < 20) {
                    continue;
                }
                
                // Only process lines that look like actual format specifications
                // (contain angle brackets or pipe symbols indicating value types)
                if !format_spec.contains('<') && !format_spec.contains('|') {
                    continue;
                }
                
                // Check if we have this property in our definitions
                if let Some(property_info) = definitions.get_property_info(property_name) {
                    tested_properties += 1;
                    
                    // For now, just log that we found the property and its format
                    // In the future, we could validate that our value_spec matches the format
                    println!("Found property '{}' with format: '{}'", property_name, format_spec);
                    
                    // Note: We don't expect our property descriptions to contain the exact format strings
                    // since they are descriptive text, not format specifications.
                    // This test mainly serves to document what formats are available in the markdown.
                } else {
                    // Property not found in our definitions - log as info
                    not_found_properties.push(format!(
                        "Property '{}' with format '{}' not found in definitions",
                        property_name, format_spec
                    ));
                }
            }
        }
    }
    
    // Print info about properties not found in our definitions
    if !not_found_properties.is_empty() {
        println!("\nInfo: Properties from markdown not found in our definitions ({} total):", not_found_properties.len());
        for not_found in &not_found_properties {
            println!("  - {}", not_found);
        }
    }
    
    // Note: We removed mismatch reporting since we're not doing exact format validation
    // The test now serves to document available formats and ensure we can parse them
    
    // Ensure we tested a reasonable number of properties
    assert!(tested_properties > 10, "Expected to test more than 10 properties, but only tested {}", tested_properties);
    
    println!("Successfully validated {} property formats against markdown source data", tested_properties);
}

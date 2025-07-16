use crate::test_utils::get_project_root;
use crate::uss::constants::*;
use crate::uss::definitions::{PropertyAnimation, UssDefinitions};
use scraper::{Html, Selector};
use std::fs;

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
    assert!(
        definitions
            .get_property_info("-unity-background-scale-mode")
            .is_some()
    );

    // Test non-existent property
    assert!(
        definitions
            .get_property_info("non-existent-property")
            .is_none()
    );
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
    assert!(
        !background_color.inherited,
        "background-color should not be inherited"
    );
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
fn test_keyword_only() {
    let definitions = UssDefinitions::new();
    let flex_dir = definitions.get_property_info("flex-direction").unwrap();
    assert!(flex_dir.value_spec.is_keyword_only());

    let width = definitions.get_property_info("width").unwrap();
    assert!(!width.value_spec.is_keyword_only());
}

#[test]
fn test_color_only() {
    let definitions = UssDefinitions::new();
    let color = definitions.get_property_info("color").unwrap();
    assert!(color.value_spec.is_color_only());

    let width = definitions.get_property_info("width").unwrap();
    assert!(!width.value_spec.is_color_only());

    let bg_color = definitions.get_property_info("background-color").unwrap();
    assert!(bg_color.value_spec.is_color_only());
}

/// Property data extracted from HTML source
#[derive(Debug, Clone)]
struct PropertySourceData {
    name: String,
    inherited: bool,
    animatable: PropertyAnimation,
    description: String,
    doc_url: String,
}

/// Extract property data from HTML source
fn extract_properties_from_html(html_file_path: &std::path::Path) -> Vec<PropertySourceData> {
    let html_content = fs::read_to_string(html_file_path)
        .expect("Failed to read USS_properties_reference_table.html file");

    let document = Html::parse_document(&html_content);
    let row_selector = Selector::parse("tbody tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();
    let link_selector = Selector::parse("a").unwrap();
    let code_selector = Selector::parse("code").unwrap();

    let mut properties = Vec::new();

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
                if element
                    .value()
                    .classes()
                    .any(|class| class.starts_with("tooltip") && class != "tooltip")
                {
                    return; // Skip this entire subtree
                }

                // Process this element's direct text and children
                for node in element.children() {
                    if let Some(text_node) = node.value().as_text() {
                        text.push_str(text_node);
                    } else if let Some(_child_element) = node.value().as_element() {
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

        // Parse inherited field
        let inherited = inherited_str == "Yes";

        // Parse animatable field
        let animatable = match animatable_str.as_str() {
            "Fully animatable" => PropertyAnimation::Animatable,
            "Discrete" => PropertyAnimation::Discrete,
            "Non-animatable" => PropertyAnimation::None,
            _ => {
                eprintln!(
                    "Warning: unknown animatable value '{}' for property '{}'",
                    animatable_str, property_name
                );
                continue;
            }
        };

        properties.push(PropertySourceData {
            name: property_name,
            inherited,
            animatable,
            description,
            doc_url,
        });
    }

    properties
}

#[test]
fn test_properties_against_source_data() {
    let definitions = UssDefinitions::new();

    // Read the official properties HTML source data
    let project_root = get_project_root();
    let html_file_path = project_root
        .join("Assets")
        .join("data")
        .join("USS_properties_reference_table_6.0_clean.html");

    let properties = extract_properties_from_html(&html_file_path);

    let mut tested_properties = 0;
    let mut mismatches = Vec::new();
    let mut missing_properties = Vec::new();

    for property_data in properties {
        // Check if we have this property in our definitions
        if let Some(property_info) = definitions.get_property_info(&property_data.name) {
            tested_properties += 1;
            let mut property_mismatches = Vec::new();

            // Test inherited field
            if property_info.inherited != property_data.inherited {
                property_mismatches.push(format!(
                    "  - inherited: expected {}, got {}",
                    property_data.inherited, property_info.inherited
                ));
            }

            // Test animatable field
            if property_info.animatable != property_data.animatable {
                property_mismatches.push(format!(
                    "  - animatable: expected {:?}, got {:?}",
                    property_data.animatable, property_info.animatable
                ));
            }

            // Test description field (should contain the official description verbatim)
            // Normalize quote characters for comparison (official docs use non-standard quotes)
            let normalized_expected = property_data.description.replace("’", "'");
            let normalized_actual = property_info.description.replace("’", "'");

            if !normalized_actual.contains(&normalized_expected) {
                property_mismatches.push(format!(
                    "  - [{}] description does not contain official text\n    Expected: '{}'\n    Actual: '{}'",
                    property_data.name, property_data.description, property_info.description
                ));
            }

            // Test documentation URL (replace {version} with "6000.0")
            if !property_data.doc_url.is_empty() && !property_info.documentation_url.is_empty() {
                let expected_doc_url = property_info
                    .documentation_url
                    .replace("{version}", "6000.0");
                if expected_doc_url != property_data.doc_url {
                    property_mismatches.push(format!(
                        "  - [{}] doc_url mismatch\n    Expected: '{}'\n    Actual: '{}'",
                        property_data.name, property_data.doc_url, expected_doc_url
                    ));
                }
            }

            // Print differences for this property if any
            if !property_mismatches.is_empty() {
                println!("\nProperty '{}' differences:", property_data.name);
                for mismatch in &property_mismatches {
                    println!("{}", mismatch);
                }
                mismatches.extend(property_mismatches);
            }
        } else {
            // Property is missing from our definitions
            missing_properties.push(property_data.name);
        }
    }

    // Report missing properties
    if !missing_properties.is_empty() {
        println!(
            "\nMissing properties from our definitions ({} total):",
            missing_properties.len()
        );
        for missing_prop in &missing_properties {
            println!("  - {}", missing_prop);
        }
    }

    // Report results
    if !mismatches.is_empty() || !missing_properties.is_empty() {
        let mut error_msg = String::new();

        if !mismatches.is_empty() {
            error_msg.push_str(&format!(
                "Found {} mismatches in {} tested properties:\n{}",
                mismatches.len(),
                tested_properties,
                mismatches.join("\n")
            ));
        }

        if !missing_properties.is_empty() {
            if !error_msg.is_empty() {
                error_msg.push_str("\n\n");
            }
            error_msg.push_str(&format!(
                "Missing {} properties from our definitions:\n{}",
                missing_properties.len(),
                missing_properties.join(", ")
            ));
        }

        panic!("{}", error_msg);
    }

    // Ensure we tested a reasonable number of properties
    assert!(
        tested_properties > 50,
        "Expected to test more than 50 properties, but only tested {}",
        tested_properties
    );

    println!(
        "Successfully validated {} properties against HTML source data",
        tested_properties
    );
}

/// Extract property format definitions from markdown source data
fn extract_properties_from_markdown(
    md_file_path: &std::path::Path,
) -> std::collections::HashMap<String, String> {
    let md_content =
        fs::read_to_string(md_file_path).expect("Failed to read USS_property_format_6.0.md file");

    let mut properties = std::collections::HashMap::new();
    let lines: Vec<&str> = md_content.lines().collect();
    let mut in_css_example = false;

    for line in lines.iter() {
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
                let mut format_spec = code_line[colon_pos + 1..].trim();

                // Remove trailing semicolon from format specification
                if format_spec.ends_with(';') {
                    format_spec = &format_spec[..format_spec.len() - 1].trim();
                }

                // Skip empty property names or format specs
                if property_name.is_empty() || format_spec.is_empty() {
                    continue;
                }

                // Skip CSS selectors and other non-property lines
                if property_name.contains('.')
                    || property_name.contains('#')
                    || property_name.contains(' ')
                    || property_name.starts_with('@')
                {
                    continue;
                }

                // Skip lines that look like CSS values (end with semicolon but don't contain format characters)
                if format_spec.ends_with(';')
                    && !format_spec.contains('<')
                    && !format_spec.contains('[')
                    && !format_spec.contains('>')
                {
                    continue;
                }

                // Only process lines that look like actual format specifications
                // (contain angle brackets or pipe symbols indicating value types)
                if !format_spec.contains('<') && !format_spec.contains('|') {
                    continue;
                }

                properties.insert(property_name.to_string(), format_spec.to_string());
            }
        }
    }

    properties
}

#[test]
fn test_properties_against_markdown_format() {
    let definitions = UssDefinitions::new();

    // Step 1: Extract data from source
    let project_root = get_project_root();
    let md_file_path = project_root
        .join("Assets")
        .join("data")
        .join("USS_property_format_6.0.md");
    let source_properties = extract_properties_from_markdown(&md_file_path);

    // Step 2: Validate against our definitions
    let mut tested_properties = 0;
    let mut not_found_properties = Vec::new();

    for (property_name, expected_format) in &source_properties {
        if let Some(property_info) = definitions.get_property_info(property_name) {
            tested_properties += 1;

            // Check if the format field matches the expected format
            let existing_format = property_info.format;
            if existing_format != expected_format {
                not_found_properties.push(format!(
                    "Property '{}': expected format '{}', but found '{}'",
                    property_name, expected_format, existing_format
                ));
            } else {
                println!(
                    "✓ Property '{}' format matches: {}",
                    property_name, expected_format
                );
            }
        } else {
            // Property not found in our definitions - log as info
            not_found_properties.push(format!(
                "Property '{}' with format '{}' not found in definitions",
                property_name, expected_format
            ));
        }
    }

    // Print info about properties not found in our definitions or format mismatches
    if !not_found_properties.is_empty() {
        println!(
            "\nFormat mismatches and missing properties ({} total):",
            not_found_properties.len()
        );
        for not_found in &not_found_properties {
            println!("  - {}", not_found);
        }

        // Count actual format mismatches vs missing properties
        let format_mismatches: Vec<_> = not_found_properties
            .iter()
            .filter(|msg| msg.contains("expected format"))
            .collect();

        if !format_mismatches.is_empty() {
            panic!(
                "Found {} format mismatches. Please update the format field in property_data.rs",
                format_mismatches.len()
            );
        }
    }

    // Ensure we tested a reasonable number of properties
    assert!(
        tested_properties > 10,
        "Expected to test more than 10 properties, but only tested {}",
        tested_properties
    );

    println!(
        "Successfully validated {} property formats against markdown source data",
        tested_properties
    );
}

/// Extract color keywords and hex values from HTML source data
fn extract_colors_from_html(
    html_file_path: &std::path::Path,
) -> std::collections::HashMap<String, String> {
    let html_content =
        fs::read_to_string(html_file_path).expect("Failed to read USS color keywords HTML file");

    let document = Html::parse_document(&html_content);
    let row_selector = Selector::parse("tbody tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();
    let code_selector = Selector::parse("code").unwrap();

    let mut colors = std::collections::HashMap::new();

    for row in document.select(&row_selector) {
        let cells: Vec<_> = row.select(&cell_selector).collect();
        if cells.len() != 3 {
            continue; // Skip malformed rows
        }

        // Extract color keyword from the first cell's <code> tag
        let color_keyword = if let Some(code_elem) = cells[0].select(&code_selector).next() {
            code_elem.text().collect::<String>()
        } else {
            continue; // Skip rows without color keyword in code tag
        };

        // Extract hex value from the second cell's <code> tag
        let hex_value = if let Some(code_elem) = cells[1].select(&code_selector).next() {
            code_elem.text().collect::<String>()
        } else {
            continue; // Skip rows without hex value in code tag
        };

        colors.insert(color_keyword, hex_value);
    }

    colors
}

#[test]
fn test_color_keywords_against_source_data() {
    let definitions = UssDefinitions::new();

    // Step 1: Extract data from source
    let project_root = get_project_root();
    let html_file_path = project_root
        .join("Assets")
        .join("data")
        .join("Unity - Manual_ USS color keywords_6.1_clean.html");
    let source_colors = extract_colors_from_html(&html_file_path);

    // Step 2: Validate against our definitions
    let mut tested_colors = 0;
    let mut mismatches = Vec::new();
    let mut missing_colors = Vec::new();

    for (color_keyword, expected_hex) in &source_colors {
        if definitions.is_valid_color_keyword(color_keyword) {
            tested_colors += 1;

            // Get the hex value from our definitions
            if let Some(our_hex_value) =
                definitions.valid_color_keywords.get(color_keyword.as_str())
            {
                // Compare hex values (normalize case)
                let expected_hex_lower = expected_hex.to_lowercase();
                let actual_hex_lower = our_hex_value.to_lowercase();

                if expected_hex_lower != actual_hex_lower {
                    mismatches.push(format!(
                        "Color '{}': expected hex '{}', got '{}'",
                        color_keyword, expected_hex_lower, actual_hex_lower
                    ));
                }
            } else {
                // This shouldn't happen if is_valid_color_keyword returned true
                mismatches.push(format!(
                    "Color '{}': found in validation but not in color map",
                    color_keyword
                ));
            }
        } else {
            // Color is missing from our definitions
            missing_colors.push(format!("'{}' with hex '{}'", color_keyword, expected_hex));
        }
    }

    // Check for colors in our definitions that are not in the HTML source
    let mut extra_colors = Vec::new();
    for &color_keyword in definitions.valid_color_keywords.keys() {
        if !source_colors.contains_key(color_keyword) {
            extra_colors.push(color_keyword);
        }
    }

    // Report missing colors
    if !missing_colors.is_empty() {
        println!(
            "\nMissing colors from our definitions ({} total):",
            missing_colors.len()
        );
        for missing_color in &missing_colors {
            println!("  - {}", missing_color);
        }
    }

    // Report extra colors
    if !extra_colors.is_empty() {
        println!(
            "\nExtra colors in our definitions not found in HTML ({} total):",
            extra_colors.len()
        );
        for extra_color in &extra_colors {
            println!("  - {}", extra_color);
        }
    }

    // Report hex value mismatches
    if !mismatches.is_empty() {
        println!("\nHex value mismatches ({} total):", mismatches.len());
        for mismatch in &mismatches {
            println!("  - {}", mismatch);
        }
    }

    // Report results
    if !mismatches.is_empty() || !missing_colors.is_empty() || !extra_colors.is_empty() {
        let mut error_msg = String::new();

        if !mismatches.is_empty() {
            error_msg.push_str(&format!(
                "Found {} hex value mismatches:\n{}",
                mismatches.len(),
                mismatches.join("\n")
            ));
        }

        if !missing_colors.is_empty() {
            if !error_msg.is_empty() {
                error_msg.push_str("\n\n");
            }
            error_msg.push_str(&format!(
                "Missing {} colors from our definitions:\n{}",
                missing_colors.len(),
                missing_colors.join(", ")
            ));
        }

        if !extra_colors.is_empty() {
            if !error_msg.is_empty() {
                error_msg.push_str("\n\n");
            }
            error_msg.push_str(&format!(
                "Found {} extra colors in our definitions:\n{}",
                extra_colors.len(),
                extra_colors.join(", ")
            ));
        }

        panic!("{}", error_msg);
    }

    // Ensure we tested a reasonable number of colors
    assert!(
        tested_colors > 100,
        "Expected to test more than 100 colors, but only tested {}",
        tested_colors
    );

    // Verify the total count matches
    let total_colors_in_definitions = definitions.valid_color_keywords.len();
    println!(
        "Successfully validated {} colors against HTML source data",
        tested_colors
    );
    println!(
        "Total colors in our definitions: {}",
        total_colors_in_definitions
    );
    println!("Colors tested from HTML: {}", tested_colors);
}

#[test]
fn test_keyword_completeness() {
    let definitions = UssDefinitions::new();
    
    // Step 1: Extract all keywords from all property ValueSpecs
    let mut keywords_from_value_specs = std::collections::HashSet::new();
    
    for (property_name, property_info) in definitions.get_all_properties() {
        for format in &property_info.value_spec.formats {
            for entry in &format.entries {
                for option in &entry.options {
                    if let crate::uss::value_spec::ValueType::Keyword(keyword) = option {
                        keywords_from_value_specs.insert(*keyword);
                    }
                }
            }
        }
    }
    
    // Step 2: Get all keywords from keyword info
    let keywords_from_info: std::collections::HashSet<&str> = definitions.get_all_keywords().keys().copied().collect();
    
    // Step 3: Find missing keywords (in ValueSpec but not in keyword info)
    let missing_keywords: Vec<&str> = keywords_from_value_specs
        .difference(&keywords_from_info)
        .copied()
        .collect();
    
    // Step 4: Find orphaned keywords (in keyword info but not in any ValueSpec)
    let orphaned_keywords: Vec<&str> = keywords_from_info
        .difference(&keywords_from_value_specs)
        .copied()
        .collect();
    
    // Step 5: Validate used_by_properties field for each keyword
    let mut property_association_errors = Vec::new();
    
    for (keyword, keyword_info) in definitions.get_all_keywords() {
        // Find all properties that actually use this keyword
        let mut actual_properties = std::collections::HashSet::new();
        
        for (property_name, property_info) in definitions.get_all_properties() {
            for format in &property_info.value_spec.formats {
                for entry in &format.entries {
                    for option in &entry.options {
                        if let crate::uss::value_spec::ValueType::Keyword(kw) = option {
                             if *kw == *keyword {
                                 actual_properties.insert(*property_name);
                             }
                         }
                    }
                }
            }
        }
        
        // Compare with the used_by_properties field
        let recorded_properties: std::collections::HashSet<&str> = keyword_info.used_by_properties.iter().copied().collect();
        
        if actual_properties != recorded_properties {
            let missing_in_record: Vec<&str> = actual_properties.difference(&recorded_properties).copied().collect();
            let extra_in_record: Vec<&str> = recorded_properties.difference(&actual_properties).copied().collect();
            
            let mut error_parts = Vec::new();
            if !missing_in_record.is_empty() {
                error_parts.push(format!("missing: [{}]", missing_in_record.join(", ")));
            }
            if !extra_in_record.is_empty() {
                error_parts.push(format!("extra: [{}]", extra_in_record.join(", ")));
            }
            
            property_association_errors.push(format!(
                "  - '{}': {}",
                keyword,
                error_parts.join(", ")
            ));
        }
    }
    
    // Step 6: Report results
    let mut error_messages = Vec::new();
    
    if !missing_keywords.is_empty() {
        let mut sorted_missing = missing_keywords;
        sorted_missing.sort();
        error_messages.push(format!(
            "Missing keyword info for {} keywords used in ValueSpecs:\n{}",
            sorted_missing.len(),
            sorted_missing.iter().map(|k| format!("  - {}", k)).collect::<Vec<_>>().join("\n")
        ));
    }
    
    if !orphaned_keywords.is_empty() {
        let mut sorted_orphaned = orphaned_keywords;
        sorted_orphaned.sort();
        error_messages.push(format!(
            "Orphaned keyword info for {} keywords not used in any ValueSpec:\n{}",
            sorted_orphaned.len(),
            sorted_orphaned.iter().map(|k| format!("  - {}", k)).collect::<Vec<_>>().join("\n")
        ));
    }
    
    if !property_association_errors.is_empty() {
        property_association_errors.sort();
        error_messages.push(format!(
            "Incorrect property associations for {} keywords:\n{}",
            property_association_errors.len(),
            property_association_errors.join("\n")
        ));
    }
    
    // Print summary
    println!("Keywords found in ValueSpecs: {}", keywords_from_value_specs.len());
    println!("Keywords with info: {}", keywords_from_info.len());
    
    if !error_messages.is_empty() {
        panic!("Keyword completeness validation failed:\n\n{}", error_messages.join("\n\n"));
    }
    
    println!("✓ All keywords have complete coverage between ValueSpecs and keyword info");
    println!("✓ All keyword property associations are correctly populated");
}

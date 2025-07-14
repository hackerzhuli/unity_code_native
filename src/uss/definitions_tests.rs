use crate::uss::{constants::*, definitions::{PropertyAnimation, UssDefinitions}};


#[test]
fn test_property_info_functionality() {
    let definitions = UssDefinitions::new();
    
    // Test getting property info
    let border_radius_info = definitions.get_property_info("border-radius");
    assert!(border_radius_info.is_some());
    
    let info = border_radius_info.unwrap();
    assert_eq!(info.name, "border-radius");
    assert!(info.description.contains("radius"));
    assert!(!info.inherited);
    assert_eq!(info.animatable, PropertyAnimation::Animatable);
    
    // Test documentation URL formatting with specific URLs
    let border_radius_info = definitions.get_property_info("border-radius").unwrap();
    let doc_url = border_radius_info.documentation_url.replace("{version}", "2023.3");
    assert!(doc_url.contains("2023.3"));
    assert!(doc_url.contains("UIE-USS-SupportedProperties.html#drawing-borders")); // Should have specific section
    assert_eq!(doc_url, "https://docs.unity3d.com/2023.3/Documentation/Manual/UIE-USS-SupportedProperties.html#drawing-borders");
    
    // Test Unity-specific property URL
    let unity_font_info = definitions.get_property_info("-unity-font").unwrap();
    let unity_url = unity_font_info.documentation_url.replace("{version}", "2023.3");
    assert!(unity_url.contains("UIE-USS-SupportedProperties.html#unity-font")); // Should have Unity font section
    
    // Test inheritance check
    let color_info = definitions.get_property_info("color").unwrap();
    assert!(color_info.inherited); // color is inherited
    let border_radius_info = definitions.get_property_info("border-radius").unwrap();
    assert!(!border_radius_info.inherited); // border-radius is not inherited
    
    // Test animation check
    let opacity_info = definitions.get_property_info("opacity").unwrap();
    assert_eq!(opacity_info.animatable, PropertyAnimation::Animatable); // opacity is animatable
    let display_info = definitions.get_property_info("display").unwrap();
    assert_eq!(display_info.animatable, PropertyAnimation::Discrete); // display is not animatable
    
    // Test description
    let color_info = definitions.get_property_info("color").unwrap();
    let desc = color_info.description;
    assert!(desc.contains("text"));
    
    // Test getting all property names
    let all_props: Vec<&str> = definitions.properties.keys().cloned().collect();
    assert!(all_props.contains(&"border-radius"));
    assert!(all_props.contains(&"-unity-font"));
    assert!(all_props.len() > 50); // Should have many properties
}

#[test]
fn test_unit_validation() {
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
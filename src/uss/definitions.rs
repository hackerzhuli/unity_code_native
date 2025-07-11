//! USS Definitions
//!
//! Contains USS property definitions, pseudo-classes, color keywords,
//! and other validation data that can be shared across different features
//! like diagnostics and autocomplete.

use crate::uss::property_data::{create_standard_properties, create_unity_properties};
use crate::uss::value_spec::ValueSpec;
use std::collections::{HashMap, HashSet};

/// Property documentation information
#[derive(Debug, Clone)]
pub struct PropertyInfo {
    /// Property name
    pub name: &'static str,
    /// Property description
    pub description: &'static str,
    /// Documentation URL (may contain {version} placeholder for Unity docs)
    pub documentation_url: String,
    /// Whether this property is inherited
    pub inherited: bool,
    /// Whether this property is animatable
    pub animatable: bool,
    /// Complete value specification for this property
    /// Note: All properties support initial keyword to reset to default, we don't put initial in here for brevity
    pub value_spec: ValueSpec,
}

/// USS language definitions and validation data
pub struct UssDefinitions {
    /// USS properties with their metadata
    pub properties: HashMap<&'static str, PropertyInfo>,
    /// Valid pseudo-classes
    pub valid_pseudo_classes: HashSet<&'static str>,
    /// Valid CSS color keywords with their hex values
    pub valid_color_keywords: HashMap<&'static str, &'static str>,
    /// Valid USS functions
    pub valid_functions: HashSet<&'static str>,
    /// Valid at-rules
    pub valid_at_rules: HashSet<&'static str>,
    /// Valid USS units
    pub valid_units: HashSet<&'static str>,
}

impl UssDefinitions {
    /// Create a new USS definitions instance
    pub fn new() -> Self {
        let mut properties = HashMap::new();
        
        // Load standard CSS properties
        let standard_props = create_standard_properties();
        for (name, prop_info) in standard_props {
            properties.insert(name, prop_info);
        }
        
        // Load Unity-specific properties
        let unity_props = create_unity_properties();
        for (name, prop_info) in unity_props {
            properties.insert(name, prop_info);
        }

        
        let mut valid_pseudo_classes = HashSet::new();
        let pseudo_classes = [
            ":hover", ":active", ":inactive", ":focus", ":disabled", 
            ":enabled", ":checked", ":root"
        ];
        for pseudo in pseudo_classes {
            valid_pseudo_classes.insert(pseudo);
        }
        
        let mut valid_color_keywords = HashMap::new();
        let color_keywords = [
            ("aliceblue", "#f0f8ff"),
            ("antiquewhite", "#faebd7"),
            ("aqua", "#00ffff"),
            ("aquamarine", "#7fffd4"),
            ("azure", "#f0ffff"),
            ("beige", "#f5f5dc"),
            ("bisque", "#ffe4c4"),
            ("black", "#000000"),
            ("blanchedalmond", "#ffebcd"),
            ("blue", "#0000ff"),
            ("blueviolet", "#8a2be2"),
            ("brown", "#a52a2a"),
            ("burlywood", "#deb887"),
            ("cadetblue", "#5f9ea0"),
            ("chartreuse", "#7fff00"),
            ("chocolate", "#d2691e"),
            ("coral", "#ff7f50"),
            ("cornflowerblue", "#6495ed"),
            ("cornsilk", "#fff8dc"),
            ("crimson", "#dc143c"),
            ("cyan", "#00ffff"),
            ("darkblue", "#00008b"),
            ("darkcyan", "#008b8b"),
            ("darkgoldenrod", "#b8860b"),
            ("darkgray", "#a9a9a9"),
            ("darkgreen", "#006400"),
            ("darkgrey", "#a9a9a9"),
            ("darkkhaki", "#bdb76b"),
            ("darkmagenta", "#8b008b"),
            ("darkolivegreen", "#556b2f"),
            ("darkorange", "#ff8c00"),
            ("darkorchid", "#9932cc"),
            ("darkred", "#8b0000"),
            ("darksalmon", "#e9967a"),
            ("darkseagreen", "#8fbc8f"),
            ("darkslateblue", "#483d8b"),
            ("darkslategray", "#2f4f4f"),
            ("darkslategrey", "#2f4f4f"),
            ("darkturquoise", "#00ced1"),
            ("darkviolet", "#9400d3"),
            ("deeppink", "#ff1493"),
            ("deepskyblue", "#00bfff"),
            ("dimgray", "#696969"),
            ("dimgrey", "#696969"),
            ("dodgerblue", "#1e90ff"),
            ("firebrick", "#b22222"),
            ("floralwhite", "#fffaf0"),
            ("forestgreen", "#228b22"),
            ("gainsboro", "#dcdcdc"),
            ("ghostwhite", "#f8f8ff"),
            ("gold", "#ffd700"),
            ("goldenrod", "#daa520"),
            ("gray", "#808080"),
            ("green", "#008000"),
            ("greenyellow", "#adff2f"),
            ("grey", "#808080"),
            ("honeydew", "#f0fff0"),
            ("hotpink", "#ff69b4"),
            ("indianred", "#cd5c5c"),
            ("indigo", "#4b0082"),
            ("ivory", "#fffff0"),
            ("khaki", "#f0e68c"),
            ("lavender", "#e6e6fa"),
            ("lavenderblush", "#fff0f5"),
            ("lawngreen", "#7cfc00"),
            ("lemonchiffon", "#fffacd"),
            ("lightblue", "#add8e6"),
            ("lightcoral", "#f08080"),
            ("lightcyan", "#e0ffff"),
            ("lightgoldenrodyellow", "#fafad2"),
            ("lightgray", "#d3d3d3"),
            ("lightgreen", "#90ee90"),
            ("lightgrey", "#d3d3d3"),
            ("lightpink", "#ffb6c1"),
            ("lightsalmon", "#ffa07a"),
            ("lightseagreen", "#20b2aa"),
            ("lightskyblue", "#87cefa"),
            ("lightslategray", "#778899"),
            ("lightslategrey", "#778899"),
            ("lightsteelblue", "#b0c4de"),
            ("lightyellow", "#ffffe0"),
            ("lime", "#00ff00"),
            ("limegreen", "#32cd32"),
            ("linen", "#faf0e6"),
            ("magenta", "#ff00ff"),
            ("maroon", "#800000"),
            ("mediumaquamarine", "#66cdaa"),
            ("mediumblue", "#0000cd"),
            ("mediumorchid", "#ba55d3"),
            ("mediumpurple", "#9370db"),
            ("mediumseagreen", "#3cb371"),
            ("mediumslateblue", "#7b68ee"),
            ("mediumspringgreen", "#00fa9a"),
            ("mediumturquoise", "#48d1cc"),
            ("mediumvioletred", "#c71585"),
            ("midnightblue", "#191970"),
            ("mintcream", "#f5fffa"),
            ("mistyrose", "#ffe4e1"),
            ("moccasin", "#ffe4b5"),
            ("navajowhite", "#ffdead"),
            ("navy", "#000080"),
            ("oldlace", "#fdf5e6"),
            ("olive", "#808000"),
            ("olivedrab", "#6b8e23"),
            ("orange", "#ffa500"),
            ("orangered", "#ff4500"),
            ("orchid", "#da70d6"),
            ("palegoldenrod", "#eee8aa"),
            ("palegreen", "#98fb98"),
            ("paleturquoise", "#afeeee"),
            ("palevioletred", "#db7093"),
            ("papayawhip", "#ffefd5"),
            ("peachpuff", "#ffdab9"),
            ("peru", "#cd853f"),
            ("pink", "#ffc0cb"),
            ("plum", "#dda0dd"),
            ("powderblue", "#b0e0e6"),
            ("purple", "#800080"),
            ("rebeccapurple", "#663399"),
            ("red", "#ff0000"),
            ("rosybrown", "#bc8f8f"),
            ("royalblue", "#4169e1"),
            ("saddlebrown", "#8b4513"),
            ("salmon", "#fa8072"),
            ("sandybrown", "#f4a460"),
            ("seagreen", "#2e8b57"),
            ("seashell", "#fff5ee"),
            ("sienna", "#a0522d"),
            ("silver", "#c0c0c0"),
            ("skyblue", "#87ceeb"),
            ("slateblue", "#6a5acd"),
            ("slategray", "#708090"),
            ("slategrey", "#708090"),
            ("snow", "#fffafa"),
            ("springgreen", "#00ff7f"),
            ("steelblue", "#4682b4"),
            ("tan", "#d2b48c"),
            ("teal", "#008080"),
            ("thistle", "#d8bfd8"),
            ("tomato", "#ff6347"),
            ("transparent", "rgba(0,0,0,0)"),
            ("turquoise", "#40e0d0"),
            ("violet", "#ee82ee"),
            ("wheat", "#f5deb3"),
            ("white", "#ffffff"),
            ("whitesmoke", "#f5f5f5"),
            ("yellow", "#ffff00"),
            ("yellowgreen", "#9acd32"),
        ];
        for (color, hex) in color_keywords {
            valid_color_keywords.insert(color, hex);
        }
        
        let mut valid_functions = HashSet::new();
        let functions = ["url", "resource", "var", "rgb", "rgba", "hsl", "hsla"];
        for func in functions {
            valid_functions.insert(func);
        }
        
        let mut valid_at_rules = HashSet::new();
        let at_rules = ["@import"];
        for rule in at_rules {
            valid_at_rules.insert(rule);
        }
        
        let mut valid_units = HashSet::new();
        let units = [
            // Length units
            "px", "%",
            // Angle units
            "deg", "rad", "grad", "turn",
            // Time units
            "s", "ms",
        ];
        for unit in units {
            valid_units.insert(unit);
        }
        
        Self {
            properties,
            valid_pseudo_classes,
            valid_color_keywords,
            valid_functions,
            valid_at_rules,
            valid_units,
        }
    }
    
    /// Check if a property name is valid
    pub fn is_valid_property(&self, property_name: &str) -> bool {
        // Allow custom properties (CSS variables)
        if property_name.starts_with("--") {
            return true;
        }
        
        self.properties.contains_key(property_name)
    }
    
    /// Check if a property is a predefined USS property (excludes custom CSS variables)
    /// This is used for features like hover that should only show info for predefined properties
    pub fn is_predefined_property(&self, property_name: &str) -> bool {
        self.properties.contains_key(property_name)
    }
    
    /// Check if a pseudo-class is valid
    pub fn is_valid_pseudo_class(&self, pseudo_class: &str) -> bool {
        self.valid_pseudo_classes.contains(pseudo_class)
    }
    
    /// Check if a color keyword is valid
    pub fn is_valid_color_keyword(&self, color: &str) -> bool {
        self.valid_color_keywords.contains_key(color)
    }
    
    /// Get the hex value for a color keyword
    pub fn get_color_hex_value(&self, color: &str) -> Option<&'static str> {
        self.valid_color_keywords.get(color).copied()
    }
    
    /// Check if a function is valid
    pub fn is_valid_function(&self, function_name: &str) -> bool {
        self.valid_functions.contains(function_name)
    }
    
    /// Check if an at-rule is valid
    pub fn is_valid_at_rule(&self, at_rule: &str) -> bool {
        self.valid_at_rules.contains(at_rule)
    }
    
    /// Check if a unit is valid
    pub fn is_valid_unit(&self, unit: &str) -> bool {
        self.valid_units.contains(unit)
    }
    
    /// Check if a unit is a length unit
    pub fn is_length_unit(&self, unit: &str) -> bool {
        matches!(unit, "px" | "%")
    }
    
    /// Check if a unit is an angle unit
    pub fn is_angle_unit(&self, unit: &str) -> bool {
        matches!(unit, "deg" | "rad" | "grad" | "turn")
    }
    
    /// Check if a unit is a time unit
    pub fn is_time_unit(&self, unit: &str) -> bool {
        matches!(unit, "s" | "ms")
    }
    
    /// Get all valid units
    pub fn get_all_units(&self) -> Vec<&str> {
        self.valid_units.iter().copied().collect()
    }
    
    /// Get units by category
    pub fn get_length_units(&self) -> Vec<&str> {
        vec!["px", "%"]
    }
    
    pub fn get_angle_units(&self) -> Vec<&str> {
        vec!["deg", "rad", "grad", "turn"]
    }
    
    pub fn get_time_units(&self) -> Vec<&str> {
        vec!["s", "ms"]
    }
    
    /// Get valid keyword values for a specific property
    pub fn get_valid_keyword_values_for_property(&self, property_name: &str) -> Option<&'static [&'static str]> {
        match property_name {
            "display" => Some(&["flex", "none", "initial"]),
            "position" => Some(&["absolute", "relative", "initial"]),
            "flex-direction" => Some(&["row", "row-reverse", "column", "column-reverse", "initial"]),
            "justify-content" => Some(&["flex-start", "flex-end", "center", "space-between", "space-around", "initial"]),
            "align-items" | "align-self" => Some(&["auto", "flex-start", "flex-end", "center", "stretch", "initial"]),
            "-unity-background-scale-mode" => Some(&["stretch-to-fill", "scale-and-crop", "scale-to-fit", "initial"]),
            "-unity-font-style" => Some(&["normal", "italic", "bold", "bold-and-italic", "initial"]),
            "-unity-text-align" => Some(&[
                "upper-left", "middle-left", "lower-left", 
                "upper-center", "middle-center", "lower-center", 
                "upper-right", "middle-right", "lower-right", "initial"
            ]),
            "white-space" => Some(&["normal", "nowrap", "initial"]),
            "text-overflow" => Some(&["clip", "ellipsis", "initial"]),
            _ => None,
        }
    }
    
    /// Check if a value is valid for a specific property
    pub fn is_valid_value_for_property(&self, property_name: &str, value: &str) -> bool {
        if let Some(valid_values) = self.get_valid_keyword_values_for_property(property_name) {
            valid_values.contains(&value)
        } else {
            // For properties without specific validation, allow any value
            // This includes numeric values, custom values, etc.
            true
        }
    }
    
    /// Get property information including description and metadata
    pub fn get_property_info(&self, property_name: &str) -> Option<&PropertyInfo> {
        self.properties.get(property_name)
    }
    
    /// Get documentation URL for a property with version formatting
    pub fn get_property_documentation_url(&self, property_name: &str, unity_version: &str) -> Option<String> {
        self.properties.get(property_name).map(|info| {
            info.documentation_url.replace("{version}", unity_version)
        })
    }
    
    /// Check if a property is inherited
    pub fn is_property_inherited(&self, property_name: &str) -> bool {
        self.properties.get(property_name)
            .map(|info| info.inherited)
            .unwrap_or(false)
    }
    
    /// Check if a property is animatable
    pub fn is_property_animatable(&self, property_name: &str) -> bool {
        self.properties.get(property_name)
            .map(|info| info.animatable)
            .unwrap_or(false)
    }
    
    /// Get property description
    pub fn get_property_description(&self, property_name: &str) -> Option<&str> {
        self.properties.get(property_name)
            .map(|info| info.description)
    }
    
    /// Get all property names
    pub fn get_all_property_names(&self) -> Vec<&str> {
        self.properties.keys().copied().collect()
    }
    
    /// Get all properties with their information
    pub fn get_all_properties(&self) -> &HashMap<&'static str, PropertyInfo> {
        &self.properties
    }
}

impl Default for UssDefinitions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(info.animatable);
        
        // Test documentation URL formatting with specific URLs
        let doc_url = definitions.get_property_documentation_url("border-radius", "2023.3");
        assert!(doc_url.is_some());
        let url = doc_url.unwrap();
        assert!(url.contains("2023.3"));
        assert!(url.contains("UIE-USS-SupportedProperties.html#drawing-borders")); // Should have specific section
        assert_eq!(url, "https://docs.unity3d.com/2023.3/Documentation/Manual/UIE-USS-SupportedProperties.html#drawing-borders");
        
        // Test Unity-specific property URL
        let unity_url = definitions.get_property_documentation_url("-unity-font", "2023.3");
        assert!(unity_url.is_some());
        let url = unity_url.unwrap();
        assert!(url.contains("UIE-USS-SupportedProperties.html#unity-font")); // Should have Unity font section
        
        // Test inheritance check
        assert!(definitions.is_property_inherited("color")); // color is inherited
        assert!(!definitions.is_property_inherited("border-radius")); // border-radius is not inherited
        
        // Test animation check
        assert!(definitions.is_property_animatable("opacity")); // opacity is animatable
        assert!(!definitions.is_property_animatable("display")); // display is not animatable
        
        // Test description
        let desc = definitions.get_property_description("color");
        assert!(desc.is_some());
        assert!(desc.unwrap().contains("text"));
        
        // Test getting all property names
        let all_props = definitions.get_all_property_names();
        assert!(all_props.contains(&"border-radius"));
        assert!(all_props.contains(&"-unity-font"));
        assert!(all_props.len() > 50); // Should have many properties
    }
    
    #[test]
    fn test_unit_validation() {
        let definitions = UssDefinitions::new();
        
        // Test valid units
        assert!(definitions.is_valid_unit("px"));
        assert!(definitions.is_valid_unit("%"));
        assert!(definitions.is_valid_unit("deg"));
        assert!(definitions.is_valid_unit("rad"));
        assert!(definitions.is_valid_unit("grad"));
        assert!(definitions.is_valid_unit("turn"));
        assert!(definitions.is_valid_unit("s"));
        assert!(definitions.is_valid_unit("ms"));
        
        // Test invalid units
        assert!(!definitions.is_valid_unit("em"));
        assert!(!definitions.is_valid_unit("rem"));
        assert!(!definitions.is_valid_unit("vh"));
        assert!(!definitions.is_valid_unit("vw"));
        assert!(!definitions.is_valid_unit("pt"));
        assert!(!definitions.is_valid_unit("cm"));
        assert!(!definitions.is_valid_unit("invalid"));
        
        // Test unit categories
        assert!(definitions.is_length_unit("px"));
        assert!(definitions.is_length_unit("%"));
        assert!(!definitions.is_length_unit("deg"));
        
        assert!(definitions.is_angle_unit("deg"));
        assert!(definitions.is_angle_unit("rad"));
        assert!(definitions.is_angle_unit("grad"));
        assert!(definitions.is_angle_unit("turn"));
        assert!(!definitions.is_angle_unit("px"));
        
        assert!(definitions.is_time_unit("s"));
        assert!(definitions.is_time_unit("ms"));
        assert!(!definitions.is_time_unit("px"));
        
        // Test getting units by category
        let length_units = definitions.get_length_units();
        assert_eq!(length_units, vec!["px", "%"]);
        
        let angle_units = definitions.get_angle_units();
        assert_eq!(angle_units, vec!["deg", "rad", "grad", "turn"]);
        
        let time_units = definitions.get_time_units();
        assert_eq!(time_units, vec!["s", "ms"]);
        
        // Test getting all units
        let all_units = definitions.get_all_units();
        assert!(all_units.contains(&"px"));
        assert!(all_units.contains(&"deg"));
        assert!(all_units.contains(&"s"));
        assert_eq!(all_units.len(), 8); // Should have exactly 8 units
    }
}
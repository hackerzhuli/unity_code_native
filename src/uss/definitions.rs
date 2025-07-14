//! USS Definitions
//!
//! Contains USS property definitions, pseudo-classes, color keywords,
//! and other validation data that can be shared across different features
//! like diagnostics and autocomplete.

use crate::uss::property_data::{create_standard_properties, create_unity_properties};
use crate::uss::keyword_data::{KeywordInfo, create_keyword_info};
use crate::uss::value_spec::ValueSpec;
use crate::uss::color::Color;
use crate::uss::constants::*;
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

/// Pseudo-class documentation information
#[derive(Debug, Clone)]
pub struct PseudoClassInfo {
    /// Pseudo-class name (without the colon prefix)
    pub name: &'static str,
    /// Description of when this pseudo-class matches
    pub description: &'static str,
    /// Documentation URL (may contain {version} placeholder for Unity docs)
    pub documentation_url: String,
    /// Whether this pseudo-class can be chained with others
    pub chainable: bool,
}

/// Create pseudo-class information with documentation
fn create_pseudo_class_info() -> HashMap<&'static str, PseudoClassInfo> {
    let mut pseudo_classes = HashMap::new();
    
    pseudo_classes.insert("hover", PseudoClassInfo {
        name: "hover",
        description: "The cursor is positioned over the element.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
        chainable: true,
    });
    
    pseudo_classes.insert("active", PseudoClassInfo {
        name: "active",
        description: "A user interacts with the element.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
        chainable: true,
    });
    
    pseudo_classes.insert("inactive", PseudoClassInfo {
        name: "inactive",
        description: "A user stops to interact with the element.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
        chainable: true,
    });
    
    pseudo_classes.insert("focus", PseudoClassInfo {
        name: "focus",
        description: "The element has focus.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
        chainable: true,
    });
    
    pseudo_classes.insert("disabled", PseudoClassInfo {
        name: "disabled",
        description: "The element is in a disabled state.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
        chainable: true,
    });
    
    pseudo_classes.insert("enabled", PseudoClassInfo {
        name: "enabled",
        description: "The element is in an enabled state.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
        chainable: true,
    });
    
    pseudo_classes.insert("checked", PseudoClassInfo {
        name: "checked",
        description: "The element is a Toggle or RadioButton element and it's selected.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
        chainable: true,
    });
    
    pseudo_classes.insert("root", PseudoClassInfo {
        name: "root",
        description: "The element is the highest-level element in the visual tree that has the stylesheet applied.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
        chainable: false,
    });
    
    pseudo_classes
}

/// USS language definitions and validation data
#[derive(Clone)]
pub struct UssDefinitions {
    /// USS properties with their metadata
    pub properties: HashMap<&'static str, PropertyInfo>,
    /// USS keywords with their documentation
    pub keywords: HashMap<&'static str, KeywordInfo>,
    /// USS pseudo-classes with their documentation
    pub pseudo_classes: HashMap<&'static str, PseudoClassInfo>,
    /// Valid pseudo-classes (for backward compatibility)
    pub valid_pseudo_classes: HashSet<&'static str>,
    /// Valid CSS color keywords with their hex values
    pub valid_color_keywords: HashMap<&'static str, &'static str>,
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

        // Load pseudo-class information
        let mut pseudo_classes = HashMap::new();
        let mut valid_pseudo_classes = HashSet::new();
        
        let pseudo_class_data = create_pseudo_class_info();
        for (name, pseudo_info) in pseudo_class_data {
            pseudo_classes.insert(name, pseudo_info);
            valid_pseudo_classes.insert(name);
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
        let functions = ["url", "resource", "var", "rgb", "rgba"];
        for func in functions {
            valid_functions.insert(func);
        }
        
        let mut valid_at_rules = HashSet::new();
        let at_rules = [NODE_IMPORT];
        for rule in at_rules {
            valid_at_rules.insert(rule);
        }
        
        let mut valid_units = HashSet::new();
        let units = [
            // Length units
            UNIT_PX, UNIT_PERCENT,
            // Angle units
            UNIT_DEG, UNIT_RAD, UNIT_GRAD, UNIT_TURN,
            // Time units
            UNIT_S, UNIT_MS,
        ];
        for unit in units {
            valid_units.insert(unit);
        }
        
        // Load keyword information
        let keywords = create_keyword_info();
        
        Self {
            properties,
            keywords,
            pseudo_classes,
            valid_pseudo_classes,
            valid_color_keywords,
            valid_units,
        }
    }
    
    /// Check if a property name is valid, ie, an existing property or a custom property (USS variable)
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
    
    /// Get parsed RGB color components for a color keyword
    /// Returns (r, g, b) values in 0-255 range
    pub fn get_color_rgb(&self, color: &str) -> Option<(u8, u8, u8)> {
        let hex_value = self.valid_color_keywords.get(color).copied()?;
        Color::from_hex(hex_value).map(|color| color.rgb())
    }

    /// Check if a unit is valid
    pub fn is_valid_unit(&self, unit: &str) -> bool {
        self.valid_units.contains(unit)
    }
    
    /// Check if a unit is a length unit
    pub fn is_length_unit(&self, unit: &str) -> bool {
        matches!(unit, UNIT_PX | UNIT_PERCENT)
    }
    
    /// Check if a unit is an angle unit
    pub fn is_angle_unit(&self, unit: &str) -> bool {
        matches!(unit, UNIT_DEG | UNIT_RAD | UNIT_GRAD | UNIT_TURN)
    }
    
    /// Check if a unit is a time unit
    pub fn is_time_unit(&self, unit: &str) -> bool {
        matches!(unit, UNIT_S | UNIT_MS)
    }
    
    /// Get all valid units
    pub fn get_all_units(&self) -> Vec<&str> {
        self.valid_units.iter().copied().collect()
    }
    
    /// Get units by category
    pub fn get_length_units(&self) -> Vec<&str> {
        vec![UNIT_PX, UNIT_PERCENT]
    }
    
    pub fn get_angle_units(&self) -> Vec<&str> {
        vec![UNIT_DEG, UNIT_RAD, UNIT_GRAD, UNIT_TURN]
    }
    
    pub fn get_time_units(&self) -> Vec<&str> {
        vec![UNIT_S, UNIT_MS]
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
    
    /// Get keyword information by name
    pub fn get_keyword_info(&self, keyword_name: &str) -> Option<&KeywordInfo> {
        self.keywords.get(keyword_name)
    }
    
    /// Check if a keyword is valid
    pub fn is_valid_keyword(&self, keyword_name: &str) -> bool {
        self.keywords.contains_key(keyword_name)
    }
    
    /// Get keyword documentation
    pub fn get_keyword_documentation(&self, keyword_name: &str) -> Option<&str> {
        self.keywords.get(keyword_name)
            .map(|info| info.doc)
    }
    
    /// Get all keyword names
    pub fn get_all_keyword_names(&self) -> Vec<&str> {
        self.keywords.keys().copied().collect()
    }
    
    /// Get all keywords with their information
    pub fn get_all_keywords(&self) -> &HashMap<&'static str, KeywordInfo> {
        &self.keywords
    }
    
    /// Get pseudo-class information by name
    pub fn get_pseudo_class_info(&self, pseudo_class_name: &str) -> Option<&PseudoClassInfo> {
        self.pseudo_classes.get(pseudo_class_name)
    }
    
    /// Get documentation URL for a pseudo-class with version formatting
    pub fn get_pseudo_class_documentation_url(&self, pseudo_class_name: &str, unity_version: &str) -> Option<String> {
        self.pseudo_classes.get(pseudo_class_name).map(|info| {
            info.documentation_url.replace("{version}", unity_version)
        })
    }
    
    /// Check if a pseudo-class is chainable with others
    pub fn is_pseudo_class_chainable(&self, pseudo_class_name: &str) -> bool {
        self.pseudo_classes.get(pseudo_class_name)
            .map(|info| info.chainable)
            .unwrap_or(false)
    }
    
    /// Get pseudo-class description
    pub fn get_pseudo_class_description(&self, pseudo_class_name: &str) -> Option<&str> {
        self.pseudo_classes.get(pseudo_class_name)
            .map(|info| info.description)
    }
    
    /// Get all pseudo-class names
    pub fn get_all_pseudo_class_names(&self) -> Vec<&str> {
        self.pseudo_classes.keys().copied().collect()
    }
    
    /// Get all pseudo-classes with their information
    pub fn get_all_pseudo_classes(&self) -> &HashMap<&'static str, PseudoClassInfo> {
        &self.pseudo_classes
    }
}

impl Default for UssDefinitions {
    fn default() -> Self {
        Self::new()
    }
}

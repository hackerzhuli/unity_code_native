//! USS Definitions
//!
//! Contains USS property definitions, pseudo-classes, color keywords,
//! and other validation data that can be shared across different features
//! like diagnostics and autocomplete.

use std::collections::HashMap;
use std::collections::HashSet;

/// USS language definitions and validation data
pub struct UssDefinitions {
    /// Valid USS properties
    pub valid_properties: HashSet<&'static str>,
    /// Valid pseudo-classes
    pub valid_pseudo_classes: HashSet<&'static str>,
    /// Valid CSS color keywords with their hex values
    pub valid_color_keywords: HashMap<&'static str, &'static str>,
    /// Valid USS functions
    pub valid_functions: HashSet<&'static str>,
    /// Valid at-rules
    pub valid_at_rules: HashSet<&'static str>,
}

impl UssDefinitions {
    /// Create a new USS definitions instance
    pub fn new() -> Self {
        let mut valid_properties = HashSet::new();
        
        // Standard CSS properties supported by USS
        let standard_properties = [
            "align-content", "align-items", "align-self", "all",
            "background-color", "background-image", "background-position", 
            "background-position-x", "background-position-y", "background-repeat", "background-size",
            "border-bottom-color", "border-bottom-left-radius", "border-bottom-right-radius", "border-bottom-width",
            "border-color", "border-left-color", "border-left-width", "border-radius",
            "border-right-color", "border-right-width", "border-top-color", "border-top-left-radius",
            "border-top-right-radius", "border-top-width", "border-width",
            "bottom", "color", "cursor", "display",
            "flex", "flex-basis", "flex-direction", "flex-grow", "flex-shrink", "flex-wrap",
            "font-size", "height", "justify-content", "left", "letter-spacing",
            "margin", "margin-bottom", "margin-left", "margin-right", "margin-top",
            "max-height", "max-width", "min-height", "min-width",
            "opacity", "overflow", "padding", "padding-bottom", "padding-left", "padding-right", "padding-top",
            "position", "right", "rotate", "scale", "text-overflow", "text-shadow",
            "top", "transform-origin", "translate", "visibility", "white-space", "width", "word-spacing"
        ];
        
        // Unity-specific properties
        let unity_properties = [
            "-unity-background-image-tint-color", "-unity-background-scale-mode",
            "-unity-font", "-unity-font-definition", "-unity-font-style",
            "-unity-paragraph-spacing", "-unity-slice-bottom", "-unity-slice-left",
            "-unity-slice-right", "-unity-slice-scale", "-unity-slice-top", "-unity-slice-type",
            "-unity-text-align", "-unity-text-generator", "-unity-text-outline",
            "-unity-text-outline-color", "-unity-text-outline-width", "-unity-text-overflow-position"
        ];
        
        for prop in standard_properties.iter().chain(unity_properties.iter()) {
            valid_properties.insert(*prop);
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
        let functions = ["url", "resource", "var", "rgb", "rgba"];
        for func in functions {
            valid_functions.insert(func);
        }
        
        let mut valid_at_rules = HashSet::new();
        let at_rules = ["@import"];
        for rule in at_rules {
            valid_at_rules.insert(rule);
        }
        
        Self {
            valid_properties,
            valid_pseudo_classes,
            valid_color_keywords,
            valid_functions,
            valid_at_rules,
        }
    }
    
    /// Check if a property name is valid
    pub fn is_valid_property(&self, property_name: &str) -> bool {
        // Allow custom properties (CSS variables)
        if property_name.starts_with("--") {
            return true;
        }
        
        self.valid_properties.contains(property_name)
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
    
    /// Get valid values for a specific property
    pub fn get_valid_values_for_property(&self, property_name: &str) -> Option<&'static [&'static str]> {
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
        if let Some(valid_values) = self.get_valid_values_for_property(property_name) {
            valid_values.contains(&value)
        } else {
            // For properties without specific validation, allow any value
            // This includes numeric values, custom values, etc.
            true
        }
    }
}

impl Default for UssDefinitions {
    fn default() -> Self {
        Self::new()
    }
}
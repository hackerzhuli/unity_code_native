//! USS Definitions
//!
//! Contains USS property definitions, pseudo-classes, color keywords,
//! and other validation data that can be shared across different features
//! like diagnostics and autocomplete.

use crate::uss::color::Color;
use crate::uss::color_keywords::create_color_keywords;
use crate::uss::constants::*;
use crate::uss::keyword_data::create_keyword_info;
use crate::uss::property_data::create_standard_properties;
use crate::uss::value_spec::{ValueSpec, ValueType};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Information about a USS keyword
#[derive(Debug, Clone)]
pub struct KeywordInfo {
    /// The keyword name
    pub name: &'static str,
    /// Markdown documentation for the keyword (default)
    doc: &'static str,
    /// from what properties are these keywords used by
    /// if a keyword is used by all properties
    pub used_by_properties: HashSet<&'static str>,
    /// docs when the keyword is used by specific property, this should override the doc fileld for a specific property
    /// this is used only when a keyword means different thing in different properties
    docs_for_property: HashMap<&'static str, &'static str>,
}

impl KeywordInfo {
    /// Create a new KeywordInfo
    pub fn new(name: &'static str, doc: &'static str) -> Self {
        Self { name, doc, used_by_properties: HashSet::new(), docs_for_property: HashMap::new() }
    }

    /// Create a new KeywordInfo
    pub fn new_with_property_docs(name: &'static str, doc: &'static str, used_by_properties: HashSet<&'static str>, docs_for_property: HashMap<&'static str, &'static str>) -> Self {
        Self { name, doc, used_by_properties, docs_for_property}
    }

    /// Create markdown documentation for the keyword
    /// If `property_name` is provided and property-specific documentation exists, returns that.
    /// Otherwise returns the default documentation.
    pub fn create_documentation(&self, property_name: Option<&str>) -> String {
        let mut content = format!("### Keyword `{}`\n", self.name);
        
        // Use property-specific documentation if available and requested
        if let Some(prop_name) = property_name {
            if let Some(property_doc) = self.docs_for_property.get(prop_name) {
                content.push_str(property_doc);
                content.push_str(&format!("\n\n*Used with property: `{}`*", prop_name));
                return content;
            }
        }
        
        // Use default documentation
        content.push_str(self.doc);
        
        // Add list of properties that use this keyword
        if !self.used_by_properties.is_empty() {
            let mut properties: Vec<&str> = self.used_by_properties.iter().copied().collect();
            properties.sort();
            
            content.push_str("\n\n**Used by properties:**\n");
            
            // Show first 10 properties, then "..." if there are more
            let display_count = std::cmp::min(properties.len(), 10);
            for property in &properties[..display_count] {
                content.push_str(&format!("- `{}`\n", property));
            }
            
            if properties.len() > 10 {
                content.push_str("- ...\n");
            }
        }
        
        content
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PropertyAnimation {
    None,
    Animatable,
    Discrete,
}

/// Property documentation information
#[derive(Debug, Clone)]
pub struct PropertyInfo {
    /// Property name
    pub name: &'static str,
    /// Property description, as a markdown string
    /// Must contain the string from Unity docs official property reference table
    /// This is because we have automatic tests that will verify this contains what the official table say
    pub description: &'static str,
    /// Examples from Unity docs
    pub examples_unity: Option<&'static str>,
    /// Examples from Mozilla docs
    pub examples_mozilla: Option<&'static str>,
    /// Official format specification from Unity or Mozzila docs
    ///
    /// We want to be the same as offcial docs, don't try to fix them, EVEN IF THEY ARE WRONG.
    ///
    /// For formats from Unity docs, MUST BE KEPT VERBATIM, because we have automatic tests that will verify this matches what official docs say.
    /// Also, this prevents mistakes from our side.
    ///
    /// Note: a few properties have formats that is written by us.
    pub format: &'static str,
    /// Documentation URL (may contain {version} placeholder for Unity docs)
    pub documentation_url: String,
    /// Whether this property is inherited
    pub inherited: bool,
    /// Whether this property is animatable
    pub animatable: PropertyAnimation,
    /// Complete value specification for this property
    /// Note: All properties support initial keyword to reset to default, we don't put initial in here for brevity
    pub value_spec: ValueSpec,
}

impl PropertyInfo {
    /// Create full markdown documentation with version-specific URL and property characteristics
    pub fn create_documentation(&self, property_name: &str, unity_version: &str) -> String {
        let doc_url = self.documentation_url.replace("{version}", unity_version);

        let mut content = format!("### Property {}\n", property_name);
        content.push_str(&format!("{}", self.description));

        // Add format specification
        content.push_str(&format!("\n\n**Format:** `{}`", self.format));

        // Add Unity examples if available
        if let Some(unity_examples) = self.examples_unity {
            content.push_str("\n\n**Examples from Unity docs:**\n```css\n");
            content.push_str(unity_examples);
            content.push_str("\n```");
        }
        // Add Mozilla examples if available
        else if let Some(mozilla_examples) = self.examples_mozilla {
            content.push_str("\n\n**Examples from Mozilla docs:**\n```css\n");
            content.push_str(mozilla_examples);
            content.push_str("\n```");
            content.push_str("\n\nNote: since these examples are from Mozilla docs, some of them may not work in Unity Engine.");
        }

        // Add property characteristics
        let mut characteristics = Vec::new();
        if self.inherited {
            characteristics.push("Inherited");
        }
        match self.animatable {
            PropertyAnimation::None => {}
            PropertyAnimation::Animatable => {
                characteristics.push("Animatable");
            }
            PropertyAnimation::Discrete => {
                characteristics.push("Discrete animatable");
            }
        }

        if !characteristics.is_empty() {
            content.push_str(&format!("\n\n*{}*", characteristics.join(", ")));
        }

        // Add documentation link
        content.push_str(&format!("\n\n[📖 Documentation]({})", doc_url));

        content
    }
}

/// Pseudo-class documentation information
#[derive(Debug, Clone)]
pub struct PseudoClassInfo {
    /// Pseudo-class name (without the colon prefix)
    pub name: &'static str,
    /// Description of this pseudo-class, in markdown format
    pub description: &'static str,
    /// Documentation URL (may contain {version} placeholder for Unity docs)
    pub documentation_url: String,
}

impl PseudoClassInfo {
    /// Create full markdown documentation with version-specific URL
    pub fn create_documentation(&self, unity_version: &str) -> String {
        let doc_url = self.documentation_url.replace("{version}", unity_version);
        format!(
            "### Pseudo Class: {}\n{}\n\n[Documentation]({})",
            self.name, self.description, doc_url
        )
    }
}

/// Create pseudo-class information with documentation
fn create_pseudo_class_info() -> HashMap<&'static str, PseudoClassInfo> {
    let mut pseudo_classes = HashMap::new();

    pseudo_classes.insert("hover", PseudoClassInfo {
        name: "hover",
        description: "Matches an element when the cursor is positioned over the element.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
    });

    pseudo_classes.insert("active", PseudoClassInfo {
        name: "active",
        description: "Matches an element when a user interacts with the element.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
    });

    pseudo_classes.insert("inactive", PseudoClassInfo {
        name: "inactive",
        description: "Matches an element when a user stops to interact with the element.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
    });

    pseudo_classes.insert("focus", PseudoClassInfo {
        name: "focus",
        description: "Matches an element when the element has focus.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
    });

    pseudo_classes.insert("disabled", PseudoClassInfo {
        name: "disabled",
        description: "Matches an element when the element is in a disabled state.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
    });

    pseudo_classes.insert("enabled", PseudoClassInfo {
        name: "enabled",
        description: "Matches an element when the element is in an enabled state.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
    });

    pseudo_classes.insert("checked", PseudoClassInfo {
        name: "checked",
        description: "Matches an element when the element is a Toggle or RadioButton element and it's selected.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
    });

    pseudo_classes.insert("root", PseudoClassInfo {
        name: "root",
        description: "Matches an element when the element is the highest-level element in the visual tree that has the stylesheet applied.",
        documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-Selectors-Pseudo-Classes.html".to_string(),
    });

    pseudo_classes
}

/// USS language definitions and validation data
#[derive(Clone, Debug)]
pub struct UssDefinitions {
    /// USS properties with their metadata (lazy-loaded)
    ///
    /// Propeties are relatively expensive to create, so only create them when needed
    properties: OnceLock<HashMap<&'static str, PropertyInfo>>,
    /// USS keywords with their documentation (lazy-loaded)
    ///
    /// There are lots of keywords, so we only create them when needed
    keywords: OnceLock<HashMap<&'static str, KeywordInfo>>,
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
    ///
    /// Relatively lightweight, expensive fields are created when needed.
    pub fn new() -> Self {
        // Load pseudo-class information
        let mut pseudo_classes = HashMap::new();
        let mut valid_pseudo_classes = HashSet::new();

        let pseudo_class_data = create_pseudo_class_info();
        for (name, pseudo_info) in pseudo_class_data {
            pseudo_classes.insert(name, pseudo_info);
            valid_pseudo_classes.insert(name);
        }

        // Load color keywords
        let valid_color_keywords = create_color_keywords();

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
            UNIT_PX,
            UNIT_PERCENT,
            // Angle units
            UNIT_DEG,
            UNIT_RAD,
            UNIT_GRAD,
            UNIT_TURN,
            // Time units
            UNIT_S,
            UNIT_MS,
        ];
        for unit in units {
            valid_units.insert(unit);
        }

        Self {
            properties: OnceLock::new(),
            keywords: OnceLock::new(),
            pseudo_classes,
            valid_pseudo_classes,
            valid_color_keywords,
            valid_units,
        }
    }

    /// Get properties (lazy-loaded)
    fn get_properties(&self) -> &HashMap<&'static str, PropertyInfo> {
        self.properties.get_or_init(|| {
            let mut properties = HashMap::new();

            // Load standard CSS properties
            let standard_props = create_standard_properties();
            for (name, prop_info) in standard_props {
                properties.insert(name, prop_info);
            }

            properties
        })
    }

    /// Get keywords (lazy-loaded)
    fn get_keywords(&self) -> &HashMap<&'static str, KeywordInfo> {
        self.keywords.get_or_init(|| {
            create_keyword_info()
        })
    }

    /// Check if a property name is valid, ie, an existing property or a custom property (USS variable)
    pub fn is_valid_property(&self, property_name: &str) -> bool {
        // Allow custom properties (CSS variables)
        if property_name.starts_with("--") {
            return true;
        }

        self.get_properties().contains_key(property_name)
    }

    /// Check if a property is a predefined USS property (excludes custom CSS variables)
    /// This is used for features like hover that should only show info for predefined properties
    pub fn is_predefined_property(&self, property_name: &str) -> bool {
        self.get_properties().contains_key(property_name)
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
        self.get_properties().get(property_name)
    }

    /// Get all properties with their information
    pub fn get_all_properties(&self) -> &HashMap<&'static str, PropertyInfo> {
        self.get_properties()
    }

    /// Get keyword information by name
    pub fn get_keyword_info(&self, keyword_name: &str) -> Option<&KeywordInfo> {
        self.get_keywords().get(keyword_name)
    }

    /// Get all keywords with their information
    pub fn get_all_keywords(&self) -> &HashMap<&'static str, KeywordInfo> {
        self.get_keywords()
    }

    /// Get pseudo-class information by name
    pub fn get_pseudo_class_info(&self, pseudo_class_name: &str) -> Option<&PseudoClassInfo> {
        self.pseudo_classes.get(pseudo_class_name)
    }

    /// Get simple completions strings for property, just keywords, colors or other simple values that will work for the property
    ///
    /// Note: no spaces, no commas, just a single value
    ///
    /// Example: `red`, `translate`, `auto`, `row`
    pub fn get_simple_completions_for_property(&self, property: &str) -> Vec<&'static str> {
        // first look for single keywords that will work by looking at value spec
        let properties = self.get_properties();
        if !properties.contains_key(property) {
            return Vec::new();
        }

        let (property_name, property_info) = properties.get_key_value(property).unwrap();

        let mut set: HashSet<&'static str> = HashSet::new();
        // see if a single value entry would work
        for format in &property_info.value_spec.formats {
            if format.entries.len() == 1 {
                let entry = &format.entries[0];
                for option in &entry.options {
                    match option {
                        ValueType::Color => {
                            for (color, _) in &self.valid_color_keywords {
                                set.insert(color);
                            }
                        }
                        ValueType::Keyword(keyword) => {
                            set.insert(keyword);
                        }
                        ValueType::PropertyName => {
                            // we assume it is for an animation property here
                            // this is our only use case now
                            for (p, p_i) in properties {
                                if p_i.animatable != PropertyAnimation::None {
                                    set.insert(p);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let r: Vec<&'static str> = set.into_iter().collect();
        r
    }
}

impl Default for UssDefinitions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path ="definitions_tests.rs"]
mod definitions_tests;
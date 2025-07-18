use std::collections::HashMap;

use crate::uss::definitions::PseudoClassInfo;

/// Create pseudo-class information with documentation
pub fn create_pseudo_class_info() -> HashMap<&'static str, PseudoClassInfo> {

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

use std::collections::HashMap;
use crate::uss::definitions::FunctionInfo;

/// Create function information with documentation
pub fn create_function_info() -> HashMap<&'static str, FunctionInfo> {
    let mut functions = HashMap::new();

    functions.insert("url", FunctionInfo {
        name: "url",
        category: "Resource",
        description: "References an asset in the project with a url or path.",
        syntax: "url(\"path/to/asset\")",
        details: Some(r#"When you reference an asset with the url() function, the path you specify can be relative or absolute:
- Relative paths must be relative to the folder that contains the USS file that references the asset.
- Absolute paths are relative to the project."#),
    });

    functions.insert("resource", FunctionInfo {
        name: "resource",
        category: "Resource",
        description: "References an asset in one of the resource folders.",
        syntax: "resource(\"path/to/asset\")",
        details: Some(r#"The resource() function can reference assets in Unity's resource folders (`Resources` and `Editor Default Resources`). You reference an asset by name.
- If the file is in the Editor Default Resources folder, you must include the file extension.
- If the file is in the Resources folder, you must not include the file extension."#),
    });

    functions.insert("rgb", FunctionInfo {
        name: "rgb",
        category: "Color",
        description: "Defines a color using red, green, and blue values.",
        syntax: "rgb(red, green, blue)",
        details: Some("Each component can be a number (0-255)"),
    });

    functions.insert("rgba", FunctionInfo {
        name: "rgba",
        category: "Color",
        description: "Defines a color using red, green, blue, and alpha values.",
        syntax: "rgba(red, green, blue, alpha)",
        details: Some("RGB components can be numbers (0-255). Alpha is a decimal from 0.0 (transparent) to 1.0 (opaque)."),
    });

    functions.insert("var", FunctionInfo {
        name: "var",
        category: "Variable",
        description: "References a custom USS variable.",
        syntax: "var(--property-name, fallback)",
        details: Some("The fallback value is optional and used when the variable is not defined."),
    });

    functions
}
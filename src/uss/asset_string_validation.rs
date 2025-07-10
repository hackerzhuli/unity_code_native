//! Asset string validation utilities for USS
//!
//! This module provides validation functions for asset strings used in USS,
//! including URL and resource function arguments. These functions can be
//! tested independently without requiring a parser.
//!
//! uss docs for url and resource function: https://docs.unity3d.com/6000.1/Documentation/Manual/UIE-USS-PropertyTypes.html
//!

use std::cell::RefCell;
use url::{SyntaxViolation, Url};

/// Error type for asset string validation
#[derive(Debug, Clone, PartialEq)]
pub struct AssetValidationError {
    pub message: String,
}

impl AssetValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AssetValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Asset validation error: {}", self.message)
    }
}

impl std::error::Error for AssetValidationError {}

/// Validates a Unity USS URL function argument string
///
/// Unity USS url() function supports:
/// - Relative paths: "../Resources/thumb.png"
/// - Absolute paths: "/Assets/Editor/Resources/thumb.png"
/// - Project scheme: "project:/Assets/Editor/Resources/thumb.png" or "project:///Assets/Editor/Resources/thumb.png"
///
/// # Arguments
/// * `url_path` - The actual URL path string (already processed, no quotes or escapes)
///
/// # Returns
/// * `Ok(())` - If the URL path is valid for Unity USS
/// * `Err(AssetValidationError)` - If the URL path is invalid
///
/// # Examples
/// ```
/// use crate::uss::asset_string_validation::validate_url_string;
///
/// // Valid project scheme URLs
/// assert!(validate_url_string("project:/Assets/image.png").is_ok());
/// assert!(validate_url_string("project:///Assets/image.png").is_ok());
///
/// // Valid relative paths - parent directory navigation
/// assert!(validate_url_string("../Resources/image.png").is_ok());
/// assert!(validate_url_string("../../Textures/background.jpg").is_ok());
///
/// // Valid relative paths - current directory
/// assert!(validate_url_string("./image.png").is_ok());
/// assert!(validate_url_string("./subfolder/icon.svg").is_ok());
///
/// // Valid relative paths - simple filenames and subdirectories
/// assert!(validate_url_string("image.png").is_ok());
/// assert!(validate_url_string("Icons/button.png").is_ok());
/// assert!(validate_url_string("UI/Textures/background.jpg").is_ok());
///
/// // Valid absolute paths
/// assert!(validate_url_string("/Assets/Resources/image.png").is_ok());
/// assert!(validate_url_string("/Packages/com.unity.ui/Runtime/icon.png").is_ok());
///
/// // Invalid - non-project scheme
/// assert!(validate_url_string("https://example.com/image.png").is_err());
/// ```
pub fn validate_url_string(url_path: &str) -> Result<(), AssetValidationError> {
    // Check if the URL path is empty
    if url_path.is_empty() {
        return Err(AssetValidationError::new("URL cannot have empty path"));
    }

    let mut actual_url_path = url_path.to_string();

    // trim spaces(not whitespace, just space)
    // spaces in the frond and end are totally ignored by unity
    actual_url_path = actual_url_path.trim_matches(' ').to_string();

    // check it, if error, it can be a relative path so make it absolute
    let r = url::Url::parse(url_path);
    // relative path is invalid, so we need to try to make it work by making giving it a schema
    if r.is_err() {
        if actual_url_path.starts_with("//") {
            return Err(AssetValidationError::new("url can't start with //"));
        }
        else if actual_url_path.starts_with("/") {
            actual_url_path = format!("project:///Assets{}", actual_url_path);
        }else{
            actual_url_path = format!("project:///Assets/{}", actual_url_path);
        }
    }
    
    // Use url crate's parsing with syntax violation detection
    let mut violations = RefCell::new(Vec::new());

    // Try to parse as a full URL first (for project: scheme)
    let url_parse_result = Url::options()
        .syntax_violation_callback(Some(&|v| violations.borrow_mut().push(v)))
        .parse(actual_url_path.as_str());

    if !violations.get_mut().is_empty() {
        match violations.get_mut()[0] {
            SyntaxViolation::Backslash => {
                return Err(AssetValidationError::new("Invalid character: backslash"));
            }
            SyntaxViolation::C0SpaceIgnored => {
                // ignore
            },
            SyntaxViolation::EmbeddedCredentials => {
                return Err(AssetValidationError::new("embedding authentication information (username or password) in an URL is not valid"));
            },
            SyntaxViolation::ExpectedDoubleSlash => {
                // double slash is not acutally expected by Unity
                //return Err(AssetValidationError::new("expected double slash"));
            },
            SyntaxViolation::ExpectedFileDoubleSlash => {
                // ignore
            },
            SyntaxViolation::FileWithHostAndWindowsDrive => {
                // ignore
            },
            SyntaxViolation::NonUrlCodePoint => {
                return Err(AssetValidationError::new("Invalid character: non-URL code point"));
            },
            SyntaxViolation::NullInFragment => {
                return Err(AssetValidationError::new("NULL characters are ignored in URL fragment identifiers"));
            },
            SyntaxViolation::PercentDecode => {
                return Err(AssetValidationError::new("expected 2 hex digits after %"));
            },
            SyntaxViolation::TabOrNewlineIgnored => {
                // this, Unity will handle them not nicely, so report an error, it is not actually ignored
                return Err(AssetValidationError::new("tabs or newlines are ignored in URLs"));
            },
            SyntaxViolation::UnencodedAtSign => {
                return Err(AssetValidationError::new("unencoded @ sign in username or password"));
            },
            _ => {
                // don't know error, so no error
                //return Err(AssetValidationError::new("Invalid URL path"));
            }
        }
    }

    match url_parse_result {
        Ok(parsed_url) => {
            // Successfully parsed as URL - validate scheme
            let scheme = parsed_url.scheme();
            if scheme == "project" {
                // Validate project URL path
                let path = parsed_url.path();
                if path.is_empty() || path == "/" {
                    return Err(AssetValidationError::new(format!(
                        "Project URL '{}' is missing a path after the scheme. Use 'project:/Assets/...' or 'project:/Packages/...'",
                        url_path
                    )));
                }

                return Ok(());
            } else {
                return Err(AssetValidationError::new(format!(
                    "Invalid URL scheme '{}' - Unity USS only supports 'project:' scheme",
                    scheme
                )));
            }
        }
        Err(_) => {
            return Err(AssetValidationError::new("Invalid URL path"));
        }
    }
}

/// Validates a Unity USS resource function argument string
///
/// Unity USS resource() function references assets in Unity's resource folders:
/// - Resources folder: reference by name without file extension ("Images/my-image")
/// - Editor Default Resources folder: reference by name with file extension ("Images/my-image.png")
///
/// # Arguments
/// * `resource_path` - The actual resource path string (already processed, no quotes or escapes)
///
/// # Returns
/// * `Ok(())` - If the resource path is valid for Unity USS
/// * `Err(AssetValidationError)` - If the resource path is invalid
///
/// # Examples
/// ```
/// use crate::uss::asset_string_validation::validate_resource_string;
///
/// // Valid resource path (Resources folder - no extension)
/// assert!(validate_resource_string("Images/my-image").is_ok());
///
/// // Valid resource path (Editor Default Resources - with extension)
/// assert!(validate_resource_string("Images/default-image.png").is_ok());
///
/// // Empty resource (invalid)
/// assert!(validate_resource_string("").is_err());
/// ```
pub fn validate_resource_string(resource_path: &str) -> Result<(), AssetValidationError> {
    // Check if the resource path is empty
    if resource_path.is_empty() {
        return Err(AssetValidationError::new("Resource cannot have empty path"));
    }

    // Check for invalid characters in resource paths
    if let Some(invalid_char) = resource_path
        .chars()
        .find(|&c| c == '<' || c == '>' || c == '|' || c == '\0' || c == '\\')
    {
        let char_name = match invalid_char {
            '<' => "less-than symbol (<)",
            '>' => "greater-than symbol (>)",
            '|' => "pipe symbol (|)",
            '\0' => "null character",
            '\\' => "backslash (\\)",
            _ => "invalid character",
        };
        return Err(AssetValidationError::new(format!(
            "Resource path contains {} (\\u{:04X}) which is not allowed. Use forward slashes (/) for path separators and remove invalid characters",
            char_name, invalid_char as u32
        )));
    }

    // Check for problematic whitespace characters (Unity ignores leading/trailing spaces but fails with other whitespace)
    if let Some(invalid_char) = resource_path
        .chars()
        .find(|&c| c == '\t' || c == '\n' || c == '\r' || c == '\x0B' || c == '\x0C')
    {
        let char_name = match invalid_char {
            '\t' => "tab character",
            '\n' => "newline character",
            '\r' => "carriage return character",
            '\x0B' => "vertical tab character",
            '\x0C' => "form feed character",
            _ => "invalid whitespace character",
        };
        return Err(AssetValidationError::new(format!(
            "Resource path contains {} (\\u{:04X}). Unity cannot handle this whitespace - remove it or replace with regular spaces",
            char_name, invalid_char as u32
        )));
    }

    // Resource paths should not start with / or contain absolute path indicators
    if resource_path.starts_with('/') {
        return Err(AssetValidationError::new(format!(
            "Resource path '{}' starts with '/' which indicates an absolute path. Resource paths should be relative names like 'Images/icon' or 'Textures/background.png'",
            resource_path
        )));
    }
    if resource_path.starts_with("Assets/") || resource_path.starts_with("Packages/") {
        return Err(AssetValidationError::new(format!(
            "Resource path '{}' starts with '{}' which is for project URLs. Resource paths should reference files in Resources folders, like 'Images/icon' instead of 'Assets/Resources/Images/icon'",
            resource_path,
            if resource_path.starts_with("Assets/") {
                "Assets/"
            } else {
                "Packages/"
            }
        )));
    }

    // Resource paths should not contain URL schemes
    if resource_path.contains("://") {
        return Err(AssetValidationError::new(format!(
            "Resource path '{}' contains '://' which indicates a URL scheme. Resource paths should be simple names like 'Images/icon', not URLs",
            resource_path
        )));
    }
    if resource_path.starts_with("project:")
        || resource_path.starts_with("http:")
        || resource_path.starts_with("https:")
    {
        let scheme = if resource_path.starts_with("project:") {
            "project:"
        } else if resource_path.starts_with("http:") {
            "http:"
        } else {
            "https:"
        };
        return Err(AssetValidationError::new(format!(
            "Resource path '{}' starts with '{}' scheme. Use url() function for URLs, or provide a simple resource name like 'Images/icon'",
            resource_path, scheme
        )));
    }

    // Validate that the path doesn't contain invalid path components
    if resource_path.contains("../") {
        return Err(AssetValidationError::new(format!(
            "Resource path '{}' contains '../' which is not allowed. Resource paths should be direct names like 'Images/icon', not relative paths",
            resource_path
        )));
    }
    if resource_path.contains("./") {
        return Err(AssetValidationError::new(format!(
            "Resource path '{}' contains './' which is not allowed. Resource paths should be direct names like 'Images/icon', not relative paths",
            resource_path
        )));
    }

    // Check for consecutive slashes or trailing slashes
    if resource_path.contains("//") {
        return Err(AssetValidationError::new(format!(
            "Resource path '{}' contains consecutive slashes (//) which is invalid. Use single slashes to separate folder names",
            resource_path
        )));
    }
    if resource_path.ends_with('/') {
        return Err(AssetValidationError::new(format!(
            "Resource path '{}' ends with a slash which indicates a folder. Resource paths should point to files, like 'Images/icon' or 'Images/icon.png'",
            resource_path
        )));
    }

    Ok(())
}

/// Validates any asset string (generic validation)
///
/// # Arguments
/// * `asset_path` - The actual asset path string (already processed, no quotes or escapes)
/// * `asset_type` - The type of asset for error messages ("url", "resource", etc.)
///
/// # Returns
/// * `Ok(())` - If the asset path is valid
/// * `Err(AssetValidationError)` - If the asset path is invalid
pub fn validate_asset_string(
    asset_path: &str,
    asset_type: &str,
) -> Result<(), AssetValidationError> {
    // Check if the asset path is empty
    if asset_path.is_empty() {
        return Err(AssetValidationError::new(format!(
            "{} cannot have empty path",
            asset_type
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_validate_url_string_valid() {
        // Test valid Unity project scheme URLs
        assert!(validate_url_string("project:/Assets/image.png").is_ok());
        assert!(validate_url_string("project:///Assets/image.png").is_ok());
        assert!(validate_url_string("project:/Packages/com.example/image.png").is_ok());
        assert!(validate_url_string("project:/Packages:/com.example/image.png").is_err());

        // Test valid Unity absolute paths
        assert!(validate_url_string("/Assets/Resources/image.png").is_ok());
        assert!(validate_url_string("/Packages/com.example/image.png").is_ok());

        // Test valid relative paths - parent directory navigation
        assert!(validate_url_string("../Resources/image.png").is_ok());
        assert!(validate_url_string("../../Textures/background.jpg").is_ok());
        assert!(validate_url_string("../../../Assets/icon.svg").is_ok());

        // Test valid relative paths - current directory
        assert!(validate_url_string("./image.png").is_ok());
        assert!(validate_url_string("./subfolder/icon.svg").is_ok());
        assert!(validate_url_string("./UI/Buttons/submit.png").is_ok());

        // Test valid relative paths - simple filenames and subdirectories (no prefix)
        assert!(validate_url_string("image.png").is_ok());
        assert!(validate_url_string("Icons/button.png").is_ok());
        assert!(validate_url_string("UI/Textures/background.jpg").is_ok());
        assert!(validate_url_string("Fonts/arial.ttf").is_ok());
        assert!(validate_url_string("simple-filename.png").is_ok());
        assert!(validate_url_string("folder/subfolder/deep-file.svg").is_ok());

        // URL with spaces (should be encoded in real usage but path validation allows it)
        assert!(validate_url_string("my image.png").is_ok());
    }

    #[test]
    fn test_validate_url_string_with_special_chars() {
        // Test URL paths with valid special characters
        assert!(validate_url_string("image-with-dashes.png").is_ok());
        assert!(validate_url_string("image_with_underscores.png").is_ok());
        assert!(validate_url_string("image%20with%20encoded%20spaces.png").is_ok());

        // Test Unity project URLs with special characters
        assert!(validate_url_string("project:/Assets/UI/button-normal.png").is_ok());
        assert!(
            validate_url_string("project:/Packages/com.unity.ui/Runtime/button_hover.png").is_ok()
        );

        // Test paths with numbers and dots
        assert!(validate_url_string("../Resources/image.v2.png").is_ok());
        assert!(validate_url_string("/Assets/Textures/texture_001.jpg").is_ok());
    }

    #[test]
    fn test_validate_url_string_invalid() {
        // Test empty URL path
        let result = validate_url_string("");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("cannot have empty path")
        );

        // Test invalid URL schemes (Unity only supports project: scheme)
        assert!(validate_url_string("https://example.com/image.png").is_err());
        assert!(validate_url_string("http://localhost/image.png").is_err());
        assert!(validate_url_string("file:///C:/path/image.png").is_err());
        assert!(validate_url_string("ftp://server/image.png").is_err());

        // Test invalid project URLs
        assert!(validate_url_string("project:").is_err()); // Missing path
        assert!(validate_url_string("project:/").is_err()); // Empty path
        assert!(validate_url_string("project:invalid").is_err()); // Wrong format
        assert!(validate_url_string("project:/InvalidFolder/image.png").is_err()); // Should be Assets/ or Packages/

        // Test invalid absolute paths
        assert!(validate_url_string("/InvalidFolder/image.png").is_err()); // Should be /Assets/ or /Packages/

        // Test invalid characters with specific error messages
        let scheme_result = validate_url_string("://missing-scheme.com");
        assert!(scheme_result.is_err());
        let scheme_error = scheme_result.unwrap_err();
        assert!(
            scheme_error
                .message
                .contains("starts with '://' without a scheme")
        );

        let bracket_result = validate_url_string("invalid<>characters.png");
        assert!(bracket_result.is_err());
        let bracket_error = bracket_result.unwrap_err();
        assert!(
            bracket_error.message.contains("less-than symbol")
                || bracket_error.message.contains("greater-than symbol")
        );

        let pipe_result = validate_url_string("path|with|pipes.png");
        assert!(pipe_result.is_err());
        let pipe_error = pipe_result.unwrap_err();
        assert!(pipe_error.message.contains("pipe symbol"));

        let null_result = validate_url_string("path\x00with\x00nulls.png");
        assert!(null_result.is_err());
        let null_error = null_result.unwrap_err();
        assert!(null_error.message.contains("null character"));

        // Test invalid whitespace characters with specific error messages
        let tab_result = validate_url_string("path\twith\ttabs.png");
        assert!(tab_result.is_err());
        let tab_error = tab_result.unwrap_err();
        assert!(tab_error.message.contains("tab character"));
        assert!(tab_error.message.contains("\\u0009"));

        let newline_result = validate_url_string("path\nwith\nnewlines.png");
        assert!(newline_result.is_err());
        let newline_error = newline_result.unwrap_err();
        assert!(newline_error.message.contains("newline character"));
        assert!(newline_error.message.contains("\\u000A"));

        let carriage_result = validate_url_string("path\rwith\rcarriage.png");
        assert!(carriage_result.is_err());
        let carriage_error = carriage_result.unwrap_err();
        assert!(carriage_error.message.contains("carriage return character"));
        assert!(carriage_error.message.contains("\\u000D"));

        assert!(validate_url_string("path\x0Bwith\x0Bvertical.png").is_err());
        assert!(validate_url_string("path\x0Cwith\x0Cform.png").is_err());
    }

    #[test]
    fn test_url_crate_integration() {
        // Test that the url crate properly handles malformed project URLs
        // This demonstrates the improved robustness from using the url crate

        // Malformed project URL with extra slashes - url crate normalizes this
        assert!(validate_url_string("project:///Assets/image.png").is_ok());

        // Project URL without proper path separator - url crate handles this gracefully
        let result = validate_url_string("project:Assets/image.png");
        // The url crate parses this as a valid URL, so we accept it
        assert!(result.is_ok());

        // Test that relative paths are properly validated through url joining
        assert!(validate_url_string("../valid/path.png").is_ok());
        assert!(validate_url_string("./valid/path.png").is_ok());
        assert!(validate_url_string("valid/path.png").is_ok());

        // Test that truly malformed URLs are still caught
        assert!(validate_url_string("project:").is_err());
        assert!(validate_url_string("://invalid").is_err());
    }

    #[test]
    fn test_validate_resource_string_valid() {
        // Test valid resource paths (Resources folder - no extension)
        assert!(validate_resource_string("image").is_ok());
        assert!(validate_resource_string("Images/logo").is_ok());
        assert!(validate_resource_string("UI/button").is_ok());
        assert!(validate_resource_string("Textures/background").is_ok());

        // Test valid resource paths (Editor Default Resources - with extension)
        assert!(validate_resource_string("Images/default-image.png").is_ok());
        assert!(validate_resource_string("Icons/folder.png").is_ok());
        assert!(validate_resource_string("Fonts/arial.ttf").is_ok());
    }

    #[test]
    fn test_validate_resource_string_with_special_chars() {
        // Test resource paths with valid special characters
        assert!(validate_resource_string("image-with-dashes").is_ok());
        assert!(validate_resource_string("image_with_underscores").is_ok());
        assert!(validate_resource_string("folder/image with spaces").is_ok());
        assert!(validate_resource_string("UI/button-normal").is_ok());

        // Test resource paths with numbers and dots (for Editor Default Resources)
        assert!(validate_resource_string("Icons/icon.v2.png").is_ok());
        assert!(validate_resource_string("Textures/texture_001.jpg").is_ok());
        assert!(validate_resource_string("Fonts/arial-bold.ttf").is_ok());
    }

    #[test]
    fn test_validate_resource_string_invalid() {
        // Test empty resource path
        let result = validate_resource_string("");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("cannot have empty path")
        );

        // Test invalid characters with specific error messages
        let bracket_result = validate_resource_string("invalid<>characters");
        assert!(bracket_result.is_err());
        let bracket_error = bracket_result.unwrap_err();
        assert!(
            bracket_error.message.contains("less-than symbol")
                || bracket_error.message.contains("greater-than symbol")
        );

        let pipe_result = validate_resource_string("path|with|pipes");
        assert!(pipe_result.is_err());
        let pipe_error = pipe_result.unwrap_err();
        assert!(pipe_error.message.contains("pipe symbol"));

        let null_result = validate_resource_string("path\x00with\x00nulls");
        assert!(null_result.is_err());
        let null_error = null_result.unwrap_err();
        assert!(null_error.message.contains("null character"));

        let backslash_result = validate_resource_string("path\\with\\backslashes");
        assert!(backslash_result.is_err());
        let backslash_error = backslash_result.unwrap_err();
        assert!(backslash_error.message.contains("backslash"));

        // Test invalid whitespace characters with specific error messages
        let tab_result = validate_resource_string("path\twith\ttabs");
        assert!(tab_result.is_err());
        let tab_error = tab_result.unwrap_err();
        assert!(tab_error.message.contains("tab character"));
        assert!(tab_error.message.contains("\\u0009"));

        let newline_result = validate_resource_string("path\nwith\nnewlines");
        assert!(newline_result.is_err());
        let newline_error = newline_result.unwrap_err();
        assert!(newline_error.message.contains("newline character"));
        assert!(newline_error.message.contains("\\u000A"));

        let carriage_result = validate_resource_string("path\rwith\rcarriage");
        assert!(carriage_result.is_err());
        let carriage_error = carriage_result.unwrap_err();
        assert!(carriage_error.message.contains("carriage return character"));
        assert!(carriage_error.message.contains("\\u000D"));

        assert!(validate_resource_string("path\x0Bwith\x0Bvertical").is_err());
        assert!(validate_resource_string("path\x0Cwith\x0Cform").is_err());

        // Test absolute path indicators (not allowed in resource paths)
        let abs_result = validate_resource_string("/Assets/image");
        assert!(abs_result.is_err());
        let abs_error = abs_result.unwrap_err();
        assert!(abs_error.message.contains("starts with '/'"));

        let assets_result = validate_resource_string("Assets/image");
        assert!(assets_result.is_err());
        let assets_error = assets_result.unwrap_err();
        assert!(assets_error.message.contains("starts with 'Assets/'"));

        let packages_result = validate_resource_string("Packages/image");
        assert!(packages_result.is_err());
        let packages_error = packages_result.unwrap_err();
        assert!(packages_error.message.contains("starts with 'Packages/'"));

        // Test URL schemes (not allowed in resource paths)
        let project_result = validate_resource_string("project:/Assets/image");
        assert!(project_result.is_err());
        let project_error = project_result.unwrap_err();
        assert!(project_error.message.contains("'project:' scheme"));

        let https_result = validate_resource_string("https://example.com/image");
        assert!(https_result.is_err());
        let https_error = https_result.unwrap_err();
        assert!(
            https_error
                .message
                .contains("'://' which indicates a URL scheme")
        );

        // Test relative path components (not allowed)
        let dotdot_result = validate_resource_string("../Images/image");
        assert!(dotdot_result.is_err());
        let dotdot_error = dotdot_result.unwrap_err();
        assert!(dotdot_error.message.contains("contains '../'"));

        let dot_result = validate_resource_string("./Images/image");
        assert!(dot_result.is_err());
        let dot_error = dot_result.unwrap_err();
        assert!(dot_error.message.contains("contains './'"));

        // Test invalid path formats
        let double_slash_result = validate_resource_string("Images//image");
        assert!(double_slash_result.is_err());
        let double_slash_error = double_slash_result.unwrap_err();
        assert!(double_slash_error.message.contains("consecutive slashes"));

        let trailing_slash_result = validate_resource_string("Images/");
        assert!(trailing_slash_result.is_err());
        let trailing_slash_error = trailing_slash_result.unwrap_err();
        assert!(trailing_slash_error.message.contains("ends with a slash"));
    }

    #[test]
    fn test_validate_asset_string_generic() {
        // Valid asset
        let result = validate_asset_string("asset.png", "custom");
        assert!(result.is_ok());

        // Empty asset with custom type
        let result = validate_asset_string("", "custom");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("custom cannot have empty path")
        );
    }

    #[test]
    fn test_special_characters() {
        // Test URL with special characters
        let result = validate_url_string("&B");
        assert!(result.is_ok());

        // Test resource with special characters
        let result = validate_resource_string("&Btest");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_display() {
        let error = AssetValidationError::new("Test error");
        assert_eq!(error.to_string(), "Asset validation error: Test error");
    }
}

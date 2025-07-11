//! Asset string validation utilities for USS
//!
//! This module provides validation functions for asset url used in USS/UXML,
//!
//! uss docs for url and resource function: https://docs.unity3d.com/6000.1/Documentation/Manual/UIE-USS-PropertyTypes.html
//!
//! Note that even though it looks like resource path in uss are all relative in official docs.
//! But actually it can be absolute just like url path, it works but it is not recommended.
//! So we treat resource path just like url path.

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

/// Validates a Unity USS `url()` or `resource()` argument or UXML's asset path
///
/// Unity USS url() function supports:
/// - Relative paths: "../Resources/thumb.png"
/// - Absolute paths: "/Assets/Editor/Resources/thumb.png"
/// - Project scheme: "project:/Assets/Editor/Resources/thumb.png" or "project:///Assets/Editor/Resources/thumb.png"
///
/// # Arguments
/// * `url_path` - The actual URL path string (already processed, no quotes or escapes)
/// * `base_url` - The base URL to resolve relative paths against (optional)
///
/// # Returns
/// * `Ok(Url)` - If the URL path is valid for Unity USS
/// * `Err(AssetValidationError)` - If the URL path is invalid
///
/// # Examples
/// ```
/// use unity_code_native::language::asset_url::validate_url;
/// use url::Url;
/// 
/// let base = Some(Url::parse("project:///Assets/UI/styles.uss").unwrap());
/// 
/// // Valid project scheme URLs
/// assert!(validate_url("project:/Assets/image.png", None).is_ok());
/// assert!(validate_url("project:///Assets/image.png", None).is_ok());
///
/// // Valid relative paths - parent directory navigation
/// assert!(validate_url("../Resources/image.png", base.as_ref()).is_ok());
/// assert!(validate_url("../../Textures/background.jpg", base.as_ref()).is_ok());
///
/// // Valid relative paths - current directory
/// assert!(validate_url("./image.png", base.as_ref()).is_ok());
/// assert!(validate_url("./subfolder/icon.svg", base.as_ref()).is_ok());
///
/// // Valid absolute paths
/// assert!(validate_url("/Assets/Resources/image.png", None).is_ok());
/// assert!(validate_url("/Packages/com.unity.ui/Runtime/icon.png", None).is_ok());
///
/// // Invalid - non-project scheme
/// assert!(validate_url("https://example.com/image.png", None).is_err());
/// ```
pub fn validate_url(url: &str, base_url: Option<&Url>) -> Result<Url, AssetValidationError> {
    // Check if the URL path is empty
    if url.is_empty() {
        return Err(AssetValidationError::new("URL cannot be empty"));
    }

    // Use provided base URL or create a default one for relative path resolution
    let default_base = url::Url::parse("project:///Assets/a/b/c/d/e/f/g/h/i/j/k/i/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z");
    
    let effective_base = match base_url {
        Some(base) => base.clone(),
        None => {
            if default_base.is_err() {
                // this should not happen, but we don't want to panic here
                return Err(AssetValidationError::new("Unknown error"));
            }
            default_base.unwrap()
        }
    };

    let parse_result = effective_base.join(url);

    match parse_result {
        Ok(parsed_url) => {
            // we need to be more strict here, and find errors if it parsed ok
            if let Some(value) = additional_error(url) {
                return Err(value);
            }

            // Successfully parsed as URL - validate scheme
            let scheme = parsed_url.scheme();
            if scheme == "project" {
                // Validate project URL path
                let path = parsed_url.path();
                if path.is_empty() || path == "/" {
                    return Err(AssetValidationError::new(format!(
                        "Project URL '{}' is missing a path after the scheme. Use 'project:/Assets/...' or 'project:/Packages/...'",
                        url
                    )));
                }

                // make sure url doesn't have anything like authority because it is not supported
                if parsed_url.authority().len() > 0 {
                    return Err(AssetValidationError::new(format!("URL should not have authority: {}", parsed_url.authority())));
                }

                // path must be absolute
                if !path.starts_with("/") {
                    return Err(AssetValidationError::new(format!("path must be absolute:`{}`, consider add a `/`", path)));
                }

                if !path.starts_with("/Assets/") && !path.starts_with("/Packages") {
                    return Err(AssetValidationError::new(format!("Asset path should start with `/Assets/` or `/Packages/` :`{}`, this is likely an error", path)));
                }

                Ok(parsed_url)
            } else {
                Err(AssetValidationError::new(format!(
                    "Invalid URL scheme '{}' - Unity only supports `project` scheme",
                    scheme
                )))
            }
        }
        Err(err) => {
            match err {
                _=>Err(AssetValidationError::new(format!("Invalid url err: {}", err))),
            }
        }
    }
}

/// checks less problematic errors
fn additional_error(url_path: &str) -> Option<AssetValidationError> {
    let mut violations = RefCell::new(Vec::new());
    let _parse_result2
        = Url::options()
        .syntax_violation_callback(Some(&|v| violations.borrow_mut().push(v)))
        .parse(url_path);
    // Use url crate's parsing with syntax violation detection

    // Try to parse as a full URL first (for project: scheme)

    if !violations.get_mut().is_empty() {
        match violations.get_mut()[0] {
            SyntaxViolation::Backslash => {
                return Some(AssetValidationError::new("Invalid character: backslash"));
            }
            SyntaxViolation::C0SpaceIgnored => {
                // ignore
            },
            SyntaxViolation::EmbeddedCredentials => {
                return Some(AssetValidationError::new("embedding authentication information (username or password) in an URL is not valid"));
            },
            SyntaxViolation::ExpectedDoubleSlash => {
                // double slash is not actually expected by Unity
                //return Err(AssetValidationError::new("expected double slash"));
            },
            SyntaxViolation::ExpectedFileDoubleSlash => {
                // ignore
            },
            SyntaxViolation::FileWithHostAndWindowsDrive => {
                // ignore
            },
            SyntaxViolation::NonUrlCodePoint => {
                return Some(AssetValidationError::new("Invalid character: non-URL code point"));
            },
            SyntaxViolation::NullInFragment => {
                return Some(AssetValidationError::new("NULL characters are ignored in URL fragment identifiers"));
            },
            SyntaxViolation::PercentDecode => {
                return Some(AssetValidationError::new("expected 2 hex digits after %"));
            },
            SyntaxViolation::TabOrNewlineIgnored => {
                // this, Unity will handle them not correctly, so report an error, it is not actually ignored
                return Some(AssetValidationError::new("tabs or newlines are ignored in URLs"));
            },
            SyntaxViolation::UnencodedAtSign => {
                return Some(AssetValidationError::new("unencoded @ sign in username or password"));
            },
            _ => {
                // don't know error, so no error
                //return Err(AssetValidationError::new("Invalid URL path"));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_validate_url_string_valid() {
        // Test valid Unity project scheme URLs
        assert!(validate_url("project:/Assets/image.png", None).is_ok());
        assert!(validate_url("project:///Assets/image.png", None).is_ok());
        assert!(validate_url("project:/Packages/com.example/image.png", None).is_ok());
        // colon in the middle is actually valid url
        assert!(validate_url("project:/Packages:/com.example/image.png", None).is_ok());

        // Test valid Unity absolute paths
        assert!(validate_url("/Assets/Resources/image.png", None).is_ok());
        assert!(validate_url("/Packages/com.example/image.png", None).is_ok());

        // Test valid relative paths - parent directory navigation
        assert!(validate_url("../Resources/image.png", None).is_ok());
        assert!(validate_url("../../Textures/background.jpg", None).is_ok());
        assert!(validate_url("../../../Assets/icon.svg", None).is_ok());

        // Test valid relative paths - current directory
        assert!(validate_url("./image.png", None).is_ok());
        assert!(validate_url("./subfolder/icon.svg", None).is_ok());
        assert!(validate_url("./UI/Buttons/submit.png", None).is_ok());

        // Test valid relative paths - simple filenames and subdirectories (no prefix)
        assert!(validate_url("image.png", None).is_ok());
        assert!(validate_url("Icons/button.png", None).is_ok());
        assert!(validate_url("UI/Textures/background.jpg", None).is_ok());
        assert!(validate_url("Fonts/arial.ttf", None).is_ok());
        assert!(validate_url("simple-filename.png", None).is_ok());
        assert!(validate_url("folder/subfolder/deep-file.svg", None).is_ok());

        // URL with spaces (should be encoded in real usage but path validation allows it)
        assert!(validate_url("my image.png", None).is_ok());
    }

    #[test]
    fn test_validate_url_string_with_special_chars() {
        // Test URL paths with valid special characters
        assert!(validate_url("image-with-dashes.png", None).is_ok());
        assert!(validate_url("image_with_underscores.png", None).is_ok());
        assert!(validate_url("image%20with%20encoded%20spaces.png", None).is_ok());

        // Test Unity project URLs with special characters
        assert!(validate_url("project:/Assets/UI/button-normal.png", None).is_ok());
        assert!(
            validate_url("project:/Packages/com.unity.ui/Runtime/button_hover.png", None).is_ok()
        );

        // Test paths with numbers and dots
        assert!(validate_url("../Resources/image.v2.png", None).is_ok());
        assert!(validate_url("/Assets/Textures/texture_001.jpg", None).is_ok());
    }

    #[test]
    fn test_validate_url_string_invalid() {
        // Test empty URL path
        let result = validate_url("", None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("empty")
        );

        // Test invalid URL schemes (Unity only supports project: scheme)
        assert!(validate_url("https://example.com/image.png", None).is_err());
        assert!(validate_url("http://localhost/image.png", None).is_err());
        assert!(validate_url("file:///C:/path/image.png", None).is_err());
        assert!(validate_url("ftp://server/image.png", None).is_err());

        // Test invalid project URLs
        assert!(validate_url("project:", None).is_err()); // Missing path
        assert!(validate_url("project:/", None).is_err()); // Empty path
        assert!(validate_url("project:invalid", None).is_err());
        assert!(validate_url("project:/InvalidFolder/image.png", None).is_err()); // Should be Assets/ or Packages/

        // Test invalid absolute paths
        assert!(validate_url("/InvalidFolder/image.png", None).is_err()); // Should be /Assets/ or /Packages/


        // Test invalid whitespace characters with specific error messages
        let tab_result = validate_url("path\twith\ttabs.png", None);
        assert!(tab_result.is_err());
        let tab_error = tab_result.unwrap_err();
        assert!(tab_error.message.contains("tab"));

        let newline_result = validate_url("path\nwith\nnewlines.png", None);
        assert!(newline_result.is_err());
        let newline_error = newline_result.unwrap_err();
        assert!(newline_error.message.contains("newline"));

        let carriage_result = validate_url("path\rwith\rcarriage.png", None);
        assert!(carriage_result.is_err());
        let carriage_error = carriage_result.unwrap_err();
        assert!(carriage_error.message.contains("newline"));
    }

    #[test]
    fn test_url_crate_integration() {
        // Test that the url crate properly handles malformed project URLs
        // This demonstrates the improved robustness from using the url crate

        // Malformed project URL with extra slashes - url crate normalizes this
        assert!(validate_url("project:///Assets/image.png", None).is_ok());

        // Requires absolute path
        let result = validate_url("project:Assets/image.png", None);
        assert!(result.is_err());

        // Test that relative paths are properly validated through url joining
        assert!(validate_url("../valid/path.png", None).is_ok());
        assert!(validate_url("./valid/path.png", None).is_ok());
        assert!(validate_url("valid/path.png", None).is_ok());

        // Test that truly malformed URLs are still caught
        assert!(validate_url("project:", None).is_err());
        // this is actually valid url because it is treated as a relative path(no scheme), and colon and double slash in the middle is valid url
        assert!(validate_url("://invalid", None).is_ok());
    }

    #[test]
    fn test_error_display() {
        let error = AssetValidationError::new("Test error");
        assert_eq!(error.to_string(), "Asset validation error: Test error");
    }
}

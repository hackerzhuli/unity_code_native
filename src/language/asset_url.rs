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

/// Validates a Unity USS `url()` or `resource()` argument or UXML's asset path
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

    // base url is needed to make relative url valid, otherwise there will be a parse error
    // note that if relative path contains lots of ..
    // we are going to run into a problem, it will eventually go up and up until the result url is no longer in Assets and report an error
    // so we need to add many layers to keep it in Assets, it should work in 99.99% cases
    let base_url = url::Url::parse("project:///Assets/a/b/c/d/e/f/g/h/i/j/k/i/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z");

    if base_url.is_err() {
        // this should not happen, but we don't want to panic here
        return Err(AssetValidationError::new("Unknown error"));
    }

    let parse_result = base_url.unwrap().join(url_path);

    // we need to be more strict here, and find errors if it parsed ok
    if parse_result.is_ok(){
        // Use url crate's parsing with syntax violation detection
        let mut violations = RefCell::new(Vec::new());

        // Try to parse as a full URL first (for project: scheme)
        let _parse_result2
            = Url::options()
            .syntax_violation_callback(Some(&|v| violations.borrow_mut().push(v)))
            .parse(url_path);

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
    }

    match parse_result {
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

                // path must be absolute
                if !path.starts_with("/") {
                    return Err(AssetValidationError::new(format!("path after scheme should be absolute:`{}`, consider add a `/`", path)));
                }

                if !path.starts_with("/Assets/") && !path.starts_with("/Packages") {
                    return Err(AssetValidationError::new(format!("Path should start with `/Assets/` or `Packages` :`{}`, it is likely an error", path)));
                }

                Ok(())
            } else {
                Err(AssetValidationError::new(format!(
                    "Invalid URL scheme '{}' - Unity only supports `project` scheme",
                    scheme
                )))
            }
        }
        Err(err) => {
            match err {
                // ParseError::EmptyHost => {}
                // ParseError::IdnaError => {}
                // ParseError::InvalidPort => {}
                // ParseError::InvalidIpv4Address => {}
                // ParseError::InvalidIpv6Address => {}
                // ParseError::InvalidDomainCharacter => {}
                // ParseError::RelativeUrlWithoutBase => {}
                // ParseError::RelativeUrlWithCannotBeABaseBase => {}
                // ParseError::SetHostOnCannotBeABaseUrl => {}
                //ParseError::Overflow => {}
                _=>Err(AssetValidationError::new("err"))
            }
        }
    }
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
        // colon in the middle is actually valid url
        assert!(validate_url_string("project:/Packages:/com.example/image.png").is_ok());

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
        assert!(validate_url_string("project:invalid").is_err());
        assert!(validate_url_string("project:/InvalidFolder/image.png").is_err()); // Should be Assets/ or Packages/

        // Test invalid absolute paths
        assert!(validate_url_string("/InvalidFolder/image.png").is_err()); // Should be /Assets/ or /Packages/


        // Test invalid whitespace characters with specific error messages
        let tab_result = validate_url_string("path\twith\ttabs.png");
        assert!(tab_result.is_err());
        let tab_error = tab_result.unwrap_err();
        assert!(tab_error.message.contains("tab"));

        let newline_result = validate_url_string("path\nwith\nnewlines.png");
        assert!(newline_result.is_err());
        let newline_error = newline_result.unwrap_err();
        assert!(newline_error.message.contains("newline"));

        let carriage_result = validate_url_string("path\rwith\rcarriage.png");
        assert!(carriage_result.is_err());
        let carriage_error = carriage_result.unwrap_err();
        assert!(carriage_error.message.contains("newline"));
    }

    #[test]
    fn test_url_crate_integration() {
        // Test that the url crate properly handles malformed project URLs
        // This demonstrates the improved robustness from using the url crate

        // Malformed project URL with extra slashes - url crate normalizes this
        assert!(validate_url_string("project:///Assets/image.png").is_ok());

        // Requires absolute path
        let result = validate_url_string("project:Assets/image.png");
        assert!(result.is_err());

        // Test that relative paths are properly validated through url joining
        assert!(validate_url_string("../valid/path.png").is_ok());
        assert!(validate_url_string("./valid/path.png").is_ok());
        assert!(validate_url_string("valid/path.png").is_ok());

        // Test that truly malformed URLs are still caught
        assert!(validate_url_string("project:").is_err());
        // this is actually valid url because it is treated as a relative path(no scheme), and colon and double slash in the middle is valid url
        assert!(validate_url_string("://invalid").is_ok());
    }

    #[test]
    fn test_error_display() {
        let error = AssetValidationError::new("Test error");
        assert_eq!(error.to_string(), "Asset validation error: Test error");
    }
}

//! Asset string validation utilities for USS
//!
//! This module provides validation functions for asset url used in USS/UXML,
//!
//! uss docs for url and resource function: https://docs.unity3d.com/6000.1/Documentation/Manual/UIE-USS-PropertyTypes.html
//!
//! Note that even though it looks like resource path in uss are all relative in official docs.
//! But actually it can be absolute just like url path, it works but it is not recommended.
//! So we treat resource path just like url path.

use std::{cell::RefCell, path::{Path, PathBuf}};
use url::{SyntaxViolation, Url};
use urlencoding::decode;

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

/// Warning type for asset string validation
#[derive(Debug, Clone, PartialEq)]
pub struct AssetValidationWarning {
    pub message: String,
}

impl AssetValidationWarning {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Result type for asset URL validation that includes the parsed URL and optional warnings
#[derive(Debug, Clone, PartialEq)]
pub struct AssetValidationResult {
    pub url: Url,
    pub warnings: Vec<AssetValidationWarning>,
}

impl AssetValidationResult {
    pub fn with_warnings(url: Url, warnings: Vec<AssetValidationWarning>) -> Self {
        Self {
            url,
            warnings,
        }
    }
}

/// Validates a Unity USS `url()` or `resource()` argument or UXML's asset path
///
/// Unity USS url() function supports:
/// - Relative paths: "../Resources/thumb.png"
/// - Absolute paths: "/Assets/Editor/Resources/thumb.png"
/// - Project scheme: "project:/Assets/Editor/Resources/thumb.png" or "project:///Assets/Editor/Resources/thumb.png"
///
/// # Arguments
/// * `url_path` - The actual URL path string (already processed, no quotes or escapes)
/// * `base_url` - The base URL to resolve relative paths against (optional), if not provided and url_path is relative, this will result in a path that is valid but does not really exist
///
/// # Returns
/// * `Ok(AssetValidationResult)` - If the URL path is valid for Unity USS, with optional warnings
/// * `Err(AssetValidationError)` - If the URL path is invalid
///
/// # Examples
/// ```
/// use unity_code_native::language::asset_url::validate_url;
/// use url::Url;
/// 
/// let base = Some(Url::parse("project:///Assets/UI/a/b/c").unwrap());
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
pub fn validate_url(url: &str, base_url: Option<&Url>) -> Result<AssetValidationResult, AssetValidationError> {
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
            // Check for additional issues that might be errors or warnings
            let mut warnings = Vec::new();
            
            // Check for errors first
            if let Some(additional_result) = additional_error(url, &effective_base) {
                match additional_result {
                    AdditionalValidationResult::Error(err) => return Err(err),
                    AdditionalValidationResult::Warning(warn) => warnings.push(warn),
                }
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

                // actually Unity does support authority field, that is totally valid
                // but it is just ignored
                // if parsed_url.authority().len() > 0 {
                //     return Err(AssetValidationError::new(format!("URL should not have authority: {}", parsed_url.authority())));
                // }

                // path must be absolute
                if !path.starts_with("/") {
                    return Err(AssetValidationError::new(format!("path must be absolute:`{}`, consider add a `/`", path)));
                }

                // should not validate that, we need to use url validation when user is still typing(eg. `/As`, not yet finish `/Assets/`), so, this validation is not needed
                // if !path.starts_with("/Assets/") && !path.starts_with("/Packages") {
                //     return Err(AssetValidationError::new(format!("Asset path should start with `/Assets/` or `/Packages/` :`{}`, this is likely an error", path)));
                // }

                Ok(AssetValidationResult::with_warnings(parsed_url, warnings))
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

/// Returns the absolute file path for a given URL.
/// the url should be in project scheme
pub fn project_url_to_path(project_root: &Path, url: &Url) -> Option<PathBuf>{
    if url.scheme() != "project" {
        return None;
    }

    if let Some(relative_path) = project_url_to_relative_path(url) {
        let mut p = PathBuf::new();
        p.push(project_root);
        p.push(relative_path.as_str()); 
        return Some(p);
    }
    
    None
}

/// Returns the absolute file path for a given URL.
/// the url should be in project scheme
pub fn project_url_to_relative_path(url: &Url) -> Option<String>{
    if url.scheme() != "project" {
        return None;
    }

    if let Ok(relative_path) = decode(url.path()) {
        return Some(relative_path.chars().skip(1).collect()); // remove leading slash
    }
    
    None
}


/// Creates a project scheme URL from normalized file path and project root path
///
/// Converts a file system path to a Unity project scheme URL.
/// The file path should be within the Unity project directory.
/// Both paths should be normalized (canonicalized) by the caller for proper comparison.
///
/// # Arguments
/// * `normalized_file_path` - The normalized absolute file system path to the file
/// * `normalized_project_root` - The normalized absolute path to the Unity project root directory
///
/// # Returns
/// * `Ok(Url)` - A project scheme URL (e.g., "project:/Assets/file.uss")
/// * `Err(AssetValidationError)` - If the file is not within the project or paths are invalid
///
/// # Examples
/// ```
/// use unity_code_native::language::asset_url::create_project_url;
/// use std::path::Path;
/// 
/// let project_root = Path::new("C:/MyProject");
/// let file_path = Path::new("C:/MyProject/Assets/UI/styles.uss");
/// let url = create_project_url(file_path, project_root).unwrap();
/// assert_eq!(url.as_str(), "project:/Assets/UI/styles.uss");
/// ```
pub fn create_project_url(normalized_file_path: &std::path::Path, normalized_project_root: &std::path::Path) -> Result<Url, AssetValidationError> {
    // Ensure both paths are absolute
    if !normalized_file_path.is_absolute() {
        return Err(AssetValidationError::new("File path must be absolute"));
    }
    if !normalized_project_root.is_absolute() {
        return Err(AssetValidationError::new("Project root path must be absolute"));
    }
    
    // Get the relative path from project root to file
    let relative_path = normalized_file_path.strip_prefix(normalized_project_root)
        .map_err(|_| AssetValidationError::new("File is not within the project directory"))?;
    
    // Convert to forward slashes for URL
    let relative_path_str = relative_path.to_string_lossy().replace('\\', "/");
    
    // Create project scheme URL
    let project_url = format!("project:/{}", relative_path_str);
    
    // Parse and validate the URL
    Url::parse(&project_url)
        .map_err(|e| AssetValidationError::new(format!("Failed to create project URL: {}", e)))
}

/// Creates a project scheme URL from file path and project root path with automatic normalization
///
/// This is a convenience function that handles path normalization automatically.
/// For better performance when dealing with multiple paths, consider normalizing paths
/// once and using `create_project_url` directly.
///
/// # Arguments
/// * `file_path` - The absolute file system path to the file
/// * `project_root` - The absolute path to the Unity project root directory
///
/// # Returns
/// * `Ok(Url)` - A project scheme URL (e.g., "project:/Assets/file.uss")
/// * `Err(AssetValidationError)` - If the file is not within the project or paths are invalid
pub fn create_project_url_with_normalization(file_path: &std::path::Path, project_root: &std::path::Path) -> Result<Url, AssetValidationError> {
    // Canonicalize both paths to handle case-insensitive comparison and resolve symlinks
    let canonical_file_path = std::fs::canonicalize(file_path)
        .map_err(|e| AssetValidationError::new(format!("Failed to canonicalize file path: {}", e)))?;
    let canonical_project_root = std::fs::canonicalize(project_root)
        .map_err(|e| AssetValidationError::new(format!("Failed to canonicalize project root: {}", e)))?;
    
    create_project_url(&canonical_file_path, &canonical_project_root)
}

/// Result type for additional validation that can be either an error or warning
#[derive(Debug, Clone, PartialEq)]
enum AdditionalValidationResult {
    Error(AssetValidationError),
    Warning(AssetValidationWarning),
}

/// checks less problematic errors
fn additional_error(url_path: &str, base_url: &Url) -> Option<AdditionalValidationResult> {
    let mut violations = RefCell::new(Vec::new());
    let parsed
        = Url::options()
        .syntax_violation_callback(Some(&|v| violations.borrow_mut().push(v)))
        .base_url(Some(base_url))
        .parse(url_path);

    // we should not get error here, we assume we call this function because there is no error
    if parsed.is_err() {
        return None;
    }

    // Use url crate's parsing with syntax violation detection

    if !violations.get_mut().is_empty() {
        match violations.get_mut()[0] {
            SyntaxViolation::Backslash => {
                return Some(AdditionalValidationResult::Error(AssetValidationError::new("Invalid character: backslash")));
            }
            SyntaxViolation::C0SpaceIgnored => {
                // ignore
            },
            SyntaxViolation::EmbeddedCredentials => {
                return Some(AdditionalValidationResult::Error(AssetValidationError::new("embedding authentication information (username or password) in an URL is not valid")));
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
                let message = "There are characters that are not valid in URLs. You may be using space or other reserved characters in URL, consider use percent encoding instead.".to_string();
                return Some(AdditionalValidationResult::Warning(AssetValidationWarning::new(message)));
            },
            SyntaxViolation::NullInFragment => {
                return Some(AdditionalValidationResult::Error(AssetValidationError::new("NULL characters are ignored in URL fragment identifiers")));
            },
            SyntaxViolation::PercentDecode => {
                return Some(AdditionalValidationResult::Error(AssetValidationError::new("expected 2 hex digits after %")));
            },
            SyntaxViolation::TabOrNewlineIgnored => {
                // this, Unity will handle them not correctly, so report an error, it is not actually ignored
                return Some(AdditionalValidationResult::Error(AssetValidationError::new("tabs or newlines are ignored in URLs")));
            },
            SyntaxViolation::UnencodedAtSign => {
                return Some(AdditionalValidationResult::Error(AssetValidationError::new("unencoded @ sign in username or password")));
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

        // Test that spaces generate warnings but still succeed
        let result = validate_url("my image.png", None);
        assert!(result.is_ok());
        let validation_result = result.unwrap();
        assert!(!validation_result.warnings.is_empty());
        assert!(validation_result.warnings[0].message.contains("characters that are not valid in URLs"));
    }

    #[test]
    fn test_validate_url_with_base_url() {
        // Create a project scheme base URL for testing relative path resolution
        let base_url = Url::parse("project:/Assets/UI/Styles/main.uss").unwrap();
        
        // Test relative paths with proper base URL
        let result = validate_url("../Images/button.png", Some(&base_url));
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.url.as_str(), "project:/Assets/UI/Images/button.png");
        
        // Test current directory relative path
        let result = validate_url("./icons/star.svg", Some(&base_url));
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.url.as_str(), "project:/Assets/UI/Styles/icons/star.svg");
        
        // Test simple filename relative path
        let result = validate_url("background.jpg", Some(&base_url));
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.url.as_str(), "project:/Assets/UI/Styles/background.jpg");
        
        // Test parent directory navigation
        let result = validate_url("../../Textures/wood.png", Some(&base_url));
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.url.as_str(), "project:/Assets/Textures/wood.png");
        
        // Test absolute paths (should ignore base URL)
        let result = validate_url("/Assets/Global/theme.png", Some(&base_url));
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.url.as_str(), "project:/Assets/Global/theme.png");
        
        // Test project scheme URLs (should ignore base URL)
        let result = validate_url("project:/Packages/com.unity.ui/icon.png", Some(&base_url));
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.url.as_str(), "project:/Packages/com.unity.ui/icon.png");
    }

    #[test]
    fn test_validate_url_with_deep_base_url() {
        // Test with a deeper nested base URL
        let base_url = Url::parse("project:/Assets/UI/Components/Buttons/Styles/button.uss").unwrap();
        
        // Test multiple parent directory navigation
        let result = validate_url("../../../Images/icons/close.svg", Some(&base_url));
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.url.as_str(), "project:/Assets/UI/Images/icons/close.svg");
        
        // Test navigation to project root and back
        let result = validate_url("../../../../../Packages/com.example/textures/metal.jpg", Some(&base_url));
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.url.as_str(), "project:/Packages/com.example/textures/metal.jpg");
    }

    #[test]
    fn test_create_project_url_basic() {
        use std::path::PathBuf;
        
        // Test with normalized paths (no file system access required)
        // Since create_project_url now expects normalized paths, we simulate what canonicalize would return
        // Use platform-appropriate absolute paths
        #[cfg(windows)]
        let (project_root, file_path, packages_file) = {
            let project_root = PathBuf::from("C:\\MyProject");
            let file_path = PathBuf::from("C:\\MyProject\\Assets\\UI\\styles.uss");
            let packages_file = PathBuf::from("C:\\MyProject\\Packages\\com.unity.ui\\Runtime\\button.png");
            (project_root, file_path, packages_file)
        };
        
        #[cfg(not(windows))]
        let (project_root, file_path, packages_file) = {
            let project_root = PathBuf::from("/home/user/MyProject");
            let file_path = PathBuf::from("/home/user/MyProject/Assets/UI/styles.uss");
            let packages_file = PathBuf::from("/home/user/MyProject/Packages/com.unity.ui/Runtime/button.png");
            (project_root, file_path, packages_file)
        };
        
        let result = create_project_url(&file_path, &project_root);
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.as_str(), "project:/Assets/UI/styles.uss");
        
        // Test with Windows-style paths explicitly
        let project_root_win = PathBuf::from("C:\\MyProject");
        let file_path_win = PathBuf::from("C:\\MyProject\\Assets\\Textures\\image.png");
        
        let result = create_project_url(&file_path_win, &project_root_win);
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.as_str(), "project:/Assets/Textures/image.png");
        
        // Test with Packages directory
        let result = create_project_url(&packages_file, &project_root);
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.as_str(), "project:/Packages/com.unity.ui/Runtime/button.png");
    }
    
    #[test]
    fn test_create_project_url_with_normalization() {
        use std::path::Path;
        use tempfile::TempDir;
        
        // Create a temporary directory structure for testing
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        
        // Create Assets directory
        let assets_dir = project_root.join("Assets");
        std::fs::create_dir_all(&assets_dir).unwrap();
        
        // Create a test file
        let test_file = assets_dir.join("test.uss");
        std::fs::write(&test_file, "/* test file */").unwrap();
        
        // Test with exact case using the normalization function
        let result = create_project_url_with_normalization(&test_file, project_root);
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.as_str(), "project:/Assets/test.uss");
        
        // On Windows, test with different case in project root path
        #[cfg(windows)]
        {
            // Convert project root to different case
            let project_root_str = project_root.to_string_lossy().to_uppercase();
            let project_root_different_case = Path::new(&project_root_str);
            
            let result = create_project_url_with_normalization(&test_file, project_root_different_case);
            assert!(result.is_ok(), "Should handle case-insensitive paths on Windows");
            let url = result.unwrap();
            assert_eq!(url.as_str(), "project:/Assets/test.uss");
        }
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

    #[test]
    #[cfg(windows)]
    fn test_project_url_to_path() {
        let path = project_url_to_path(Path::new("f:\\a\\b\\c"), &Url::parse("project:///Assets/hello.txt").unwrap());
        assert_eq!(path.unwrap().to_string_lossy(), "f:\\a\\b\\c\\Assets/hello.txt");
    }

    #[test]
    fn test_project_url_to_relative_path() {
        let path = project_url_to_relative_path(&Url::parse("project:///Assets/hello.txt").unwrap());
        assert_eq!(path.unwrap(), "Assets/hello.txt");
    }
}

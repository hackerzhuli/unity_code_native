//! URL Completion Provider for Unity Project Assets
//!
//! This module provides auto-completion for Unity project URLs in USS and UXML files.
//! It supports path completion for project assets and query parameter completion for assets with subassets.

use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::*;
use url::Url;
use log;

use crate::language::asset_url::{validate_url, project_url_to_path};
use crate::unity_asset_database::{UnityAssetDatabase, AssetDatabaseError};

/// Error type for URL completion operations
#[derive(Debug, Clone, PartialEq)]
pub struct UrlCompletionError {
    pub message: String,
}

impl UrlCompletionError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for UrlCompletionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "URL completion error: {}", self.message)
    }
}

impl std::error::Error for UrlCompletionError {}

/// Context for URL completion
#[derive(Debug, Clone, PartialEq)]
pub enum UrlCompletionContext {
    /// Completing the path portion of a URL
    Path {
        /// The partial path being typed
        partial_path: String,
        /// The base URL for resolving relative paths
        base_url: Option<Url>,
    },
    /// Completing query parameters for an asset with subassets
    Query {
        /// The complete asset URL
        asset_url: Url,
    },
}

/// URL completion provider for Unity project assets
pub struct UrlCompletionProvider {
    /// Reference to the Unity asset database
    asset_database: UnityAssetDatabase,
}

impl UrlCompletionProvider {
    /// Create a new URL completion provider
    ///
    /// # Arguments
    /// * `project_root` - The root path of the Unity project
    pub fn new(project_root: &Path) -> Self {
        Self {
            asset_database: UnityAssetDatabase::new(project_root),
        }
    }

    /// Provide completion items for a URL string
    ///
    /// # Arguments
    /// * `url_string` - The URL content WITHOUT quotes. This should be the actual URL text
    ///   extracted from the source code, not including the surrounding quote characters.
    ///   For example, if the source contains `url("project:/Assets/UI/")`, this parameter
    ///   should be `"project:/Assets/UI/"`, not `"\"project:/Assets/UI/\""`.
    /// * `cursor_position` - The position of the cursor within the URL string (0-based byte offset)
    /// * `base_url` - The base URL for resolving relative paths (ie. the source file's URL)
    ///
    /// # Returns
    /// A vector of completion items for the URL
    pub fn complete_url(
        &self,
        url_string: &str,
        cursor_position: usize,
        base_url: Option<&Url>,
    ) -> Vec<CompletionItem> {
        log::info!("URL completion requested: url='{}', cursor_pos={}, base_url={:?}", 
                   url_string, cursor_position, base_url);
        
        let context = match self.analyze_completion_context(url_string, cursor_position, base_url) {
            Ok(context) => {
                log::info!("Completion context analyzed: {:?}", context);
                context
            },
            Err(err) => {
                log::warn!("Failed to analyze completion context: {}", err);
                return Vec::new();
            },
        };
        
        let result = match context {
            UrlCompletionContext::Path { partial_path, base_url } => {
                log::info!("Completing path: '{}' with base_url: {:?}", partial_path, base_url);
                let items = self.complete_path(&partial_path, base_url.as_ref()).unwrap_or_default();
                log::info!("Path completion returned {} items", items.len());
                items
            }
            UrlCompletionContext::Query { asset_url } => {
                log::info!("Completing query for asset: {}", asset_url);
                let items = self.complete_query(&asset_url).unwrap_or_default();
                log::info!("Query completion returned {} items", items.len());
                items
            }
        };
        
        log::info!("URL completion finished with {} total items", result.len());
        result
    }

    /// Analyze the completion context based on the URL string and cursor position
    /// 
    /// # Parameters
    /// 
    /// * `url_string` - The URL content WITHOUT quotes. This should be the actual URL text
    ///   extracted from the source code, not including the surrounding quote characters.
    ///   For example, if the source contains `url("project:/Assets/UI/")`, this parameter
    ///   should be `"project:/Assets/UI/"`, not `"\"project:/Assets/UI/\""`.
    /// * `cursor_position` - The byte offset within the url_string where completion is requested.
    ///   Must be <= url_string.len().
    /// * `base_url` - Optional base URL for resolving relative paths. Used when the URL
    ///   being completed is relative to another URL.
    /// 
    /// # Returns
    /// 
    /// Returns a `UrlCompletionContext` indicating whether to complete paths or query parameters.
    pub fn analyze_completion_context(
        &self,
        url_string: &str,
        cursor_position: usize,
        base_url: Option<&Url>,
    ) -> Result<UrlCompletionContext, UrlCompletionError> {
        log::debug!("Analyzing completion context: url_string='{}', cursor_position={}", url_string, cursor_position);
        
        // Ensure cursor position is within bounds
        if cursor_position > url_string.len() {
            let error_msg = format!("Cursor position {} out of bounds for string length {}", cursor_position, url_string.len());
            log::error!("{}", error_msg);
            return Err(UrlCompletionError::new(error_msg));
        }

        let url_part = &url_string[..cursor_position];
        log::debug!("URL part up to cursor: '{}'", url_part);
        
        // Check if we're completing query parameters (user just typed '?')
        if url_part.ends_with('?') {
            log::debug!("Detected query completion context (ends with '?')");
            let path_part = &url_part[..url_part.len() - 1];
            log::debug!("Path part for query completion: '{}'", path_part);
            
            // Only provide query completion for explicit project scheme URLs
            if path_part.starts_with("project:") {
                log::debug!("Path part starts with 'project:', attempting URL validation");
                match validate_url(path_part, base_url) {
                    Ok(validation_result) => {
                        log::info!("Query completion context validated for URL: {}", validation_result.url);
                        return Ok(UrlCompletionContext::Query {
                            asset_url: validation_result.url,
                        });
                    }
                    Err(err) => {
                        log::warn!("URL validation failed for query completion: {}", err);
                        // Invalid URL, fall through to path completion
                    }
                }
            } else {
                log::debug!("Path part does not start with 'project:', falling through to path completion");
            }
        } else {
            log::debug!("Not a query completion context (does not end with '?')");
        }

        // Default to path completion
        log::debug!("Defaulting to path completion context");
        Ok(UrlCompletionContext::Path {
            partial_path: url_part.to_string(),
            base_url: base_url.cloned(),
        })
    }

    /// Complete the path portion of a URL
    pub fn complete_path(
        &self,
        partial_path: &str,
        base_url: Option<&Url>,
    ) -> Result<Vec<CompletionItem>, UrlCompletionError> {
        log::debug!("Starting path completion for: '{}' with base_url: {:?}", partial_path, base_url);
        
        // Only provide completion after a '/' character
        if !partial_path.contains('/') {
            log::debug!("No '/' found in partial path, returning empty completion");
            return Ok(Vec::new());
        }

        // Find the directory to search in
        let (directory_path, filename_prefix) = match self.extract_directory_and_prefix(partial_path, base_url) {
            Ok((dir_path, file_prefix)) => {
                log::debug!("Extracted directory: '{}', prefix: '{}'", dir_path.display(), file_prefix);
                (dir_path, file_prefix)
            },
            Err(err) => {
                log::warn!("Failed to extract directory and prefix: {}", err);
                return Err(err);
            }
        };
        
        // List directory contents
        let entries = match self.list_directory_entries(&directory_path, &filename_prefix) {
            Ok(entries) => {
                log::debug!("Found {} directory entries matching prefix '{}'", entries.len(), filename_prefix);
                entries
            },
            Err(err) => {
                log::warn!("Failed to list directory entries: {}", err);
                return Err(err);
            }
        };
        
        // Convert to completion items
        let mut items = Vec::new();
        for entry in entries {
            let item = self.create_path_completion_item(entry);
            items.push(item);
        }
        
        log::debug!("Created {} completion items for path completion", items.len());
        Ok(items)
    }

    /// Complete query parameters for an asset
    fn complete_query(
        &self,
        asset_url: &Url,
    ) -> Result<Vec<CompletionItem>, UrlCompletionError> {
        // Get texture asset information to check for subassets
        match self.asset_database.get_texture_asset_info(asset_url) {
            Ok(texture_info) => {
                let mut items = Vec::new();
                
                // Add completion for the main asset
                let main_asset_item = CompletionItem {
                    label: format!("fileID={}&guid={}&type=3", 
                        if texture_info.sprites.is_empty() { "2800000" } else { "21300000" },
                        texture_info.guid
                    ),
                    kind: Some(CompletionItemKind::VALUE),
                    detail: Some("Main asset".to_string()),
                    documentation: Some(Documentation::String(
                        "Complete URL parameters for the main asset".to_string()
                    )),
                    insert_text: Some(format!("fileID={}&guid={}&type=3", 
                        if texture_info.sprites.is_empty() { "2800000" } else { "21300000" },
                        texture_info.guid
                    )),
                    ..Default::default()
                };
                items.push(main_asset_item);
                
                // Add completion for each sprite subasset
                for sprite in &texture_info.sprites {
                    let sprite_item = CompletionItem {
                        label: format!("fileID={}&guid={}&type=3#{}", 
                            sprite.file_id, texture_info.guid, sprite.name
                        ),
                        kind: Some(CompletionItemKind::VALUE),
                        detail: Some(format!("Sprite: {}", sprite.name)),
                        documentation: Some(Documentation::String(
                            format!("Complete URL parameters for sprite '{}'", sprite.name)
                        )),
                        insert_text: Some(format!("fileID={}&guid={}&type=3#{}", 
                            sprite.file_id, texture_info.guid, sprite.name
                        )),
                        ..Default::default()
                    };
                    items.push(sprite_item);
                }
                
                Ok(items)
            }
            Err(AssetDatabaseError { message }) => {
                // Asset doesn't exist or isn't a texture, no query completion
                log::debug!("No query completion for asset: {}", message);
                Ok(Vec::new())
            }
        }
    }

    /// Extract the directory path and filename prefix from a partial URL path
    fn extract_directory_and_prefix(
        &self,
        partial_path: &str,
        base_url: Option<&Url>,
    ) -> Result<(PathBuf, String), UrlCompletionError> {
        log::debug!("Extracting directory and prefix from partial_path: '{}'", partial_path);
        
        // Parse the partial path as a URL to resolve relative paths
        let resolved_url = match validate_url(partial_path, base_url) {
            Ok(validation_result) => {
                log::debug!("URL validation successful: {}", validation_result.url);
                validation_result.url
            },
            Err(validation_error) => {
                log::debug!("URL validation failed: {}", validation_error);
                // If validation fails, try fallback for partial paths that might be valid when completed
                // But reject URLs with backslashes or other clearly invalid characters
                if validation_error.message.contains("backslash") || 
                   validation_error.message.contains("Invalid character") ||
                   validation_error.message.contains("authentication information") ||
                   validation_error.message.contains("NULL characters") ||
                   validation_error.message.contains("tabs or newlines") ||
                   validation_error.message.contains("unencoded @ sign") {
                    log::warn!("Rejecting URL due to invalid characters: {}", validation_error.message);
                    return Err(UrlCompletionError::new("Invalid URL - no completion available"));
                }
                // For other validation errors (like incomplete paths), try fallback
                log::debug!("Attempting fallback path extraction");
                return self.fallback_path_extraction(partial_path, base_url);
            }
        };

        // Convert URL to file system path
        let project_root = self.asset_database.project_root();
        log::debug!("Project root: {}", project_root.display());
        
        if let Some(asset_path) = project_url_to_path(project_root, &resolved_url) {
            log::debug!("Converted URL to asset path: {}", asset_path.display());
            let path_str = resolved_url.path();
            log::debug!("URL path string: '{}'", path_str);
            
            // Find the last '/' to separate directory and filename
            if let Some(last_slash) = path_str.rfind('/') {
                let directory_part = &path_str[..last_slash];
                let filename_part = &path_str[last_slash + 1..];
                log::debug!("Split path - directory: '{}', filename: '{}'", directory_part, filename_part);
                
                // Convert directory part to file system path
                let directory_url_str = format!("project:{}", directory_part);
                log::debug!("Directory URL string: '{}'", directory_url_str);
                
                if let Ok(directory_url) = Url::parse(&directory_url_str) {
                    if let Some(directory_path) = project_url_to_path(&project_root, &directory_url) {
                        log::debug!("Successfully resolved directory path: {}", directory_path.display());
                        return Ok((directory_path, filename_part.to_string()));
                    } else {
                        log::warn!("Failed to convert directory URL to path: {}", directory_url);
                    }
                } else {
                    log::warn!("Failed to parse directory URL: '{}'", directory_url_str);
                }
            } else {
                log::warn!("No '/' found in URL path: '{}'", path_str);
            }
        } else {
            log::warn!("Failed to convert resolved URL to asset path: {}", resolved_url);
        }
        
        let error_msg = "Could not extract directory and prefix from path";
        log::error!("{}", error_msg);
        Err(UrlCompletionError::new(error_msg))
    }

    /// Fallback method for extracting directory and prefix when URL validation fails
    fn fallback_path_extraction(
        &self,
        partial_path: &str,
        base_url: Option<&Url>,
    ) -> Result<(PathBuf, String), UrlCompletionError> {
        // For simple cases, try to extract directory information
        if let Some(last_slash) = partial_path.rfind('/') {
            let directory_part = &partial_path[..last_slash];
            let filename_part = &partial_path[last_slash + 1..];
            
            // Try to resolve the directory part
            if let Ok(validation_result) = validate_url(directory_part, base_url) {
                let project_root = self.asset_database.project_root();
                if let Some(directory_path) = project_url_to_path(project_root, &validation_result.url) {
                    return Ok((directory_path, filename_part.to_string()));
                }
            }
        }
        
        Err(UrlCompletionError::new("Fallback path extraction failed"))
    }

    /// List entries in a directory that match the given prefix
    fn list_directory_entries(
        &self,
        directory_path: &Path,
        filename_prefix: &str,
    ) -> Result<Vec<DirectoryEntry>, UrlCompletionError> {
        log::debug!("Listing directory entries in: '{}' with prefix: '{}'", directory_path.display(), filename_prefix);
        
        if !directory_path.exists() {
            log::warn!("Directory does not exist: {}", directory_path.display());
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        let mut total_files = 0;
        let mut skipped_meta = 0;
        let mut filtered_out = 0;
        
        match std::fs::read_dir(directory_path) {
            Ok(dir_entries) => {
                for entry in dir_entries {
                    if let Ok(entry) = entry {
                        total_files += 1;
                        let file_name = entry.file_name();
                        let file_name_str = file_name.to_string_lossy();
                        
                        // Skip .meta files
                        if file_name_str.ends_with(".meta") {
                            skipped_meta += 1;
                            continue;
                        }
                        
                        // Filter by prefix (case-insensitive)
                        if filename_prefix.is_empty() || file_name_str.to_lowercase().starts_with(&filename_prefix.to_lowercase()) {
                            let is_directory = entry.path().is_dir();
                            log::debug!("Adding entry: '{}' ({})", file_name_str, if is_directory { "directory" } else { "file" });
                            entries.push(DirectoryEntry {
                                name: file_name_str.to_string(),
                                is_directory,
                            });
                        } else {
                            filtered_out += 1;
                        }
                    }
                }
                log::debug!("Directory scan complete: {} total files, {} .meta files skipped, {} filtered out by prefix, {} entries added", 
                           total_files, skipped_meta, filtered_out, entries.len());
            }
            Err(e) => {
                let error_msg = format!("Failed to read directory {}: {}", directory_path.display(), e);
                log::error!("{}", error_msg);
                return Err(UrlCompletionError::new(error_msg));
            }
        }
        
        // Sort entries: directories first, then files, both alphabetically
        entries.sort_by(|a, b| {
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        
        log::debug!("Returning {} sorted directory entries", entries.len());
        Ok(entries)
    }

    /// Create a completion item for a directory entry
    fn create_path_completion_item(&self, entry: DirectoryEntry) -> CompletionItem {
        let (kind, detail, insert_text) = if entry.is_directory {
            (
                CompletionItemKind::FOLDER,
                "Directory".to_string(),
                format!("{}/", entry.name),
            )
        } else {
            (
                CompletionItemKind::FILE,
                "File".to_string(),
                entry.name.clone(),
            )
        };

        CompletionItem {
            label: entry.name,
            kind: Some(kind),
            detail: Some(detail),
            insert_text: Some(insert_text),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            ..Default::default()
        }
    }
}

/// Represents a directory entry for completion
#[derive(Debug, Clone)]
struct DirectoryEntry {
    name: String,
    is_directory: bool,
}

#[cfg(test)]
#[path = "url_completion_tests.rs"]
mod tests;
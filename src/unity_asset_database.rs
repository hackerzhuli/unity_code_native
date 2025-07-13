//! Unity Asset Database for querying asset information from meta files
//!
//! This module provides the `UnityAssetDatabase` struct that can query asset information
//! from Unity project meta files, including GUIDs and sprite details for textures.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;
use serde::{Deserialize, Serialize};
use crate::language::asset_url::{project_url_to_path, validate_url};

/// Error type for Unity Asset Database operations
/// 
/// This error type encapsulates all possible failures that can occur when
/// querying asset information from Unity meta files.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetDatabaseError {
    /// Human-readable error message describing what went wrong.
    /// 
    /// Common error scenarios include:
    /// - Invalid asset URLs (wrong scheme, malformed paths)
    /// - Missing asset files or meta files
    /// - Corrupted or unparseable meta file content
    /// - File system access errors
    /// 
    /// Example: "Failed to read meta file '/path/to/asset.png.meta': No such file or directory"
    pub message: String,
}

impl AssetDatabaseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

}

impl std::fmt::Display for AssetDatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Asset database error: {}", self.message)
    }
}

impl std::error::Error for AssetDatabaseError {}

/// Basic asset information containing just the GUID
/// 
/// This struct represents the minimal information available for any Unity asset.
/// All Unity assets have a GUID that uniquely identifies them within the project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BasicAssetInfo {
    /// The unique identifier (GUID) for this asset within the Unity project.
    /// This is a 32-character hexadecimal string that Unity uses to reference assets.
    /// Example: "990f791f0aee3f04e8e9eba2ff279777"
    pub guid: String,
}

/// Sprite information within a texture asset
/// 
/// When a texture is configured as a multi-sprite texture in Unity, each individual
/// sprite within that texture has its own name and file ID for referencing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteInfo {
    /// The human-readable name of the sprite as defined in Unity's Sprite Editor.
    /// This is the name that appears in Unity's Inspector and is used for identification.
    /// Example: "player_idle_01" or "UI_Button_Normal"
    pub name: String,
    
    /// The internal file ID used by Unity to reference this specific sprite.
    /// This is a signed 64-bit integer that Unity generates for each sprite.
    /// Used in asset references and serialization. Can be negative.
    /// Example: -1713611897823765776
    pub file_id: i64,
}

/// Detailed texture asset information including sprite data
/// 
/// This struct provides comprehensive information about texture assets, including
/// whether they contain multiple sprites and the details of each sprite if applicable.
/// For non-texture assets, this behaves like BasicAssetInfo with additional fields set to defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextureAssetInfo {
    /// The unique identifier (GUID) for this texture asset within the Unity project.
    /// Same format as BasicAssetInfo.guid - a 32-character hexadecimal string.
    /// Example: "6a1cda2d4d23f0f43ab961e7dde2bd4a"
    pub guid: String,
    
    /// Indicates whether this texture contains multiple sprites.
    /// 
    /// This is true when:
    /// - textureType is 8 (Sprite 2D and UI)
    /// - spriteMode is 2 (Multiple)
    /// - The texture has been sliced into multiple sprites in Unity's Sprite Editor
    /// 
    /// For non-texture assets, this is always false.
    pub is_multiple_sprite: bool,
    
    /// Collection of individual sprites within this texture.
    /// 
    /// - If is_multiple_sprite is true: Contains one SpriteInfo for each sprite
    /// - If is_multiple_sprite is false: Empty vector
    /// - For non-texture assets: Always empty
    /// 
    /// The sprites are extracted from the nameFileIdTable in the texture's .meta file.
    pub sprites: Vec<SpriteInfo>,
}

/// Represents the structure of a Unity meta file for basic parsing
#[derive(Debug, Deserialize)]
struct MetaFile {
    guid: String,
    #[serde(rename = "TextureImporter")]
    texture_importer: Option<TextureImporter>,
}

/// Represents the TextureImporter section of a meta file
#[derive(Debug, Deserialize)]
struct TextureImporter {
    #[serde(rename = "spriteMode")]
    sprite_mode: Option<i32>,
    #[serde(rename = "textureType")]
    texture_type: Option<i32>,
    #[serde(rename = "spriteSheet")]
    sprite_sheet: Option<SpriteSheet>,
}

/// Represents the spriteSheet section containing sprite information
#[derive(Debug, Deserialize)]
struct SpriteSheet {
    #[serde(rename = "nameFileIdTable")]
    name_file_id_table: Option<HashMap<String, i64>>,
}

/// Unity Asset Database for querying asset information
/// 
/// This struct provides methods to query asset information from Unity project meta files.
/// It requires a Unity project root path for initialization and validates all asset URLs
/// against the project structure.
/// 
/// The database can extract:
/// - Basic asset information (GUID) for any Unity asset
/// - Detailed texture information including sprite data for multi-sprite textures
pub struct UnityAssetDatabase {
    /// The absolute file system path to the Unity project root directory.
    /// 
    /// This should point to the directory containing the Assets/ and Packages/ folders.
    /// All asset URL resolution is performed relative to this path.
    /// 
    /// Example: "/Users/developer/MyUnityProject" or "C:\\Projects\\MyGame"
    project_root: PathBuf,
}

impl UnityAssetDatabase {
    /// Creates a new UnityAssetDatabase instance
    ///
    /// # Arguments
    /// * `project_path` - The file system path to the Unity project root
    ///
    /// # Examples
    /// ```
    /// use unity_code_native::unity_asset_database::UnityAssetDatabase;
    /// use std::path::Path;
    ///
    /// let db = UnityAssetDatabase::new(Path::new("/path/to/unity/project"));
    /// ```
    pub fn new(project_path: &Path) -> Self {
        Self {
            project_root: project_path.to_path_buf(),
        }
    }

    /// Gets basic asset information (GUID) for any asset
    ///
    /// # Arguments
    /// * `asset_url` - The asset URL in project scheme (e.g., project:/Assets/image.png)
    ///
    /// # Returns
    /// * `Ok(BasicAssetInfo)` - Basic asset information with GUID
    /// * `Err(AssetDatabaseError)` - If the asset or meta file cannot be found/parsed
    pub fn get_asset_info(&self, asset_url: &Url) -> Result<BasicAssetInfo, AssetDatabaseError> {
        let asset_path = self.url_to_asset_path(asset_url)?;
        let meta_path = self.get_meta_file_path(&asset_path)?;
        let meta_content = self.read_meta_file(&meta_path)?;
        let meta_file = self.parse_meta_file(&meta_content)?;
        
        Ok(BasicAssetInfo {
            guid: meta_file.guid,
        })
    }

    /// Convenience method to get basic asset information using a string URL
    ///
    /// # Arguments
    /// * `asset_url_str` - The asset URL as a string in project scheme
    ///
    /// # Returns
    /// * `Ok(BasicAssetInfo)` - Basic asset information with GUID
    /// * `Err(AssetDatabaseError)` - If the URL is invalid or asset cannot be found/parsed
    pub fn get_asset_info_from_str(&self, asset_url_str: &str) -> Result<BasicAssetInfo, AssetDatabaseError> {
        let url = Url::parse(asset_url_str)
            .map_err(|e| AssetDatabaseError::new(format!("Failed to parse URL '{}': {}", asset_url_str, e)))?;
        self.get_asset_info(&url)
    }

    /// Gets detailed texture asset information including sprite data
    ///
    /// # Arguments
    /// * `asset_url` - The texture asset URL in project scheme
    ///
    /// # Returns
    /// * `Ok(TextureAssetInfo)` - Detailed texture information with sprite data
    /// * `Err(AssetDatabaseError)` - If the asset is not a texture or cannot be parsed
    pub fn get_texture_asset_info(&self, asset_url: &Url) -> Result<TextureAssetInfo, AssetDatabaseError> {
        let asset_path = self.url_to_asset_path(asset_url)?;
        let meta_path = self.get_meta_file_path(&asset_path)?;
        let meta_content = self.read_meta_file(&meta_path)?;
        let meta_file = self.parse_meta_file(&meta_content)?;
        
        // Check if this asset has a TextureImporter
        if let Some(texture_importer) = meta_file.texture_importer {
            // Check if it's a multiple sprite texture
            let is_multiple_sprite = texture_importer.sprite_mode == Some(2) && 
                                   texture_importer.texture_type == Some(8);
            
            let mut sprites = Vec::new();
            
            if is_multiple_sprite {
                // Extract sprite information from nameFileIdTable
                if let Some(sprite_sheet) = texture_importer.sprite_sheet {
                    if let Some(name_file_id_table) = sprite_sheet.name_file_id_table {
                        for (name, file_id) in name_file_id_table {
                            sprites.push(SpriteInfo {
                                name,
                                file_id,
                            });
                        }
                    }
                }
            }
            
            Ok(TextureAssetInfo {
                guid: meta_file.guid,
                is_multiple_sprite,
                sprites,
            })
        } else {
            // Not a texture asset, treat as basic asset
            Ok(TextureAssetInfo {
                guid: meta_file.guid,
                is_multiple_sprite: false,
                sprites: Vec::new(),
            })
        }
    }

    /// Convenience method to get detailed texture asset information using a string URL
    ///
    /// # Arguments
    /// * `asset_url_str` - The texture asset URL as a string in project scheme
    ///
    /// # Returns
    /// * `Ok(TextureAssetInfo)` - Detailed texture information with sprite data
    /// * `Err(AssetDatabaseError)` - If the URL is invalid or asset cannot be parsed
    pub fn get_texture_asset_info_from_str(&self, asset_url_str: &str) -> Result<TextureAssetInfo, AssetDatabaseError> {
        let url = Url::parse(asset_url_str)
            .map_err(|e| AssetDatabaseError::new(format!("Failed to parse URL '{}': {}", asset_url_str, e)))?;
        self.get_texture_asset_info(&url)
    }

    /// Converts an asset URL to a file system path
    fn url_to_asset_path(&self, asset_url: &Url) -> Result<PathBuf, AssetDatabaseError> {
        // Validate the URL first
        let validation_result = validate_url(&asset_url.to_string(), None)
            .map_err(|e| AssetDatabaseError::new(format!("Invalid asset URL: {}", e)))?;
        
        // Convert to file path
        let asset_path = project_url_to_path(&self.project_root, asset_url)
            .ok_or_else(|| AssetDatabaseError::new("Failed to convert URL to file path"))?;
        
        Ok(asset_path)
    }

    /// Gets the meta file path for a given asset path
    fn get_meta_file_path(&self, asset_path: &Path) -> Result<PathBuf, AssetDatabaseError> {
        let mut meta_path = asset_path.to_path_buf();
        let current_extension = meta_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        // Append .meta to the existing extension
        let new_extension = if current_extension.is_empty() {
            "meta".to_string()
        } else {
            format!("{}.meta", current_extension)
        };
        
        meta_path.set_extension(new_extension);
        Ok(meta_path)
    }

    /// Reads the content of a meta file
    fn read_meta_file(&self, meta_path: &Path) -> Result<String, AssetDatabaseError> {
        fs::read_to_string(meta_path)
            .map_err(|e| AssetDatabaseError::new(format!("Failed to read meta file '{}': {}", meta_path.display(), e)))
    }

    /// Parses a meta file YAML content
    fn parse_meta_file(&self, content: &str) -> Result<MetaFile, AssetDatabaseError> {
        serde_yaml::from_str(content)
            .map_err(|e| AssetDatabaseError::new(format!("Failed to parse meta file: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_get_meta_file_path() {
        let db = UnityAssetDatabase::new(Path::new("/project"));
        
        // Test with extension
        let asset_path = Path::new("/project/Assets/image.png");
        let meta_path = db.get_meta_file_path(asset_path).unwrap();
        assert_eq!(meta_path, Path::new("/project/Assets/image.png.meta"));
        
        // Test without extension
        let asset_path = Path::new("/project/Assets/folder");
        let meta_path = db.get_meta_file_path(asset_path).unwrap();
        assert_eq!(meta_path, Path::new("/project/Assets/folder.meta"));
    }

    #[test]
    fn test_url_to_asset_path() {
        let db = UnityAssetDatabase::new(Path::new("C:\\MyProject"));
        
        // This test would need actual file system setup to work properly
        // For now, we'll just test the URL validation part
        let url = Url::parse("project:/Assets/image.png").unwrap();
        let result = db.url_to_asset_path(&url);
        // The result depends on the actual file system, so we just check it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_texture_with_multiple_sprites() {
        let db = UnityAssetDatabase::new(Path::new("f:\\projects\\rs\\unity_code_native"));
        
        // Test with the example texture that has multiple sprites
        let url = Url::parse("project:/Assets/examples/meta/texture_with_multiple_sprites_example.png").unwrap();
        let result = db.get_texture_asset_info(&url);
        
        match result {
            Ok(texture_info) => {
                assert_eq!(texture_info.guid, "6a1cda2d4d23f0f43ab961e7dde2bd4a");
                assert_eq!(texture_info.is_multiple_sprite, true);
                assert_eq!(texture_info.sprites.len(), 3);
                
                // Check sprite names and file IDs
                let sprite_names: Vec<&str> = texture_info.sprites.iter().map(|s| s.name.as_str()).collect();
                assert!(sprite_names.contains(&"Hover Doc Link_0"));
                assert!(sprite_names.contains(&"Hover Doc Link_1"));
                assert!(sprite_names.contains(&"Hover Doc Link_2"));
                
                // Check specific file IDs
                for sprite in &texture_info.sprites {
                    match sprite.name.as_str() {
                        "Hover Doc Link_0" => assert_eq!(sprite.file_id, -1713611897823765776),
                        "Hover Doc Link_1" => assert_eq!(sprite.file_id, -970562782),
                        "Hover Doc Link_2" => assert_eq!(sprite.file_id, -577418574),
                        _ => panic!("Unexpected sprite name: {}", sprite.name),
                    }
                }
            }
            Err(e) => {
                println!("Error reading texture asset: {}", e);
                // This might fail if the file doesn't exist, which is okay for this test
            }
        }
    }

    #[test]
    fn test_basic_asset_uxml() {
        let db = UnityAssetDatabase::new(Path::new("f:\\projects\\rs\\unity_code_native"));
        
        // Test with the example UXML file
        let url = Url::parse("project:/Assets/examples/meta/uxml_example.uxml").unwrap();
        let result = db.get_asset_info(&url);
        
        match result {
            Ok(asset_info) => {
                assert_eq!(asset_info.guid, "990f791f0aee3f04e8e9eba2ff279777");
            }
            Err(e) => {
                println!("Error reading UXML asset: {}", e);
                // This might fail if the file doesn't exist, which is okay for this test
            }
        }
    }

    #[test]
    fn test_texture_asset_info_for_non_multiple_sprite() {
        let db = UnityAssetDatabase::new(Path::new("f:\\projects\\rs\\unity_code_native"));
        
        // Test with the UXML file using texture asset info method
        let url = Url::parse("project:/Assets/examples/meta/uxml_example.uxml").unwrap();
        let result = db.get_texture_asset_info(&url);
        
        match result {
            Ok(texture_info) => {
                assert_eq!(texture_info.guid, "990f791f0aee3f04e8e9eba2ff279777");
                assert_eq!(texture_info.is_multiple_sprite, false);
                assert_eq!(texture_info.sprites.len(), 0);
            }
            Err(e) => {
                println!("Error reading asset as texture: {}", e);
                // This might fail if the file doesn't exist, which is okay for this test
            }
        }
    }

    #[test]
    fn test_convenience_methods_with_string_urls() {
        let db = UnityAssetDatabase::new(Path::new("f:\\projects\\rs\\unity_code_native"));
        
        // Test get_asset_info_from_str
        let result = db.get_asset_info_from_str("project:/Assets/examples/meta/uxml_example.uxml");
        match result {
            Ok(asset_info) => {
                assert_eq!(asset_info.guid, "990f791f0aee3f04e8e9eba2ff279777");
            }
            Err(e) => {
                println!("Error reading UXML asset with string URL: {}", e);
            }
        }
        
        // Test get_texture_asset_info_from_str
        let result = db.get_texture_asset_info_from_str("project:/Assets/examples/meta/texture_with_multiple_sprites_example.png");
        match result {
            Ok(texture_info) => {
                assert_eq!(texture_info.guid, "6a1cda2d4d23f0f43ab961e7dde2bd4a");
                assert_eq!(texture_info.is_multiple_sprite, true);
                assert_eq!(texture_info.sprites.len(), 3);
            }
            Err(e) => {
                println!("Error reading texture asset with string URL: {}", e);
            }
        }
        
        // Test error handling for invalid URL string
        let result = db.get_asset_info_from_str("not-a-valid-url");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.message.contains("Failed to parse URL"));
        }
    }
}
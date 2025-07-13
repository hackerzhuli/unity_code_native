//! Unity Asset Database for querying asset information from meta files
//!
//! This module provides the `UnityAssetDatabase` struct that can query asset information
//! from Unity project meta files, including GUIDs and sprite details for textures.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;
use serde::{Deserialize, Serialize};
use crate::language::asset_url::{project_url_to_path};

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
    
    /// Indicates whether this texture is configured for multiple sprites.
    /// 
    /// This is true when:
    /// - textureType is 8 (Sprite 2D and UI)
    /// - spriteMode is 2 (Multiple)
    /// 
    /// Note: This field indicates the sprite mode configuration, not the actual number of sprites.
    /// A texture can have is_multiple_sprite=true but contain zero sprites.
    /// 
    /// For non-texture assets, this is always false.
    pub is_multiple_sprite: bool,
    
    /// Collection of individual sprites that have been created within this texture.
    /// 
    /// - If is_multiple_sprite is true: Contains one SpriteInfo for each sprite that has been
    ///   created in Unity's Sprite Editor (may be empty if no sprites have been sliced yet)
    /// - If is_multiple_sprite is false: Always empty
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

    /// Gets the project root path
    ///
    /// # Returns
    /// The absolute file system path to the Unity project root directory
    pub fn project_root(&self) -> &Path {
        &self.project_root
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
            // Check if it's configured for multiple sprites (spriteMode=2, textureType=8)
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

    /// Converts an asset URL to a file system path
    fn url_to_asset_path(&self, asset_url: &Url) -> Result<PathBuf, AssetDatabaseError> {
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
#[path = "unity_asset_database_tests.rs"]
mod tests;
//! Demo showing how to use the UnityAssetDatabase
//!
//! This example demonstrates how to query asset information from Unity meta files
//! using the UnityAssetDatabase struct.

use unity_code_native::unity_asset_database::UnityAssetDatabase;
use std::path::Path;
use url::Url;

fn main() {
    // Initialize the database with the project root path
    let project_root = Path::new("f:\\projects\\rs\\unity_code_native");
    let db = UnityAssetDatabase::new(project_root);
    
    println!("Unity Asset Database Demo");
    println!("========================\n");
    
    // Example 1: Get basic asset info for a UXML file
    println!("1. Basic Asset Info (UXML file):");
    let uxml_url = Url::parse("project:/Assets/examples/meta/uxml_example.uxml").unwrap();
    match db.get_asset_info(&uxml_url) {
        Ok(asset_info) => {
            println!("   GUID: {}", asset_info.guid);
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }
    println!();
    
    // Example 2: Get texture asset info for a texture with multiple sprites
    println!("2. Texture Asset Info (Multiple Sprites):");
    let texture_url = Url::parse("project:/Assets/examples/meta/texture_with_multiple_sprites_example.png").unwrap();
    match db.get_texture_asset_info(&texture_url) {
        Ok(texture_info) => {
            println!("   GUID: {}", texture_info.guid);
            println!("   Is Multiple Sprite: {}", texture_info.is_multiple_sprite);
            println!("   Number of Sprites: {}", texture_info.sprites.len());
            
            if texture_info.is_multiple_sprite {
                println!("   Sprites:");
                for sprite in &texture_info.sprites {
                    println!("     - Name: '{}', File ID: {}", sprite.name, sprite.file_id);
                }
            }
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }
    println!();
    
    // Example 3: Get texture asset info for a non-texture asset (should show is_multiple_sprite = false)
    println!("3. Texture Asset Info for Non-Texture (UXML file):");
    match db.get_texture_asset_info(&uxml_url) {
        Ok(texture_info) => {
            println!("   GUID: {}", texture_info.guid);
            println!("   Is Multiple Sprite: {}", texture_info.is_multiple_sprite);
            println!("   Number of Sprites: {}", texture_info.sprites.len());
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }
    println!();
    
    // Example 4: Demonstrate error handling with invalid URL
    println!("4. Error Handling (Invalid URL):");
    let invalid_url = Url::parse("invalid:/not/a/valid/url").unwrap();
    match db.get_asset_info(&invalid_url) {
        Ok(asset_info) => {
            println!("   Unexpected success: GUID = {}", asset_info.guid);
        }
        Err(e) => {
            println!("   Expected error: {}", e);
        }
    }
    println!();
    
    println!("Demo completed!");
}
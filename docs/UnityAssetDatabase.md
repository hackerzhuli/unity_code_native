# Unity Asset Database

The `UnityAssetDatabase` struct provides a way to query asset information from Unity project meta files. It can extract GUIDs and detailed sprite information from texture assets.

## Features

- **Basic Asset Info**: Get GUID for any Unity asset
- **Texture Asset Info**: Get detailed information about texture assets, including sprite data for multi-sprite textures
- **URL Validation**: Validates Unity project scheme URLs
- **Meta File Parsing**: Parses Unity `.meta` files (YAML format)
- **Type-Safe URLs**: Primary methods use `url::Url` type for better type safety and validation
- **Convenience Methods**: String-based convenience methods for easier integration

## Usage

### Creating a Database Instance

```rust
use unity_code_native::unity_asset_database::UnityAssetDatabase;
use std::path::Path;
use url::Url;

let project_root = Path::new("/path/to/unity/project");
let db = UnityAssetDatabase::new(project_root);
```

### Getting Basic Asset Information

For most assets, you only need the GUID:

```rust
// Method 1: Using proper URL type (recommended for type safety)
let asset_url = Url::parse("project:/Assets/UI/MainMenu.uxml").unwrap();
match db.get_asset_info(&asset_url) {
    Ok(asset_info) => {
        println!("GUID: {}", asset_info.guid);
    }
    Err(e) => {
        println!("Error: {}", e);
    }
}

// Method 2: Using string URL (convenience method)
match db.get_asset_info_from_str("project:/Assets/UI/MainMenu.uxml") {
    Ok(asset_info) => {
        println!("GUID: {}", asset_info.guid);
    }
    Err(e) => {
        println!("Error: {}", e);
    }
}
```

### Getting Texture Asset Information

For texture assets, especially those with multiple sprites:

```rust
// Method 1: Using proper URL type (recommended for type safety)
let texture_url = Url::parse("project:/Assets/Textures/UI_Sprites.png").unwrap();
match db.get_texture_asset_info(&texture_url) {
    Ok(texture_info) => {
        println!("GUID: {}", texture_info.guid);
        println!("Is Multiple Sprite: {}", texture_info.is_multiple_sprite);
        
        if texture_info.is_multiple_sprite {
            for sprite in &texture_info.sprites {
                println!("Sprite: '{}', File ID: {}", sprite.name, sprite.file_id);
            }
        }
    }
    Err(e) => {
        println!("Error: {}", e);
    }
}

// Method 2: Using string URL (convenience method)
match db.get_texture_asset_info_from_str("project:/Assets/Textures/UI_Sprites.png") {
    Ok(texture_info) => {
        println!("GUID: {}", texture_info.guid);
        println!("Is Multiple Sprite: {}", texture_info.is_multiple_sprite);
        
        if texture_info.is_multiple_sprite {
            for sprite in &texture_info.sprites {
                println!("Sprite: '{}', File ID: {}", sprite.name, sprite.file_id);
            }
        }
    }
    Err(e) => {
        println!("Error: {}", e);
    }
}
```

## Data Structures

### BasicAssetInfo

```rust
pub struct BasicAssetInfo {
    pub guid: String,
}
```

### TextureAssetInfo

```rust
pub struct TextureAssetInfo {
    pub guid: String,
    pub is_multiple_sprite: bool,
    pub sprites: Vec<SpriteInfo>,
}
```

### SpriteInfo

```rust
pub struct SpriteInfo {
    pub name: String,
    pub file_id: i64,
}
```

## API Design

### Method Types

The `UnityAssetDatabase` provides two types of methods for accessing asset information:

1. **Primary Methods** (Recommended for type safety):
   - `get_asset_info(&Url)` - Takes a `url::Url` type
   - `get_texture_asset_info(&Url)` - Takes a `url::Url` type
   - These methods provide compile-time URL validation and better type safety
   - Use these when you want maximum type safety and are working with URLs as first-class types

2. **Convenience Methods** (For easier integration):
   - `get_asset_info_from_str(&str)` - Takes a string URL
   - `get_texture_asset_info_from_str(&str)` - Takes a string URL
   - These methods internally parse the string into a `Url` and call the primary methods
   - Use these when you have URLs as strings and want simpler integration
   - URL parsing errors are included in the returned `Result`

### When to Use Which Method

- **Use Primary Methods** when:
  - You want maximum type safety
  - You're building a robust system where URL validation is critical
  - You're already working with `Url` types in your codebase
  - Performance is critical (avoids string parsing overhead)

- **Use Convenience Methods** when:
  - You have URLs as strings from external sources
  - You want simpler, more direct integration
  - You're prototyping or building quick tools
  - The slight performance overhead of string parsing is acceptable

## Asset URL Format

The database expects asset URLs in Unity's project scheme format:

- `project:/Assets/path/to/asset.ext`
- `project:/Packages/com.example.package/path/to/asset.ext`

The URL validation ensures:
- Correct scheme (`project`)
- Valid path structure
- Proper encoding

## Meta File Support

The database supports parsing Unity meta files for:

1. **Basic Assets**: Extracts GUID from any meta file
2. **Texture Assets**: Extracts GUID and sprite information from TextureImporter meta files

### Multiple Sprite Detection

A texture is considered to have multiple sprites when:
- `textureType` is `8` (Sprite 2D and UI)
- `spriteMode` is `2` (Multiple)

When these conditions are met, the `is_multiple_sprite` field is set to `true`. The `sprites` vector is populated with individual sprite information from the `nameFileIdTable` if any sprites have been created in Unity's Sprite Editor.

**Important**: A texture can have `is_multiple_sprite=true` but contain zero sprites if it has been configured for multiple sprites but hasn't been sliced yet in Unity's Sprite Editor.

## Error Handling

The database provides detailed error messages for:
- Invalid URLs
- Missing files
- Malformed meta files
- Parsing errors

## Example

See `examples/unity_asset_database_demo.rs` for a complete working example.

## Testing

Run the tests with:

```bash
cargo test unity_asset_database
```

The tests use example meta files located in `Assets/examples/meta/`.
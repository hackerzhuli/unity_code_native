# Unity Project
Here is a typical Unity project structure.

```
Assets/
ProjectSettings/
|-- ProjectVersion.txt
Library/
Packages/
```

The core is that `ProjectVersion.txt` is used to store the Unity version of the project(the format is YAML).

Example:
```
m_EditorVersion: 6000.0.51f1
m_EditorVersionWithRevision: 6000.0.51f1 (01c3ff5872c5)
```

just grab the `6000.0.51f1` part and use it as the version Unity.

## Unity Editor Version Detection

The `UnityProjectManager` provides methods to detect Unity projects and parse the Unity editor version from the `ProjectVersion.txt` file using a proper YAML parser.

**Key Methods:**
- `is_valid_unity_project()` - Returns `true` if the project has a valid Unity version that can be parsed
- `detect_unity_version()` - Returns `Result<String, UnityProjectError>` with the Unity version or an error
- `get_unity_version()` - Convenience method that returns `Option<String>`
- `get_unity_version_for_docs()` - Returns `Option<String>` for documentation URL generation

**Example usage:**
```rust
use std::path::PathBuf;
use unity_project_manager::UnityProjectManager;

let manager = UnityProjectManager::new(PathBuf::from("/path/to/unity/project"));

// Check if it's a valid Unity project
if manager.is_valid_unity_project() {
    // Get the Unity version
    match manager.detect_unity_version() {
        Ok(version) => println!("Unity version: {}", version),
        Err(e) => println!("Error: {}", e),
    }
}

// Or use the convenience method
if let Some(version) = manager.get_unity_version() {
    println!("Unity version: {}", version);
}
```

**Common version string formats:**
- `"6000.0.51f1"` - Unity 6000.0.51f1
- `"2023.3.15f1"` - Unity 2023.3.15f1
- `"2022.3.42f1"` - Unity 2022.3.42f1
- `"2021.3.35f1"` - Unity 2021.3.35f1

**Error Handling:**
The system handles various error cases:
- Missing `ProjectSettings/ProjectVersion.txt` file
- Invalid YAML format
- Missing or empty `m_EditorVersion` field
- I/O errors when reading the file


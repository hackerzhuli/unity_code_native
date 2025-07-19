use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::time::sleep;
use tokio::fs;
use crate::dir_changed::{DirChanged};

/// Errors that can occur during UXML schema processing
#[derive(Error, Debug)]
pub enum UxmlSchemaError {
    /// File system I/O operation failed
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// XML parsing failed due to malformed content
    #[error("XML parsing error: {0}")]
    XmlParsing(#[from] quick_xml::Error),
    /// UTF-8 string conversion failed
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

/// Information about a Unity UI visual element extracted from UXML schema files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualElementInfo {
    /// The simple name of the visual element (e.g., "Button", "Label")
    pub name: String,
    /// The namespace containing this element (e.g., "UnityEngine.UIElements")
    pub namespace: String,
    /// The fully qualified name combining namespace and element name (e.g., "UnityEngine.UIElements.Button")
    pub fully_qualified_name: String,
}

#[derive(Debug)]
struct SchemaFileInfo {
    last_modified: SystemTime,
    namespace: String,
    elements: Vec<String>,
}

/// Core data structure containing visual elements and providing lookup functionality
/// This struct is designed to be shared between threads using Arc<Mutex<T>>
#[derive(Debug, Default)]
pub struct VisualElementsData {
    /// Visual elements by fully qualified name
    visual_elements: HashMap<String, VisualElementInfo>,
    /// Visual element name to fully qualified name(there might be name collisions, new names will override old name)
    name_to_full_name: HashMap<String, String>,
}

impl VisualElementsData {
    /// Creates a new empty VisualElementsData instance
    pub fn new() -> Self {
        Self {
            visual_elements: HashMap::new(),
            name_to_full_name: HashMap::new(),
        }
    }

    /// Looks up a visual element by its fully qualified name
    /// 
    /// # Arguments
    /// 
    /// * `fully_qualified_name` - The fully qualified name (e.g., "UnityEngine.UIElements.Button")
    /// 
    /// # Returns
    /// 
    /// * `Some(VisualElementInfo)` if the element exists (cloned)
    /// * `None` if no element with that name is found
    pub fn lookup(&self, fully_qualified_name: &str) -> Option<&VisualElementInfo> {
        self.visual_elements.get(fully_qualified_name)
    }

    /// Looks up visual elements by name (without namespace)
    /// 
    /// # Arguments
    /// 
    /// * `name` - The simple name of the visual element (e.g., "Button")
    /// 
    /// # Returns
    /// 
    /// * `Some(&VisualElementInfo)` if elements with that name exist
    /// * `None` if no elements with that name are found
    pub fn lookup_by_name(&self, name: &str) -> Option<&VisualElementInfo> {
        let full_name = self.name_to_full_name.get(name);
        if let Some(full_name) = full_name {
            return self.lookup(full_name);
        }
        None
    }

    /// Returns all available visual elements from all loaded schema files
    pub fn get_all_elements(&self) -> &HashMap<String, VisualElementInfo> {
        &self.visual_elements
    }

    /// Returns all available visual elements names(without namespace) from all loaded schema files
    /// Maps to full name
    pub fn get_all_names(&self) -> &HashMap<String, String> {
        &self.name_to_full_name
    }

    /// Clears all visual elements
    pub fn clear(&mut self) {
        self.visual_elements.clear();
        self.name_to_full_name.clear();
    }

    /// Inserts a visual element into the collection
    pub fn insert(&mut self, fully_qualified_name: String, element_info: VisualElementInfo) {
        let name = element_info.name.clone();
        self.visual_elements.insert(fully_qualified_name.clone(), element_info);
        self.name_to_full_name.insert(name, fully_qualified_name);
    }
}

/// Manages Unity UXML schema files and provides lookup functionality for UI elements
/// 
/// This manager monitors a directory of XSD schema files, parses them to extract
/// visual element definitions, and provides efficient lookup capabilities for
/// Unity UI elements by their fully qualified names.
pub struct UxmlSchemaManager {
    schema_directory: PathBuf,
    schema_files: HashMap<PathBuf, SchemaFileInfo>,
    visual_elements_data: Arc<Mutex<VisualElementsData>>,
    dir_changed: DirChanged,
    last_scan_timestamp: u64,
}

impl UxmlSchemaManager {
    /// Creates a new UxmlSchemaManager instance for the specified schema directory
    /// 
    /// # Arguments
    /// 
    /// * `schema_directory` - Path to the directory containing Unity UXML schema (.xsd) files
    pub fn new(schema_directory: PathBuf) -> Self {
        let schema_dir = schema_directory.clone();
        
        // Set up directory change monitoring for .xsd files
        let dir_changed = match DirChanged::new(&schema_dir, Some("xsd")) {
            Ok(watcher) => watcher,
            Err(_) => {
                // Fallback to no-watcher mode if setup fails
                DirChanged::new_without_watcher()
            }
        };
        
        Self {
            schema_directory: schema_dir,
            schema_files: HashMap::new(),
            visual_elements_data: Arc::new(Mutex::new(VisualElementsData::new())),
            dir_changed,
            last_scan_timestamp: 0,
        }
    }

    /// Returns a clone of the Arc<Mutex<VisualElementsData>> for sharing with other components
    pub fn get_visual_elements_data(&self) -> Arc<Mutex<VisualElementsData>> {
        Arc::clone(&self.visual_elements_data)
    }

    pub async fn some(&mut self) -> (){
        sleep(Duration::from_millis(1000)).await;
    }

    /// Updates the schema data by scanning for file changes
    /// 
    /// Only performs directory scanning if changes have been detected by the file watcher.
    /// This optimization avoids expensive file system operations when no changes occurred.
    /// 
    /// ## Returns
    /// 
    /// * `Ok(())` if the update completed successfully
    /// * `Err(UxmlSchemaError)` if file I/O or XML parsing failed
    pub async fn update(&mut self) -> Result<(), UxmlSchemaError> {        
        // Check if directory has changed since last scan
        let current_timestamp = self.dir_changed.last_change_timestamp();
        
        // Skip scan if no changes detected since last scan
        if current_timestamp <= self.last_scan_timestamp {
            return Ok(());
        }

        log::info!("Schema directory changed scanning for .xsd files in: {}", 
                   self.schema_directory.display());
                   
        let start_time = std::time::Instant::now();

        let mut current_files = HashSet::new();
        let mut any_changes = false;
        let mut processed_files_count = 0;
        
        // Read directory entries
        let mut dir_entries = fs::read_dir(&self.schema_directory).await?;
        
        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("xsd") {
                current_files.insert(path.clone());
                
                let metadata = entry.metadata().await?;
                let last_modified = metadata.modified()?;
                
                // Check if file needs to be processed
                let needs_update = match self.schema_files.get(&path) {
                    Some(file_info) => file_info.last_modified != last_modified,
                    None => true,
                };
                
                if needs_update {
                    self.process_schema_file(&path, last_modified).await?;
                    processed_files_count += 1;
                    any_changes = true;
                }
            }
        }
        
        // Remove files that no longer exist
        let removed_files: Vec<PathBuf> = self.schema_files.keys()
            .filter(|path| !current_files.contains(*path))
            .cloned()
            .collect();
            
        if !removed_files.is_empty() {
            for path in removed_files {
                self.schema_files.remove(&path);
            }
            any_changes = true;
        }
        
        // If any changes occurred, rebuild the visual_elements from all cached files
        if any_changes {
            self.rebuild_visual_elements();
        }
        
        // Update last scan timestamp
        self.last_scan_timestamp = current_timestamp;
        
        let duration = start_time.elapsed();
        log::info!("Schema update completed in {:.2}ms (found {} files, processed {} files, changes detected: {})", 
                   duration.as_secs_f64() * 1000.0, current_files.len(), processed_files_count, any_changes);
        
        Ok(())
    }

    async fn process_schema_file(&mut self, path: &Path, last_modified: SystemTime) -> Result<(), UxmlSchemaError> {
        let content = fs::read_to_string(path).await?;
        let (namespace, elements) = self.parse_schema_content(&content)?;
        
        // Update file info cache
        let file_info = SchemaFileInfo {
            last_modified,
            namespace,
            elements,
        };
        self.schema_files.insert(path.to_path_buf(), file_info);
        
        Ok(())
    }
    
    /// Rebuilds the visual_elements HashMap from all cached schema files
    fn rebuild_visual_elements(&mut self) {
        if let Ok(mut data) = self.visual_elements_data.lock() {
            data.clear();
            
            for file_info in self.schema_files.values() {
                for element_name in &file_info.elements {
                    let fqn = format!("{}.{}", file_info.namespace, element_name);
                    let element_info = VisualElementInfo {
                        name: element_name.clone(),
                        namespace: file_info.namespace.clone(),
                        fully_qualified_name: fqn.clone(),
                    };
                    data.insert(fqn, element_info);
                }
            }
        } else {
            log::error!("Failed to acquire lock on visual_elements_data for rebuilding");
        }
    } 

    fn parse_schema_content(&self, content: &str) -> Result<(String, Vec<String>), UxmlSchemaError> {
        let mut reader = Reader::from_str(content);
        
        let mut namespace = String::new();
        let mut elements = Vec::new();
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(ref e) | Event::Empty(ref e) => {
                    match e.name().as_ref() {
                        b"xs:schema" => {
                            // Extract targetNamespace attribute
                            for attr in e.attributes() {
                                match attr {
                                    Ok(attr) => {
                                        if attr.key.as_ref() == b"targetNamespace" {
                                            match std::str::from_utf8(&attr.value) {
                                                Ok(value) => namespace = value.to_string(),
                                                Err(_) => continue,
                                            }
                                        }
                                    }
                                    Err(_) => continue,
                                }
                            }
                        }
                        b"xs:element" => {
                            // Extract element name attribute
                            for attr in e.attributes() {
                                match attr {
                                    Ok(attr) => {
                                        if attr.key.as_ref() == b"name" {
                                            match std::str::from_utf8(&attr.value) {
                                                Ok(value) => {
                                                    elements.push(value.to_string());
                                                    break;
                                                }
                                                Err(_) => continue,
                                            }
                                        }
                                    }
                                    Err(_) => continue,
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }
        
        Ok((namespace, elements))
    }
}

#[cfg(test)]
#[path = "uxml_schema_manager_tests.rs"]
mod tests;
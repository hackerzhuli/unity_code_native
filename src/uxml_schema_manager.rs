use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::fs;
use notify::{Watcher, RecursiveMode, Event as NotifyEvent, EventKind};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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
    path: PathBuf,
    last_modified: SystemTime,
    namespace: String,
    elements: Vec<String>,
}

/// Manages Unity UXML schema files and provides lookup functionality for UI elements
/// 
/// This manager monitors a directory of XSD schema files, parses them to extract
/// visual element definitions, and provides efficient lookup capabilities for
/// Unity UI elements by their fully qualified names.
pub struct UxmlSchemaManager {
    schema_directory: PathBuf,
    schema_files: HashMap<PathBuf, SchemaFileInfo>,
    visual_elements: HashMap<String, VisualElementInfo>,
    _watcher: Option<notify::RecommendedWatcher>,
    _receiver: Option<mpsc::Receiver<Result<NotifyEvent, notify::Error>>>,
    has_changes: Arc<AtomicBool>,
}

impl UxmlSchemaManager {
    /// Creates a new UxmlSchemaManager instance for the specified schema directory
    /// 
    /// # Arguments
    /// 
    /// * `schema_directory` - Path to the directory containing Unity UXML schema (.xsd) files
    pub fn new(schema_directory: PathBuf) -> Self {
        let schema_dir = schema_directory;
        let has_changes = Arc::new(AtomicBool::new(true)); // Start with true to trigger initial scan
        
        // Try to set up file watcher
        let (watcher, receiver) = Self::setup_watcher(&schema_dir, has_changes.clone())
            .unwrap_or((None, None));
        
        Self {
            schema_directory: schema_dir,
            schema_files: HashMap::new(),
            visual_elements: HashMap::new(),
            _watcher: watcher,
            _receiver: receiver,
            has_changes,
        }
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
    /// 
    /// ## Note
    /// This method can't be async because the struct is stored inside of a mutex. Async operations are not possible. 
    pub fn update(&mut self) -> Result<(), UxmlSchemaError> {
        // Check if any changes have been detected by the file watcher
        if !self.has_changes.load(Ordering::Relaxed) {
            return Ok(()); // No changes detected, skip expensive directory scan
        }
        
        // Reset the change flag
        self.has_changes.store(false, Ordering::Relaxed);
        
        let mut current_files = HashSet::new();
        let mut any_changes = false;
        
        // Read directory entries
        let dir_entries = fs::read_dir(&self.schema_directory)?;
        
        for entry in dir_entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("xsd") {
                current_files.insert(path.clone());
                
                let metadata = entry.metadata()?;
                let last_modified = metadata.modified()?;
                
                // Check if file needs to be processed
                let needs_update = match self.schema_files.get(&path) {
                    Some(file_info) => file_info.last_modified != last_modified,
                    None => true,
                };
                
                if needs_update {
                    self.process_schema_file(&path, last_modified)?;
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
        
        Ok(())
    }

    /// Looks up a visual element by its fully qualified name
    /// 
    /// # Arguments
    /// 
    /// * `fully_qualified_name` - The fully qualified name (e.g., "UnityEngine.UIElements.Button")
    /// 
    /// # Returns
    /// 
    /// * `Some(&VisualElementInfo)` if the element exists
    /// * `None` if no element with that name is found
    pub fn lookup(&self, fully_qualified_name: &str) -> Option<&VisualElementInfo> {
        self.visual_elements.get(fully_qualified_name)
    }

    /// Returns all available visual elements from all loaded schema files
    /// 
    /// # Returns
    /// 
    /// A vector containing references to all `VisualElementInfo` instances
    pub fn get_all_elements(&self) -> Vec<&VisualElementInfo> {
        self.visual_elements.values().collect()
    }

    /// Returns all visual elements belonging to a specific namespace
    /// 
    /// # Arguments
    /// 
    /// * `namespace` - The namespace to filter by (e.g., "UnityEngine.UIElements")
    /// 
    /// # Returns
    /// 
    /// A vector containing references to `VisualElementInfo` instances in the specified namespace
    pub fn get_elements_in_namespace(&self, namespace: &str) -> Vec<&VisualElementInfo> {
        self.visual_elements.values()
            .filter(|element| element.namespace == namespace)
            .collect()
    }

    fn process_schema_file(&mut self, path: &Path, last_modified: SystemTime) -> Result<(), UxmlSchemaError> {
        let content = fs::read_to_string(path)?;
        let (namespace, elements) = self.parse_schema_content(&content)?;
        
        // Update file info cache
        let file_info = SchemaFileInfo {
            path: path.to_path_buf(),
            last_modified,
            namespace,
            elements,
        };
        self.schema_files.insert(path.to_path_buf(), file_info);
        
        Ok(())
    }
    
    /// Rebuilds the visual_elements HashMap from all cached schema files
    fn rebuild_visual_elements(&mut self) {
        self.visual_elements.clear();
        
        for file_info in self.schema_files.values() {
            for element_name in &file_info.elements {
                let fqn = format!("{}.{}", file_info.namespace, element_name);
                let element_info = VisualElementInfo {
                    name: element_name.clone(),
                    namespace: file_info.namespace.clone(),
                    fully_qualified_name: fqn.clone(),
                };
                self.visual_elements.insert(fqn, element_info);
            }
        }
    } 

    fn setup_watcher(
        schema_directory: &Path,
        has_changes: Arc<AtomicBool>,
    ) -> Result<(Option<notify::RecommendedWatcher>, Option<mpsc::Receiver<Result<NotifyEvent, notify::Error>>>), notify::Error> {
        let (tx, rx) = mpsc::channel();
        let changes_flag = has_changes.clone();
        
        let mut watcher = notify::recommended_watcher(move |res: Result<NotifyEvent, notify::Error>| {
             match &res {
                 Ok(event) => {
                     // Check if the event is relevant (file creation, modification, or deletion)
                     match event.kind {
                         EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                             // Check if any of the paths have .xsd extension
                             for path in &event.paths {
                                 if path.extension().and_then(|s| s.to_str()) == Some("xsd") {
                                     changes_flag.store(true, Ordering::Relaxed);
                                     break;
                                 }
                             }
                         }
                         _ => {}
                     }
                 }
                 Err(_) => {
                     // On error, assume changes occurred to be safe
                     changes_flag.store(true, Ordering::Relaxed);
                 }
             }
             
             // Forward the event to the receiver (though we don't use it currently)
             let _ = tx.send(res);
         })?;
        
        // Watch the schema directory (non-recursive to match the original behavior)
        watcher.watch(schema_directory, RecursiveMode::NonRecursive)?;
        
        Ok((Some(watcher), Some(rx)))
    }

    fn parse_schema_content(&self, content: &str) -> Result<(String, Vec<String>), UxmlSchemaError> {
        let mut reader = Reader::from_str(content);
        reader.trim_text(true);
        
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
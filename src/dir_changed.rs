use std::path::Path;
use std::sync::{atomic::{AtomicU64, Ordering}, Arc};
use std::time::{SystemTime, UNIX_EPOCH};
use notify::{Watcher, RecursiveMode, Event as NotifyEvent, EventKind};
use std::sync::mpsc;
use thiserror::Error;

/// Errors that can occur during directory change monitoring
#[derive(Error, Debug)]
pub enum DirChangedError {
    /// File watcher setup failed
    #[error("Watcher setup error: {0}")]
    WatcherSetup(#[from] notify::Error),
}

/// Tracks directory changes using file system events and timestamps
/// 
/// This struct provides a thread-safe way to monitor directory changes
/// using a file watcher and maintains a timestamp of the last change.
/// It's designed to be used with Mutex for concurrent access.
pub struct DirChanged {
    /// Timestamp of the last detected change (nanoseconds since UNIX epoch)
    last_change_timestamp: Arc<AtomicU64>,
    /// File watcher instance (kept alive to continue monitoring)
    _watcher: Option<notify::RecommendedWatcher>,
    /// Receiver for file system events (not actively used but kept for completeness)
    _receiver: Option<mpsc::Receiver<Result<NotifyEvent, notify::Error>>>,
}

impl DirChanged {
    /// Creates a new DirChanged instance that monitors the specified directory
    /// 
    /// # Arguments
    /// 
    /// * `directory` - Path to the directory to monitor
    /// * `file_extension` - Optional file extension to filter events (e.g., "xsd")
    /// 
    /// # Returns
    /// 
    /// * `Ok(DirChanged)` if the watcher was set up successfully
    /// * `Err(DirChangedError)` if the watcher setup failed
    pub fn new(directory: &Path, file_extension: Option<&str>) -> Result<Self, DirChangedError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        let last_change_timestamp = Arc::new(AtomicU64::new(now));
        
        let (watcher, receiver) = Self::setup_watcher(
            directory,
            file_extension,
            last_change_timestamp.clone(),
        )?;
        
        Ok(Self {
            last_change_timestamp,
            _watcher: Some(watcher),
            _receiver: Some(receiver),
        })
    }
    
    /// Creates a new DirChanged instance without file watching (for testing or when watching is not needed)
    pub fn new_without_watcher() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        Self {
            last_change_timestamp: Arc::new(AtomicU64::new(now)),
            _watcher: None,
            _receiver: None,
        }
    }
    
    /// Gets the timestamp of the last detected change
    /// 
    /// # Returns
    /// 
    /// Timestamp as nanoseconds since UNIX epoch
    pub fn last_change_timestamp(&self) -> u64 {
        self.last_change_timestamp.load(Ordering::Relaxed)
    }
    
    fn setup_watcher(
        directory: &Path,
        file_extension: Option<&str>,
        timestamp: Arc<AtomicU64>,
    ) -> Result<(notify::RecommendedWatcher, mpsc::Receiver<Result<NotifyEvent, notify::Error>>), notify::Error> {
        let (tx, rx) = mpsc::channel();
        let ext_filter = file_extension.map(|s| s.to_string());
        
        let mut watcher = notify::recommended_watcher(move |res: Result<NotifyEvent, notify::Error>| {
            match &res {
                Ok(event) => {
                    // Check if the event is relevant (file creation, modification, or deletion)
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                            let should_trigger = if let Some(ref ext) = ext_filter {
                                // Check if any of the paths have the specified extension
                                event.paths.iter().any(|path| {
                                    path.extension().and_then(|s| s.to_str()) == Some(ext)
                                })
                            } else {
                                // No filter, trigger on any file change
                                true
                            };
                            
                            if should_trigger {
                                let now = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_nanos() as u64;
                                
                                timestamp.store(now, Ordering::Relaxed);
                            }
                        }
                        _ => {}
                    }
                }
                Err(_) => {
                    // On error, assume changes occurred to be safe
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64;
                    
                    timestamp.store(now, Ordering::Relaxed);
                }
            }
            
            // Forward the event to the receiver (though we don't use it currently)
            let _ = tx.send(res);
        })?;
        
        // Watch the directory (non-recursive to match the original behavior)
        watcher.watch(directory, RecursiveMode::NonRecursive)?;
        
        Ok((watcher, rx))
    }
}

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use log::{LevelFilter, Log, Metadata, Record};

struct FileLogger {
    file: Mutex<std::fs::File>,
}

impl FileLogger {
    fn new(file_path: PathBuf) -> io::Result<Self> {
        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_path)?;
        
        Ok(FileLogger {
            file: Mutex::new(file),
        })
    }
}

impl Log for FileLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if let Ok(mut file) = self.file.lock() {
                let _ = writeln!(
                    file,
                    "[{}] [{}] {}",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    record.level(),
                    record.args()
                );
                let _ = file.flush();
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

/// Get the platform-specific log file path
fn get_log_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let data_dir = dirs::data_local_dir()
        .ok_or("Could not determine local data directory")?;
    
    let unity_code_dir = data_dir.join("UnityCode");
    let log_file_path = unity_code_dir.join("unity_code_native.log");
    
    Ok(log_file_path)
}

/// Initialize the logger to write to a single file in local app data, overwriting previous logs
pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    let log_file_path = get_log_file_path()?;
    let logger = FileLogger::new(log_file_path)?;
    
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(LevelFilter::Info))?;
    
    Ok(())
}
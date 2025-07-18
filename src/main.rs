// Allow warnings, so we don't see so many warnings everytime we run tests or build
// We will clean up warnings once in a while
#![allow(warnings)] 
mod logging;
mod monitor;
mod server;
mod unity_project_manager;
mod unity_asset_database;
pub mod uxml_schema_manager;
mod dir_changed;
mod uss;
mod language;
mod cs;
#[cfg(test)]
pub mod test_utils;

use std::env;
use std::path::PathBuf;
use std::process;
use server::Server;
use unity_project_manager::UnityProjectManager;
use uss::server::start_uss_language_server;
use uxml_schema_manager::UxmlSchemaManager;
use log::{error, info};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // Use eprintln for usage info since logger isn't initialized yet
        eprintln!("Usage: {} <project_path>", args[0]);
        eprintln!("  <project_path>: Start Unity monitor server with USS Language Server");
        eprintln!("Example: {} F:\\projects\\unity\\MyProject", args[0]);
        eprintln!("Note: Both UDP server and USS Language Server run concurrently.");
        process::exit(1);
    }

    // Initialize file logging for combined mode
    if let Err(e) = logging::init_logger() {
        eprintln!("Failed to initialize logger: {}", e);
        process::exit(1);
    }
    
    // Log startup information
    info!("Unity Code Native starting with both UDP server and USS Language Server");
    info!("Command line arguments: {:?}", args);

    let target_project_path = monitor::normalize_path(&args[1]);
    info!("Monitoring project path: {}", target_project_path);
    
    // Create Unity project manager instance
    let unity_project_manager = UnityProjectManager::new(PathBuf::from(&target_project_path));
    match unity_project_manager.detect_unity_version() {
        Ok(version) => info!("Detected Unity version: {}", version),
        Err(e) => info!("Unity project detection failed: {}", e),
    }
    
    // Create UXML schema manager once for the entire application
    let uxml_schema_manager = UxmlSchemaManager::new(PathBuf::from(&target_project_path).join("UIElementsSchema"));
    info!("UXML schema manager created");

    // Start UDP server first
    let target_project_path_clone = target_project_path.clone();
    let udp_server_task = async move {
        match Server::new(target_project_path_clone).await {
            Ok(mut server) => {
                info!("UDP server started successfully");
                server.run().await;
                Ok(())
            }
            Err(e) => {
                error!("Failed to create UDP server: {}", e);
                Err(e)
            }
        }
    };
    
    // Start USS Language Server concurrently
    let project_path_for_lsp = PathBuf::from(&target_project_path);
    let lsp_server_task = async move {
        info!("Starting USS Language Server (will handle LSP requests when connected)");
        if let Err(e) = start_uss_language_server(project_path_for_lsp, uxml_schema_manager).await {
            error!("USS Language Server error: {:?}", e);
        }
        info!("USS Language Server stopped");
    };
    
    // Run both servers concurrently - if either stops, continue with the other
    tokio::select! {
        result = udp_server_task => {
            if let Err(_) = result {
                error!("UDP server failed, shutting down");
                process::exit(1);
            }
            info!("UDP server stopped");
        }
        _ = lsp_server_task => {
             info!("LSP server task completed, UDP server continues running");
             // Continue running UDP server even if LSP server stops
             match Server::new(target_project_path).await {
                 Ok(mut server) => {
                     server.run().await;
                 }
                 Err(e) => {
                     error!("Failed to restart UDP server: {}", e);
                     process::exit(1);
                 }
             }
         }
    }
    
    info!("Unity Code Native shutting down");
}

mod logging;
mod monitor;
mod server;
mod uss;

use std::env;
use std::process;
use server::Server;
use uss::server::start_uss_language_server;
use log::{error, info};

#[tokio::main]
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
    let lsp_server_task = async {
        info!("Starting USS Language Server (will handle LSP requests when connected)");
        if let Err(e) = start_uss_language_server().await {
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

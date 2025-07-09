mod monitor;
mod server;
mod uss;

use std::env;
use std::process;
use server::Server;
use uss::server::start_uss_language_server;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} [--uss-lsp | <project_path>]", args[0]);
        eprintln!("  --uss-lsp: Start USS Language Server");
        eprintln!("  <project_path>: Start Unity monitor server");
        eprintln!("Example: {} F:\\projects\\unity\\MyProject", args[0]);
        process::exit(1);
    }

    // Check if starting USS language server
    if args[1] == "--uss-lsp" {
        eprintln!("Starting USS Language Server...");
        if let Err(e) = start_uss_language_server().await {
            eprintln!("USS Language Server error: {:?}", e);
            process::exit(1);
        }
        return;
    }

    let target_project_path = monitor::normalize_path(&args[1]);
    eprintln!("Monitoring project path: {}", target_project_path);

    // Create and run the UDP server
    match Server::new(target_project_path) {
        Ok(mut server) => {
            server.run();
        }
        Err(e) => {
            eprintln!("Failed to create server: {}", e);
            process::exit(1);
        }
    }
}

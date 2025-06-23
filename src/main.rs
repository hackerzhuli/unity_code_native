mod monitor;
mod server;

use std::env;
use std::process;
use server::Server;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <project_path>", args[0]);
        eprintln!("Example: {} F:\\projects\\unity\\MyProject", args[0]);
        process::exit(1);
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

# Server Usage

The `Server` struct implements the UDP messaging protocol defined in `MessagingProtocol.md`.

## Features

- **UDP Communication**: Listens on port `50000 + (PID % 1000)` for incoming messages
- **Client Management**: Tracks connected clients and removes inactive ones after 30 seconds
- **Message Protocol**: Supports the defined message format with type, payload length, and payload
- **Real-time State Updates**: Calls `monitor.update()` when state requests are received
- **Periodic Updates**: Automatically updates monitor every 10 seconds when no requests are received
- **Change Detection**: Monitors process ID changes and broadcasts updates to all connected clients
- **State Broadcasting**: Automatically notifies all clients when Unity or HotReload process states change
- **Blocking Socket**: Uses a blocking UDP socket with 1-second read timeout for efficient operation

## Usage

```rust
use unity_code_native::Server;

fn main() {
    let project_path = "F:\\projects\\unity\\MyProject".to_string();
    
    match Server::new(project_path) {
        Ok(mut server) => {
            server.run(); // Runs indefinitely
        }
        Err(e) => {
            eprintln!("Failed to create server: {}", e);
        }
    }
}
```

## Supported Messages

### GetUnityState (Type 1)
- **Request**: Empty payload
- **Response**: JSON-serialized `ProcessState` struct

Example response:
```json
{
  "IsUnityRunning": true,
  "IsHotReloadEnabled": false
}
```

## Testing

You can test the server using the provided example client:

```bash
# Terminal 1: Start the server
cargo run -- "F:\projects\unity\MyProject"

# Terminal 2: Run the test client
cargo run --example test_client
```

## Implementation Details

- **Synchronous**: Uses blocking UDP socket with 1-second read timeout
- **Simple**: No async/await complexity, straightforward message loop
- **Robust**: Handles malformed messages gracefully
- **Efficient**: Minimal memory allocations, reuses buffers where possible
- **Responsive**: Always calls monitor.update() on GetUnityState requests
- **Configurable**: 10-second monitor update interval (MONITOR_UPDATE_INTERVAL const)
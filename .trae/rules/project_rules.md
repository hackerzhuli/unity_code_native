# Unity Code Native - Project Rules

## Project Overview

**unity_code_native** is the native Rust backend for the Unity Code Pro VS Code extension. It provides Unity Editor detection, Unity-specific language servers (USS, etc.), and other Unity development utilities in a single performant binary.

## Tech Stack & Versions

- **Rust Edition**: 2024
- **Async Runtime**: Tokio (single-threaded async only)
- **Language Server Protocol**: tower-lsp
- **Parser**: tree-sitter for syntax analysis
- **Communication**: UDP-based messaging for custom protocols
- **Process Detection**: sysinfo crate
- **Serialization**: serde + serde_json

## Architecture Principles

- **Single-threaded async**: Use tokio but avoid multi-threading to keep complexity low
- **Minimal dependencies**: Only include necessary crates to maintain fast build times
- **Cross-platform**: Support Windows, macOS, and Linux

## Code Guidelines

### General
- Use `cargo fmt` for consistent formatting
- Run `cargo clippy` and address all warnings
- Prefer explicit error handling with `Result<T, E>`
- Use meaningful variable and function names

### Async Code
- Use `async/await` syntax consistently
- Avoid `spawn` unless absolutely necessary (single-threaded principle)
- Use `tokio::select!` for concurrent operations
- Prefer `async fn` over `impl Future`

### Error Handling
- Create custom error types using `thiserror` when needed
- Use `anyhow` for application-level error handling
- Always handle errors explicitly, avoid `unwrap()` in production code
- Use `expect()` with descriptive messages for truly impossible failures

### Language Server Implementation
- Follow LSP specification strictly
- Use tower-lsp's provided traits and structures
- Implement incremental parsing with tree-sitter
- Cache parse trees for performance

### Unity Integration
- Monitor Unity processes efficiently using sysinfo
- Detect project associations through file system analysis
- Support Unity's file formats (USS, UXML, etc.)
- Handle Unity Editor state changes gracefully

### Testing
- Write unit tests for core functionality that is easy to write test for
- Integration tests are not needed

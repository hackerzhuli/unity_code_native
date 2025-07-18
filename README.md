# Unity Code Native

## Description

Unity Code Native is a high-performance Rust backend that powers advanced Unity development tools. Originally designed for Unity process detection, it has evolved into a comprehensive language server suite that provides intelligent IDE features for Unity's UI Toolkit and C# documentation systems.

## Main Features

### 🎨 USS Language Server (Primary Feature)
A feature-complete language server for Unity Style Sheets (USS) that brings modern IDE capabilities to Unity's UI Toolkit development:

- **Syntax Highlighting**: Of course!
- **High performance**: Written in rust, built from ground up for performance. Get instant feedback(diagnostics) on syntax and values as you type! 100% high performance as a Game Dev's code should be!
- **Comprehensive Auto-completion**: Property names, values, selectors, pseudo-classes, and asset URLs. For element names, it know all Unity Engine element's lile `Button` and `Label` and will provide auto completion if you type them. For url, auto completion will complete from `Assets` all the way down to the individual sprite in the image(if it is a multiple sprite image asset). 
- **Advanced Diagnostics**: Syntax validation, duplicate property detection, and asset path validation, property value checks, everything you ever need and more. 100% USS native, completely validates every property that USS has and can check the value you provide it with high accuracy, almost produce the same errors (and more) as Unity itself does. It goes above and beyond and try to validate property values even if it has var() in it, which no one, not Unity, or any CSS language server does(though it is not 100% accurate because we never know what variables real value will be at runtime).
- **Intelligent Hover Documentation**: Rich tooltips with syntax examples and keyword explanations. No need to check official docs when you have quick hover docs that is completely Unity specific, no CSS shenanigans(almost). Also, a link to official (mostly Unity's) docs is provided.
- **Code Formatting**: Document and selection formatting for USS and TSS files
- **Refactoring**: Rename operations for ID and class selectors

### 📚 C# Documentation System
Automated XML documentation extraction and compilation for Unity projects:

- **Package Documentation**: Extracts XML docs from Unity packages in `Library/PackageCache`
- **User Code Documentation**: Processes user assemblies with full member documentation
- **Assembly Watching**: Real-time updates when Unity recompiles assemblies
- **Inheritance Resolution**: Resolves `<inheritdoc>` references across assemblies
- **Efficient Storage**: Compiles documentation into optimized JSON format

### 🔍 Unity Process Detection
Original core functionality for Unity Editor integration:

- Detects running Unity Editor instances for specific projects
- Monitors Hot Reload for Unity status
- UDP-based messaging protocol for real-time communication
- Cross-platform process monitoring

## Why Rust?

Rust provides the perfect balance of performance and safety for this project:

- **Process Detection**: JavaScript's process detection is too slow (requires PowerShell on Windows), and C# lacks parent process ID access
- **Performance**: Rust allows precise control over system resource usage without unnecessary overhead
- **Memory Safety**: Critical for long-running language server processes
- **Cross-platform**: Single codebase supports Windows, macOS, and Linux
- **Async Efficiency**: Tokio provides excellent single-threaded async performance

## Usage

### As a Language Server
The primary use case is as a language server for USS files in Unity projects. The binary integrates with VS Code extensions and other LSP-compatible editors to provide:

- Real-time syntax highlighting and error detection
- Context-aware auto-completion as you type
- Hover documentation for properties and values
- Code formatting and refactoring capabilities

### Command Line Usage
For Unity process detection and monitoring:

```bash
unity_code_native.exe "C:\path\to\your\Unity\Project"
```

The tool will detect running Unity Editor instances and provide status information via UDP messaging.

## Test
Some tests rely on the embedded Unity Project in `UnityProject` directory. Unity Engine generated files is needed for some tests to pass. So if you want to run full tests, you need to use Unity Engine to open the embedded Unity Project in `UnityProject` directory before running the tests.

## Build

Ensure you have the latest Rust toolchain installed:

```bash
cargo build --release
```

The resulting binary in `target/release/` is self-contained and can be deployed anywhere.

**Cross-platform**: Build on your target platform for optimal compatibility.

## Development

For rapid development iterations:

```powershell
# Fast build without optimizations
cargo build --profile release-fast

# Copy to target directory (adjust path as needed)
Copy-Item -Path target\release-fast\unity_code_native.exe -Destination F:\projects\js\UnityCode\bin\win_x64
```

## Architecture

- **Single-threaded async**: Uses Tokio for efficient async operations without threading complexity
- **tree-sitter**: Leverages tree-sitter-css for robust USS parsing
- **tower-lsp**: Implements Language Server Protocol for editor integration
- **Cross-platform**: Supports Windows, macOS, and Linux

## Version 1.0 Release

This 1.0 release represents a mature, production-ready language server with comprehensive USS support and robust C# documentation features. The USS language server is particularly feature-complete, offering an experience comparable to modern CSS tooling but tailored specifically for Unity's UI Toolkit development workflow.
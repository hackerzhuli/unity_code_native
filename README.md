# Unity Code Native - Process Information Tool

A Rust executable that retrieves process information and outputs it in CSV format.

## Description

This tool takes a single command-line argument (process name, e.g., "Unity.exe") and searches for all running processes matching that name. It then outputs detailed process information in CSV format.

## Output Format

The tool outputs CSV data with the following columns:
- **Process Name**: The name of the process
- **Process ID**: The unique process identifier (PID)
- **Parent ID**: The process ID of the parent process (PPID)
- **Command Line**: The full command line used to start the process

## Usage

```bash
# Build the project
cargo build --release

# Run with a process name
cargo run -- Unity.exe

# Or run the compiled executable
./target/release/unity_code_native Unity.exe
```

## Example Output

```csv
Process Name,Process ID,Parent ID,Command Line
Unity.exe,1234,5678,"C:\Program Files\Unity\Hub\Editor\2023.1.0f1\Editor\Unity.exe" -projectPath "C:\MyProject"
Unity.exe,9876,5432,"C:\Program Files\Unity\Hub\Editor\2023.2.0f1\Editor\Unity.exe" -batchmode
```

## Dependencies

- `sysinfo`: For cross-platform system and process information
- `csv`: For CSV output formatting

## Features

- Cross-platform compatibility (Windows, macOS, Linux)
- Efficient process enumeration
- Clean CSV output format
- Command-line argument validation
- Error handling for invalid process names

## Requirements

- Rust 2024 edition or later
- Cargo package manager

## License

This project is open source.
# Unity Code Native
## Description

This tool serves as a bridge between VS Code extension Unity Code and Unity Editor instances. It can detect running Unity processes, extract project information, and provide real-time Unity state monitoring through a UDP-based messaging protocol.

## Why rust, not C# or javascript?

It turns out process detection is very tricky in javascript, which involves running powershell in windows, which takes seconds, too inefficient for our case. What about C#? It turns out that C# don't have a way of getting parent id of a process, which makes the implementation tricky again, we do need parent id. So rust is the way to go.

## Key Features

### Unity Process Detection
- Automatically detects running Unity Editor instances
- Extracts Unity project paths from command line arguments
- Supports various Unity command line options (`-projectpath`, `-createproject`, etc.)
- Handles quoted/unquoted paths and international characters

### Real-time Communication
- UDP-based messaging protocol for Unity state monitoring
- Automatic Unity state change notifications
- Hot reload detection capabilities
- Keep-alive messaging system

## Build
Make sure you update your Rust toolchain to the latest version if needed. Run the following command:

```bash
cargo build --release 
```

That's it, find the binary in `target/release/`, it is a single file, copy it to where you need it.

Note that I assume you build the executable in the platform that you build for. I don't provide info about how to cross compile here. 
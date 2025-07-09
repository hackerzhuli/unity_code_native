# Unity Code Native
## Description

This command line tool is a utility for Unity related process detection. It can detect running Unity process for the specified project path, and provide real-time Unity state monitoring through a UDP-based messaging protocol.

## Why rust, not C# or javascript?

It turns out process detection is very tricky in javascript, which involves running powershell in windows, which takes seconds, too inefficient for our case. What about C#? It turns out that C# don't have a way of getting parent id of a process, which makes the implementation tricky again, we do need parent id. So rust is the way to go. Also performance is very good with rust, because we can specify exactly what info we need for a process, without wasting time retriving info like CPU usage, memory usage and so on.

## Key Features
- Run the executable with one argument the path of the Unity project
- Automatically detects running Unity Editor instance for the project
- Detects whether Hot Reload for Unity is running for the project when requested
- UDP-based messaging protocol for communication

## Build
Make sure you update your Rust toolchain to the latest version if needed. Run the following command:

```bash
cargo build --release 

```

```powershell
# use this line to quicly build and copy to target directory with powershell
cargo build --release; Copy-Item -Path target\release\unity_code_native.exe -Destination F:\projects\js\UnityCode\bin\win_x64 
```

That's it, find the binary in `target/release/`, it is a single file, copy it to where you need it.

Note that I assume you build the executable in the platform that you build for. I don't provide info about how to cross compile here. 
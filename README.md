# Unity Code Native - Unity Development Integration Tool

A Rust-based tool designed for Unity development workflow integration that monitors Unity Editor processes and provides real-time communication capabilities for development tools.

## Description

This tool serves as a bridge between external development tools and Unity Editor instances. It can detect running Unity processes, extract project information, and provide real-time Unity state monitoring through a UDP-based messaging protocol.

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

## Use Cases

- **IDE Extensions**: Development tools that need to know which Unity project is currently active
- **Hot Reload Systems**: Tools that want to integrate with Unity's hot reload functionality
- **Development Workflow Automation**: Scripts that need to monitor Unity Editor state
- **Multi-project Management**: Tools managing multiple Unity projects simultaneously
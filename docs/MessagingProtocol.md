# Messaging Protocol

## Base configuration
We communicate through udp socket.

port: 50000 + (pid % 1000)

drops the client if no message is sent for 30 seconds.

## Message format
a byte for message type, 4 bytes (little endian) for payload length, the rest is a utf8 string, that is serialized json from struct, can be empty.

### Message table
| message type | name | payload | description |
| --- | --- | --- | --- |
| 0 | None | empty| does nothing, but can be used to keep the connection alive
| 1 | GetUnityState | request is empty, response is ProcessState | Get the current state of Unity process, including whether Hot Reload is enabled.

``` rust
pub enum MessageType{
    None,
    GetUnityState,
}

pub struct ProcessState {
    UnityProcessId: u32, // 0 if Unity is not running
    IsHotReloadEnabled: bool,
}
```

Notes for GetUnityState:
- Even if there is no request, if Unity state change is detected, the client will get the message, note that whether Hot Reload is enabled is not reliable, because it is not detected unless requested
- If client needs to know if Hot Reload for Unity is enabled for certain, client must send the request
- Checking process state can be slow, expect it to take 100ms

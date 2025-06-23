use std::net::UdpSocket;
use std::io;

fn main() -> io::Result<()> {
    // Connect to the server
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let server_addr = "127.0.0.1:50000"; // Assuming PID % 1000 = 0 for simplicity
    
    println!("Sending GetUnityState request to {}", server_addr);
    
    // Create GetUnityState message (message type 1, empty payload)
    let mut message = Vec::new();
    message.push(1u8); // MessageType::GetUnityState
    message.extend_from_slice(&0u32.to_le_bytes()); // payload length = 0
    
    // Send the message
    socket.send_to(&message, server_addr)?;
    
    // Receive response
    let mut buffer = [0u8; 1024];
    let (size, _) = socket.recv_from(&mut buffer)?;
    
    if size >= 5 {
        let message_type = buffer[0];
        let payload_length = u32::from_le_bytes([
            buffer[1], buffer[2], buffer[3], buffer[4]
        ]) as usize;
        
        if size >= 5 + payload_length {
            let payload = std::str::from_utf8(&buffer[5..5 + payload_length])
                .unwrap_or("<invalid utf8>");
            
            println!("Received response:");
            println!("  Message Type: {}", message_type);
            println!("  Payload Length: {}", payload_length);
            println!("  Payload: {}", payload);
        }
    }
    
    Ok(())
}
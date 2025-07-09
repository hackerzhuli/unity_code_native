use std::{
    collections::HashMap,
    io,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use sysinfo::PidExt;
use tokio::{
    net::UdpSocket,
    time::interval,
};
use log::{debug, error, info, warn};
use crate::monitor::ProcessMonitor;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum MessageType {
    None = 0,
    GetUnityState = 1,
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            0 => MessageType::None,
            1 => MessageType::GetUnityState,
            _ => MessageType::None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ProcessState {
    #[serde(rename = "UnityProcessId")]
    pub unity_process_id: u32,
    #[serde(rename = "IsHotReloadEnabled")]
    pub is_hot_reload_enabled: bool,
}

// Time interval for periodic detect Unity when Unity is not yet detected
const DETECT_UNITY_INTERVAL: Duration = Duration::from_secs(10);

struct ClientInfo {
    last_message_time: Instant,
}

pub struct Server {
    socket: UdpSocket,
    clients: HashMap<std::net::SocketAddr, ClientInfo>,
    monitor: ProcessMonitor,
    last_monitor_update: Instant,
}

impl Server {
    pub async fn new(project_path: String) -> io::Result<Self> {
        let pid = std::process::id();
        let port = 50000 + (pid % 1000);
        let addr = format!("127.0.0.1:{}", port);

        let socket = UdpSocket::bind(&addr).await?;

        info!("Server listening on {}", addr);

        Ok(Server {
            socket,
            clients: HashMap::new(),
            monitor: ProcessMonitor::new(project_path),
            last_monitor_update: Instant::now() - DETECT_UNITY_INTERVAL, // we want to update immediately
        })
    }

    pub async fn run(&mut self) {
        let mut buffer = [0u8; 1024];
        let mut cleanup_interval = interval(Duration::from_secs(5));
        let mut monitor_interval = interval(Duration::from_millis(100));

        loop {
            tokio::select! {
                // Handle incoming messages
                result = self.socket.recv_from(&mut buffer) => {
                    match result {
                        Ok((size, addr)) => {
                            self.handle_message(&buffer[..size], addr).await;
                        }
                        Err(e) => {
                            error!("Error receiving message: {}", e);
                        }
                    }
                }
                
                // Clean up inactive clients periodically
                _ = cleanup_interval.tick() => {
                    self.cleanup_inactive_clients();
                }
                
                // Monitor Unity processes
                _ = monitor_interval.tick() => {
                    // check if unity is already detected or DETECT_UNITY_INTERVAL is reached
                    // this will make detect new Unity instance slow and find out Unity shutdown fast
                    if self.last_monitor_update.elapsed() >= DETECT_UNITY_INTERVAL || self.monitor.unity_pid().is_some() {
                        // only checks unity
                        if self.monitor_update(false) {
                            info!("state changed to {:?}, broadcast to clients", self.get_process_state());
                            self.broadcast_state().await;
                        }
                    }
                }
            }
        }
    }

    /** 
     * update the process monitor
     * 
     * returns true if the state has changed
     */
    fn monitor_update(&mut self, is_full:bool) -> bool {
        let start = Instant::now();

        let old_state = self.get_process_state();
        
        self.monitor.update(is_full);
        self.last_monitor_update = Instant::now();
        
        let new_state = self.get_process_state();

        #[cfg(debug_assertions)]{
            debug!("monitor update took: {:?}, is_full:{}, state is:{:?}", start.elapsed(), is_full, new_state);
        }
        old_state != new_state
    }

    async fn handle_message(&mut self, data: &[u8], addr: std::net::SocketAddr) {
        if data.len() < 9 {
            warn!("Invalid message format: too short");
            return;
        }

        // Update client last message time
        self.clients.insert(
            addr,
            ClientInfo {
                last_message_time: Instant::now(),
            },
        );

        let message_type = MessageType::from(data[0]);
        let request_id = u32::from_le_bytes([data[1], data[2], data[3], data[4]]);
        let payload_length = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as usize;

        if data.len() < 9 + payload_length {
            warn!("Invalid message format: payload length mismatch");
            return;
        }

        let _payload = if payload_length > 0 {
            match std::str::from_utf8(&data[9..9 + payload_length]) {
                Ok(s) => s,
                Err(_) => {
                    warn!("Invalid UTF-8 in payload");
                    return;
                }
            }
        } else {
            ""
        };

        match message_type {
            MessageType::None => {
                // Do nothing
            }
            MessageType::GetUnityState => {
                self.handle_get_unity_state(addr, request_id).await;
            }
        }
    }

    async fn handle_get_unity_state(&mut self, addr: std::net::SocketAddr, request_id: u32) {
        // Always update monitor when state is requested(full check)
        let _changed = self.monitor_update(true);

        self.send_state(addr, request_id).await;
    }

    async fn send_state(&mut self, addr: std::net::SocketAddr, request_id: u32) {
        // Return real process state data from monitor
        let state = self.get_process_state();

        match serde_json::to_string(&state) {
            Ok(json) => {
                self.send_response(MessageType::GetUnityState, request_id, &json, addr).await;
            }
            Err(e) => {
                error!("Error serializing ProcessState: {}", e);
            }
        }
    }

    fn get_process_state(&mut self) -> ProcessState {
        ProcessState {
            unity_process_id: match self.monitor.unity_pid() {
                Some(pid) => pid.as_u32(),
                None => 0,
            },
            is_hot_reload_enabled: self.monitor.hot_reload_pid().is_some(),
        }
    }
    
    async fn broadcast_state(&mut self) {
        // Return real process state data from monitor
        let state = self.get_process_state();

        match serde_json::to_string(&state) {
            Ok(json) => {
                self.broadcast(MessageType::GetUnityState, json).await;
            }
            Err(e) => {
                error!("Error serializing ProcessState for broadcast: {}", e);
            }
        }
    }

    async fn broadcast(&mut self, message_type: MessageType, json: String) {
        // Send to all connected clients
        let clients: Vec<std::net::SocketAddr> = self.clients.keys().cloned().collect();
        for addr in clients {
            self.send_response(message_type, 0, &json, addr).await; // request_id = 0 for broadcasts
        }
    }
    
    async fn send_response(&self, message_type: MessageType, request_id: u32, payload: &str, addr: std::net::SocketAddr) {
        let payload_bytes = payload.as_bytes();
        let payload_length = payload_bytes.len() as u32;

        let mut response = Vec::with_capacity(9 + payload_bytes.len());
        response.push(message_type as u8);
        response.extend_from_slice(&request_id.to_le_bytes());
        response.extend_from_slice(&payload_length.to_le_bytes());
        response.extend_from_slice(payload_bytes);

        if let Err(e) = self.socket.send_to(&response, addr).await {
            error!("Error sending response to {}: {}", addr, e);
        }
    }

    fn cleanup_inactive_clients(&mut self) {
        let now = Instant::now();
        let timeout = Duration::from_secs(30);

        self.clients.retain(|addr, client| {
            let is_active = now.duration_since(client.last_message_time) < timeout;
            if !is_active {
                info!("Dropping inactive client: {}", addr);
            }
            is_active
        });
    }
}

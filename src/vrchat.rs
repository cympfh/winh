use rosc::encoder;
use rosc::{OscMessage, OscPacket, OscType};
use std::net::UdpSocket;

#[derive(Debug)]
pub enum VRChatError {
    SocketError(String),
    SendError(String),
}

impl std::fmt::Display for VRChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VRChatError::SocketError(msg) => write!(f, "Socket error: {}", msg),
            VRChatError::SendError(msg) => write!(f, "Send error: {}", msg),
        }
    }
}

impl std::error::Error for VRChatError {}

pub struct VRChatClient {
    pub target_addr: String,
}

impl VRChatClient {
    pub fn new() -> Self {
        Self {
            target_addr: "127.0.0.1:9091".to_string(),
        }
    }

    /// Send a message to VRChat via OSC
    pub fn send_message(&self, message: &str) -> Result<(), VRChatError> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| VRChatError::SocketError(format!("Failed to bind socket: {}", e)))?;

        self.send_chatbox_input(&socket, message, true)
    }

    fn send_chatbox_input(
        &self,
        socket: &UdpSocket,
        text: &str,
        notify: bool,
    ) -> Result<(), VRChatError> {
        let msg = OscMessage {
            addr: "/chatbox/input".to_string(),
            args: vec![
                OscType::String(text.to_string()),
                OscType::Bool(true),   // immediate
                OscType::Bool(notify), // notify sound
            ],
        };

        self.send_osc_message(socket, msg)
    }

    fn send_osc_message(&self, socket: &UdpSocket, msg: OscMessage) -> Result<(), VRChatError> {
        let packet = OscPacket::Message(msg);
        let msg_buf = encoder::encode(&packet)
            .map_err(|e| VRChatError::SendError(format!("Failed to encode OSC message: {}", e)))?;

        socket
            .send_to(&msg_buf, &self.target_addr)
            .map_err(|e| VRChatError::SendError(format!("Failed to send OSC message: {}", e)))?;

        Ok(())
    }
}

impl Default for VRChatClient {
    fn default() -> Self {
        Self::new()
    }
}

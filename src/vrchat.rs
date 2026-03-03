use rosc::decoder;
use rosc::encoder;
use rosc::{OscMessage, OscPacket, OscType};
use std::net::UdpSocket;
use std::sync::mpsc::Sender;

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
            target_addr: "127.0.0.1:9000".to_string(),
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

/// VRChat から OSC (port=9001) で MuteSelf パラメータを受信し、
/// 1秒以内に False→True と切り替わったら sender に GestureRight の値を送信する
pub fn start_mute_listener(sender: Sender<i32>) {
    std::thread::spawn(move || {
        let socket = match UdpSocket::bind("0.0.0.0:9001") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[VRChat OSC Listener] Failed to bind port 9001: {}", e);
                return;
            }
        };
        socket
            .set_read_timeout(Some(std::time::Duration::from_millis(500)))
            .ok();
        println!(
            "[VRChat OSC Listener] Listening on port 9001 for MuteSelf/GestureRight parameters"
        );

        let mut buf = [0u8; 65535];
        // False を受け取った時刻を記録する
        let mut unmute_time: Option<std::time::Instant> = None;
        // GestureRight の現在値
        let mut gesture_right: i32 = 0;

        loop {
            match socket.recv_from(&mut buf) {
                Ok((size, _addr)) => {
                    if let Ok((_, OscPacket::Message(msg))) = decoder::decode_udp(&buf[..size]) {
                        if msg.addr == "/avatar/parameters/GestureRight" {
                            gesture_right = match msg.args.first() {
                                Some(OscType::Int(i)) => *i,
                                Some(OscType::Float(f)) => *f as i32,
                                _ => gesture_right,
                            };
                            println!("[VRChat OSC Listener] GestureRight={}", gesture_right);
                        } else if msg.addr == "/avatar/parameters/MuteSelf" {
                            let is_muted = match msg.args.first() {
                                Some(OscType::Bool(b)) => *b,
                                Some(OscType::Int(i)) => *i != 0,
                                Some(OscType::Float(f)) => *f != 0.0,
                                _ => continue,
                            };
                            println!("[VRChat OSC Listener] MuteSelf={}", is_muted);

                            if !is_muted {
                                // False (ミュート解除) → 時刻を記録
                                unmute_time = Some(std::time::Instant::now());
                            } else if let Some(t) = unmute_time.take() {
                                // True (ミュート) → 直前の False から 1秒以内なら録音開始
                                if t.elapsed() <= std::time::Duration::from_secs(1) {
                                    println!(
                                        "[VRChat OSC Listener] Mute toggle detected → trigger recording (gesture_right={})",
                                        gesture_right
                                    );
                                    if sender.send(gesture_right).is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => {
                    eprintln!("[VRChat OSC Listener] recv error: {}", e);
                    break;
                }
            }
        }
        println!("[VRChat OSC Listener] Stopped");
    });
}

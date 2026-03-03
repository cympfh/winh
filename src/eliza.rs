use serde::Serialize;

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    messages: Vec<Message>,
    use_memory: bool,
    detect_sleep: bool,
}

pub struct ElizaClient {
    url: String,
}

impl ElizaClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    /// Send transcribed text to eliza-agent-server (fire-and-forget)
    pub fn send_chat(&self, text: &str) -> Result<(), String> {
        let request = ChatRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: text.to_string(),
            }],
            use_memory: false,
            detect_sleep: false,
        };

        let client = reqwest::blocking::Client::new();
        let endpoint = format!("{}/chat", self.url.trim_end_matches('/'));

        client
            .post(&endpoint)
            .json(&request)
            .send()
            .map_err(|e| format!("Failed to send to eliza: {}", e))?;

        println!("[ElizaClient] Sent to {}: {}", endpoint, text);
        Ok(())
    }
}

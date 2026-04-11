use serde::{Deserialize, Serialize};
use serde_json;

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

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: ChatResponseMessage,
}

pub struct ElizaClient {
    url: String,
}

impl ElizaClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    /// Send transcribed text to eliza-agent-server and return the response message
    pub fn send_chat(&self, text: &str) -> Result<String, String> {
        let request = ChatRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: text.to_string(),
            }],
            use_memory: false,
            detect_sleep: false,
        };

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_default();
        let endpoint = format!("{}/chat", self.url.trim_end_matches('/'));

        let response = client
            .post(&endpoint)
            .json(&request)
            .send()
            .map_err(|e| format!("Failed to send to eliza: {}", e))?;

        let raw = response
            .text()
            .map_err(|e| format!("Failed to read eliza response: {}", e))?;
        println!("[ElizaClient] Raw response from {}: {}", endpoint, raw);

        let body: ChatResponse = serde_json::from_str(&raw)
            .map_err(|e| format!("Failed to parse eliza response: {}. Body was: {}", e, raw))?;

        println!(
            "[ElizaClient] Response from {}: {}",
            endpoint, body.message.content
        );
        Ok(body.message.content)
    }
}

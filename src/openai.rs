use serde::{Deserialize, Serialize};
use std::path::Path;

const OPENAI_API_URL: &str = "https://api.openai.com/v1/audio/transcriptions";

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptionResponse {
    pub text: String,
}

#[derive(Debug)]
pub enum OpenAIError {
    NetworkError(String),
    ApiError(String),
    FileError(String),
    ParseError(String),
}

impl std::fmt::Display for OpenAIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenAIError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            OpenAIError::ApiError(msg) => write!(f, "API error: {}", msg),
            OpenAIError::FileError(msg) => write!(f, "File error: {}", msg),
            OpenAIError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for OpenAIError {}

pub struct OpenAIClient {
    api_key: String,
    model: String,
}

impl OpenAIClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }

    pub fn transcribe_audio(&self, audio_file_path: &Path) -> Result<String, OpenAIError> {
        // Check if file exists
        if !audio_file_path.exists() {
            return Err(OpenAIError::FileError(format!(
                "Audio file not found: {:?}",
                audio_file_path
            )));
        }

        println!("Transcribing audio file: {:?}", audio_file_path);
        println!("Using model: {}", self.model);

        // Read the audio file
        let audio_data = std::fs::read(audio_file_path)
            .map_err(|e| OpenAIError::FileError(format!("Failed to read audio file: {}", e)))?;

        // Get filename
        let filename = audio_file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.wav");

        // Create multipart form
        let form = reqwest::blocking::multipart::Form::new()
            .part(
                "file",
                reqwest::blocking::multipart::Part::bytes(audio_data)
                    .file_name(filename.to_string())
                    .mime_str("audio/wav")
                    .map_err(|e| {
                        OpenAIError::FileError(format!("Failed to set MIME type: {}", e))
                    })?,
            )
            .text("model", self.model.clone())
            .text(
                "prompt",
                "以下は日本語で話された音声データです。テキストに書き起こせ。---".to_string(),
            );

        // Send request
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .map_err(|e| OpenAIError::NetworkError(format!("Failed to send request: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .map_err(|e| OpenAIError::NetworkError(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(OpenAIError::ApiError(format!(
                "API returned status {}: {}",
                status, response_text
            )));
        }

        // Parse response
        let transcription: TranscriptionResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                OpenAIError::ParseError(format!(
                    "Failed to parse response: {}. Response was: {}",
                    e, response_text
                ))
            })?;

        println!("Transcription result: {}", transcription.text);
        Ok(transcription.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OpenAIClient::new("test_key".to_string(), "whisper-1".to_string());
        assert_eq!(client.api_key, "test_key");
        assert_eq!(client.model, "whisper-1");
    }
}

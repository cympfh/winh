use serde::{Deserialize, Serialize};
use std::path::Path;

const XAI_STT_URL: &str = "https://api.x.ai/v1/stt";

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptionResponse {
    pub text: String,
}

#[derive(Debug)]
pub enum SttError {
    NetworkError(String),
    ApiError(String),
    FileError(String),
    ParseError(String),
}

impl std::fmt::Display for SttError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SttError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            SttError::ApiError(msg) => write!(f, "API error: {}", msg),
            SttError::FileError(msg) => write!(f, "File error: {}", msg),
            SttError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for SttError {}

pub struct SpeechToTextClient {
    api_key: String,
}

impl SpeechToTextClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub fn transcribe_audio(&self, audio_file_path: &Path) -> Result<String, SttError> {
        if !audio_file_path.exists() {
            return Err(SttError::FileError(format!(
                "Audio file not found: {:?}",
                audio_file_path
            )));
        }

        println!("Transcribing audio via x.ai STT: {:?}", audio_file_path);

        let audio_data = std::fs::read(audio_file_path)
            .map_err(|e| SttError::FileError(format!("Failed to read audio file: {}", e)))?;

        let filename = audio_file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.wav");

        // x.ai requires file to be the LAST field in multipart form
        let form = reqwest::blocking::multipart::Form::new()
            .text("language", "ja,en,zh")
            .part(
                "file",
                reqwest::blocking::multipart::Part::bytes(audio_data)
                    .file_name(filename.to_string())
                    .mime_str("audio/wav")
                    .map_err(|e| {
                        SttError::FileError(format!("Failed to set MIME type: {}", e))
                    })?,
            );

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(XAI_STT_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .map_err(|e| SttError::NetworkError(format!("Failed to send request: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .map_err(|e| SttError::NetworkError(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(SttError::ApiError(format!(
                "x.ai STT returned status {}: {}",
                status, response_text
            )));
        }

        let transcription: TranscriptionResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                SttError::ParseError(format!(
                    "Failed to parse response: {}. Response was: {}",
                    e, response_text
                ))
            })?;

        println!("x.ai STT result: {}", transcription.text);
        let text = remove_punctuation(&transcription.text);
        println!("After punctuation removal: {}", text);
        Ok(text)
    }
}

fn remove_punctuation(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '、' | '。' | '，' | '．' | '！' | '？' | ',' | '.' | '!' | '?' => ' ',
            _ => c,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_punctuation() {
        assert_eq!(remove_punctuation("こんにちは、世界。"), "こんにちは 世界");
        assert_eq!(remove_punctuation("Hello, world!"), "Hello world");
        assert_eq!(remove_punctuation("Yes? No."), "Yes No");
        assert_eq!(
            remove_punctuation("それは、すごいですね！"),
            "それは すごいですね"
        );
    }
}

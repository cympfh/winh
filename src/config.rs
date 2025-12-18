use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub silence_duration_secs: f32,
    pub silence_threshold: f32,
    #[serde(default)]
    pub input_device_name: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "gpt-4o-transcribe".to_string(),
            silence_duration_secs: 2.0,
            silence_threshold: 0.01,
            input_device_name: None,
        }
    }
}

impl Config {
    /// Get the config file path
    pub fn config_path() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir().ok_or("Failed to get config directory")?;

        let app_config_dir = config_dir.join("winh");

        // Create directory if it doesn't exist
        if !app_config_dir.exists() {
            fs::create_dir_all(&app_config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        Ok(app_config_dir.join("config.json"))
    }

    /// Load config from file
    pub fn load() -> Self {
        match Self::config_path() {
            Ok(path) => {
                if path.exists() {
                    match fs::read_to_string(&path) {
                        Ok(content) => match serde_json::from_str(&content) {
                            Ok(config) => {
                                println!("Config loaded from: {:?}", path);
                                return config;
                            }
                            Err(e) => {
                                eprintln!("Failed to parse config: {}", e);
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to read config file: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get config path: {}", e);
            }
        }

        println!("Using default config");
        Self::default()
    }

    /// Save config to file
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path()?;

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&path, json).map_err(|e| format!("Failed to write config file: {}", e))?;

        println!("Config saved to: {:?}", path);
        Ok(())
    }

    /// Apply command line arguments
    pub fn apply_args(&mut self, args: &[String]) {
        for arg in args {
            if let Some(key) = arg.strip_prefix("--api-key=") {
                self.api_key = key.to_string();
                println!("API key set from command line");
            } else if let Some(key) = arg.strip_prefix("OPENAI_API_KEY=") {
                self.api_key = key.to_string();
                println!("API key set from command line (OPENAI_API_KEY=...)");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.model, "gpt-4o-transcribe");
        assert_eq!(config.silence_duration_secs, 2.0);
        assert_eq!(config.silence_threshold, 0.01);
    }
}

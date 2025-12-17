mod audio;
mod config;
mod openai;

use audio::{save_audio_to_wav, AudioRecorder};
use config::Config;
use eframe::egui;
use openai::OpenAIClient;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

fn main() -> eframe::Result<()> {
    // Load config and apply command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut config = Config::load();
    config.apply_args(&args);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 400.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "winh - Voice Transcription",
        options,
        Box::new(move |cc| {
            // Setup Japanese font
            let mut fonts = egui::FontDefinitions::default();

            // Add Japanese font
            fonts.font_data.insert(
                "japanese".to_owned(),
                egui::FontData::from_static(include_bytes!("../fonts/NotoSansJP-Regular.ttf")),
            );

            // Set Japanese font as highest priority for proportional text
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "japanese".to_owned());

            // Set Japanese font as highest priority for monospace text
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "japanese".to_owned());

            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(WinhApp::new(config)))
        }),
    )
}

enum TranscriptionMessage {
    InProgress,
    Success(String),
    Error(String),
}

struct WinhApp {
    is_recording: bool,
    transcribed_text: String,
    audio_recorder: Option<AudioRecorder>,
    status_message: String,
    recording_info: String,
    audio_file_path: Option<PathBuf>,

    // Config
    config: Config,

    // Background transcription
    transcription_receiver: Option<Receiver<TranscriptionMessage>>,
    is_transcribing: bool,

    // Settings UI
    show_settings: bool,
    settings_api_key: String,
    settings_model: String,
    settings_silence_duration: f32,
}

impl WinhApp {
    fn new(config: Config) -> Self {
        Self {
            is_recording: false,
            transcribed_text: String::new(),
            audio_recorder: None,
            status_message: String::new(),
            recording_info: String::new(),
            audio_file_path: None,
            settings_api_key: config.api_key.clone(),
            settings_model: config.model.clone(),
            settings_silence_duration: config.silence_duration_secs,
            config,
            transcription_receiver: None,
            is_transcribing: false,
            show_settings: false,
        }
    }
}

impl eframe::App for WinhApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for transcription results
        if let Some(receiver) = &self.transcription_receiver {
            if let Ok(message) = receiver.try_recv() {
                match message {
                    TranscriptionMessage::InProgress => {
                        // Already handled
                    }
                    TranscriptionMessage::Success(text) => {
                        self.transcribed_text = text.clone();

                        // Copy to clipboard
                        match arboard::Clipboard::new() {
                            Ok(mut clipboard) => {
                                match clipboard.set_text(&text) {
                                    Ok(_) => {
                                        self.status_message =
                                            "Transcription completed! Text copied to clipboard."
                                                .to_string();
                                        println!(
                                            "Transcription successful and copied to clipboard: {}",
                                            text
                                        );
                                    }
                                    Err(e) => {
                                        self.status_message = format!("Transcription completed, but clipboard copy failed: {}", e);
                                        eprintln!("Failed to copy to clipboard: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                self.status_message = format!(
                                    "Transcription completed, but clipboard init failed: {}",
                                    e
                                );
                                eprintln!("Failed to initialize clipboard: {}", e);
                            }
                        }

                        self.is_transcribing = false;
                        self.transcription_receiver = None;
                    }
                    TranscriptionMessage::Error(error) => {
                        self.status_message = format!("Transcription failed: {}", error);
                        self.is_transcribing = false;
                        self.transcription_receiver = None;
                        eprintln!("Transcription error: {}", error);
                    }
                }
            }
        }

        // Settings modal window
        if self.show_settings {
            egui::Window::new("Settings")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("OpenAI API Key:");
                    ui.text_edit_singleline(&mut self.settings_api_key);
                    ui.add_space(10.0);

                    ui.label("Model:");
                    ui.text_edit_singleline(&mut self.settings_model);
                    ui.add_space(10.0);

                    ui.label("Silence Duration (seconds):");
                    ui.add(egui::Slider::new(
                        &mut self.settings_silence_duration,
                        0.5..=10.0,
                    ));
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.config.api_key = self.settings_api_key.clone();
                            self.config.model = self.settings_model.clone();
                            self.config.silence_duration_secs = self.settings_silence_duration;

                            match self.config.save() {
                                Ok(_) => {
                                    self.status_message = "Settings saved!".to_string();
                                }
                                Err(e) => {
                                    self.status_message = format!("Failed to save settings: {}", e);
                                }
                            }

                            self.show_settings = false;
                        }

                        if ui.button("Cancel").clicked() {
                            // Revert to current config
                            self.settings_api_key = self.config.api_key.clone();
                            self.settings_model = self.config.model.clone();
                            self.settings_silence_duration = self.config.silence_duration_secs;
                            self.show_settings = false;
                        }
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

                // Header with title and settings button
                ui.horizontal(|ui| {
                    ui.heading("Voice Transcription");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("⚙ Settings").clicked() {
                            self.show_settings = true;
                        }
                    });
                });
                ui.add_space(10.0);

                // Status message
                if !self.status_message.is_empty() {
                    ui.colored_label(egui::Color32::from_rgb(100, 150, 255), &self.status_message);
                }

                // Recording info (buffer size, sample rate)
                if !self.recording_info.is_empty() {
                    ui.label(&self.recording_info);
                }

                ui.add_space(20.0);

                // Large Start/Stop button
                let button_text = if self.is_recording {
                    "⏹ Stop"
                } else {
                    "⏺ Start"
                };

                let button_size = egui::vec2(200.0, 80.0);
                let button = egui::Button::new(egui::RichText::new(button_text).size(24.0));

                if ui.add_sized(button_size, button).clicked() {
                    self.is_recording = !self.is_recording;
                    if self.is_recording {
                        self.on_start_recording();
                    } else {
                        self.on_stop_recording();
                    }
                }

                ui.add_space(30.0);

                // Transcribed text display area
                ui.label("Transcribed Text:");
                ui.add_space(5.0);

                egui::ScrollArea::vertical()
                    .max_height(100.0)
                    .show(ui, |ui| {
                        ui.text_edit_multiline(&mut self.transcribed_text);
                    });
            });
        });

        // Update recording info during recording and check for silence
        if self.is_recording {
            if let Some(recorder) = &self.audio_recorder {
                let buffer_size = recorder.get_buffer_size();
                let sample_rate = recorder.get_sample_rate();
                let duration_secs = if sample_rate > 0 {
                    buffer_size as f32 / sample_rate as f32
                } else {
                    0.0
                };

                let silence_elapsed = recorder.get_silence_duration().as_secs_f32();

                self.recording_info = format!(
                    "Recording: {:.1}s | Silence: {:.1}s/{:.1}s",
                    duration_secs, silence_elapsed, self.config.silence_duration_secs
                );

                // Auto-stop if silence duration exceeded
                if recorder.is_silent(self.config.silence_duration_secs) && buffer_size > 0 {
                    println!(
                        "Silence detected for {:.1}s - auto-stopping",
                        self.config.silence_duration_secs
                    );
                    self.is_recording = false;
                    self.on_stop_recording();
                }
            }
            ctx.request_repaint();
        }

        // Keep updating UI while transcribing
        if self.is_transcribing {
            ctx.request_repaint();
        }
    }
}

impl WinhApp {
    fn on_start_recording(&mut self) {
        println!("Recording started");
        self.status_message = "Starting recording...".to_string();
        self.recording_info.clear();

        match AudioRecorder::new() {
            Ok(mut recorder) => match recorder.start_recording() {
                Ok(_) => {
                    self.status_message = "Recording... Speak now!".to_string();
                    self.audio_recorder = Some(recorder);
                }
                Err(e) => {
                    self.status_message = format!("Error: {}", e);
                    self.is_recording = false;
                    eprintln!("Failed to start recording: {}", e);
                }
            },
            Err(e) => {
                self.status_message = format!("Error: {}", e);
                self.is_recording = false;
                eprintln!("Failed to create audio recorder: {}", e);
            }
        }
    }

    fn on_stop_recording(&mut self) {
        println!("Recording stopped");
        self.status_message = "Processing audio...".to_string();

        if let Some(mut recorder) = self.audio_recorder.take() {
            let audio_data = recorder.stop_recording();
            let sample_rate = recorder.get_sample_rate();

            println!("Recorded {} samples at {}Hz", audio_data.len(), sample_rate);
            println!(
                "Duration: {:.2} seconds",
                audio_data.len() as f32 / sample_rate as f32
            );

            if audio_data.is_empty() {
                self.status_message = "No audio recorded".to_string();
                self.recording_info.clear();
                return;
            }

            // Save audio to WAV file
            match save_audio_to_wav(&audio_data, sample_rate) {
                Ok(path) => {
                    let duration = audio_data.len() as f32 / sample_rate as f32;
                    self.audio_file_path = Some(path.clone());
                    println!("Audio file saved: {:?}", path);

                    // Check if API key is set
                    if self.config.api_key.is_empty() {
                        self.status_message =
                            "Audio saved. Set API key in Settings to enable transcription."
                                .to_string();
                    } else {
                        // Start transcription in background thread
                        self.status_message = "Transcribing audio...".to_string();
                        self.start_transcription(path);
                    }
                }
                Err(e) => {
                    self.status_message = format!("Failed to save audio: {}", e);
                    eprintln!("Error saving audio: {}", e);
                }
            }

            self.recording_info.clear();
        } else {
            self.status_message = "No recording found".to_string();
        }
    }

    fn start_transcription(&mut self, audio_path: PathBuf) {
        let (sender, receiver) = channel();
        self.transcription_receiver = Some(receiver);
        self.is_transcribing = true;

        let api_key = self.config.api_key.clone();
        let model = self.config.model.clone();

        // Spawn background thread for transcription
        std::thread::spawn(move || {
            let client = OpenAIClient::new(api_key, model);

            match client.transcribe_audio(&audio_path) {
                Ok(text) => {
                    let _ = sender.send(TranscriptionMessage::Success(text));
                }
                Err(e) => {
                    let _ = sender.send(TranscriptionMessage::Error(e.to_string()));
                }
            }
        });
    }
}

mod audio;
mod config;
mod openai;

use audio::{save_audio_to_wav, AudioRecorder};
use config::Config;
use eframe::egui;
use openai::OpenAIClient;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};

fn main() -> eframe::Result<()> {
    // Load config and apply command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut config = Config::load();
    config.apply_args(&args);

    // Load application icon
    let icon_data = load_icon();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 400.0])
            .with_resizable(false)
            .with_icon(icon_data),
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
    settings_silence_threshold: f32,
    settings_input_device: Option<String>,

    // Device management
    available_devices: Vec<String>,
    selected_device_index: usize,

    // Error tracking
    last_error: Option<String>,
}

impl WinhApp {
    fn new(config: Config) -> Self {
        // Get available input devices
        let mut available_devices = audio::get_input_devices().unwrap_or_else(|e| {
            eprintln!("Failed to get input devices: {}", e);
            vec![]
        });

        // Add "Windows既定" as first option
        available_devices.insert(0, "Windows既定".to_string());

        // Find the index of the configured device
        let selected_device_index = if let Some(ref device_name) = config.input_device_name {
            available_devices
                .iter()
                .position(|d| d == device_name)
                .unwrap_or(0)
        } else {
            0
        };

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
            settings_silence_threshold: config.silence_threshold,
            settings_input_device: config.input_device_name.clone(),
            available_devices,
            selected_device_index,
            config,
            transcription_receiver: None,
            is_transcribing: false,
            show_settings: false,
            last_error: None,
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
                        self.status_message = "Transcribing audio...".to_string();
                        self.last_error = None;
                    }
                    TranscriptionMessage::Success(text) => {
                        self.transcribed_text = text.clone();
                        self.last_error = None;

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
                        self.status_message = format!("❌ Transcription failed: {}", error);
                        self.last_error = Some(error.clone());
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

                    ui.label("Silence Threshold (0.001-0.3):");
                    ui.add(
                        egui::Slider::new(&mut self.settings_silence_threshold, 0.001..=0.3)
                            .logarithmic(true),
                    );
                    ui.label(format!("Current: {:.4}", self.settings_silence_threshold));
                    ui.add_space(10.0);

                    ui.label("Input Device:");
                    egui::ComboBox::from_id_salt("input_device_combo")
                        .selected_text(
                            self.available_devices
                                .get(self.selected_device_index)
                                .unwrap_or(&"Default".to_string()),
                        )
                        .show_ui(ui, |ui| {
                            for (idx, device_name) in self.available_devices.iter().enumerate() {
                                ui.selectable_value(
                                    &mut self.selected_device_index,
                                    idx,
                                    device_name,
                                );
                            }
                        });
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.config.api_key = self.settings_api_key.trim().to_string();
                            self.config.model = self.settings_model.trim().to_string();
                            self.config.silence_duration_secs = self.settings_silence_duration;
                            self.config.silence_threshold = self.settings_silence_threshold;
                            self.config.input_device_name = self
                                .available_devices
                                .get(self.selected_device_index)
                                .cloned();

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
                            self.settings_silence_threshold = self.config.silence_threshold;
                            self.settings_input_device = self.config.input_device_name.clone();
                            // Restore device index
                            self.selected_device_index =
                                if let Some(ref device_name) = self.config.input_device_name {
                                    self.available_devices
                                        .iter()
                                        .position(|d| d == device_name)
                                        .unwrap_or(0)
                                } else {
                                    0
                                };
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

                // Large Start/Stop button with progress indicator
                let button_text = if self.is_recording {
                    "⏹ Stop"
                } else {
                    "⏺ Start"
                };

                let button_size = egui::vec2(200.0, 80.0);

                // Calculate silence progress ratio if recording
                let silence_progress = if self.is_recording {
                    if let Some(recorder) = &self.audio_recorder {
                        let silence_elapsed = recorder.get_silence_duration().as_secs_f32();
                        (silence_elapsed / self.config.silence_duration_secs).min(1.0)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                // Allocate space for custom button
                let (rect, response) = ui.allocate_exact_size(button_size, egui::Sense::click());

                // Get visual style based on interaction
                let visuals = ui.style().interact(&response);

                // Draw button background
                ui.painter()
                    .rect_filled(rect, visuals.rounding, visuals.bg_fill);

                // Draw progress bar if recording (fill from bottom)
                if self.is_recording && silence_progress > 0.0 {
                    let progress_height = rect.height() * silence_progress;
                    let progress_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.min.x, rect.max.y - progress_height),
                        egui::vec2(rect.width(), progress_height),
                    );
                    ui.painter().rect_filled(
                        progress_rect,
                        visuals.rounding,
                        egui::Color32::from_rgb(100, 200, 255),
                    );
                }

                // Draw button border
                ui.painter()
                    .rect_stroke(rect, visuals.rounding, visuals.bg_stroke);

                // Draw button text
                let text_color = visuals.text_color();
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    button_text,
                    egui::FontId::proportional(24.0),
                    text_color,
                );

                // Handle click
                if response.clicked() {
                    self.is_recording = !self.is_recording;
                    if self.is_recording {
                        self.on_start_recording();
                    } else {
                        self.on_stop_recording();
                    }
                }

                ui.add_space(10.0);

                // Volume indicator bar
                if self.is_recording {
                    if let Some(recorder) = &self.audio_recorder {
                        let max_amplitude = recorder.get_max_amplitude();
                        let bar_width = 200.0;
                        let bar_height = 10.0;

                        // Clip to 1.2
                        let clipped_amplitude = max_amplitude.min(1.2);
                        let bar_fill_width = (clipped_amplitude / 1.2) * bar_width;

                        // Allocate space for the bar
                        let (bar_rect, _) = ui.allocate_exact_size(
                            egui::vec2(bar_width, bar_height),
                            egui::Sense::hover(),
                        );

                        // Draw background (dark gray)
                        ui.painter().rect_filled(
                            bar_rect,
                            2.0,
                            egui::Color32::from_rgb(50, 50, 50),
                        );

                        // Draw filled portion with color coding
                        if bar_fill_width > 0.0 {
                            let fill_rect = egui::Rect::from_min_size(
                                bar_rect.min,
                                egui::vec2(bar_fill_width, bar_height),
                            );

                            // Color coding based on amplitude
                            let color = if max_amplitude < self.config.silence_threshold {
                                // Gray: below threshold
                                egui::Color32::from_rgb(150, 150, 150)
                            } else if max_amplitude < 1.0 {
                                // Green: normal range
                                egui::Color32::from_rgb(0, 200, 0)
                            } else {
                                // Red: clipping range
                                egui::Color32::from_rgb(255, 0, 0)
                            };

                            ui.painter().rect_filled(fill_rect, 2.0, color);
                        }

                        // Draw border
                        ui.painter().rect_stroke(
                            bar_rect,
                            2.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
                        );

                        // Display amplitude value
                        ui.label(format!("Level: {:.3}", max_amplitude));
                    }
                }

                ui.add_space(20.0);

                // Transcribed text display area (click to copy)
                ui.label("Transcribed Text (click to copy):");
                ui.add_space(5.0);

                let text_response = egui::ScrollArea::vertical()
                    .max_height(100.0)
                    .show(ui, |ui| {
                        let output = ui.add(
                            egui::TextEdit::multiline(&mut self.transcribed_text)
                                .interactive(false),
                        );
                        // Add click sense on top of the text area
                        let rect = output.rect;
                        ui.allocate_rect(rect, egui::Sense::click())
                    });

                // Copy to clipboard when clicked
                if text_response.inner.clicked() && !self.transcribed_text.is_empty() {
                    match arboard::Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(&self.transcribed_text) {
                            Ok(_) => {
                                self.status_message = "Text copied to clipboard!".to_string();
                            }
                            Err(e) => {
                                self.status_message = format!("Failed to copy: {}", e);
                            }
                        },
                        Err(e) => {
                            self.status_message = format!("Failed to access clipboard: {}", e);
                        }
                    }
                }

                ui.add_space(10.0);

                // Error display area
                if let Some(error) = &self.last_error {
                    ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
                }

                // Warning if API key is not set
                if self.config.api_key.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 165, 0),
                        "⚠ API key not set. Please configure in Settings.",
                    );
                }
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
                if recorder.is_silent(self.config.silence_duration_secs) {
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

        match AudioRecorder::new(self.config.silence_threshold) {
            Ok(mut recorder) => {
                // Use configured device if set, otherwise use default
                // If "Windows既定" is selected, use None to get default device
                let device_name = self
                    .config
                    .input_device_name
                    .as_ref()
                    .filter(|name| name.as_str() != "Windows既定")
                    .map(|s| s.as_str());
                match recorder.start_recording_with_device(device_name) {
                    Ok(_) => {
                        self.status_message = "Recording... Speak now!".to_string();
                        self.audio_recorder = Some(recorder);
                    }
                    Err(e) => {
                        self.status_message = format!("Error: {}", e);
                        self.is_recording = false;
                        eprintln!("Failed to start recording: {}", e);
                    }
                }
            }
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
                    let _duration = audio_data.len() as f32 / sample_rate as f32;
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
            // Send InProgress message
            let _ = sender.send(TranscriptionMessage::InProgress);

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

fn load_icon() -> egui::IconData {
    let icon_bytes = include_bytes!("icon.png");
    let image = image::load_from_memory(icon_bytes).expect("Failed to load icon");
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();

    egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    }
}

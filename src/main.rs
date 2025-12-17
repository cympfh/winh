mod audio;

use eframe::egui;
use audio::{AudioRecorder, save_audio_to_wav};
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 350.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "winh - Voice Transcription",
        options,
        Box::new(|_cc| Ok(Box::new(WinhApp::default()))),
    )
}

struct WinhApp {
    is_recording: bool,
    transcribed_text: String,
    audio_recorder: Option<AudioRecorder>,
    status_message: String,
    recording_info: String,
    silence_duration_secs: f32,
    audio_file_path: Option<PathBuf>,
}

impl Default for WinhApp {
    fn default() -> Self {
        Self {
            is_recording: false,
            transcribed_text: String::new(),
            audio_recorder: None,
            status_message: String::new(),
            recording_info: String::new(),
            silence_duration_secs: 2.0, // Default 2 seconds
            audio_file_path: None,
        }
    }
}

impl eframe::App for WinhApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // Title
                ui.heading("Voice Transcription");
                ui.add_space(10.0);

                // Status message
                if !self.status_message.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(100, 150, 255),
                        &self.status_message
                    );
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
                let button = egui::Button::new(
                    egui::RichText::new(button_text).size(24.0)
                );

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
                    duration_secs, silence_elapsed, self.silence_duration_secs
                );

                // Auto-stop if silence duration exceeded
                if recorder.is_silent(self.silence_duration_secs) && buffer_size > 0 {
                    println!("Silence detected for {:.1}s - auto-stopping", self.silence_duration_secs);
                    self.is_recording = false;
                    self.on_stop_recording();
                }
            }
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
            Ok(mut recorder) => {
                match recorder.start_recording() {
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
            println!("Duration: {:.2} seconds", audio_data.len() as f32 / sample_rate as f32);

            if audio_data.is_empty() {
                self.status_message = "No audio recorded".to_string();
                self.recording_info.clear();
                return;
            }

            // Save audio to WAV file
            match save_audio_to_wav(&audio_data, sample_rate) {
                Ok(path) => {
                    let duration = audio_data.len() as f32 / sample_rate as f32;
                    self.status_message = format!(
                        "Saved {:.2}s of audio to WAV",
                        duration
                    );
                    self.audio_file_path = Some(path.clone());
                    println!("Audio file saved: {:?}", path);

                    // TODO: Phase 4 - Send to OpenAI API for transcription
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
}

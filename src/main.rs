use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "winh - Voice Transcription",
        options,
        Box::new(|_cc| Ok(Box::new(WinhApp::default()))),
    )
}

#[derive(Default)]
struct WinhApp {
    is_recording: bool,
    transcribed_text: String,
}

impl eframe::App for WinhApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // Title
                ui.heading("Voice Transcription");
                ui.add_space(30.0);

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
    }
}

impl WinhApp {
    fn on_start_recording(&mut self) {
        println!("Recording started");
        // TODO: Phase 2 - Implement audio capture
    }

    fn on_stop_recording(&mut self) {
        println!("Recording stopped");
        // TODO: Phase 3 - Implement silence detection and audio processing
    }
}

use std::{error::Error, io::Write, net::TcpStream};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
#[derive(Default)]
pub struct LinbpqApp {
    // Server interaction stuff:
    received_text: String,
    command_input: String,
}

impl LinbpqApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

fn tester() -> Result<String, Box<dyn Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8001")?;

    // KISS frame boundary
    let frame_start_end = 0xC0;
    // KISS command/data byte for data frames (assuming port 0)
    let command = 0x00;

    // Example KISS frame to send (replace with actual frame content)
    let data = b"Hello, Direwolf!";
    let mut frame = Vec::new();
    frame.push(frame_start_end);
    frame.push(command);
    frame.extend_from_slice(data);
    frame.push(frame_start_end);

    stream.write_all(&frame)?;
    stream.flush()?;
    println!("KISS frame sent successfully!");

    Ok("KISS frame sent".into())
}

impl eframe::App for LinbpqApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Connect").clicked() {
                    match tester() {
                        Ok(text) => self.received_text.push_str(&text),
                        Err(e) => self.received_text.push_str(&format!("Error: {}", e)),
                    }
                }
                if ui.button("Disconnect").clicked() {
                    // Disconnect command
                }
                if ui.button("Status").clicked() {
                    // Status command
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Linbpq Server Interface");

                ui.separator();

                ui.label("Received:");
                ui.add_sized(
                    [ui.available_width(), ui.available_height() - 60.0],
                    egui::Label::new(&self.received_text).wrap(true),
                );
                ui.separator();
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [ui.available_width() - 60.0, 20.0],
                        egui::TextEdit::singleline(&mut self.command_input),
                    );
                    if ui.button("Send").clicked() {
                        // Send the command_input to the server
                        // Clear command_input field
                        self.command_input.clear();
                    }
                });
            });
        });
    }
}

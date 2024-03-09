use ax25::frame::{
    Address, Ax25Frame, CommandResponse, FrameContent, ProtocolIdentifier, UnnumberedInformation,
};
use ax25_tnc::tnc::{Tnc, TncAddress};
use std::error::Error;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // Use default for deserialization of missing fields
#[derive(Default)]
pub struct LinbpqApp {
    // Server interaction stuff:
    received_text: String,
    command_input: String,
    #[serde(skip)] // Correctly used to skip both serialization and deserialization
    received_messages_rx: Option<std::sync::mpsc::Receiver<String>>,
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

    pub fn start_listening(
        &mut self,
        tnc_address_str: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (tx, rx) = std::sync::mpsc::channel();
        self.received_messages_rx = Some(rx);

        let addr = tnc_address_str.parse::<TncAddress>()?;
        std::thread::spawn(move || {
            let tnc = Tnc::open(&addr).expect("Failed to open TNC");
            let receiver = tnc.incoming();
            while let Ok(frame) = receiver.recv().unwrap() {
                let message = format!("{}", frame);
                tx.send(message).expect("Failed to send message");
            }
        });

        Ok(())
    }
}

fn tester() -> Result<String, Box<dyn Error>> {
    // Example values for demonstration purposes
    let tnc_address_str = "tnc:tcpkiss:127.0.0.1:8001";
    let source_callsign = "CALLSIGN";
    let dest_callsign = "DEST";
    let message = "Your message here";

    let addr = tnc_address_str.parse::<TncAddress>()?;
    let src = source_callsign.parse::<Address>()?;
    let dest = dest_callsign.parse::<Address>()?;
    let tnc = Tnc::open(&addr)?;

    let frame = Ax25Frame {
        source: src,
        destination: dest,
        route: Vec::new(),
        command_or_response: Some(CommandResponse::Command),
        content: FrameContent::UnnumberedInformation(UnnumberedInformation {
            pid: ProtocolIdentifier::None,
            info: message.as_bytes().to_vec(),
            poll_or_final: false,
        }),
    };

    tnc.send_frame(&frame)?;
    println!("Transmitted!");

    Ok("AX.25 frame sent".into())
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
                    if let Err(e) = self.start_listening("tnc:tcpkiss:127.0.0.1:8001") {
                        self.received_text.push_str(&format!("Error starting listener: {}", e));
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
                        match tester() {
                            Ok(text) => self.received_text.push_str(&text),
                            Err(e) => self.received_text.push_str(&format!("Error: {}", e)),
                        }
                        // Clear command_input field
                        self.command_input.clear();
                    }
                });
            });
        });

        if let Some(rx) = &self.received_messages_rx {
            while let Ok(message) = rx.try_recv() {
                self.received_text.push_str(&message);
                self.received_text.push('\n');
            }
        }
    }
}

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
    received_messages: Vec<Message>,
    command_input: String,
    destination_address: String, // New field for destination address
    message_input: String,       // New field for message input
    #[serde(skip)] // Correctly used to skip both serialization and deserialization
    received_messages_rx: Option<std::sync::mpsc::Receiver<String>>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Message {
    source: String,
    destination: String,
    content: String,
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

    fn send_ax25_frame(
        &self,
        destination_address: &str,
        message: &str,
    ) -> Result<String, Box<dyn Error>> {
        let tnc_address_str =
            std::env::var("TNC_URL").unwrap_or_else(|_| "tnc:tcpkiss:localhost:8001".to_string());
        let source_callsign =
            std::env::var("CALLSIGN").unwrap_or_else(|_| "DEFAULT_CALLSIGN".to_string());
        let dest_callsign = destination_address;

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
                    if let Err(e) = self.start_listening(
                        std::env::var("TNC_URL")
                            .unwrap_or_else(|_| "tnc:tcpkiss:localhost:8001".to_string())
                            .as_str(),
                    ) {
                        let error_message = format!("Error starting listener: {}", e);
                        let error_display = Message {
                            source: "System".to_string(),
                            destination: "N/A".to_string(),
                            content: error_message,
                        };
                        self.received_messages.push(error_display);
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
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for message in &self.received_messages {
                            ui.label(format!(
                                "**{} -> {}:**\t{}",
                                message.source, message.destination, message.content
                            ));
                        }
                        ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                    });
                ui.separator();
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Destination:");
                    ui.add_sized(
                        [150.0, 20.0],
                        egui::TextEdit::singleline(&mut self.destination_address),
                    );
                    ui.label("Message:");
                    ui.add_sized(
                        [ui.available_width() - 310.0, 20.0],
                        egui::TextEdit::singleline(&mut self.message_input),
                    );
                    if ui.button("Send").clicked() {
                        match self.send_ax25_frame(&self.destination_address, &self.message_input) {
                            Ok(text) => {
                                let message = Message {
                                    source: "Me".to_string(),
                                    destination: self.destination_address.clone(),
                                    content: self.message_input.clone(),
                                };
                                self.received_messages.push(message);
                            }
                            Err(e) => {
                                let error_message = format!("Error: {}", e);
                                let error_display = Message {
                                    source: "System".to_string(),
                                    destination: "N/A".to_string(),
                                    content: error_message,
                                };
                                self.received_messages.push(error_display);
                            }
                        }
                        // Clear input fields
                        self.destination_address.clear();
                        self.message_input.clear();
                    }
                });
            });
        });

        if let Some(rx) = &self.received_messages_rx {
            while let Ok(message_str) = rx.try_recv() {
                if let Some((source, content)) = parse_message(&message_str) {
                    let message = Message {
                        source,
                        destination: self.destination_address.clone(), // Assuming destination is stored in self.destination_address
                        content,
                    };
                    self.received_messages.push(message);
                }
            }
        }
    }
}

// Placeholder function; implement according to your message format
fn parse_message(message: &str) -> Option<(String, String)> {
    // Parse the message to extract the source and content
    // Return them as a tuple
    Some(("Source".to_string(), message.to_string()))
}

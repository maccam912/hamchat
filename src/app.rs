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
    callsign: String, // Add this line
    message_input: String,       // New field for message input
    #[serde(skip)] // Correctly used to skip both serialization and deserialization
    received_messages_rx: Option<std::sync::mpsc::Receiver<Message>>,
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
                // Extract source, destination, and content from the frame
                let source = format!("{}", frame.source);
                let destination = format!("{}", frame.destination);
                let content = match dbg!(frame.content) {
                    FrameContent::UnnumberedInformation(info) => {
                        String::from_utf8(info.info).unwrap_or_default()
                    }
                    FrameContent::Information(info) => {
                        String::from_utf8(info.info).unwrap_or_default()
                    }
                    FrameContent::ReceiveReady(rr) => {
                        format!("Receive Ready with sequence: {}", rr.receive_sequence)
                    }
                    FrameContent::ReceiveNotReady(rnr) => {
                        format!("Receive Not Ready with sequence: {}", rnr.receive_sequence)
                    }
                    // Add handling for other FrameContent variants as needed
                    _ => "Unknown or unsupported frame content".to_string(),
                };
                let message = Message {
                    source,
                    destination,
                    content,
                };
                // Send the structured message instead of a string representation
                tx.send(message).expect("Failed to send message");
            }
        });

        Ok(())
    }

    fn send_ax25_frame(
        &self,
        _destination_address: &str,
        message: &str,
    ) -> Result<String, Box<dyn Error>> {
        let tnc_address_str =
            std::env::var("TNC_URL").unwrap_or_else(|_| "tnc:tcpkiss:localhost:8001".to_string());
        let source_callsign = self.callsign.clone();
        let dest_callsign = "HARECH-0";

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
                ui.label("Callsign:");
                ui.text_edit_singleline(&mut self.callsign);
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
                if ui.button("Clear").clicked() {
                    self.received_messages.clear();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Chattateria");

                ui.separator();

                ui.label("Received:");
                // Adjust the height of the ScrollArea to ensure the bottom row of text is visible
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .max_height(ui.available_height() - 60.0) // Adjusted height to leave space for the text input area
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
                    ui.label("Message:");
                    ui.add_sized(
                        [ui.available_width() - 160.0, 20.0],
                        egui::TextEdit::singleline(&mut self.message_input),
                    );
                    if ui.button("Send").clicked() {
                        match self.send_ax25_frame("HARECH-0", &self.message_input) {
                            Ok(_) => {
                                let message = Message {
                                    source: self.callsign.clone(),
                                    destination: "HARECH-0".to_string(),
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
                        self.message_input.clear();
                    }
                });
            });
        });
        if let Some(rx) = &self.received_messages_rx {
            while let Ok(message) = rx.try_recv() {
                self.received_messages.push(message);
            }
        }
    }
}

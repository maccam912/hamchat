# Hamchat: A Rust-based Chat Application for Ham Radio Packet Radio

Hamchat is a Rust application designed for chatting over ham radio packet radio using the AX.25 protocol. It leverages the egui framework for the GUI and communicates with a Terminal Node Controller (TNC) to send and receive AX.25 frames. This document outlines the structure of the Hamchat application and provides instructions on how to use it.
## Getting Started

### Prerequisites

- Rust and Cargo installed.
- A Terminal Node Controller (TNC) accessible over TCP/IP for AX.25 communication.

### Installation

1. Clone the repository.
2. Navigate to the project directory.
3. Build the project:
   ```shell
   cargo build --release
   ```

### Configuration

1. Set the TNC address in an environment variable `TNC_URL` if it's not the default `localhost:8001`.
2. Modify `src/app.rs` if you need to customize AX.25 frame parameters or handling logic.

### Running Hamchat

  ```shell
  cargo run --release
  ```

### Usage

1. Upon launching Hamchat, enter your callsign in the "Callsign" field.
2. Click "Connect" to start listening for incoming AX.25 frames.
3. To send a message, enter the message in the "Message" field and click "Send".
4. Received messages will be displayed in the main window. You can filter to show only HRRC traffic by checking "Show only HRRC traffic".

### GitHub Actions

- Automated CI/CD is set up using GitHub Actions for building, testing, and deploying the application.

## Contributing

Contributions are welcome! Please follow the standard GitHub pull request workflow.

## License

MIT

## Acknowledgments

- Thanks to the `egui` and `ax25` crate authors and contributors.
- This project is inspired by the need for modern, easy-to-use software for ham radio operators.

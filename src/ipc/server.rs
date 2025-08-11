use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::OnceLock;
use tokio::sync::broadcast;

use crate::ipc;

static SENDER: OnceLock<broadcast::Sender<String>> = OnceLock::new();

pub fn drop_socket() -> io::Result<()> {
    std::fs::remove_file(ipc::get_socket_path())
}

pub fn start() -> io::Result<()> {
    let socket_path = ipc::get_socket_path();

    // Ensure the socket is removed before starting
    if ipc::client::get_stream().is_err() {
        let _ = drop_socket();
    } else {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "IPC server is already running",
        ));
    }

    let listener = UnixListener::bind(&socket_path)?;
    println!("IPC server started, listening on {}", socket_path);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || handle_client(stream));
            },

            Err(e) => eprintln!("Error accepting connection: {}", e)
        }
    }

    Ok(())
}

pub fn subscribe() -> broadcast::Receiver<String> {
    SENDER.get_or_init(|| broadcast::channel(100).0).subscribe()
}

pub fn handle_client(mut stream: UnixStream) {
    let mut buffer = [0; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, // Connection closed
            
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[..n]).to_string();

                // Send the message to all subscribers
                if let Some(sender) = SENDER.get() {
                    let _ = sender.send(message.clone());
                }

                stream.write_all(b"Message received").unwrap();
            }

            Err(e) => {
                eprintln!("Error reading from stream: {}", e);
                break;
            }
        }
    }
}
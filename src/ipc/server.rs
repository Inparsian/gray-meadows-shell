use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use once_cell::sync::OnceCell;
use tokio::sync::broadcast;

use crate::ipc;

static SENDER: OnceCell<broadcast::Sender<String>> = OnceCell::new();

pub fn drop_socket() {
    let _ = std::fs::remove_file(ipc::get_socket_path());
}

pub fn start() -> std::io::Result<()> {
    let socket_path = ipc::get_socket_path();

    // Ensure the socket is removed before starting
    drop_socket(); 

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
    let sender = SENDER.get_or_init(|| broadcast::channel(100).0.clone());
    sender.subscribe()
}

pub fn handle_client(mut stream: UnixStream) {
    let mut buffer = [0; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, // Connection closed
            
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[..n]).to_string();
                println!("Received message: {}", message);

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
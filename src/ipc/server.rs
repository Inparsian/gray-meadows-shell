use std::io::{self, Read as _, Write as _};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::OnceLock;
use async_broadcast::Receiver;

use crate::utils::broadcast::BroadcastChannel;

static CHANNEL: OnceLock<BroadcastChannel<String>> = OnceLock::new();

pub fn drop_socket() -> io::Result<()> {
    std::fs::remove_file(super::get_socket_path())
}

pub async fn start() -> io::Result<()> {
    let socket_path = super::get_socket_path();

    // Ensure the socket is removed before starting
    if super::client::get_stream().is_err() {
        let _ = drop_socket();
    } else {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "IPC server is already running",
        ));
    }

    let listener = UnixListener::bind(&socket_path)?;
    info!(%socket_path, "IPC server started");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream).await;
            },

            Err(e) => error!(%e, "Error accepting connection")
        }
    }

    Ok(())
}

pub fn subscribe() -> Receiver<String> {
    CHANNEL.get_or_init(|| BroadcastChannel::new(10)).subscribe()
}

pub async fn handle_client(mut stream: UnixStream) {
    let mut buffer = [0; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, // Connection closed
            
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[..n]).to_string();

                // Send the message to all subscribers
                if let Some(sender) = CHANNEL.get() {
                    sender.send(message.clone()).await;
                }

                stream.write_all(b"Message received").unwrap();
            }

            Err(e) => {
                error!(%e, "Error reading from stream");
                break;
            }
        }
    }
}
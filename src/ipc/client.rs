use std::os::unix::net::UnixStream;
use std::io::{Read, Write};

use crate::ipc;

pub fn get_stream() -> Option<UnixStream> {
    match UnixStream::connect(ipc::get_socket_path()) {
        Ok(stream) => Some(stream),
        Err(e) => {
            eprintln!("Failed to connect to IPC socket: {}", e);
            None
        }
    }
}

pub fn send_message(message: &str) -> Result<String, std::io::Error> {
    if let Some(mut stream) = get_stream() {
        stream.write_all(message.as_bytes())?;
        stream.shutdown(std::net::Shutdown::Write)?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;

        Ok(response)
    } else {
        Err(std::io::Error::other("Failed to get IPC stream"))
    }
}
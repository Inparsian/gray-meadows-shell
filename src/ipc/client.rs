use std::os::unix::net::UnixStream;
use std::io::{self, Read, Write};
use std::time::Duration;

pub fn get_stream() -> io::Result<UnixStream> {
    let stream = UnixStream::connect(super::get_socket_path())?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    Ok(stream)
}

pub fn send_message(message: &str) -> io::Result<String> {
    let mut stream = get_stream()?;
    let mut response = String::new();

    stream.write_all(message.as_bytes())?;
    stream.flush()?;
    stream.shutdown(std::net::Shutdown::Write)?;
    stream.read_to_string(&mut response)?;

    Ok(response)
}
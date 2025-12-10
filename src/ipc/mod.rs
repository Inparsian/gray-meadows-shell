pub mod client;
pub mod server;

pub const SOCKET_FILE_NAME: &str = "gray-meadows-shell.sock";

pub fn get_socket_path() -> String {
    format!(
        "{}/{}",
        crate::filesystem::get_xdg_runtime_directory(),
        SOCKET_FILE_NAME
    )
}

/// Listens for incoming IPC messages and invokes the provided callback
/// function whenever a new message is received on the GTK main thread.
pub fn listen_for_messages_local<F>(callback: F)
where
    F: Fn(String) + 'static,
{
    gtk4::glib::spawn_future_local(async move {
        let mut receiver = server::subscribe();
        while let Ok(message) = receiver.recv().await {
            callback(message);
        }
    });
}
pub mod client;
pub mod server;

use tokio::sync::broadcast;

pub const SOCKET_FILE_NAME: &str = "gray-meadows-shell.sock";

pub fn get_socket_path() -> String {
    format!(
        "{}/{}",
        crate::filesystem::get_xdg_runtime_directory(),
        SOCKET_FILE_NAME
    )
}

/// Listens for incoming IPC messages and invokes the provided callback
/// function whenever a new message is received on a separate Tokio task.
pub fn listen_for_messages<F>(callback: F)
where
    F: Fn(String) + Send + 'static,
{
    let mut receiver = server::subscribe();

    tokio::spawn(async move {
        loop {
            match receiver.recv().await {
                Ok(message) => callback(message),
                Err(broadcast::error::RecvError::Closed) => break, // Channel closed
                Err(broadcast::error::RecvError::Lagged(_)) => {}, // Lagged messages
            }
        }
    });
}

/// Listens for incoming IPC messages and invokes the provided callback
/// function whenever a new message is received on the GTK main thread.
pub fn listen_for_messages_local<F>(callback: F)
where
    F: Fn(String) + 'static,
{
    let mut receiver = server::subscribe();
    let (tx, rx) = async_channel::bounded::<String>(1);

    tokio::spawn(async move {
        loop {
            match receiver.recv().await {
                Ok(message) => tx.send(message).await.unwrap(),
                Err(broadcast::error::RecvError::Closed) => break, // Channel closed
                Err(broadcast::error::RecvError::Lagged(_)) => {}, // Lagged messages
            }
        }
    });

    gtk4::glib::spawn_future_local(async move {
        while let Ok(message) = rx.recv().await {
            callback(message);
        }
    });
}
pub mod client;
pub mod server;

pub const SOCKET_FILE_NAME: &str = "gray-meadows-shell.sock";

pub fn get_socket_path() -> String {
    format!(
        "{}/{}",
        crate::helpers::filesystem::get_xdg_runtime_directory(),
        SOCKET_FILE_NAME
    )
}
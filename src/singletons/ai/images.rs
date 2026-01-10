use base64::Engine as _;
use uuid::Uuid;

use crate::{filesystem, sql::wrappers::aichats};

pub fn cache_image_data(base64: &str) -> Result<String, anyhow::Error> {
    let config_dir = filesystem::get_local_data_directory();
    let images_dir = format!("{}/ai_images", config_dir);
    std::fs::create_dir_all(&images_dir)?;

    let uuid = Uuid::new_v4().to_string();
    let image_path = format!("{}/{}.png", images_dir, uuid);

    let image_data = base64::engine::general_purpose::STANDARD.decode(base64)?;
    std::fs::write(&image_path, image_data)?;

    Ok(image_path)
}

pub fn load_image_data(path: &str) -> Result<String, anyhow::Error> {
    let image_data = std::fs::read(path)?;
    let base64 = base64::engine::general_purpose::STANDARD.encode(&image_data);
    Ok(base64)
}

pub fn collect_garbage() {
    let config_dir = filesystem::get_local_data_directory();
    let images_dir = format!("{}/ai_images", config_dir);
    let valid_paths = aichats::get_all_image_item_paths().unwrap_or_default();

    if let Ok(entries) = std::fs::read_dir(&images_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(path_str) = path.to_str()
                && !valid_paths.contains(&path_str.to_owned())
            {
                let _ = std::fs::remove_file(path_str);
            }
        }
    }
}
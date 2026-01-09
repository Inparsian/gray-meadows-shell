use base64::Engine as _;
use uuid::Uuid;

use crate::filesystem;

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
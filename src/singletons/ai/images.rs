use std::collections::HashSet;
use base64::Engine as _;
use uuid::Uuid;

use crate::utils::filesystem;
use crate::sql::wrappers::aichats;

fn get_ai_images_directory() -> String {
    let local_data_dir = filesystem::get_local_data_directory();
    format!("{}/ai_images", local_data_dir)
}

pub fn cache_image_data(base64: &str) -> Result<String, anyhow::Error> {
    let images_dir = get_ai_images_directory();
    std::fs::create_dir_all(&images_dir)?;

    let uuid = Uuid::new_v4().to_string();
    let image_path = format!("{}/{}.png", images_dir, uuid);

    let image_data = base64::engine::general_purpose::STANDARD.decode(base64)?;
    std::fs::write(&image_path, image_data)?;

    Ok(uuid)
}

pub fn uuid_to_file_path(uuid: &str) -> String {
    let images_dir = get_ai_images_directory();
    format!("{}/{}.png", images_dir, uuid)
}

pub fn load_image_data(uuid: &str) -> Result<String, anyhow::Error> {
    let image_data = std::fs::read(uuid_to_file_path(uuid))?;
    let base64 = base64::engine::general_purpose::STANDARD.encode(&image_data);
    Ok(base64)
}

pub async fn collect_garbage() {
    let images_dir = get_ai_images_directory();
    let used_uuids = aichats::get_all_image_item_uuids().await.unwrap_or_default();
    let used_set: HashSet<String> = used_uuids.into_iter()
        .map(|uuid| format!("{}.png", uuid))
        .collect();

    if let Ok(entries) = std::fs::read_dir(&images_dir) {
        for entry in entries.flatten() {
            if !used_set.contains(&entry.file_name().to_string_lossy().to_string()) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
}
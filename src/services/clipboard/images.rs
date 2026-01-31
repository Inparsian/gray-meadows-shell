use std::sync::LazyLock;
use regex::Regex;
use image::codecs::png::PngEncoder;
use image::imageops::FilterType;
use image::ImageEncoder as _;

use crate::utils::filesystem;

static IMAGE_BINARY_DATA_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\[ binary data (\d+) ((?:[KMGT]i)?B) (\w+) (\d+x\d+) \]\]").expect("Failed to compile image binary data regex")
});

static IMAGE_SIZE: u32 = 192;
static IMAGE_SEMAPHORE: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(3);

pub struct ImagePreviewData {
    pub size: u32,
    pub size_unit: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
}

impl ImagePreviewData {
    pub fn from_clipboard_preview(preview: &str) -> Option<Self> {
        let captures = IMAGE_BINARY_DATA_PATTERN.captures(preview)?;
        let size = captures.get(1)?.as_str().parse::<u32>().ok()?;
        let size_unit = captures.get(2)?.as_str().to_owned();
        let mime_type = captures.get(3)?.as_str().to_owned();
        let dimensions = captures.get(4)?.as_str().split_once('x')?;
        let width = dimensions.0.parse::<u32>().ok()?;
        let height = dimensions.1.parse::<u32>().ok()?;
        Some(Self {
            size,
            size_unit,
            mime_type,
            width,
            height,
        })
    }
    
    fn to_filename(&self) -> String {
        format!("{}{}_{}_{}x{}.png", self.size, self.size_unit, self.mime_type, self.width, self.height)
    }
}

fn get_images_directory() -> String {
    let local_data_dir = filesystem::get_local_data_directory();
    format!("{}/clipboard_image_previews", local_data_dir)
}

pub fn is_an_image_clipboard_entry(preview: &str) -> bool {
    IMAGE_BINARY_DATA_PATTERN.is_match(preview)
}

pub fn get_downscale_image_resolution(width: u32, height: u32) -> (u32, u32) {
    if width > IMAGE_SIZE || height > IMAGE_SIZE {
        let aspect = width as f32 / height as f32;
        let new_width = if aspect >= 1.0 {
            IMAGE_SIZE
        } else {
            (IMAGE_SIZE as f32 * aspect) as u32
        };
        let new_height = if aspect >= 1.0 {
            (IMAGE_SIZE as f32 / aspect) as u32
        } else {
            IMAGE_SIZE
        };
        (new_width, new_height)
    } else {
        (width, height)
    }
}

async fn downscale_image_data(decoded: Vec<u8>) -> Result<(u32, u32, Vec<u8>), anyhow::Error> {
    let _permit = IMAGE_SEMAPHORE.acquire().await.unwrap();
    let image = image::load_from_memory(&decoded).ok();
    image.map_or_else(|| Err(anyhow::anyhow!("Failed to load image")), |img| {
        let scaled_img = if img.width() > IMAGE_SIZE || img.height() > IMAGE_SIZE {
            let (width, height) = get_downscale_image_resolution(img.width(), img.height());
            img.resize(width, height, FilterType::Lanczos3)
        } else {
            img
        };

        let mut buf = Vec::new();
        if PngEncoder::new(&mut buf).write_image(
            scaled_img.as_bytes(),
            scaled_img.width(),
            scaled_img.height(),
            scaled_img.color().into()
        ).is_ok() {
            Ok((scaled_img.width(), scaled_img.height(), buf))
        } else {
            Err(anyhow::anyhow!("Failed to encode image"))
        }
    })
}

fn cache_image_clipboard_entry(id: i32, data: Vec<u8>) -> Result<String, anyhow::Error>  {
    let Some(preview) = ({
        let preview_cache = super::PREVIEW_CACHE.read().unwrap();
        preview_cache.get(&id).cloned()
    }) else {
        return Err(anyhow::anyhow!("Failed to get preview or decoded data for image clipboard entry"));
    };
    
    if is_an_image_clipboard_entry(&preview)
        && let Some(preview_data) = ImagePreviewData::from_clipboard_preview(&preview)
    {
        debug!(
            id,
            size = preview_data.size,
            size_unit = preview_data.size_unit,
            mime_type = preview_data.mime_type,
            width = preview_data.width,
            height = preview_data.height,
            "image preview data"
        );
        
        let images_dir = get_images_directory();
        std::fs::create_dir_all(&images_dir)?;
        
        let image_path = format!("{}/{id}_{}", images_dir, preview_data.to_filename());
        std::fs::write(&image_path, data)?;
        Ok(image_path)
    } else {
        Err(anyhow::anyhow!("Failed to parse image preview data"))
    }
}

pub async fn get_image_entry(id: i32) -> Option<(u32, u32, Vec<u8>)> {
    let preview = super::get_preview(id)?;
    if is_an_image_clipboard_entry(&preview) {
        let preview_data = ImagePreviewData::from_clipboard_preview(&preview)?;
        let images_dir = get_images_directory();
        let image_path = format!("{}/{id}_{}", images_dir, preview_data.to_filename());
        if std::path::Path::new(&image_path).exists() {
            debug!(id, "Image clipboard cache hit");
            let data = std::fs::read(image_path).ok()?;
            if let Ok(image) = image::load_from_memory(&data) {
                return Some((image.width(), image.height(), data));
            }
        } else {
            debug!(id, "Image clipboard cache miss");
            let decoded = super::decode_clipboard_entry(&id.to_string())?;
            if let Ok((width, height, data)) = downscale_image_data(decoded).await {
                tokio::spawn(clone!(
                    #[strong] data,
                    async move {
                        if let Err(err) = cache_image_clipboard_entry(id, data) {
                            error!(%err, "Failed to cache image clipboard entry");
                        }
                    }
                ));
                
                return Some((width, height, data));
            }
        }
    }
    
    None
}
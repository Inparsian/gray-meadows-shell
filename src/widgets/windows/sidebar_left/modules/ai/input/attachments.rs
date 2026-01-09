use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use uuid::Uuid;
use base64::Engine as _;
use image::{ImageBuffer, Rgba};

#[derive(Clone)]
pub struct RawImage {
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

impl RawImage {
    pub fn from_texture(texture: &gdk4::Texture) -> Option<Self> {
        let width = texture.width();
        let height = texture.height();
        let stride = width * 4;
        let buffer_size = (stride * height) as usize;
        let mut bytes = vec![0_u8; buffer_size];
        texture.download(&mut bytes, stride as usize);

        Some(Self {
            width,
            height,
            data: bytes,
        })
    }

    pub fn into_image_attachment(self, texture: Option<&gdk4::Texture>) -> Result<ImageAttachment, anyhow::Error> {
        let buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
            self.width as u32,
            self.height as u32,
            self.data
        ).ok_or_else(|| anyhow::anyhow!("Failed to create image buffer from texture"))?;

        let mut cursor = std::io::Cursor::new(Vec::new());
        buffer.write_to(&mut cursor, image::ImageFormat::Png)?;

        let base64 = base64::engine::general_purpose::STANDARD.encode(cursor.get_ref());
        let glib_bytes = gtk4::glib::Bytes::from_owned(cursor.into_inner());
        let thumbnail = texture.cloned().or_else(|| gdk4::Texture::from_bytes(&glib_bytes).ok());
        let uuid = Uuid::new_v4().to_string();

        Ok(ImageAttachment {
            thumbnail,
            uuid,
            mime: "image/png".to_owned(),
            base64,
        })
    }
}

#[derive(Clone)]
pub struct ImageAttachment {
    pub thumbnail: Option<gdk4::Texture>,
    pub uuid: String,
    pub mime: String,
    pub base64: String,
}

impl ImageAttachment {
    pub fn from_texture(
        texture: &gdk4::Texture,
    ) -> Result<Self, anyhow::Error> {
        let raw_image = RawImage::from_texture(texture)
            .ok_or_else(|| anyhow::anyhow!("Failed to get raw image from texture"))?;
        raw_image.into_image_attachment(Some(texture))
    }

    #[allow(dead_code)]
    pub fn to_data_url(&self) -> String {
        format!("data:{};base64,{}", self.mime, self.base64)
    }
}

#[derive(Clone)]
pub struct ImageAttachmentWidget {
    pub attachment: ImageAttachment,
    pub widget: gtk4::Box,
}

impl ImageAttachmentWidget {
    pub fn new(attachments_ref: ImageAttachments, attachment: ImageAttachment) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let overlay = gtk4::Overlay::new();
        widget.append(&overlay);

        let w_clamp = libadwaita::Clamp::new();
        w_clamp.set_maximum_size(100);
        let h_clamp = libadwaita::Clamp::new();
        h_clamp.set_maximum_size(100);
        h_clamp.set_orientation(gtk4::Orientation::Vertical);
        w_clamp.set_child(Some(&h_clamp));
        overlay.set_child(Some(&w_clamp));

        let picture = gtk4::Picture::new();
        picture.set_paintable(attachment.thumbnail.as_ref());
        picture.set_width_request(100);
        h_clamp.set_child(Some(&picture));

        let remove_button = gtk4::Button::new();
        remove_button.set_css_classes(&["ai-chat-input-attachment-remove-button"]);
        remove_button.set_halign(gtk4::Align::End);
        remove_button.set_valign(gtk4::Align::Start);
        remove_button.set_label("close");
        remove_button.connect_clicked({
            let uuid = attachment.uuid.clone();
            move |_| {
                let index = {
                    let attachments = attachments_ref.attachments.borrow();
                    attachments.iter().position(|a| a.attachment.uuid == uuid)
                };

                if let Some(index) = index {
                    attachments_ref.remove(index);
                }
            }
        });
        overlay.add_overlay(&remove_button);

        Self {
            attachment,
            widget,
        }
    }
}

#[derive(Clone)]
pub struct ImageAttachments {
    pub container: gtk4::Revealer,
    pub bx: gtk4::Box,
    attachments: Rc<RefCell<Vec<ImageAttachmentWidget>>>,
}

impl Default for ImageAttachments {
    fn default() -> Self {
        let container = gtk4::Revealer::new();
        container.set_transition_type(gtk4::RevealerTransitionType::SlideUp);
        container.set_reveal_child(false);

        let bx = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        bx.add_css_class("ai-chat-input-attachments-box");
        container.set_child(Some(&bx));

        Self {
            container,
            bx,
            attachments: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl ImageAttachments {
    pub fn push(&self, attachment: ImageAttachment) {
        let widget = ImageAttachmentWidget::new(
            self.clone(),
            attachment,
        );

        self.bx.append(&widget.widget);
        self.attachments.borrow_mut().push(widget);
        self.container.set_reveal_child(!self.attachments.borrow().is_empty());
    }

    pub fn remove(&self, index: usize) {
        self.bx.remove(&self.attachments.borrow_mut().remove(index).widget);
        self.container.set_reveal_child(!self.attachments.borrow().is_empty());
    }
}
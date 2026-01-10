use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use gtk4::prelude::*;
use gtk4::{glib, gio};
use uuid::Uuid;
use base64::Engine as _;
use image::{ImageBuffer, Rgba};

#[derive(Clone)]
pub struct RawImage {
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

struct ProcessedImage {
    thumbnail_bytes: Vec<u8>,
    base64: String,
}

impl RawImage {
    pub fn from_texture(texture: &gdk4::Texture) -> Option<Self> {
        let width = texture.width();
        let height = texture.height();
        let stride = width * 4;
        let buffer_size = (stride * height) as usize;
        let mut bytes = vec![0_u8; buffer_size];
        texture.download(&mut bytes, stride as usize);

        // GDK4 downloads as BGRA, convert to RGBA
        for pixel in bytes.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        Some(Self {
            width,
            height,
            data: bytes,
        })
    }

    fn process_blocking(self) -> Result<ProcessedImage, anyhow::Error> {
        let buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
            self.width as u32,
            self.height as u32,
            self.data
        ).ok_or_else(|| anyhow::anyhow!("Failed to create image buffer from texture"))?;

        let mut cursor = std::io::Cursor::new(Vec::new());
        buffer.write_to(&mut cursor, image::ImageFormat::Png)?;

        let base64 = base64::engine::general_purpose::STANDARD.encode(cursor.get_ref());
        
        let thumbnail_bytes = {
            let aspect_ratio = self.width as f32 / self.height as f32;
            let (new_width, new_height) = if aspect_ratio > 1.0 {
                (100, (100.0 / aspect_ratio) as u32)
            } else {
                ((100.0 * aspect_ratio) as u32, 100)
            };
        
            let thumbnail_buffer = image::imageops::resize(
                &buffer,
                new_width,
                new_height,
                image::imageops::FilterType::Lanczos3,
            );
        
            let mut thumb_cursor = std::io::Cursor::new(Vec::new());
            thumbnail_buffer.write_to(&mut thumb_cursor, image::ImageFormat::Png)?;
            thumb_cursor.into_inner()
        };
        
        Ok(ProcessedImage {
            thumbnail_bytes,
            base64,
        })
    }
}

#[derive(Clone)]
pub struct ImageAttachment {
    pub base64: String,
}

#[derive(Clone)]
enum AttachmentState {
    Loading,
    Ready(ImageAttachment),
}

#[derive(Clone)]
pub struct ImageAttachmentWidget {
    state: Rc<RefCell<AttachmentState>>,
    cancelled: Arc<AtomicBool>,
    uuid: String,
    pub widget: gtk4::Box,
}

impl ImageAttachmentWidget {
    pub fn new_async(
        attachments_ref: ImageAttachments,
        raw_image: RawImage,
    ) -> Self {
        let uuid = Uuid::new_v4().to_string();
        let cancelled = Arc::new(AtomicBool::new(false));
        let state = Rc::new(RefCell::new(AttachmentState::Loading));
        
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        widget.set_size_request(100, 100);
        
        let overlay = gtk4::Overlay::new();
        widget.append(&overlay);

        let w_clamp = libadwaita::Clamp::new();
        w_clamp.set_maximum_size(100);
        w_clamp.set_unit(libadwaita::LengthUnit::Px);
        let h_clamp = libadwaita::Clamp::new();
        h_clamp.set_maximum_size(100);
        h_clamp.set_unit(libadwaita::LengthUnit::Px);
        h_clamp.set_orientation(gtk4::Orientation::Vertical);
        w_clamp.set_child(Some(&h_clamp));
        overlay.set_child(Some(&w_clamp));

        let spinner = gtk4::Label::new(Some("progress_activity"));
        spinner.set_css_classes(&["ai-chat-input-attachment-spinner"]);
        spinner.set_size_request(100, 100);
        h_clamp.set_child(Some(&spinner));

        let remove_button = gtk4::Button::new();
        remove_button.set_css_classes(&["ai-chat-input-attachment-remove-button"]);
        remove_button.set_halign(gtk4::Align::End);
        remove_button.set_valign(gtk4::Align::Start);
        remove_button.set_label("close");
        remove_button.connect_clicked({
            let uuid = uuid.clone();
            move |_| {
                let index = {
                    let attachments = attachments_ref.attachments.borrow();
                    attachments.iter().position(|a| a.uuid == uuid)
                };

                if let Some(index) = index {
                    attachments_ref.remove(index);
                }
            }
        });
        overlay.add_overlay(&remove_button);

        let attachment_widget = Self {
            state: state.clone(),
            cancelled: cancelled.clone(),
            uuid,
            widget,
        };

        glib::spawn_future_local(async move {
            let result = gio::spawn_blocking(move || {
                raw_image.process_blocking()
            }).await;

            if cancelled.load(Ordering::SeqCst) {
                return;
            }

            match result {
                Ok(Ok(processed)) => {
                    let thumb_bytes = glib::Bytes::from_owned(processed.thumbnail_bytes);
                    let thumbnail = gdk4::Texture::from_bytes(&thumb_bytes).ok();

                    let attachment = ImageAttachment {
                        base64: processed.base64,
                    };

                    *state.borrow_mut() = AttachmentState::Ready(attachment);

                    spinner.set_visible(false);
                    let picture = gtk4::Picture::new();
                    picture.set_paintable(thumbnail.as_ref());
                    picture.set_width_request(100);
                    h_clamp.set_child(Some(&picture));
                }
                Ok(Err(e)) => {
                    eprintln!("Failed to process image: {e}");
                    spinner.set_visible(false);
                    let label = gtk4::Label::new(Some("Error"));
                    h_clamp.set_child(Some(&label));
                }
                Err(e) => {
                    eprintln!("Background task failed: {e:?}");
                    spinner.set_visible(false);
                    let label = gtk4::Label::new(Some("Error"));
                    h_clamp.set_child(Some(&label));
                }
            }
        });

        attachment_widget
    }

    pub fn get_attachment(&self) -> Option<ImageAttachment> {
        match &*self.state.borrow() {
            AttachmentState::Loading => None,
            AttachmentState::Ready(attachment) => Some(attachment.clone()),
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(&*self.state.borrow(), AttachmentState::Ready(_))
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
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

        let scrolled_window = gtk4::ScrolledWindow::new();
        scrolled_window.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Never);
        container.set_child(Some(&scrolled_window));

        let bx = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        bx.add_css_class("ai-chat-input-attachments-box");
        scrolled_window.set_child(Some(&bx));

        Self {
            container,
            bx,
            attachments: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl ImageAttachments {
    pub fn get_attachments(&self) -> Vec<ImageAttachment> {
        self.attachments
            .borrow()
            .iter()
            .filter_map(|widget| widget.get_attachment())
            .collect()
    }

    pub fn all_ready(&self) -> bool {
        self.attachments.borrow().iter().all(|w| w.is_ready())
    }

    pub fn push_texture(&self, texture: &gdk4::Texture) {
        if let Some(raw_image) = RawImage::from_texture(texture) {
            let widget = ImageAttachmentWidget::new_async(self.clone(), raw_image);
            self.bx.append(&widget.widget);
            self.attachments.borrow_mut().push(widget);
            self.container.set_reveal_child(true);
        }
    }

    pub fn remove(&self, index: usize) {
        let widget = self.attachments.borrow_mut().remove(index);
        widget.cancel();
        self.bx.remove(&widget.widget);
        self.container.set_reveal_child(!self.attachments.borrow().is_empty());
    }

    pub fn clear(&self) {
        for widget in self.attachments.borrow_mut().drain(..) {
            widget.cancel();
            self.bx.remove(&widget.widget);
        }
        self.container.set_reveal_child(false);
    }
}
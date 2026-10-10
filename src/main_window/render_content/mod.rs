use gtk::{Widget, gio::Settings};

use crate::clipboard_object::ClipboardObject;

const MAX_IMG_WIDTH: f64 = 150.00;
const MAX_IMG_HEIGHT: f64 = 100.00;
const PREVIEW_ALWAYS: u32 = 0;
const PREVIEW_IMAGE_FILES_ONLY: u32 = 1;
const PREVIEW_IMAGES_ONLY: u32 = 2;
const PREVIEW_NEVER: u32 = 3;

mod file;
mod image;
mod text;

pub fn item_widget(clipboard_object: &ClipboardObject, settings: Settings) -> Option<Widget> {
    let item_bytes = clipboard_object.item_bytes();
    let mime_type = clipboard_object.mime_type();

    let is_image = mime_type.starts_with("image/");
    let is_file = mime_type == "text/uri-list";
    let is_text = mime_type.starts_with("text/");

    if is_image {
        return image::widget(item_bytes, settings);
    }

    if is_file {
        return file::widget(item_bytes, settings);
    }

    if is_text {
        return text::widget(item_bytes);
    }

    None
}

pub fn image_preview_size(width: i32, height: i32) -> (i32, i32) {
    let width = width as f64;
    let height = height as f64;
    let scale = (MAX_IMG_WIDTH / width as f64)
        .min(MAX_IMG_HEIGHT / height as f64)
        .min(1.0);
    (
        (width * scale).round() as i32,
        (height * scale).round() as i32,
    )
}

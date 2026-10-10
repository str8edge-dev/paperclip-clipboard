use std::io::Cursor;

use gtk::{
    Label, Picture, Widget,
    gdk::{Texture, prelude::TextureExt},
    gio::{Settings, prelude::SettingsExt},
    glib::{Bytes, object::Cast},
    pango,
    prelude::WidgetExt,
};
use image::ImageReader;

use crate::main_window::render_content::{
    PREVIEW_ALWAYS, PREVIEW_IMAGE_FILES_ONLY, PREVIEW_IMAGES_ONLY, PREVIEW_NEVER,
    image_preview_size,
};

pub fn widget(image_bytes: Bytes, settings: Settings) -> Option<Widget> {
    let mode = settings.uint("image-preview-mode");
    match mode {
        PREVIEW_ALWAYS | PREVIEW_IMAGES_ONLY => {
            create_picture(&image_bytes).map(|p| p.upcast::<Widget>())
        }
        PREVIEW_IMAGE_FILES_ONLY | PREVIEW_NEVER => {
            Some(text_image_preview(&image_bytes).upcast::<Widget>())
        }
        _ => None,
    }
}

fn create_picture(image_bytes: &Bytes) -> Option<Picture> {
    match Texture::from_bytes(image_bytes) {
        Ok(texture) => {
            let (width, height) = image_preview_size(texture.width(), texture.height());
            let picture = Picture::builder()
                .paintable(&texture)
                .width_request(width)
                .height_request(height)
                .build();
            picture.add_css_class("row-image");
            Some(picture)
        }
        Err(e) => {
            eprintln!("Error creating Picture: {:#}", e);
            None
        }
    }
}

fn image_dimensions(bytes: &Bytes) -> Option<(u32, u32)> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;

    reader.into_dimensions().ok()
}

fn text_image_preview(bytes: &Bytes) -> Label {
    let text = match image_dimensions(bytes) {
        Some((width, height)) => format!("Image - {width} x {height}"),
        None => "Image".to_string(),
    };
    let label = Label::builder()
        .label(text)
        .halign(gtk::Align::Start)
        .ellipsize(pango::EllipsizeMode::End)
        .single_line_mode(true)
        .build();
    label.add_css_class("row-label");
    label.add_css_class("dim-label");
    label
}

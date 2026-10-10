use gtk::{
    Box, Label, Picture, Widget,
    gdk::Texture,
    gio::{File, Settings},
    glib::{Bytes, object::Cast},
    pango,
    prelude::*,
};
use std::{
    env::home_dir,
    path::{Path, PathBuf},
};
use url::Url;

use crate::main_window::render_content::{
    PREVIEW_ALWAYS, PREVIEW_IMAGE_FILES_ONLY, PREVIEW_IMAGES_ONLY, PREVIEW_NEVER,
    image_preview_size,
};

pub fn widget(item_bytes: Bytes, settings: Settings) -> Option<Widget> {
    let files = bytes_to_pathbuf(item_bytes);
    match files.as_slice() {
        [] => None,

        [file] => match mimetype_detector::detect_file(&file) {
            Ok(mime) if mime.to_string().starts_with("image/") => {
                let mode = settings.uint("image-preview-mode");
                match mode {
                    PREVIEW_ALWAYS | PREVIEW_IMAGE_FILES_ONLY => {
                        create_picture_from_file(file).map(|p| p.upcast::<Widget>())
                    }
                    PREVIEW_IMAGES_ONLY | PREVIEW_NEVER => {
                        single_file_label(file).map(|b| b.upcast::<Widget>())
                    }
                    _ => None,
                }
            }
            _ => return single_file_label(file).map(|b| b.upcast::<Widget>()),
        },

        [_, _, ..] => multiple_files_label(files).map(|b| b.upcast::<Widget>()),
    }
}

fn bytes_to_pathbuf(item_bytes: Bytes) -> Vec<PathBuf> {
    let uri_string = String::from_utf8_lossy(&item_bytes).to_string();
    uri_string
        .lines()
        .map(str::trim)
        .filter_map(|line| Url::parse(line).ok()?.to_file_path().ok())
        .collect()
}

fn split_parent_and_filename(path: &PathBuf) -> Option<(String, &str)> {
    let name = path.file_name().and_then(|n| n.to_str())?;
    let parent = path.parent()?;
    let parent_path = match home_dir() {
        Some(home) => parent
            .strip_prefix(home)
            .map(|p| Path::new("~").join(p))
            .unwrap_or_else(|_| parent.to_path_buf()),
        None => parent.to_path_buf(),
    };

    let parent_str = parent_path.to_string_lossy().into_owned();
    Some((parent_str, name))
}

fn single_file_label(path: &PathBuf) -> Option<Box> {
    let (parent, filename) = split_parent_and_filename(path)?;

    let container = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(1)
        .build();
    container.add_css_class("main-box");
    let label = Label::builder()
        .label(filename)
        .halign(gtk::Align::Start)
        .ellipsize(pango::EllipsizeMode::End)
        .single_line_mode(true)
        .build();
    label.add_css_class("row-label");
    let sub_label = Label::builder()
        .label(parent)
        .halign(gtk::Align::Start)
        .ellipsize(pango::EllipsizeMode::End)
        .single_line_mode(true)
        .build();
    sub_label.add_css_class("dim-label");
    sub_label.add_css_class("caption");
    container.append(&label);
    container.append(&sub_label);

    Some(container)
}

fn multiple_files_label(files: Vec<PathBuf>) -> Option<Box> {
    let file_names = files
        .iter()
        .map(|f| f.file_name().and_then(|n| n.to_str()).unwrap())
        .collect::<Vec<_>>();

    let file_count = format!("{} Files", file_names.len());
    let file_names_str = file_names.join(", ");

    let container = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(1)
        .build();
    container.add_css_class("main-box");
    let label = Label::builder()
        .label(file_count)
        .halign(gtk::Align::Start)
        .ellipsize(pango::EllipsizeMode::End)
        .single_line_mode(true)
        .build();
    label.add_css_class("dim-label");
    label.add_css_class("caption");
    let sub_label = Label::builder()
        .label(file_names_str)
        .halign(gtk::Align::Start)
        .ellipsize(pango::EllipsizeMode::End)
        .single_line_mode(true)
        .build();
    sub_label.add_css_class("row-label");
    container.append(&label);
    container.append(&sub_label);

    Some(container)
}

fn create_picture_from_file(path: &PathBuf) -> Option<Picture> {
    let file = File::for_path(path);
    match Texture::from_file(&file) {
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
            eprintln!("Texture error: {:#}", e);
            None
        }
    }
}

use chrono::Utc;
use glib::Properties;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, glib::Bytes};
use std::cell::{Cell, RefCell};

#[derive(Properties)]
#[properties(wrapper_type = super::ClipboardObject)]
pub struct ClipboardObject {
    #[property(get, construct_only, type = glib::Bytes)]
    item_bytes: RefCell<Bytes>,
    #[property(get, construct_only, type = glib::Bytes)]
    hash: RefCell<Bytes>,
    #[property(get, construct_only)]
    mime_type: RefCell<String>,
    #[property(get, set)]
    timestamp: Cell<i64>,
}

impl Default for ClipboardObject {
    fn default() -> Self {
        Self {
            item_bytes: RefCell::new(Bytes::from_static(b"")),
            mime_type: RefCell::new(String::new()),
            hash: RefCell::new(Bytes::from_owned([0; 32])),
            timestamp: Cell::new(Utc::now().timestamp_millis()),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ClipboardObject {
    const NAME: &'static str = "MyGtkAppClipboardObject";
    type Type = super::ClipboardObject;
}

#[glib::derived_properties]
impl ObjectImpl for ClipboardObject {}

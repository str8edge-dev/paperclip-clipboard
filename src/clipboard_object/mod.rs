mod imp;

use glib::Object;
use gtk::glib::{self, Bytes};
use sha2::{Digest, Sha256};

glib::wrapper! {
    pub struct ClipboardObject(ObjectSubclass<imp::ClipboardObject>);
}

pub fn hash_bytes(data: &[u8]) -> Bytes {
    Bytes::from_owned(Sha256::digest(data))
}

impl ClipboardObject {
    pub fn new(item_bytes: glib::Bytes, mime_type: String) -> Self {
        let hash = hash_bytes(&item_bytes);
        Object::builder()
            .property("item_bytes", item_bytes)
            .property("mime_type", mime_type)
            .property("hash", hash)
            .build()
    }
}

pub mod wayland;

pub struct ClipboardEvent {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

pub fn start_clipboard_listener(sender: async_channel::Sender<ClipboardEvent>) {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        if let Err(e) = wayland::send_event(sender) {
            eprintln!("Clipboard listener error: {e:#}");
        }
    }
}

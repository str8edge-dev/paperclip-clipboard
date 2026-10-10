use anyhow::{self, Context};
use async_channel::Sender;
use gtk::gio;
use wayland_clipboard_listener::{WlClipboardPasteStream, WlListenType};

use crate::backend::ClipboardEvent;

pub const PASSWORD_HINT_MIME: &str = "x-kde-passwordManagerHint";

pub fn send_event(sender: Sender<ClipboardEvent>) -> anyhow::Result<()> {
    let mut stream = WlClipboardPasteStream::init(WlListenType::ListenOnCopy)
        .context("Wayland Clipboard Protocol not support by this compositor")?;

    stream.set_priority(vec![
        "image/png".into(),
        "image/webp".into(),
        "image/jpeg".into(),
        "text/uri-list".into(),
        "x-special/gnome-copied-files".into(),
        "text/plain;charset=utf-8".into(),
        "text/plain".into(),
        "text/html".into(),
    ]);

    gio::spawn_blocking(move || {
        for contextmsg in stream.paste_stream().flatten() {
            let mime_types = contextmsg.mime_types;
            if mime_types.iter().any(|m| m == PASSWORD_HINT_MIME) {
                continue;
            }
            let listen_context = contextmsg.context;
            let event = ClipboardEvent {
                bytes: listen_context.context,
                mime_type: listen_context.mime_type,
            };
            if let Err(e) = sender.send_blocking(event) {
                eprintln!("Failed to send clipboard event: {e}");
            }
        }
    });
    Ok(())
}

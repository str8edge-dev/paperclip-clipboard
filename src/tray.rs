use async_channel::{Receiver, Sender};
use gtk::{
    Application,
    gio::prelude::ApplicationExt,
    prelude::{GtkApplicationExt, GtkWindowExt},
};

use crate::config;

pub enum TrayEvent {
    ShowClipboard,
    Exit,
}

#[derive(Debug)]
pub struct ClipboardTray {
    pub sender: Sender<TrayEvent>,
}

impl ksni::Tray for ClipboardTray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }
    fn icon_name(&self) -> String {
        config::app_id().to_string()
    }
    fn title(&self) -> String {
        "ClipboardTray".into()
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        let sender = || self.sender.clone();

        vec![
            StandardItem {
                label: "Show Clipboard".into(),
                activate: Box::new({
                    let sender = sender();
                    move |_| {
                        if let Err(e) = sender.send_blocking(TrayEvent::ShowClipboard) {
                            eprintln!("Failed to send TrayEvent::ShowClipboard: {e}")
                        } else {
                            println!("[DEBUG] Toggled ShowClipboard")
                        }
                    }
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Exit".into(),
                activate: Box::new({
                    let sender = sender();
                    move |_| {
                        if let Err(e) = sender.send_blocking(TrayEvent::Exit) {
                            eprintln!("Failed to send TrayEvent::Exit: {e}")
                        }
                    }
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub async fn tray_event_handler(receiver: Receiver<TrayEvent>, app: &Application) {
    while let Ok(event) = receiver.recv().await {
        match event {
            TrayEvent::ShowClipboard => {
                if let Some(window) = app.active_window() {
                    window.present();
                }
            }
            TrayEvent::Exit => {
                app.quit();
            }
        }
    }
}

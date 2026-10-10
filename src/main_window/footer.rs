use gtk::{
    ApplicationWindow, Box, Button,
    gio::{ListStore, Settings},
    glib::{self, clone},
    prelude::*,
};

use crate::{config, database::DbState, settings};

pub fn build(window: &ApplicationWindow, store: ListStore, db: DbState) -> Box {
    let settings = Settings::new(config::app_id());

    let main_box = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(0)
        .build();
    main_box.add_css_class("footer-box");

    let preferences_label = gtk::Label::builder()
        .label("Preferences")
        .halign(gtk::Align::Start)
        .build();
    let preferences_btn = Button::builder()
        .child(&preferences_label)
        .halign(gtk::Align::Fill)
        .hexpand(true)
        .build();
    preferences_btn.add_css_class("flat");
    preferences_btn.add_css_class("preferences-button");
    preferences_btn.add_css_class("footer-button");
    preferences_btn.connect_clicked(clone!(
        #[weak]
        window,
        #[strong]
        settings,
        move |_| {
            settings::window::open_settings(&window, &settings);
        },
    ));
    main_box.append(&preferences_btn);

    let clear_label = gtk::Label::builder()
        .label("Clear")
        .halign(gtk::Align::Start)
        .build();
    let clear_btn = Button::builder()
        .child(&clear_label)
        .halign(gtk::Align::Fill)
        .hexpand(true)
        .build();
    clear_btn.add_css_class("flat");
    clear_btn.add_css_class("clear-button");
    clear_btn.add_css_class("footer-button");
    clear_btn.connect_clicked(clone!(
        #[weak]
        store,
        #[weak]
        settings,
        move |_| {
            let db = db.clone();
            if settings.boolean("save-history") {
                glib::spawn_future_local(async move {
                    if let Err(e) = db.clear_all().await {
                        eprintln!("{:#}", e);
                    }
                });
            }
            store.remove_all();
        }
    ));
    main_box.append(&clear_btn);

    main_box
}

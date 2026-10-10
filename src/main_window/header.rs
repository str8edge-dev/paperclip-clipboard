use gtk::{
    Box, Label, SearchEntry,
    gio::{self, ListStore, Settings},
    glib::{self, clone},
    prelude::*,
};

use crate::config;

pub fn build_header(store: ListStore, search_entry: SearchEntry) -> Box {
    let settings = Settings::new(config::app_id());
    let header_name = settings.string("header-name");

    let main_box = Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(10)
        .build();
    main_box.add_css_class("header-box");

    let header_label = Label::builder()
        .label(header_name)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();
    header_label.set_visible(!header_label.label().is_empty());
    header_label.connect_notify_local("label".into(), |label, _| {
        label.set_visible(!label.label().is_empty());
    });
    main_box.add_css_class("header-label");
    main_box.append(&header_label);

    settings
        .bind("header-name", &header_label, "label")
        .flags(gio::SettingsBindFlags::GET)
        .build();

    let count_label = Label::builder()
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();
    count_label.add_css_class("count-label");
    main_box.append(&count_label);

    settings
        .bind("show-count", &count_label, "visible")
        .flags(gio::SettingsBindFlags::DEFAULT)
        .build();

    let update_count_label = clone!(
        #[weak]
        count_label,
        #[weak]
        store,
        move || {
            count_label.set_label(&store.n_items().to_string());
        }
    );
    update_count_label();
    store.connect_items_changed(move |_, _, _, _| {
        update_count_label();
    });

    main_box.append(&search_entry);

    main_box
}

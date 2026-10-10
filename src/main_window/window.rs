use crate::clipboard_object::ClipboardObject;
use crate::config;
use crate::database::DbState;
use crate::main_window::footer;

use super::header::build_header;
use super::listview::build_list;

use gtk::gio::{ListStore, Settings};
use gtk::{Application, ApplicationWindow, Box, PolicyType, ScrolledWindow, gio};
use gtk::{SearchEntry, Separator, prelude::*};

pub fn build_ui(app: &Application, db: DbState) {
    let main_box = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(1)
        .build();
    main_box.add_css_class("main-box");

    let settings = Settings::new(config::app_id());

    let window = ApplicationWindow::builder()
        .application(app)
        .resizable(false)
        .child(&main_box)
        .build();
    // window.set_decorated(false);
    settings
        .bind("window-width", &window, "default-width")
        .flags(gio::SettingsBindFlags::DEFAULT)
        .build();

    let store = ListStore::new::<ClipboardObject>();

    let search_entry = SearchEntry::builder().hexpand(true).build();
    search_entry.add_css_class("search-entry");

    let header = build_header(store.clone(), search_entry.clone());
    main_box.append(&header);

    let list_view = build_list(store.clone(), db.clone(), search_entry);
    list_view.set_vexpand(false);

    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .propagate_natural_height(true)
        .hexpand(false)
        .vexpand(false)
        .child(&list_view)
        .build();
    main_box.append(&scrolled_window);
    settings
        .bind("max-content-height", &scrolled_window, "max-content-height")
        .flags(gio::SettingsBindFlags::DEFAULT)
        .build();

    let separator = Separator::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    main_box.append(&separator);

    let footer = footer::build(&window, store, db);
    main_box.append(&footer);

    window.connect_visible_notify(move |window| {
        if window.is_visible() {
            if let Some(model) = list_view.model() {
                if model.n_items() > 0 {
                    list_view.scroll_to(0, gtk::ListScrollFlags::SELECT, None);
                }
            }
        }
    });

    let hide_action = gtk::gio::ActionEntry::builder("hide")
        .activate(move |win: &ApplicationWindow, _, _| win.set_visible(false))
        .build();
    window.add_action_entries([hide_action]);
}

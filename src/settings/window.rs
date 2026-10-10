use gtk::{
    Align, ApplicationWindow, Box, DropDown, Entry, Image, Label, ListBox, ListBoxRow, Orientation,
    StringList, Switch, Window,
    gio::Settings,
    glib::{self, clone},
    prelude::*,
};

use crate::config;

pub fn open_settings(parent: &ApplicationWindow, settings: &Settings) {
    let window = Window::builder()
        .title("Preferences")
        .default_width(540)
        .transient_for(parent)
        .destroy_with_parent(true)
        .modal(true)
        .resizable(true)
        .build();
    window.add_css_class("preferences-window");

    let preferences_list = ListBox::builder().build();
    preferences_list.add_css_class("preferences-list");

    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(20)
        .build();
    main_box.add_css_class("preferences-box");
    main_box.append(&preferences_list);

    window.set_child(Some(&main_box));

    let show_tray_switch = Switch::builder().build();
    settings
        .bind("show-tray", &show_tray_switch, "active")
        .build();
    append_settings_row(
        "Show Tray",
        "Shows the app icon in system tray. Requires restart.",
        show_tray_switch,
        &preferences_list,
    );

    let save_history_switch = Switch::builder().build();
    settings
        .bind("save-history", &save_history_switch, "active")
        .build();
    append_settings_row(
        "Save History",
        "Saves clipboard history between sessions. When disabled, history is hidden on the next launch but can be restored by re-enabling this setting.",
        save_history_switch,
        &preferences_list,
    );

    let adj = gtk::Adjustment::new(
        400.0,  // default
        200.0,  // minimum
        1000.0, // maximum
        1.0,    // step
        100.0,  // page increment
        0.0,    // page size
    );
    let win_width_spin = gtk::SpinButton::new(Some(&adj), 1.0, 0);
    settings
        .bind("window-width", &win_width_spin, "value")
        .build();
    append_settings_row(
        "Window Width",
        "Width of the application window.",
        win_width_spin,
        &preferences_list,
    );

    let adj = gtk::Adjustment::new(
        300.0,  // default
        150.0,  // minimum
        1000.0, // maximum
        1.0,    // step
        100.0,  // page increment
        0.0,    // page size
    );
    let max_height_spin = gtk::SpinButton::new(Some(&adj), 1.0, 0);
    settings
        .bind("max-content-height", &max_height_spin, "value")
        .build();
    append_settings_row(
        "Max Content Height",
        "Maximum height of clipboard content before scrolling is enabled.",
        max_height_spin,
        &preferences_list,
    );

    let show_count_switch = Switch::builder().build();
    settings
        .bind("show-count", &show_count_switch, "active")
        .build();
    append_settings_row(
        "Show Count",
        "Shows the count of total items copied.",
        show_count_switch,
        &preferences_list,
    );

    let options = StringList::new(&["Always", "Image files only", "Images only", "Never"]);
    let image_preview_mode_dropdown = DropDown::builder().model(&options).build();
    settings
        .bind(
            "image-preview-mode",
            &image_preview_mode_dropdown,
            "selected",
        )
        .build();
    append_settings_row(
        "Preview Image",
        "Controls when image previews are displayed. New entries update immediately. Restart to refresh existing ones.",
        image_preview_mode_dropdown,
        &preferences_list,
    );

    let adjustment = gtk::Adjustment::new(
        100.0,   // default
        5.0,     // minimum
        10000.0, // maximum
        1.0,     // step
        100.0,   // page increment
        0.0,     // page size
    );

    let spin = gtk::SpinButton::new(Some(&adjustment), 1.0, 0);

    settings.bind("history-limit", &spin, "value").build();

    append_settings_row(
        "History Limit",
        "Maximum number of clipboard items to retain.",
        spin,
        &preferences_list,
    );

    let entry = Entry::builder()
        .placeholder_text("Custom Text")
        .hexpand(true)
        .build();
    settings.bind("header-name", &entry, "text").build();
    append_settings_row(
        "Header Name",
        "Set a custom name to be displayed on the header of the app.",
        entry,
        &preferences_list,
    );

    let about_button = gtk::Button::builder()
        .label("About")
        .halign(Align::Fill)
        .hexpand(true)
        .build();
    about_button.add_css_class("about-button");
    about_button.add_css_class("flat");
    main_box.append(&about_button);
    about_button.connect_clicked(clone!(
        #[weak]
        window,
        move |_| {
            about_window(&window);
        }
    ));

    window.present();
}

fn append_settings_row(
    main_text: &str,
    sub_text: &str,
    widget: impl IsA<gtk::Widget>,
    listbox: &ListBox,
) {
    let row = ListBoxRow::builder()
        .selectable(false)
        .activatable(false)
        .build();
    row.add_css_class("preferences-row");

    let main_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(20)
        .build();

    let main_label = Label::builder()
        .label(main_text)
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let sub_label = Label::builder()
        .label(sub_text)
        .halign(Align::Start)
        .wrap(true)
        .max_width_chars(38)
        .hexpand(true)
        .build();
    sub_label.add_css_class("caption");
    sub_label.add_css_class("dim-label");

    let label_box = Box::builder().orientation(Orientation::Vertical).build();
    label_box.append(&main_label);
    label_box.append(&sub_label);

    widget.set_valign(Align::Center);

    main_box.append(&label_box);
    main_box.append(&widget);
    row.set_child(Some(&main_box));

    listbox.append(&row);
}

pub fn about_window(parent: &Window) {
    let window = Window::builder()
        .title("About")
        .transient_for(parent)
        .destroy_with_parent(true)
        .modal(true)
        .resizable(false)
        .build();
    window.add_css_class("about-window");

    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .halign(Align::Center)
        .build();
    main_box.add_css_class("about-box");
    window.set_child(Some(&main_box));

    let app_icon = Image::from_resource(&format!("{}/app-icon.svg", config::gresource_path()));
    app_icon.add_css_class("app-icon");
    app_icon.set_pixel_size(128);
    app_icon.set_halign(gtk::Align::Center);
    app_icon.set_valign(gtk::Align::Center);
    main_box.append(&app_icon);

    let app_name_label = Label::builder()
        .label(config::app_name())
        .halign(Align::Center)
        .wrap(true)
        .build();
    app_name_label.add_css_class("title-2");
    main_box.append(&app_name_label);

    let version_label = Label::builder()
        .label(&format!("Version {}", env!("CARGO_PKG_VERSION")))
        .halign(Align::Center)
        .wrap(true)
        .build();
    version_label.add_css_class("dim-label");
    main_box.append(&version_label);

    let app_credit_label = Label::builder()
        .label(r#"By <a href="https://github.com/str8edge-dev">@str8edge-dev</a>"#)
        .use_markup(true)
        .halign(Align::Center)
        .wrap(true)
        .build();
    main_box.append(&app_credit_label);

    let legal_label = Label::builder()
        .label("Distributed under the GNU General Public License, version 3")
        .halign(Align::Fill)
        .justify(gtk::Justification::Center)
        .wrap(true)
        .max_width_chars(32)
        .build();
    legal_label.add_css_class("dim-label");
    main_box.append(&legal_label);

    window.present();
}

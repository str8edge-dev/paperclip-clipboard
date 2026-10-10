mod backend;
mod clipboard_object;
mod config;
mod database;
mod main_window;
mod settings;
mod tray;

use anyhow::Context;
use gtk::{
    Application, CssProvider,
    gdk::Display,
    gio::{self, Settings},
    glib::{self, ExitCode, clone},
    prelude::*,
};
use ksni::TrayMethods;
use main_window::window::build_ui;

use crate::{
    database::DbState,
    tray::{TrayEvent, tray_event_handler},
};

fn load_css() -> anyhow::Result<()> {
    let display = Display::default().context("Could not connect to a display")?;

    // Default css
    let default_provider = CssProvider::new();
    default_provider.load_from_resource(&format!("{}/style.css", config::gresource_path()));
    gtk::style_context_add_provider_for_display(
        &display,
        &default_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // User css
    let user_css_path = config::app_config_dir()?.join("style.css");
    if user_css_path.is_file() {
        let user_provider = CssProvider::new();
        user_provider.load_from_file(&gio::File::for_path(&user_css_path));
        gtk::style_context_add_provider_for_display(
            &display,
            &user_provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let res = gio::Resource::load(config::resources_file()).expect("Could not load gresource file");
    gio::resources_register(&res);

    let app = Application::builder()
        .application_id(config::app_id())
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();
    let settings = Settings::new(config::app_id());

    app.connect_startup(clone!(
        #[strong]
        settings,
        move |app| {
            match load_css() {
                Ok(_) => println!("CSS loaded successfully"),
                Err(e) => eprintln!("Error while loading the CSS: {e:#}"),
            }
            app.set_accels_for_action("win.hide", &["Escape"]);

            let (sender, receiver) = async_channel::unbounded::<TrayEvent>();
            let tray = tray::ClipboardTray { sender };

            if settings.boolean("show-tray") {
                glib::spawn_future_local(async move {
                    if let Err(e) = tray.spawn().await {
                        eprintln!("Failed to spawn tray: {e:#}");
                    }
                });

                glib::spawn_future_local(clone!(
                    #[strong]
                    app,
                    async move {
                        tray_event_handler(receiver, &app).await;
                    }
                ));
            }
        }
    ));

    app.connect_activate(move |app| {
        if app.active_window().is_some() {
            return;
        }
        let limit: usize = settings.int("history-limit") as usize;
        match DbState::init(limit) {
            Ok(db) => build_ui(app, db),
            Err(e) => eprintln!("Failed to initialize the database: {e:#}"),
        }
    });

    app.connect_command_line(|app, cmd_line| {
        let args = cmd_line.arguments();
        app.activate();
        let show_window = args.contains(&"--show".into());
        if let Some(window) = app.windows().first() {
            if show_window {
                window.present();
            }
        }
        ExitCode::SUCCESS
    });

    app.run();
    Ok(())
}

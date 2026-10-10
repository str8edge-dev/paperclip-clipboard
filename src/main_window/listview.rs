use super::render_content::item_widget;

use crate::backend::{ClipboardEvent, start_clipboard_listener};
use crate::clipboard_object::ClipboardObject;
use crate::config;
use crate::database::DbState;

use async_channel::unbounded;
use gtk::gdk::{Clipboard, ContentProvider};
use gtk::gio::{ListStore, Settings};
use gtk::glib::{Bytes, clone};
use gtk::{
    Box, Button, CustomFilter, CustomSorter, FilterListModel, ListItem, ListView, Overlay,
    SignalListItemFactory, SingleSelection, SortListModel, glib,
};
use gtk::{SearchEntry, prelude::*};

pub fn build_list(store: ListStore, db: DbState, search_entry: SearchEntry) -> ListView {
    // IPC for getting the clipboard context
    let (sender, receiver) = unbounded::<ClipboardEvent>();
    start_clipboard_listener(sender);

    // Initializing GtkListView widget
    let list_view = ListView::builder()
        .single_click_activate(true)
        .vexpand(true)
        .build();
    list_view.add_css_class("main-listview");

    let settings = Settings::new(config::app_id());

    glib::spawn_future_local(clone!(
        #[weak]
        list_view,
        #[weak]
        store,
        #[weak]
        settings,
        #[strong]
        db,
        async move {
            // Initialize the items from database onto the listview
            if settings.boolean("save-history") {
                match db.get_history().await {
                    Ok(history) => {
                        for item in history.into_iter() {
                            let clipboard_obj =
                                ClipboardObject::new(Bytes::from_owned(item.bytes), item.mime_type);
                            clipboard_obj.set_timestamp(item.timestamp);
                            store.append(&clipboard_obj);
                        }
                    }
                    Err(e) => eprintln!("Failed to load history: {:#}", e),
                }
            }

            list_view.grab_focus();

            // Listens for changes in clipboard
            while let Ok(clipboard_event) = receiver.recv().await {
                insert_into_listview(
                    &list_view,
                    &store,
                    settings.clone(),
                    clipboard_event,
                    db.clone(),
                );
            }
            println!("Clipboard listener stopped");
        }
    ));

    let factory = SignalListItemFactory::new();
    factory.connect_setup(glib::clone!(
        #[weak]
        store,
        #[strong]
        settings,
        #[strong]
        db,
        move |_, list_item| {
            let container = Box::new(gtk::Orientation::Horizontal, 1);
            let overlay = Overlay::builder().child(&container).build();
            let list_item = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem");

            list_item.set_child(Some(&overlay));

            let remove_btn = Button::builder()
                .icon_name("window-close-symbolic")
                .halign(gtk::Align::End)
                .valign(gtk::Align::Center)
                .build();
            overlay.add_overlay(&remove_btn);
            remove_btn.add_css_class("remove-button");
            remove_btn.add_css_class("circular");

            remove_btn.connect_clicked(glib::clone!(
                #[weak]
                store,
                #[weak]
                list_item,
                #[strong]
                settings,
                #[strong]
                db,
                move |_| {
                    let Some(item) = list_item.item() else {
                        return;
                    };

                    let clipboard_object = item
                        .downcast::<ClipboardObject>()
                        .expect("Needs to be ClipboardObject");
                    let Some(pos) = store.find(&clipboard_object) else {
                        return;
                    };
                    let hash = clipboard_object.hash();
                    if settings.boolean("save-history") {
                        glib::spawn_future_local(glib::clone!(
                            #[weak]
                            store,
                            #[strong]
                            db,
                            async move {
                                if let Err(e) = db.remove(&hash).await {
                                    eprintln!("{:#}", e);
                                } else {
                                    store.remove(pos);
                                }
                            }
                        ));
                    } else {
                        store.remove(pos);
                    }
                }
            ));
        }
    ));

    factory.connect_bind(clone!(
        #[weak]
        settings,
        move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem");

            let clipboard_object = list_item
                .item()
                .and_downcast::<ClipboardObject>()
                .expect("Needs to be ClipboardObject");

            let overlay = list_item
                .child()
                .and_downcast::<Overlay>()
                .expect("The child has to be Overlay");

            let container = overlay
                .child()
                .and_downcast::<Box>()
                .expect("The child has to be Box");

            while let Some(child) = container.first_child() {
                container.remove(&child);
            }

            if let Some(widget) = item_widget(&clipboard_object, settings) {
                container.append(&widget);
            }
        }
    ));

    factory.connect_unbind(move |_, list_item| {
        let list_item = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListView");
        let overlay = list_item
            .child()
            .and_downcast::<Overlay>()
            .expect("Needs to be Overlay");
        let container = overlay
            .child()
            .and_downcast::<Box>()
            .expect("Needs to be Box");

        while let Some(child) = container.first_child() {
            container.remove(&child);
        }
    });

    let filter = CustomFilter::new(clone!(
        #[strong]
        search_entry,
        move |obj| {
            let clipboard_object = obj
                .downcast_ref::<ClipboardObject>()
                .expect("Needs to be ClipboardObject");

            let bytes = clipboard_object.item_bytes();
            let mime = clipboard_object.mime_type();
            if !is_valid_item(&bytes, &mime) {
                return false;
            }

            let query = search_entry.text();
            is_search_matching(&query, &bytes, &mime)
        }
    ));

    search_entry.connect_search_changed(clone!(
        #[weak]
        filter,
        move |_| {
            filter.changed(gtk::FilterChange::Different);
        }
    ));

    let filter_model = FilterListModel::new(Some(store), Some(filter));

    let sorter = CustomSorter::new(move |obj1, obj2| {
        let clipboard_object_1 = obj1
            .downcast_ref::<ClipboardObject>()
            .expect("Needs to be ClipboardObject");
        let clipboard_object_2 = obj2
            .downcast_ref::<ClipboardObject>()
            .expect("Needs to be ClipboardObject");

        let timestamp_1 = clipboard_object_1.timestamp();
        let timestamp_2 = clipboard_object_2.timestamp();

        // Reverse sorting order -> large numbers come first
        timestamp_2.cmp(&timestamp_1).into()
    });

    let sort_model = SortListModel::new(Some(filter_model), Some(sorter.clone()));
    let selection_model = SingleSelection::builder()
        .model(&sort_model)
        .autoselect(true)
        .build();

    list_view.set_model(Some(&selection_model));
    list_view.set_factory(Some(&factory));

    list_view.connect_activate(move |list_view, position| {
        let model = list_view.model().expect("The model has to exist.");
        let clipboard_object = model
            .item(position)
            .and_downcast::<ClipboardObject>()
            .expect("Needs to be `ClipboardObject`.");

        // Set system clipboard on click using GDK Clipboard
        let clipboard = list_view.display().clipboard();
        handle_copy(&clipboard, &clipboard_object);
        if let Err(e) = list_view.activate_action("win.hide", None) {
            eprintln!("Failed to activate action: {:#}", e);
        }
    });

    list_view
}

fn insert_into_listview(
    list_view: &ListView,
    store: &ListStore,
    settings: Settings,
    clipboard_event: ClipboardEvent,
    db: DbState,
) {
    let item_bytes = clipboard_event.bytes;
    let mime_type = clipboard_event.mime_type;

    if !is_valid_item(&item_bytes, &mime_type) {
        return;
    }

    let gbytes = glib::Bytes::from_owned(item_bytes);
    let clipboard_obj = ClipboardObject::new(gbytes, mime_type);

    let mut duplicate_item: Option<u32> = None;
    for i in 0..store.n_items() {
        if let Some(item) = store.item(i).and_downcast::<ClipboardObject>() {
            if item.hash() == clipboard_obj.hash() {
                duplicate_item = Some(i);
                break;
            }
        }
    }

    if let Some(index) = duplicate_item {
        let item = store
            .item(index)
            .and_downcast::<ClipboardObject>()
            .expect("Needs to be ClipboardObject");

        // Updating timestamp
        let timestamp = chrono::Utc::now().timestamp_millis();
        item.set_timestamp(timestamp);

        // Update timestamp in db
        if settings.boolean("save-history") {
            glib::spawn_future_local(clone!(
                #[weak]
                item,
                async move {
                    if let Err(e) = db.store(item).await {
                        eprintln!("{:#}", e);
                    }
                }
            ));
        }

        // To make the CustomSorter and CustomFilter re-evaluate
        store.items_changed(index, 1, 1);
    } else {
        store.append(&clipboard_obj);

        // Store the clipboard object into the database
        if settings.boolean("save-history") {
            glib::spawn_future_local(clone!(
                #[weak]
                clipboard_obj,
                async move {
                    if let Err(e) = db.store(clipboard_obj).await {
                        eprintln!("{:#}", e);
                    }
                }
            ));
        }
    }

    while store.n_items() > settings.int("history-limit") as u32 {
        store.remove(0);
    }

    glib::idle_add_local_once(glib::clone!(
        #[weak]
        list_view,
        move || {
            list_view.scroll_to(
                0,
                gtk::ListScrollFlags::SELECT | gtk::ListScrollFlags::FOCUS,
                None,
            );
            list_view.grab_focus();
        }
    ));
}

pub fn handle_copy(clipboard: &Clipboard, clipboard_object: &ClipboardObject) {
    let item_bytes = clipboard_object.item_bytes();
    let mime_type = clipboard_object.mime_type();
    let content_provider = ContentProvider::for_bytes(&mime_type, &item_bytes);

    if let Err(e) = clipboard.set_content(Some(&content_provider)) {
        eprintln!("Error in copying to system clipboard: {:#}", e);
    }
}

fn is_valid_item(bytes: &[u8], mime_type: &str) -> bool {
    if bytes.is_empty() {
        return false;
    }

    if mime_type.starts_with("text/") {
        if let Ok(text) = std::str::from_utf8(bytes) {
            return !text.trim().is_empty();
        }
    }

    true
}

fn is_search_matching(query: &str, bytes: &[u8], mime_type: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    if mime_type.starts_with("text/") {
        if let Ok(text) = std::str::from_utf8(&bytes) {
            return text.to_lowercase().contains(&query.to_lowercase());
        }
    }

    false
}

use gtk::{
    Label, Widget,
    glib::{Bytes, object::Cast},
    pango,
    prelude::WidgetExt,
};

pub fn widget(item_bytes: Bytes) -> Option<Widget> {
    Some(label_from_bytes(item_bytes).upcast::<Widget>())
}

fn label_from_bytes(item_bytes: Bytes) -> Label {
    let text = String::from_utf8_lossy(&item_bytes).to_string();
    let label = Label::builder()
        .label(text)
        .halign(gtk::Align::Start)
        .ellipsize(pango::EllipsizeMode::End)
        .single_line_mode(true)
        .build();
    label.add_css_class("row-label");
    label
}

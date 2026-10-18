use gtk::prelude::*;
use gtk::{HeaderBar, Revealer, SearchEntry, ToggleButton};

pub struct Header {
    pub bar: HeaderBar,
    pub search: SearchEntry,
}

//header bar: a sidebar toggle on the left and search
//entry in the title area
pub fn build(revealer: &Revealer) -> Header {
    let toggle = ToggleButton::builder()
        .icon_name("sidebar-show-symbolic")
        .active(true)
        .css_classes(["sidebar-toggle"])
        .build();
    toggle
        .bind_property("active", revealer, "reveal-child")
        .sync_create()
        .build();

    let search = SearchEntry::builder()
        .placeholder_text("Search")
        .width_request(300)
        .build();
    search.connect_search_changed(|entry| {
        let query = entry.text();
        println!("searching for: {query}");
    });

    let bar = HeaderBar::new();
    bar.pack_start(&toggle);
    bar.set_title_widget(Some(&search));

    Header { bar, search }
}

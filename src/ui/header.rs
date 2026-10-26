//src/ui/header.rs
use super::accounts::Accounts;
use super::notifications::Notifications;
use super::state::Shared;
use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Button, HeaderBar, Orientation, Revealer, SearchEntry, Stack, ToggleButton,
};
use webkit6::prelude::*;
use webkit6::WebView;

use super::menu;
use super::pages::PAGES;
use super::webview::current_webview;

pub struct Header {
    pub bar: HeaderBar,
    pub search: SearchEntry,
    pub notifications: Notifications,
    pub toggle: ToggleButton,
    pub accounts: Accounts,
}
// ...

/// Build the header bar: sidebar toggle, back/forward buttons, and a
/// search entry in the title area.
pub fn build(revealer: &Revealer, stack: &Stack, state: &Shared) -> Header {
    // Sidebar toggle, bound directly to the revealer's state.
    let toggle = ToggleButton::builder()
        .icon_name("sidebar-show-symbolic")
        .active(true)
        .css_classes(["sidebar-toggle"])
        .build();
    toggle
        .bind_property("active", revealer, "reveal-child")
        .sync_create()
        .build();

    // Back / forward buttons, acting on whichever page is visible.
    let back = Button::from_icon_name("go-previous-symbolic");
    let forward = Button::from_icon_name("go-next-symbolic");
    back.set_sensitive(true);
    forward.set_sensitive(true);

    back.connect_clicked({
        let stack = stack.clone();
        move |_| {
            if let Some(webview) = current_webview(&stack) {
                webview.go_back();
            }
        }
    });
    forward.connect_clicked({
        let stack = stack.clone();
        move |_| {
            if let Some(webview) = current_webview(&stack) {
                webview.go_forward();
            }
        }
    });

    // Greys the buttons out when there is nowhere to go.
    let refresh = {
        let (stack, back, forward) = (stack.clone(), back.clone(), forward.clone());
        move || {
            let webview = current_webview(&stack);
            back.set_sensitive(webview.as_ref().is_some_and(|w| w.can_go_back()));
            forward.set_sensitive(webview.as_ref().is_some_and(|w| w.can_go_forward()));
        }
    };

    // Refresh when switching pages...
    stack.connect_visible_child_notify({
        let refresh = refresh.clone();
        move |_| refresh()
    });
    // ...and whenever any page navigates.
    for page in PAGES {
        if let Some(webview) = stack
            .child_by_name(page.id)
            .and_then(|w| w.downcast::<WebView>().ok())
        {
            webview.connect_load_changed({
                let refresh = refresh.clone();
                move |_, _| refresh()
            });
        }
    }

    // Group the two buttons visually.
    let nav = GtkBox::new(Orientation::Horizontal, 0);
    nav.add_css_class("linked");
    nav.append(&back);
    nav.append(&forward);

    // Search entry.
    let search = SearchEntry::builder()
        .placeholder_text("Search")
        .width_request(300)
        .build();

    search.connect_search_changed(|entry| {
        let query = entry.text();
        println!("searching for: {query}");
    });

    let notifications = Notifications::new();

    let accounts = Accounts::new(state, {
        let notifications = notifications.clone();
        move |acc| match acc {
            Some(name) => notifications.push(&format!("Switched to {name}")),
            None => notifications.push("Signed out"),
        }
    });

    let bar = HeaderBar::new();
    bar.pack_start(&toggle);
    bar.pack_start(&nav);
    bar.pack_end(&menu::build());
    bar.pack_end(&accounts.button);
    bar.pack_end(&notifications.button);
    //bar.set_title_widget(Some(&search));

    let clamp = adw::Clamp::builder()
        .maximum_size(300)
        .tightening_threshold(200)
        .hexpand(true)
        .child(&search)
        .build();
    bar.set_title_widget(Some(&clamp));

    Header {
        bar,
        search,
        notifications,
        toggle,
        accounts,
    }
}

// src/ui/pages.rs
// single source truth: the sidebar rows and the stack pages are both
// ge from this table in this order
use super::settings;
use super::state::Shared;
use gtk::prelude::*;
use gtk::{Stack, StackTransitionType};

use webkit6::prelude::*;
use webkit6::WebView;

use super::webview;

pub struct Page {
    pub id: &'static str,
    pub title: &'static str,
    pub icon: &'static str,
    pub url: &'static str,
}

/// Single source of truth: the sidebar rows and the stack pages are both
/// generated from this table, in this order.
pub const PAGES: &[Page] = &[
    Page {
        id: "home",
        title: "Home",
        icon: "go-home-symbolic",
        url: "https://google.com",
    },
    Page {
        id: "search",
        title: "Search",
        icon: "system-search-symbolic",
        url: "http://127.0.0.1/",
    },
    Page {
        id: "dashboard",
        title: "Dashboard",
        icon: "view-grid-symbolic",
        url: "https://hypr.land/news/26_lua/",
    },
    Page {
        id: "orders",
        title: "Orders",
        icon: "view-list-symbolic",
        url: "http://127.0.0.1/",
    },
    Page {
        id: "business",
        title: "Business",
        icon: "system-users-symbolic",
        url: "http://127.0.0.1/",
    },
    Page {
        id: "assets",
        title: "Assets ",
        icon: "package-x-generic-symbolic",
        url: "http://127.0.0.1/",
    },
    Page {
        id: "invoice",
        title: "Invoice",
        icon: "x-office-document-symbolic", // a document page, closest to an invoice
        // alternatives:
        //icon: "text-x-generic-symbolic", // plain text page
        //icon: "emblem-documents-symbolic", // documents emblem
        url: "http://127.0.0.1/",
    },
    Page {
        id: "library",
        title: "Library",
        icon: "folder-symbolic",
        url: "https://hypr.land/news/26_lua/",
    },
    Page {
        id: "inbox",
        title: "Inbox",
        icon: "mail-unread-symbolic",
        //icon: "mail-message-new-symbolic",
        url: "http://127.0.0.1/",
    },
    Page {
        id: "help",
        title: "Help",
        icon: "help-faq-symbolic",
        url: "https://google.com",
    },
    Page {
        id: "settings",
        title: "Settings",
        icon: "applications-system-symbolic",
        url: "https://hypr.land/news/26_lua/",
    },
    Page {
        id: "trash",
        title: "Trash",
        icon: "user-trash-symbolic",
        url: "https://hypr.land/news/26_lua/",
    },
];

/// Build the stack with one WebView per page. A page's URL is loaded
/// the first time it becomes visible.
pub fn build(state: &Shared) -> Stack {
    let stack = Stack::builder()
        .transition_type(StackTransitionType::Crossfade)
        .hexpand(true)
        .vexpand(true)
        .build();

    //stack.connect_visible_child_name_notify(load_visible_page);

    for page in PAGES {
        if page.id == "settings" {
            continue;
        }
        stack.add_titled(&webview::build(page.url), Some(page.id), page.title);
    }
    stack.add_titled(
        &settings::build(&stack, state),
        Some("settings"),
        "Settings",
    );

    stack.connect_visible_child_notify(|stack| {
        let Some(name) = stack.visible_child_name() else {
            return;
        };
        let Some(page) = PAGES.iter().find(|p| p.id == name.as_str()) else {
            return;
        };
        let Some(child) = stack.child_by_name(page.id) else {
            return;
        };
        if child.is::<WebView>() || page.id == "settings" {
            return;
        }
    });
    stack
}

fn load_visible_page(stack: &Stack) {
    let (Some(name), Some(child)) = (stack.visible_child_name(), stack.visible_child()) else {
        return;
    };
    let Ok(webview) = child.downcast::<WebView>() else {
        return;
    };
    if webview.uri().is_some() {
        return; // already loaded
    }
    if let Some(page) = PAGES.iter().find(|p| p.id == name.as_str()) {
        webview.load_uri(page.url);
    }
}

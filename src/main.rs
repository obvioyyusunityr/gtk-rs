use gtk::prelude::*;
use gtk::{
    gdk, glib, Application, ApplicationWindow, Box as GtkBox, CssProvider, HeaderBar, Label,
    Orientation, Revealer, SearchEntry, Stack, StackSidebar, ToggleButton,
};
use webkit6::prelude::*;
use webkit6::WebView;

const APP_ID: &str = "org.example.GtkSidebar";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    unsafe {
        std::env::set_var("GDK_SCALE", "1");
        std::env::set_var("GDK_DPI_SCALE", "1");
    }
    app.connect_activate(build_ui);
    app.run()
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_string(
        "
         button.sidebar-toggle:checked {background: #3584e4; color: white; }
         button.sidebar-toggle:active {background: #1c71d8; color: white; }
        ",
    );
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("no display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

/// Helper: a simple page with a centered label.
fn page(text: &str) -> Label {
    Label::builder()
        .label(text)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build()
}

const PAGES: &[(&str, &str, &str)] = &[
    ("home ", "Home", "https://hypr.land/news/26_lua/"),
    ("dashboard ", "Dashboard", "https://hypr.land/news/26_lua/"),
    ("library ", "Library", "https://hypr.land/news/26_lua/"),
    ("settings ", "Settings", "https://hypr.land/news/26_lua/"),
];
fn new_webview() -> WebView {
    let webview = WebView::new();
    enable_inspector(&webview);
    webview.set_hexpand(true);
    webview.set_vexpand(true);
    webview
}

fn build_ui(app: &Application) {
    load_css();
    // 1. The Stack holds the pages. Each page gets a name (id) and a title,
    //    and the title is what shows up in the sidebar.
    let stack = Stack::builder()
        .transition_type(gtk::StackTransitionType::Crossfade)
        .hexpand(true)
        .vexpand(true)
        .build();

    stack.connect_visible_child_name_notify(|stack| {
        let (Some(name), Some(child)) = (stack.visible_child_name(), stack.visible_child()) else {
            return;
        };
        if let Ok(webview) = child.downcast::<WebView>() {
            if webview.uri().is_none() {
                if let Some((_, _, url)) = PAGES.iter().find(|(id, _, _)| *id == name.as_str()) {
                    webview.load_uri(url);
                }
            }
        }
    });
    for (id, title, _) in PAGES {
        stack.add_titled(&new_webview(), Some(id), title);
    }
    // 2. The StackSidebar is a ready-made list that controls the Stack.
    let sidebar = StackSidebar::new();
    sidebar.set_stack(&stack);
    sidebar.set_width_request(200);

    // 3. Revealer wraps the sidebar so it can slide in and out.
    let revealer = Revealer::builder()
        .transition_type(gtk::RevealerTransitionType::SlideRight)
        .transition_duration(250)
        .reveal_child(true)
        .child(&sidebar)
        .build();

    let content = GtkBox::new(Orientation::Horizontal, 0);
    content.append(&revealer);
    content.append(&gtk::Separator::new(Orientation::Vertical));
    content.append(&stack);

    // 4. Toggle button in a header bar, bound to the revealer.
    let toggle = ToggleButton::builder()
        .icon_name("sidebar-show-symbolic")
        .active(true)
        .css_classes(["sidebar-toggle"])
        .build();
    toggle
        .bind_property("active", &revealer, "reveal-child")
        .sync_create()
        .build();

    let header = HeaderBar::new();
    header.pack_start(&toggle);

    //search bar
    let search = SearchEntry::builder()
        .placeholder_text("Search")
        .width_request(300)
        .build();
    header.set_title_widget(Some(&search));

    search.connect_search_changed(|entry| {
        let query = entry.text();
        println!("searching for: {query}");
    });

    let window = ApplicationWindow::builder()
        .application(app)
        .title("GTK Sidebar")
        .default_width(800)
        .default_height(500)
        //.titlebar(&header)
        .child(&content)
        .build();

    window.set_titlebar(Some(&header));
    search.set_key_capture_widget(Some(&window));
    window.present();
}
fn enable_inspector(webview: &webkit6::WebView) {
    // 1. Enable developer extras
    if let Some(settings) = WebViewExt::settings(webview) {
        settings.set_enable_developer_extras(true);
    }
}

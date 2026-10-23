//src/ui/style.rs
use gtk::{gdk, CssProvider};

const CSS: &str = "
    button.sidebar-toggle:checked { background: #3584e4; color: white; }
    button.sidebar-toggle:active  { background: #1c71d8; color: white; }
.notif-badge {
    background: #e01b24;
    color: white;
    border-radius: 999px;
    min-width: 16px;
    min-height: 16px;
    padding: 0 4px;
    font-size: 10px;
    font-weight: bold;
}
";

/// Load the app's CSS for the whole display.
pub fn load() {
    let provider = CssProvider::new();
    provider.load_from_string(CSS);
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("no display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

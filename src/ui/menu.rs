use gtk::prelude::*;
use gtk::{gio, Application, ApplicationWindow, MenuButton, Stack};
use webkit6::prelude::*;

use super::webview;

/// Build the hamburger button and its popover menu.
/// Each item triggers an action registered in `install_actions`.
pub fn build() -> MenuButton {
    let menu = gio::Menu::new();
    menu.append(Some("Reload Page"), Some("app.reload"));
    menu.append(Some("Preferences"), Some("app.preferences"));

    let section = gio::Menu::new();
    section.append(Some("About"), Some("app.about"));
    section.append(Some("Quit"), Some("app.quit"));
    menu.append_section(None, &section);

    MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .build()
}

/// Register the actions the menu items point at, plus keyboard shortcuts.
pub fn install_actions(app: &Application, window: &ApplicationWindow, stack: &Stack) {
    // Reload whichever page is currently visible.
    let reload = gio::ActionEntry::builder("reload")
        .activate({
            let stack = stack.clone();
            move |_app: &Application, _, _| {
                if let Some(webview) = webview::current_webview(&stack) {
                    webview.reload();
                }
            }
        })
        .build();

    let preferences = gio::ActionEntry::builder("preferences")
        .activate({
            let stack = stack.clone();
            move |_app: &Application, _, _| stack.set_visible_child_name("settings")
        })
        .build();

    let about = gio::ActionEntry::builder("about")
        .activate({
            let window = window.clone();
            move |_app: &Application, _, _| {
                let dialog = gtk::AboutDialog::builder()
                    .program_name("GTK Sidebar")
                    .version("0.1.0")
                    .modal(true)
                    .build();
                dialog.set_transient_for(Some(&window));
                dialog.present();
            }
        })
        .build();

    let quit = gio::ActionEntry::builder("quit")
        .activate({
            let window = window.clone();
            move |_app: &Application, _, _| window.close()
        })
        .build();

    app.add_action_entries([reload, preferences, about, quit]);
    app.set_accels_for_action("app.reload", &["<Ctrl>r"]);
    app.set_accels_for_action("app.quit", &["<Alt>q"]);
}

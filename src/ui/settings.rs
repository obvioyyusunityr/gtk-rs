use super::state::Shared;
use gtk::prelude::*;
use gtk::{
    Align, Box as GtkBox, Label, Orientation, ScrolledWindow, Separator, SpinButton, Stack, Switch,
    Widget,
};
use webkit6::prelude::*;
use webkit6::WebView;

use super::pages::PAGES;

/// Every WebView currently in the stack (the settings page is skipped
/// automatically because it isn't a WebView).
fn webviews(stack: &Stack) -> Vec<WebView> {
    PAGES
        .iter()
        .filter_map(|p| stack.child_by_name(p.id)?.downcast::<WebView>().ok())
        .collect()
}

/// One settings row: title + subtitle on the left, control on the right.
fn row(title: &str, subtitle: &str, control: &impl IsA<Widget>) -> GtkBox {
    let text = GtkBox::new(Orientation::Vertical, 2);
    text.set_hexpand(true);
    text.set_valign(Align::Center);
    text.append(&Label::builder().label(title).halign(Align::Start).build());
    text.append(
        &Label::builder()
            .label(subtitle)
            .halign(Align::Start)
            .css_classes(["dim-label"])
            .build(),
    );

    control.set_valign(Align::Center);

    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.set_margin_top(12);
    row.set_margin_bottom(12);
    row.append(&text);
    row.append(control);
    row
}

pub fn build(stack: &Stack, state: &Shared) -> ScrolledWindow {
    // --- Dark theme ---
    let apply_dark = {
        let stack = stack.clone();
        move |on: bool| {
            adw::StyleManager::default().set_color_scheme(if on {
                adw::ColorScheme::ForceDark
            } else {
                adw::ColorScheme::ForceLight
            });
            for wv in webviews(&stack) {
                super::webview::set_dark(&wv, on);
            }
        }
    };

    let manager = adw::StyleManager::default();
    let dark = Switch::new();
    dark.set_active(manager.is_dark());
    dark.connect_active_notify({
        let (state, apply) = (state.clone(), apply_dark.clone());
        move |sw| {
            state.borrow_mut().dark = Some(sw.is_active());
            apply(sw.is_active());
        }
    });
    let saved_dark = state.borrow().dark;
    if let Some(on) = saved_dark {
        dark.set_active(on);
        apply_dark(on); // also covers the case where the switch value didn't change
    }
    // --- Developer tools ---
    let devtools = Switch::builder().active(true).build();
    devtools.connect_active_notify({
        let (stack, state) = (stack.clone(), state.clone());
        move |sw| {
            state.borrow_mut().devtools = sw.is_active();
            for wv in webviews(&stack) {
                if let Some(s) = WebViewExt::settings(&wv) {
                    s.set_enable_developer_extras(sw.is_active());
                }
            }
        }
    });
    let saved = state.borrow().devtools;
    devtools.set_active(saved);

    // --- Zoom ---
    let zoom = SpinButton::with_range(0.5, 3.0, 0.1);
    zoom.set_digits(1);
    zoom.connect_value_changed({
        let (stack, state) = (stack.clone(), state.clone());
        move |sb| {
            state.borrow_mut().zoom = sb.value();
            for wv in webviews(&stack) {
                wv.set_zoom_level(sb.value());
            }
        }
    });
    let saved = state.borrow().zoom;
    zoom.set_value(saved);

    dark.connect_active_notify({
        let stack = stack.clone();
        move |sw| {
            adw::StyleManager::default().set_color_scheme(if sw.is_active() {
                adw::ColorScheme::ForceDark
            } else {
                adw::ColorScheme::ForceLight
            });
            for wv in webviews(&stack) {
                super::webview::set_dark(&wv, sw.is_active());
            }
        }
    });

    // --- Developer tools (right-click > Inspect Element) ---
    let devtools = Switch::builder().active(true).build();
    devtools.connect_active_notify({
        let stack = stack.clone();
        move |sw| {
            for wv in webviews(&stack) {
                if let Some(settings) = WebViewExt::settings(&wv) {
                    settings.set_enable_developer_extras(sw.is_active());
                }
            }
        }
    });

    // --- Zoom ---
    let zoom = SpinButton::with_range(0.5, 3.0, 0.1);
    zoom.set_digits(1);
    zoom.set_value(1.0);
    zoom.connect_value_changed({
        let stack = stack.clone();
        move |sb| {
            for wv in webviews(&stack) {
                wv.set_zoom_level(sb.value());
            }
        }
    });

    // --- Layout ---
    let heading = Label::builder()
        .label("Settings")
        .halign(Align::Start)
        .css_classes(["title-1"])
        .build();

    let page = GtkBox::new(Orientation::Vertical, 0);
    page.set_halign(Align::Center);
    page.set_width_request(520);
    page.set_margin_top(24);
    page.set_margin_bottom(24);
    page.append(&heading);
    page.append(&row("Dark theme", "Use the dark GTK theme", &dark));
    page.append(&Separator::new(Orientation::Horizontal));
    page.append(&row(
        "Developer tools",
        "Enable Inspect Element in pages",
        &devtools,
    ));
    page.append(&Separator::new(Orientation::Horizontal));
    page.append(&row("Page zoom", "Applies to every page", &zoom));

    ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .child(&page)
        .build()
}
//let settings

//src/ui/window.rs
use super::state;
use super::{header, menu, pages, sidebar, style};
use gtk::glib;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Orientation, Separator};

//assemble every  piece to main window
pub fn build(app: &Application) {
    style::load();

    let state = state::load();
    let stack = pages::build(&state);
    let revealer = sidebar::build(&stack);
    let header = header::build(&revealer, &stack, &state);
    header.notifications.push("Welcome to Duck Finance");
    header.notifications.push("Welcome to Duck Finance");
    header.accounts.button.set_tooltip_text(Some("Accounts"));

    let (w, h, maximized, sidebar, page) = {
        let s = state.borrow();
        (
            s.width,
            s.height,
            s.maximized,
            s.sidebar_visible,
            s.page.clone(),
        )
    };

    let card = GtkBox::new(Orientation::Horizontal, 0);
    card.add_css_class("content-card");
    card.set_overflow(gtk::Overflow::Hidden);
    card.set_hexpand(true);
    card.append(&stack);

    let content = GtkBox::new(Orientation::Horizontal, 0);
    content.add_css_class("app-bg");
    content.append(&revealer);
    //content.append(&Separator::new(Orientation::Vertical));
    content.append(&card);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Gtk sidebar")
        .default_width(w)
        .default_height(h)
        .child(&content)
        .build();
    if maximized {
        window.maximize();
    }

    header.toggle.set_active(sidebar);
    if stack.child_by_name(&page).is_some() {
        stack.set_visible_child_name(&page);
    }
    window.connect_close_request({
        let (state, stack, toggle) = (state.clone(), stack.clone(), header.toggle.clone());
        move |win| {
            let mut s = state.borrow_mut();
            s.maximized = win.is_maximized();
            if !s.maximized {
                s.width = win.width();
                s.height = win.height();
            }
            s.sidebar_visible = toggle.is_active();
            if let Some(name) = stack.visible_child_name() {
                s.page = name.to_string();
            }
            state::save(&s);
            glib::Propagation::Proceed
        }
    });
    let manager = adw::StyleManager::default();
    let apply = {
        let window = window.clone();
        move |m: &adw::StyleManager| {
            if m.is_dark() {
                window.add_css_class("dark");
            } else {
                window.remove_css_class("dark");
            }
        }
    };

    apply(&manager);
    println!(
        "dark={} classes={:?}",
        manager.is_dark(),
        window.css_classes()
    );
    manager.connect_dark_notify(move |m| apply(m));

    menu::install_actions(app, &window, &stack);

    window.set_titlebar(Some(&header.bar));
    header.search.set_key_capture_widget(Some(&window));
    window.present();
}

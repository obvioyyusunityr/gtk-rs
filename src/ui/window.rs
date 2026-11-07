//src/ui/window.rs
use super::state;
use super::{header, menu, pages, sidebar, style};
use gtk::glib;
use gtk::prelude::*;
use gtk::{gio, Application, ApplicationWindow, Box as GtkBox, Orientation, Separator};
use gtk::{EventControllerMotion, PropagationPhase, Revealer, RevealerTransitionType};
use std::{cell::Cell, rc::Rc};

//assemble every  piece to main window
pub fn build(app: &Application) {
    style::load();

    let state = state::load();
    let stack = pages::build(&state);
    let revealer = sidebar::build(&stack, &state);
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
    content.append(&Separator::new(Orientation::Vertical));
    content.append(&card);

    let header_visible = state.borrow().header_visible;
    header.bar.set_visible(header_visible);

    let header_rev = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideDown)
        .reveal_child(header_visible)
        .child(&header.bar)
        .build();

    let root = GtkBox::new(Orientation::Vertical, 0);
    root.append(&header_rev);
    root.append(&content);
    content.set_vexpand(true);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Gtk sidebar")
        .default_width(w)
        .default_height(h)
        //.child(&content)
        .child(&root)
        .build();
    if maximized {
        window.maximize();
    }

    header.toggle.set_active(sidebar);
    if stack.child_by_name(&page).is_some() {
        stack.set_visible_child_name(&page);
    }
    window.connect_close_request({
        let (state, stack, toggle) = (
            state.clone(),
            stack.clone(),
            header.toggle.clone(),
            //header.bar.clone(),
        );
        move |win| {
            let mut s = state.borrow_mut();
            //s.header_visible = header.bar.is_visible();
            s.maximized = win.is_maximized();
            if !s.maximized {
                s.width = win.width();
                s.height = win.height();
            }
            s.sidebar_visible = toggle.is_active();
            s.header_visible = header.bar.is_visible();
            if let Some(name) = stack.visible_child_name() {
                s.page = name.to_string();
            }
            state::save(&s);
            glib::Propagation::Proceed
        }
    });

    // true = header hides itself and appears on hover at the top edge
    let auto = Rc::new(Cell::new(!header_visible));

    // Reveal when the pointer touches the top edge. Capture phase so the
    // WebView can't swallow the event.
    let motion = EventControllerMotion::new();
    motion.set_propagation_phase(PropagationPhase::Capture);
    motion.connect_motion({
        let (rev, auto) = (header_rev.clone(), auto.clone());
        move |_, _, y| {
            if auto.get() && y < 6.0 {
                rev.set_reveal_child(true);
            }
        }
    });
    root.add_controller(motion);

    // Hide again shortly after the pointer leaves the header.
    let leave = EventControllerMotion::new();
    leave.connect_leave({
        let (rev, auto, search) = (header_rev.clone(), auto.clone(), header.search.clone());
        move |_| {
            let (rev, auto, search) = (rev.clone(), auto.clone(), search.clone());
            glib::timeout_add_local_once(std::time::Duration::from_millis(700), move || {
                if auto.get() && !search.has_focus() {
                    rev.set_reveal_child(false);
                }
            });
        }
    });
    let focus_search = gio::ActionEntry::builder("focus-search")
        .activate({
            let (search, header_rev) = (header.search.clone(), header_rev.clone());
            move |_app: &Application, _, _| {
                header_rev.set_reveal_child(true);
                search.grab_focus();
            }
        })
        .build();
    app.add_action_entries([focus_search]);
    app.set_accels_for_action("app.focus-search", &["<Ctrl>k", "<Ctrl>f"]);

    header_rev.add_controller(leave);
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
    //close_request
    apply(&manager);
    println!(
        "dark={} classes={:?}",
        manager.is_dark(),
        window.css_classes()
    );
    manager.connect_dark_notify(move |m| apply(m));

    menu::install_actions(app, &window, &stack);

    header.search.set_key_capture_widget(Some(&window));
    window.present();
}

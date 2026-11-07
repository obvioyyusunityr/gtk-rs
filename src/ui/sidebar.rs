use super::pages;
use super::pages::{Group, Page, PAGES};
use super::state::{CustomPage, Shared};
use gtk::prelude::*;
use gtk::{
    glib, Align, Box as GtkBox, Button, Entry, Image, Label, ListBox, ListBoxRow, Orientation,
    Revealer, RevealerTransitionType, SelectionMode, Separator, Stack, Window,
};
use std::rc::Rc;

// ---------- custom (user-made) pages ----------

fn fill_custom(list: &ListBox, stack: &Stack, state: &Shared) {
    while let Some(c) = list.first_child() {
        list.remove(&c);
    }
    let items = state.borrow().custom_pages.clone();
    list.set_visible(!items.is_empty());

    for cp in items {
        let content = GtkBox::new(Orientation::Horizontal, 12);
        content.set_margin_top(4);
        content.set_margin_bottom(4);
        content.set_margin_start(8);
        content.append(&Image::from_icon_name("web-browser-symbolic"));
        content.append(
            &Label::builder()
                .label(&cp.title)
                .hexpand(true)
                .xalign(0.0)
                .build(),
        );

        let del = Button::builder()
            .icon_name("user-trash-symbolic")
            .css_classes(["flat"])
            .tooltip_text("Delete page")
            .build();
        del.connect_clicked({
            let (list, stack, state, id) =
                (list.clone(), stack.clone(), state.clone(), cp.id.clone());
            move |_| {
                // Deferred: the rebuild destroys this button.
                let (list, stack, state, id) =
                    (list.clone(), stack.clone(), state.clone(), id.clone());
                glib::idle_add_local_once(move || delete_custom(&list, &stack, &state, &id));
            }
        });
        content.append(&del);

        let row = ListBoxRow::new();
        row.set_child(Some(&content));
        row.set_widget_name(&cp.id);
        list.append(&row);
    }
}

fn delete_custom(list: &ListBox, stack: &Stack, state: &Shared, id: &str) {
    state.borrow_mut().custom_pages.retain(|p| p.id != id);
    if stack.visible_child_name().as_deref() == Some(id) {
        stack.set_visible_child_name("home");
    }
    if let Some(child) = stack.child_by_name(id) {
        stack.remove(&child);
    }
    super::state::save(&state.borrow());
    fill_custom(list, stack, state);
}

fn add_custom(list: &ListBox, stack: &Stack, state: &Shared, title: String, url: String) {
    let cp = CustomPage {
        id: format!("custom-{}", glib::real_time()),
        title,
        url,
    };
    state.borrow_mut().custom_pages.push(cp.clone());
    super::state::save(&state.borrow());
    pages::add_custom_view(stack, &cp);
    fill_custom(list, stack, state);
    stack.set_visible_child_name(&cp.id);
}

/// Small modal dialog asking for a title and a URL.
fn ask_new_page(from: &impl IsA<gtk::Widget>, on_ok: impl Fn(String, String) + 'static) {
    let title = Entry::builder().placeholder_text("Title").build();
    let url = Entry::builder()
        .placeholder_text("https://example.com")
        .build();
    let cancel = Button::with_label("Cancel");
    let create = Button::with_label("Create");
    create.add_css_class("suggested-action");

    let buttons = GtkBox::new(Orientation::Horizontal, 8);
    buttons.set_halign(Align::End);
    buttons.append(&cancel);
    buttons.append(&create);

    let body = GtkBox::new(Orientation::Vertical, 10);
    body.append(&title);
    body.append(&url);
    body.append(&buttons);
    body.set_margin_top(16);
    body.set_margin_bottom(16);
    body.set_margin_start(16);
    body.set_margin_end(16);

    let mut win = Window::builder()
        .title("New page")
        .modal(true)
        .default_width(360)
        .child(&body);
    if let Some(parent) = from.root().and_then(|r| r.downcast::<Window>().ok()) {
        win = win.transient_for(&parent);
    }
    let win = win.build();

    let submit = Rc::new({
        let (win, title, url) = (win.clone(), title.clone(), url.clone());
        move || {
            let mut u = url.text().trim().to_string();
            if u.is_empty() {
                return;
            }
            if !u.starts_with("http://") && !u.starts_with("https://") {
                u = format!("https://{u}");
            }
            let t = title.text().trim().to_string();
            let t = if t.is_empty() { u.clone() } else { t };
            on_ok(t, u);
            win.close();
        }
    });
    create.connect_clicked({
        let s = submit.clone();
        move |_| s()
    });
    url.connect_activate({
        let s = submit.clone();
        move |_| s()
    });
    cancel.connect_clicked({
        let w = win.clone();
        move |_| w.close()
    });
    win.present();
}

// ---------- rows and lists ----------

fn make_row(page: &Page) -> ListBoxRow {
    let content = GtkBox::new(Orientation::Horizontal, 12);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.append(&Image::from_icon_name(page.icon));
    content.append(&Label::new(Some(page.title.trim())));

    let row = ListBoxRow::new();
    row.set_child(Some(&content));
    row.set_widget_name(page.id); // used to find the page, no index needed
    row
}

fn divider() -> Separator {
    let sep = Separator::new(Orientation::Horizontal);
    sep.set_margin_top(6);
    sep.set_margin_bottom(6);
    sep.set_margin_start(8);
    sep.set_margin_end(8);
    sep
}

fn make_list(group: Group) -> ListBox {
    let list = ListBox::builder()
        .selection_mode(SelectionMode::Single)
        .css_classes(["navigation-sidebar"])
        .build();

    for page in PAGES
        .iter()
        .filter(|p| p.group == group && p.parent.is_none())
    {
        let row = make_row(page);
        let has_children = PAGES.iter().any(|c| c.parent == Some(page.id));

        if has_children {
            if let Some(content) = row.child().and_then(|c| c.downcast::<GtkBox>().ok()) {
                let chevron = Image::from_icon_name("pan-end-symbolic");
                chevron.set_hexpand(true);
                chevron.set_halign(Align::End);
                chevron.set_margin_end(8);
                content.append(&chevron);
            }
        }

        list.append(&row);

        for child in PAGES.iter().filter(|c| c.parent == Some(page.id)) {
            let crow = make_row(child);
            crow.set_margin_start(20); // indent
            crow.set_visible(false); // collapsed by default
            list.append(&crow);
        }
    }

    // Clicking a parent toggles its children.
    list.connect_row_activated(|list, row| {
        let id = row.widget_name();
        toggle_children(list, id.as_str(), None);
    });

    list
}

/// Show/hide the children of `parent_id`. `None` flips the current state.
fn toggle_children(list: &ListBox, parent_id: &str, show: Option<bool>) {
    let mut target = show;
    let mut child = list.first_child();

    while let Some(w) = child {
        let name = w.widget_name();
        if PAGES
            .iter()
            .any(|p| p.id == name.as_str() && p.parent == Some(parent_id))
        {
            let v = target.unwrap_or(!w.is_visible());
            target = Some(v);
            w.set_visible(v);
        }
        child = w.next_sibling();
    }
}

/// Select the row named `name` in whichever list contains it.
fn select_by_name(lists: &[ListBox], name: &str) {
    let parent = PAGES.iter().find(|p| p.id == name).and_then(|p| p.parent);

    for list in lists {
        if let Some(parent) = parent {
            toggle_children(list, parent, Some(true));
        }

        let mut child = list.first_child();
        while let Some(w) = child {
            if w.widget_name() == name {
                if let Ok(row) = w.clone().downcast::<ListBoxRow>() {
                    list.select_row(Some(&row));
                }
                return;
            }
            child = w.next_sibling();
        }
    }
}

// ---------- sidebar filter (used by the header search, ">" mode) ----------

/// Show only rows whose title contains `query`. Empty query restores the
/// normal view (children collapsed, except a selected one).
pub fn filter(root: &Revealer, query: &str) {
    let q = query.trim().to_lowercase();
    if let Some(child) = root.child() {
        walk(&child, &q);
    }
}

fn walk(w: &gtk::Widget, q: &str) {
    if let Some(row) = w.downcast_ref::<ListBoxRow>() {
        let name = row.widget_name();
        if let Some(page) = PAGES.iter().find(|p| p.id == name.as_str()) {
            let show = if q.is_empty() {
                page.parent.is_none() || row.is_selected()
            } else {
                page.title.to_lowercase().contains(q)
            };
            row.set_visible(show);
        }
        return; // don't descend into the row's contents
    }
    let mut c = w.first_child();
    while let Some(x) = c {
        walk(&x, q);
        c = x.next_sibling();
    }
}

// ---------- the sidebar ----------

pub fn build(stack: &Stack, state: &Shared) -> Revealer {
    let title = Label::builder()
        .label("Duck Finance Management")
        .halign(Align::Start)
        .margin_top(14)
        .margin_bottom(10)
        .margin_start(12)
        .css_classes(["app-title"])
        .build();

    let column = GtkBox::new(Orientation::Vertical, 0);
    column.set_width_request(200);
    column.add_css_class("sidebar-col");
    column.append(&title);

    // Built-in groups.
    let groups = [Group::Top, Group::Overview, Group::Work];
    let body_lists: Vec<ListBox> = groups.iter().map(|g| make_list(*g)).collect();
    let footer = make_list(Group::Footer);

    // User-made pages.
    let custom = ListBox::builder()
        .selection_mode(SelectionMode::Single)
        .css_classes(["navigation-sidebar"])
        .build();
    fill_custom(&custom, stack, state);

    // Every list, so selection and syncing cover all of them.
    let mut lists: Vec<ListBox> = body_lists.clone();
    lists.push(custom.clone());
    lists.push(footer.clone());

    // Built-in groups, with a line between each.
    for (i, list) in body_lists.iter().enumerate() {
        if i > 0 {
            column.append(&divider());
        }
        column.append(list);
    }

    // Custom pages: the divider follows the list's visibility (hidden while empty).
    let custom_div = divider();
    custom
        .bind_property("visible", &custom_div, "visible")
        .sync_create()
        .build();
    column.append(&custom_div);
    column.append(&custom);

    let add = Button::builder()
        .label("New page")
        .icon_name("list-add-symbolic")
        .css_classes(["flat"])
        .margin_start(4)
        .margin_end(4)
        .build();
    add.connect_clicked({
        let (custom, stack, state) = (custom.clone(), stack.clone(), state.clone());
        move |b| {
            let (custom, stack, state) = (custom.clone(), stack.clone(), state.clone());
            ask_new_page(b, move |t, u| add_custom(&custom, &stack, &state, t, u));
        }
    });
    column.append(&add);

    // Spacer pushes the footer to the bottom.
    let spacer = GtkBox::new(Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    column.append(&spacer);
    column.append(&divider());
    column.append(&footer);

    // Selecting a row: open its page and clear selection in the other lists.
    for list in &lists {
        list.connect_row_selected({
            let (stack, lists) = (stack.clone(), lists.clone());
            move |this, row| {
                let Some(row) = row else { return };
                for other in lists.iter().filter(|l| *l != this) {
                    other.unselect_all();
                }
                stack.set_visible_child_name(&row.widget_name());
            }
        });
    }

    // Page changed from elsewhere (menu, search, restored state): follow it.
    stack.connect_visible_child_name_notify({
        let lists = lists.clone();
        move |stack| {
            if let Some(name) = stack.visible_child_name() {
                select_by_name(&lists, &name);
            }
        }
    });

    // Initial selection.
    let start = stack
        .visible_child_name()
        .map(|n| n.to_string())
        .unwrap_or_else(|| "home".into());
    select_by_name(&lists, &start);

    Revealer::builder()
        .transition_type(RevealerTransitionType::SlideRight)
        .transition_duration(250)
        .reveal_child(true)
        .child(&column)
        .build()
}

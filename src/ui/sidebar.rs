use super::pages::{Group, Page, PAGES};
use gtk::prelude::*;
use gtk::{
    Align, Box as GtkBox, Image, Label, ListBox, ListBoxRow, Orientation, Revealer,
    RevealerTransitionType, SelectionMode, Separator, Stack,
};

fn make_row(page: &Page) -> ListBoxRow {
    let content = GtkBox::new(Orientation::Horizontal, 12);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.append(&Image::from_icon_name(page.icon));
    content.append(&Label::new(Some(page.title)));

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

pub fn build(stack: &Stack) -> Revealer {
    let title = Label::builder()
        .label("GTK Sidebar")
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

    // One list per group (footer is built separately, after the spacer).
    let groups = [Group::Top, Group::Overview, Group::Work];
    let mut lists: Vec<ListBox> = groups.iter().map(|g| make_list(*g)).collect();
    let footer = make_list(Group::Footer);
    lists.push(footer.clone());

    let body: Vec<&ListBox> = lists.iter().filter(|l| *l != &footer).collect();

    for (i, list) in body.iter().enumerate() {
        if i > 0 {
            column.append(&divider()); // line between groups
        }
        column.append(*list);
    }

    // Spacer pushes the footer to the bottom.
    let spacer = GtkBox::new(Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    column.append(&spacer);
    column.append(&divider()); // line above the footer
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

    // Page changed from elsewhere (menu, restored state): follow it.
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

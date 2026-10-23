//src/ui/sidebar.rs
use gtk::prelude::*;
use gtk::{
    Align, Box as GtkBox, Image, Label, ListBox, Orientation, Revealer, RevealerTransitionType,
    SelectionMode, Stack,
};

//use crate::PAGES;
use super::pages::PAGES;

//icon sidebar, wrapped in revealer so it slide in and out
//select row and switch stack to the matching page
pub fn build(stack: &Stack) -> Revealer {
    // Program name at the top of the sidebar.
    let title = Label::builder()
        .label("Duck Finance") // your program name
        .halign(Align::Start)
        .margin_top(14)
        .margin_bottom(10)
        .margin_start(12)
        .css_classes(["title-3"])
        .build();

    let list = ListBox::builder()
        .selection_mode(SelectionMode::Single)
        .css_classes(["navigation-sidebar"])
        .width_request(200)
        .vexpand(true)
        .build();

    let column = GtkBox::new(Orientation::Vertical, 0);
    column.set_width_request(200);
    column.append(&title);
    column.append(&list);

    for page in PAGES {
        let row = GtkBox::new(Orientation::Horizontal, 12);
        row.set_margin_top(8);
        row.set_margin_bottom(8);
        row.set_margin_start(8);
        row.append(&Image::from_icon_name(page.icon));
        row.append(&Label::new(Some(page.title)));
        list.append(&row);
    }
    list.connect_row_selected({
        let stack = stack.clone();
        move |_, row| {
            if let Some(page) = row.and_then(|r| PAGES.get(r.index() as usize)) {
                stack.set_visible_child_name(page.id);
            }
        }
    });

    stack.connect_visible_child_notify({
        let list = list.clone();
        move |stack| {
            if let Some(name) = stack.visible_child_name() {
                if let Some(i) = PAGES.iter().position(|p| p.id == name.as_str()) {
                    list.select_row(list.row_at_index(i as i32).as_ref());
                }
            }
        }
    });
    //first row highlighted
    list.select_row(list.row_at_index(0).as_ref());

    Revealer::builder()
        .transition_type(RevealerTransitionType::SlideRight)
        .transition_duration(250)
        .reveal_child(true)
        //.child(&list)
        .child(&column)
        .build()
}

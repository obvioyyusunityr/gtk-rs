use gtk::prelude::*;
use gtk::{
    Align, Box as GtkBox, Button, Image, Label, ListBox, MenuButton, Orientation, Overlay, Popover,
    ScrolledWindow, SelectionMode,
};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
pub struct Notifications {
    pub button: MenuButton,
    list: ListBox,
    badge: Label,
    empty: Label,
    unread: Rc<Cell<u32>>,
}

impl Notifications {
    pub fn new() -> Self {
        // Bell icon with a badge overlaid on the top-right corner.
        let badge = Label::builder()
            .css_classes(["notif-badge"])
            .halign(Align::End)
            .valign(Align::Start)
            .visible(false)
            .build();
        let overlay = Overlay::new();
        overlay.set_child(Some(&Image::from_icon_name(
            "preferences-system-notifications-symbolic",
        )));
        overlay.add_overlay(&badge);

        // Popover contents: title row, empty-state label, scrolling list.
        let list = ListBox::builder()
            .selection_mode(SelectionMode::None)
            .build();
        let empty = Label::builder()
            .label("No notifications")
            .css_classes(["dim-label"])
            .margin_top(24)
            .margin_bottom(24)
            .build();
        let clear = Button::with_label("Clear all");
        clear.add_css_class("flat");

        let title = Label::builder()
            .label("Notifications")
            .halign(Align::Start)
            .hexpand(true)
            .css_classes(["heading"])
            .build();
        let top = GtkBox::new(Orientation::Horizontal, 6);
        top.append(&title);
        top.append(&clear);

        let scroller = ScrolledWindow::builder()
            .child(&list)
            .min_content_height(60)
            .max_content_height(320)
            .propagate_natural_height(true)
            .build();

        let content = GtkBox::new(Orientation::Vertical, 6);
        content.set_margin_top(10);
        content.set_margin_bottom(10);
        content.set_margin_start(10);
        content.set_margin_end(10);
        content.set_width_request(300);
        content.append(&top);
        content.append(&empty);
        content.append(&scroller);

        let popover = Popover::new();
        popover.set_child(Some(&content));

        let button = MenuButton::builder()
            .child(&overlay)
            .popover(&popover)
            .build();

        let this = Self {
            button,
            list,
            badge,
            empty,
            unread: Rc::new(Cell::new(0)),
        };

        // Opening the popover marks everything as read.
        popover.connect_show({
            let this = this.clone();
            move |_| {
                this.unread.set(0);
                this.refresh();
            }
        });
        clear.connect_clicked({
            let this = this.clone();
            move |_| this.clear()
        });

        this.refresh();
        this
    }

    /// Add a notification at the top of the list.
    pub fn push(&self, text: &str) {
        let label = Label::builder()
            .label(text)
            .wrap(true)
            .xalign(0.0)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();
        self.list.prepend(&label);
        self.unread.set(self.unread.get() + 1);
        self.refresh();
    }

    pub fn clear(&self) {
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        self.unread.set(0);
        self.refresh();
    }

    fn refresh(&self) {
        let n = self.unread.get();
        self.badge.set_visible(n > 0);
        self.badge
            .set_label(&if n > 9 { "9+".into() } else { n.to_string() });
        self.empty.set_visible(self.list.first_child().is_none());
    }
}

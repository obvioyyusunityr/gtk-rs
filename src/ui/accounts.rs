use adw::prelude::*;
use gtk::{
    Align, Box as GtkBox, Button, Entry, Image, Label, ListBox, MenuButton, Orientation, Popover,
    Revealer, SelectionMode, Separator,
};
use std::rc::Rc;

use super::state::Shared;

#[derive(Clone)]
pub struct Accounts {
    pub button: MenuButton,
    state: Shared,
    popover: Popover,
    list: ListBox,
    small_avatar: adw::Avatar,
    big_avatar: adw::Avatar,
    name: Label,
    sign_out: Button,
    on_change: Rc<dyn Fn(Option<String>)>,
}

impl Accounts {
    /// `on_change` runs whenever the active account changes (None = signed out).
    pub fn new(state: &Shared, on_change: impl Fn(Option<String>) + 'static) -> Self {
        let small_avatar = adw::Avatar::new(24, None, true);
        let big_avatar = adw::Avatar::new(48, None, true);

        let name = Label::builder()
            .halign(Align::Start)
            .css_classes(["heading"])
            .build();
        let top = GtkBox::new(Orientation::Horizontal, 12);
        top.append(&big_avatar);
        top.append(&name);

        let list = ListBox::builder()
            .selection_mode(SelectionMode::None)
            .css_classes(["navigation-sidebar"])
            .build();

        // "Add account" reveals an entry inline, which avoids a separate dialog.
        let entry = Entry::builder().placeholder_text("Account name").build();
        let entry_revealer = Revealer::builder().child(&entry).build();
        let add_btn = Button::with_label("Add account");
        add_btn.add_css_class("flat");
        let sign_out = Button::with_label("Sign out");
        sign_out.add_css_class("flat");

        let content = GtkBox::new(Orientation::Vertical, 6);
        for m in ["top", "bottom", "start", "end"] {
            match m {
                "top" => content.set_margin_top(10),
                "bottom" => content.set_margin_bottom(10),
                "start" => content.set_margin_start(10),
                _ => content.set_margin_end(10),
            }
        }
        content.set_width_request(260);
        content.append(&top);
        content.append(&Separator::new(Orientation::Horizontal));
        content.append(&list);
        content.append(&Separator::new(Orientation::Horizontal));
        content.append(&entry_revealer);
        content.append(&add_btn);
        content.append(&sign_out);

        let popover = Popover::new();
        popover.set_child(Some(&content));
        let button = MenuButton::builder()
            .child(&small_avatar)
            .popover(&popover)
            .build();

        let this = Self {
            button,
            state: state.clone(),
            popover,
            list,
            small_avatar,
            big_avatar,
            name,
            sign_out: sign_out.clone(),
            on_change: Rc::new(on_change),
        };

        add_btn.connect_clicked({
            let (r, e) = (entry_revealer.clone(), entry.clone());
            move |_| {
                r.set_reveal_child(true);
                e.grab_focus();
            }
        });
        entry.connect_activate({
            let (this, r) = (this.clone(), entry_revealer.clone());
            move |e| {
                this.add(e.text().trim());
                e.set_text("");
                r.set_reveal_child(false);
            }
        });
        sign_out.connect_clicked({
            let this = this.clone();
            move |_| this.set_active(None)
        });
        this.popover.connect_closed({
            let r = entry_revealer.clone();
            move |_| r.set_reveal_child(false)
        });

        this.refresh();
        this
    }

    fn add(&self, name: &str) {
        if name.is_empty() {
            return;
        }
        {
            let mut s = self.state.borrow_mut();
            if !s.accounts.iter().any(|a| a == name) {
                s.accounts.push(name.to_string());
            }
        }
        self.set_active(Some(name.to_string()));
    }

    fn remove(&self, name: &str) {
        let was_active = {
            let mut s = self.state.borrow_mut();
            s.accounts.retain(|a| a != name);
            s.account.as_deref() == Some(name)
        };
        if was_active {
            self.set_active(None);
        } else {
            self.refresh();
        }
    }

    fn set_active(&self, name: Option<String>) {
        self.state.borrow_mut().account = name.clone();
        self.refresh();
        self.popover.popdown();
        (self.on_change)(name);
    }

    fn refresh(&self) {
        let (accounts, active) = {
            let s = self.state.borrow();
            (s.accounts.clone(), s.account.clone())
        };

        self.name
            .set_label(active.as_deref().unwrap_or("Signed out"));
        self.small_avatar.set_text(active.as_deref());
        self.big_avatar.set_text(active.as_deref());
        self.sign_out.set_sensitive(active.is_some());

        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        for acc in accounts {
            let label_box = GtkBox::new(Orientation::Horizontal, 8);
            label_box.append(&adw::Avatar::new(24, Some(&acc), true));
            label_box.append(
                &Label::builder()
                    .label(&acc)
                    .hexpand(true)
                    .halign(Align::Start)
                    .build(),
            );
            if active.as_deref() == Some(acc.as_str()) {
                label_box.append(&Image::from_icon_name("object-select-symbolic"));
            }

            let pick = Button::builder()
                .child(&label_box)
                .hexpand(true)
                .css_classes(["flat"])
                .build();
            pick.connect_clicked({
                let (this, acc) = (self.clone(), acc.clone());
                move |_| this.set_active(Some(acc.clone()))
            });

            let del = Button::builder()
                .icon_name("user-trash-symbolic")
                .css_classes(["flat"])
                .tooltip_text("Remove account")
                .build();
            del.connect_clicked({
                let (this, acc) = (self.clone(), acc.clone());
                move |_| this.remove(&acc)
            });

            let row = GtkBox::new(Orientation::Horizontal, 0);
            row.append(&pick);
            row.append(&del);
            self.list.append(&row);
        }
    }
}

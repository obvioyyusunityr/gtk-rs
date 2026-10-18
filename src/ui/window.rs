use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Orientation, Separator};

use super::{header, pages, sidebar, style};

//assemble every  piece to main window
pub fn build(app: &Application) {
    style::load();

    let stack = pages::build();
    let revealer = sidebar::build(&stack);
    let header = header::build(&revealer);

    let content = GtkBox::new(Orientation::Horizontal, 0);
    content.append(&revealer);
    content.append(&Separator::new(Orientation::Vertical));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Gtk sidebar")
        .default_width(800)
        .default_height(500)
        .child(&content)
        .build();

    window.set_titlebar(Some(&header.bar));
    header.search.set_key_capture_widget(Some(&window));
    window.present();
}

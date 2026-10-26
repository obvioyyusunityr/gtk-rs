//sidebar-gtkrs/src/main.rs
use gtk::glib;
use gtk::prelude::*;

mod ui;

const APP_ID: &str = "org.example.GtkSidebar";

fn main() -> glib::ExitCode {
    unsafe {
        std::env::set_var("GDK_SCALE", "1");
        std::env::set_var("GDK_DPI_SCALE", "1");
    }
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| ui::window::build(app.upcast_ref::<gtk::Application>()));
    app.run()
    // stack
}

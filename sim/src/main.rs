mod ui;

use gtk::{glib, prelude::*};

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("ru.example.stacksim")
        .build();

    app.connect_activate(ui::build_ui);

    app.run()
}
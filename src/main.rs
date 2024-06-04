use adw::prelude::*;
use comhra::{config::APP_ID, window::build_ui};
use gtk::glib;

#[tokio::main]
async fn main() -> glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run()
}

mod config;
mod phi_chat;
mod window;

use crate::config::APP_ID;

use gtk::prelude::*;
use gtk::{glib, Application};
use window::build_ui;

#[tokio::main]
async fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run()
}

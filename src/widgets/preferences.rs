use adw::prelude::*;
use gtk::glib;

use super::model_manager::ModelManagerWidget;

pub struct PreferencesWidget {
    pub dialog: gtk::Dialog,
}

impl PreferencesWidget {
    pub fn new(main_context: glib::MainContext) -> Self {
        let model_manager_widget = ModelManagerWidget::new(main_context);
        let dialog = gtk::Dialog::builder()
            .title("Preferences")
            .default_height(300)
            .default_width(300)
            .child(&model_manager_widget.main_box)
            .build();
        Self { dialog }
    }
}

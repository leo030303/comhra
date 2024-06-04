use adw::prelude::*;

use gtk::glib;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
/*
- Prompt entry text field
- Collapsible RAG search query text field
- Submit prompt/stop generating button
- Open file button
*/
pub struct PromptEntryWidget {
    pub main_box: gtk::Box,
    pub prompt_entry: gtk::Entry,
    pub prompt_entry_buffer: gtk::EntryBuffer,
    pub submit_button: gtk::Button,
    pub submit_button_signal_id: Arc<Mutex<Option<glib::SignalHandlerId>>>,
    pub prompt_entry_signal_id: Arc<Mutex<Option<glib::SignalHandlerId>>>,
    pub selected_file: Arc<Mutex<Option<PathBuf>>>,
}
impl PromptEntryWidget {
    pub fn new() -> Self {
        // Create state holders
        let prompt_entry_signal_id: Arc<Mutex<Option<glib::SignalHandlerId>>> =
            Arc::new(Mutex::new(None));
        let prompt_button_signal_id: Arc<Mutex<Option<glib::SignalHandlerId>>> =
            Arc::new(Mutex::new(None));
        let selected_file = Arc::new(Mutex::new(None));

        // Create the widgets
        let prompt_entry_buffer = gtk::EntryBuffer::builder().text("").build();
        let prompt_entry = gtk::Entry::builder()
            .buffer(&prompt_entry_buffer)
            .hexpand(true)
            .placeholder_text("Enter a prompt")
            .build();
        let prompt_button = gtk::Button::builder()
            .icon_name("emblem-ok-symbolic")
            .tooltip_text("Send prompt")
            .width_request(50)
            .build();
        let add_file_button = gtk::Button::builder()
            .tooltip_text("Add file")
            .icon_name("document-open-symbolic")
            .build();
        let prompt_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(4)
            .build();

        let file_chooser = Self::create_file_chooser(Arc::clone(&selected_file));

        add_file_button.connect_clicked(move |_| {
            file_chooser.show();
        });

        prompt_box.append(&prompt_entry);
        prompt_box.append(&add_file_button);
        prompt_box.append(&prompt_button);

        PromptEntryWidget {
            prompt_entry,
            prompt_entry_buffer,
            submit_button: prompt_button,
            submit_button_signal_id: prompt_button_signal_id,
            prompt_entry_signal_id,
            main_box: prompt_box,
            selected_file,
        }
    }

    fn create_file_chooser(selected_file: Arc<Mutex<Option<PathBuf>>>) -> gtk::FileChooserNative {
        let file_filter = gtk::FileFilter::new();
        file_filter.add_mime_type("image/png");
        file_filter.add_mime_type("image/jpeg");
        file_filter.add_mime_type("text/*");
        file_filter.add_mime_type("application/pdf");

        let file_chooser = gtk::FileChooserNative::builder()
            .title("Select a file")
            .action(gtk::FileChooserAction::Open)
            .filter(&file_filter)
            .build();
        file_chooser.connect_response(move |file_chooser, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file) = file_chooser.file() {
                    if let Some(name) = file.path() {
                        if let Some(name_str) = name.to_str() {
                            println!("Selected file: {}", name_str);
                        }
                        *selected_file.lock().unwrap() = Some(name);
                    }
                }
            }
            file_chooser.hide();
        });
        file_chooser
    }
}

impl Default for PromptEntryWidget {
    fn default() -> Self {
        Self::new()
    }
}

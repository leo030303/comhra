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
    pub prompt_button_send_icon: gtk::Image, // remove todo
    pub selected_file: Arc<Mutex<Option<PathBuf>>>,
}
impl PromptEntryWidget {
    pub fn new() -> Self {
        let prompt_entry_signal_id: Arc<Mutex<Option<glib::SignalHandlerId>>> =
            Arc::new(Mutex::new(None));
        let prompt_button_signal_id: Arc<Mutex<Option<glib::SignalHandlerId>>> =
            Arc::new(Mutex::new(None));

        let selected_file = Arc::new(Mutex::new(None));

        // Create the prompt entry section
        let prompt_entry_buffer = gtk::EntryBuffer::builder().text("").build();
        let prompt_entry = gtk::Entry::builder()
            .buffer(&prompt_entry_buffer)
            .hexpand(true)
            .placeholder_text("Enter a prompt")
            .build();
        let prompt_button_send_icon = gtk::Image::from_icon_name("emblem-ok-symbolic");
        let prompt_button = gtk::Button::builder()
            .tooltip_text("Send prompt")
            .width_request(50)
            .child(&prompt_button_send_icon)
            .build();
        let add_file_button = gtk::Button::builder()
            .icon_name("document-open-symbolic")
            .build();

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
        {
            let selected_file = Arc::clone(&selected_file);
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
        }
        add_file_button.connect_clicked(move |_| {
            file_chooser.show();
        });
        let prompt_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(4)
            .build();
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
            prompt_button_send_icon,
            selected_file,
        }
    }
}

impl Default for PromptEntryWidget {
    fn default() -> Self {
        Self::new()
    }
}

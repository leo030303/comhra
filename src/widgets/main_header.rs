use crate::models::api_model::ApiModel;
use crate::models::ollama_model::OllamaModel;
use crate::models::{CoreLLM, SavedModel};
use crate::utils::get_filenames_from_folder;
use crate::RagSource;
use adw::prelude::*;

use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use super::preferences::PreferencesWidget;

#[derive(Clone, Debug)]
pub struct RagDropdown {
    pub rag_options: Vec<RagSource>,
    pub dropdown: gtk::DropDown,
}

impl RagDropdown {
    pub fn new() -> Self {
        let rag_source_scripts = get_filenames_from_folder(PathBuf::from("./rag_sources"));
        let mut rag_options = vec![RagSource::NoRag];
        rag_source_scripts
            .iter()
            .for_each(|source| rag_options.push(RagSource::BashScript(source.clone())));
        let option_list =
            gtk::StringList::from_iter(rag_options.iter().map(|rag_source| rag_source.name()));

        let dropdown = gtk::DropDown::builder().model(&option_list).build();

        dropdown.connect_selected_notify(move |drop_down| {
            let selected_index = drop_down.selected();

            let selected_text = option_list.string(selected_index).unwrap();
            println!("Selected: {}", selected_text);
        });
        Self {
            rag_options,
            dropdown,
        }
    }
}

impl Default for RagDropdown {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct ModelDropdown {
    pub model_list: Vec<SavedModel>,
    pub dropdown: gtk::DropDown,
}

impl ModelDropdown {
    pub fn new(chat_model: Arc<Mutex<Box<dyn CoreLLM>>>) -> Self {
        let model_list = SavedModel::get_all();

        let option_list = gtk::StringList::from_iter(model_list.iter().map(|local_model| {
            format!("{0}: {1}", local_model.model_type.name(), &local_model.name)
        }));

        let dropdown = gtk::DropDown::builder().model(&option_list).build();

        let model_list_for_closure = model_list.clone();
        dropdown.connect_selected_notify(Self::dropdown_on_selected(
            option_list,
            model_list_for_closure,
            chat_model,
        ));
        Self {
            dropdown,
            model_list,
        }
    }

    fn dropdown_on_selected(
        option_list: gtk::StringList,
        model_list_for_closure: Vec<SavedModel>,
        chat_model: Arc<Mutex<Box<dyn CoreLLM>>>,
    ) -> impl Fn(&gtk::DropDown) {
        move |drop_down| {
            let selected_index = drop_down.selected();
            let selected_text = option_list.string(selected_index).unwrap();
            let saved_model = model_list_for_closure.get(selected_index as usize).unwrap();

            let current_conversation = chat_model.lock().unwrap().get_conversation();
            *chat_model.lock().unwrap() = match &saved_model.model_type {
                crate::models::ModelType::Ollama => {
                    Box::new(OllamaModel::new_from_conversation_and_model_name(
                        current_conversation,
                        saved_model.name.clone(),
                    ))
                }
                crate::models::ModelType::Api(api_key, api_type) => {
                    Box::new(ApiModel::new_from_conversation_and_model_name(
                        current_conversation,
                        saved_model.name.clone(),
                        api_key.clone(),
                        api_type.clone(),
                    ))
                }
            };
            println!("Selected: {}", selected_text);
        }
    }
}

/*
- Other dropdown menu
    - Open about page button
    - Open settings button
    - Show onboarding button
    - Show keyboard shortcuts button
    - Donate button
*/
pub struct HeaderWidget {
    pub main_bar: gtk::HeaderBar,
    pub rag_dropdown: RagDropdown,
    pub sidebar_toggle_button: gtk::ToggleButton,
}

impl HeaderWidget {
    pub fn new(
        sidebar_widget: gtk::Box,
        main_content_box: gtk::Box,
        conversation_file_option_sender: Sender<Option<PathBuf>>,
        chat_model: Arc<Mutex<Box<dyn CoreLLM>>>,
    ) -> Self {
        // Create new chat button, this restarts the conversation, saves the current one, and clears the conversation list
        let new_chat_button = Self::create_new_chat_button(conversation_file_option_sender);
        let sidebar_toggle_button =
            Self::create_sidebar_toggle_button(sidebar_widget, main_content_box);

        let menu_popover = Self::create_menu_popover();
        let menu_button = gtk::Button::builder()
            .icon_name("open-menu-symbolic")
            .build();
        menu_popover.set_parent(&menu_button);
        menu_button.connect_clicked(move |_| {
            menu_popover.popup();
        });

        let rag_dropdown = RagDropdown::new();
        let model_dropdown = ModelDropdown::new(chat_model);

        let main_bar = gtk::HeaderBar::builder().show_title_buttons(true).build();
        main_bar.pack_start(&sidebar_toggle_button);
        main_bar.pack_start(&new_chat_button);
        main_bar.pack_start(&rag_dropdown.dropdown);
        main_bar.pack_end(&menu_button);
        main_bar.pack_end(&model_dropdown.dropdown);

        Self {
            main_bar,
            rag_dropdown,
            sidebar_toggle_button,
        }
    }

    fn create_new_chat_button(
        conversation_file_option_sender: Sender<Option<PathBuf>>,
    ) -> gtk::Button {
        let new_chat_button = gtk::Button::builder()
            .tooltip_text("New conversation")
            .icon_name("tab-new-symbolic")
            .build();
        new_chat_button.connect_clicked(move |_| {
            conversation_file_option_sender.send(None).unwrap();
        });
        new_chat_button
    }

    fn create_sidebar_toggle_button(
        sidebar_widget: gtk::Box,
        main_content_box: gtk::Box,
    ) -> gtk::ToggleButton {
        let sidebar_toggle_button = gtk::ToggleButton::builder()
            .tooltip_text("Toggle sidebar")
            .icon_name("view-dual-symbolic")
            .build();

        sidebar_toggle_button.connect_toggled(move |button| {
            if button.is_active() {
                sidebar_widget.show();
                main_content_box.hide();
            } else {
                sidebar_widget.hide();
                main_content_box.show();
            }
        });
        sidebar_toggle_button
    }

    fn create_about_window() -> adw::AboutWindow {
        adw::AboutWindow::builder()
            .application_name("Comhrá")
            .developer_name("Leo Ring")
            .website("https://github.com/leo030303")
            .build()
    }

    fn create_menu_popover() -> gtk::Popover {
        let menu_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .homogeneous(true)
            .build();

        let preferences_button = gtk::Button::builder().label("Preferences").build();
        let onboarding_button = gtk::Button::builder().label("Onboarding").build();
        let shortcuts_button = gtk::Button::builder().label("Shortcuts").build();
        let about_button = gtk::Button::builder().label("About").build();
        let donate_button = gtk::Button::builder().label("Buy me a coffee").build();

        donate_button.connect_clicked(|_| {
            if let Err(error) = open::that("https://github.com/leo030303") {
                println!("Error opening web browser: {}", error);
            };
        });

        let about_dialog = Self::create_about_window();
        about_button.connect_clicked(move |_| {
            about_dialog.show();
            about_dialog.grab_focus();
        });

        let preferences_widget = PreferencesWidget::new();
        preferences_button.connect_clicked(move |_| {
            preferences_widget.dialog.show();
            preferences_widget.dialog.grab_focus();
        });

        menu_box.append(&preferences_button);
        menu_box.append(&onboarding_button);
        menu_box.append(&shortcuts_button);
        menu_box.append(&about_button);
        menu_box.append(&donate_button);

        let menu_popover = gtk::Popover::builder().autohide(true).build();
        menu_popover.set_position(gtk::PositionType::Bottom);
        menu_popover.set_child(Some(&menu_box));
        menu_popover
    }
}

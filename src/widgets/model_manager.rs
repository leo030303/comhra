use core::time;
use std::{
    fs::File,
    io::Read,
    path::PathBuf,
    process::Command,
    sync::mpsc::{self, Receiver, Sender},
};

use adw::prelude::*;
use gtk::glib;
use ollama_rs::Ollama;

use crate::{
    models::{
        ollama_model::{ModelInfo, OllamaModel},
        ModelType, UtilsLLM,
    },
    utils::get_root_folder,
};
/*
two tabs
    Local
        On open, list models, if fail, have a dialog for selecting a different ollama url or installing ollama
            Entry for new url
            Button for install ollama
                Show output from installation process
        Open list of all available ollama models
        If downloaded, have delete button
        If not downloaded, have download button
        Have name, short description, and size
    Remote
        Will work on this later
Start on Local
*/

struct ModelListItem {
    button: gtk::Button,
    main_box: gtk::Box,
    model_info: ModelInfo,
}

impl ModelListItem {
    pub fn new(model_info: ModelInfo, main_context: glib::MainContext) -> Self {
        let detail_box = gtk::Box::builder()
            .spacing(5)
            .hexpand(true)
            .orientation(gtk::Orientation::Vertical)
            .build();
        let display_name_label = gtk::Label::builder()
            .selectable(false)
            .wrap(true)
            .label(&model_info.display_name)
            .build();
        let download_name_label = gtk::Label::builder()
            .selectable(false)
            .wrap(true)
            .label(&model_info.download_name)
            .build();
        let size_in_b_label = gtk::Label::builder()
            .selectable(false)
            .wrap(true)
            .label(model_info.size_in_b.to_string())
            .build();
        let description_label = gtk::Label::builder()
            .selectable(false)
            .wrap(true)
            .label(&model_info.description)
            .build();
        let model_download_progress_bar = gtk::ProgressBar::builder()
            .text(format!("Downloading model: {}", model_info.download_name))
            .visible(false)
            .show_text(true)
            .build();
        detail_box.append(&display_name_label);
        detail_box.append(&download_name_label);
        detail_box.append(&size_in_b_label);
        detail_box.append(&description_label);
        detail_box.append(&model_download_progress_bar);
        let main_box = gtk::Box::builder()
            .spacing(5)
            .orientation(gtk::Orientation::Horizontal)
            .build();
        let button = if model_info.is_downloaded {
            let temp_button = gtk::Button::builder()
                .icon_name("user-trash-symbolic")
                .css_classes(["is-downloaded-button"])
                .build();
            {
                let download_name = model_info.download_name.clone();
                let model_download_progress_bar = model_download_progress_bar.clone();
                temp_button.connect_clicked(move |button| {
                    ModelListItem::delete_button(
                        button,
                        download_name.clone(),
                        model_download_progress_bar.clone(),
                        main_context.clone(),
                    )
                });
            }
            temp_button
        } else {
            let temp_button = gtk::Button::builder()
                .icon_name("document-save-symbolic")
                .css_classes(["not-downloaded-button"])
                .build();
            {
                let download_name = model_info.download_name.clone();
                let model_download_progress_bar = model_download_progress_bar.clone();
                temp_button.connect_clicked(move |button| {
                    ModelListItem::download_button(
                        button,
                        download_name.clone(),
                        model_download_progress_bar.clone(),
                        main_context.clone(),
                    )
                });
            }
            temp_button
        };
        main_box.append(&detail_box);
        main_box.append(&button);
        Self {
            button,
            main_box,
            model_info,
        }
    }

    fn delete_button(
        button: &gtk::Button,
        download_name: String,
        model_download_progress_bar: gtk::ProgressBar,
        main_context: glib::MainContext,
    ) {
        main_context.spawn_local(OllamaModel::delete_model(download_name.clone()));
        button.set_icon_name("document-save-symbolic");
        button.set_css_classes(&["not-downloaded-button"]);
        button.connect_clicked(move |button| {
            ModelListItem::download_button(
                button,
                download_name.clone(),
                model_download_progress_bar.clone(),
                main_context.clone(),
            )
        });
    }

    fn download_button(
        button: &gtk::Button,
        download_name: String,
        model_download_progress_bar: gtk::ProgressBar,
        main_context: glib::MainContext,
    ) {
        let model_download_progress_bar_for_thread = model_download_progress_bar.clone();
        let download_name_for_thread = download_name.clone();
        main_context.spawn_local(async move {
            OllamaModel::pull_model(
                download_name_for_thread,
                &model_download_progress_bar_for_thread,
            )
            .await;
        });
        button.set_icon_name("user-trash-symbolic");
        button.set_css_classes(&["is-downloaded-button"]);
        button.connect_clicked(move |button| {
            ModelListItem::delete_button(
                button,
                download_name.clone(),
                model_download_progress_bar.clone(),
                main_context.clone(),
            )
        });
    }
}

pub struct ModelManagerWidget {
    pub main_box: gtk::Box,
    ollama_model_list: Vec<ModelInfo>,
}

impl ModelManagerWidget {
    pub fn new(main_context: glib::MainContext) -> Self {
        let ollama_model_list_path =
            get_root_folder().join(PathBuf::from("./ollama_model_list.json"));
        let mut ollama_model_list = if ollama_model_list_path.exists() {
            let mut list_file = File::open(&ollama_model_list_path).expect("Could not open file");

            let mut json_data = String::new();
            list_file
                .read_to_string(&mut json_data)
                .expect("Failed to read data from file");

            let loaded_model_list: Vec<ModelInfo> =
                serde_json::from_str(&json_data).expect("Failed to deserialize JSON data");
            loaded_model_list
        } else {
            vec![]
        };
        let main_box = gtk::Box::builder()
            .spacing(5)
            .orientation(gtk::Orientation::Vertical)
            .build();
        if let Ok(saved_models) = OllamaModel::list_models() {
            let scroll_window = gtk::ScrolledWindow::builder()
                .hexpand(true)
                .vexpand(true)
                .build();
            let list_widget = gtk::ListBox::builder().hexpand(true).vexpand(true).build();
            let saved_models_names = saved_models
                .iter()
                .filter(|saved_model| matches!(saved_model.model_type, ModelType::Ollama))
                .map(|saved_model| saved_model.name.clone())
                .collect::<Vec<String>>();
            ollama_model_list.iter_mut().for_each(|model_info| {
                model_info.is_downloaded = saved_models_names.contains(&model_info.download_name);
                let model_list_item_box =
                    ModelListItem::new(model_info.clone(), main_context.clone()).main_box;
                list_widget.append(&model_list_item_box);
            });
            scroll_window.set_child(Some(&list_widget));
            main_box.append(&scroll_window);
        } else {
            let default_ollama_uri = Ollama::default().uri();
            let error_label = gtk::Label::builder()
                .label(format!(
                "Ollama was not found at {} Either enter the URI you're using or install Ollama.",
                default_ollama_uri
            ))
                .wrap(true)
                .build();
            main_box.append(&error_label);
            let install_ollama_button = gtk::Button::builder().label("Install Ollama").build();
            let installation_progress_spinner = gtk::Spinner::new();
            main_box.append(&install_ollama_button);
            main_box.append(&installation_progress_spinner);
            install_ollama_button.connect_clicked(move |_| {
                let installation_progress_spinner = installation_progress_spinner.clone();
                let error_label = error_label.clone();
                main_context.spawn_local(async move {
                    execute_command(&installation_progress_spinner, &error_label).await;
                });
                println!("TEST3");
            });
        }
        Self {
            main_box,
            ollama_model_list,
        }
    }
}

async fn execute_command(installation_progress_spinner: &gtk::Spinner, error_label: &gtk::Label) {
    let command = "curl -fsSL https://ollama.com/install.sh | sh";
    installation_progress_spinner.start();
    let ollama_install_script = Command::new("curl")
        .arg("-fsSL")
        .arg("https://ollama.com/install.sh")
        .output()
        .expect("Failed to execute command")
        .stdout;
    println!("eeeeee");
    let (ollama_install_sender, ollama_install_receiver): (Sender<bool>, Receiver<bool>) =
        mpsc::channel();
    std::thread::spawn(move || {
        Command::new("sh")
            .arg("-c")
            .arg(String::from_utf8_lossy(&ollama_install_script).into_owned())
            .spawn()
            .expect("Err")
            .wait_with_output()
            .expect("err2");
        println!("teeeeee");
        ollama_install_sender.send(true).unwrap();
    });
    loop {
        match ollama_install_receiver.try_recv() {
            Ok(is_done) => {
                if is_done {
                    installation_progress_spinner.stop();
                    error_label.set_text("Ollama is now installed.")
                }
            }
            Err(mpsc::TryRecvError::Empty) => {
                glib::timeout_future(time::Duration::from_millis(500)).await;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                // Handle the case where the sender is disconnected
                println!("The model manager channel is disconnected.");
                break;
            }
        }
    }
}

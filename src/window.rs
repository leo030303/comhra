use crate::models::ollama_model::OllamaModel;
use crate::models::{CoreLLM, Message, UtilsLLM};
use crate::utils::generate_unique_filename;
use crate::widgets::chat_list_item::ChatMessageListItem;
use crate::widgets::main_header::{HeaderWidget, RagDropdown};
use crate::widgets::prompt_entry::PromptEntryWidget;
use crate::widgets::sidebar::create_sidebar;
use crate::{ModelMessageState, RagSource};
use adw::{gdk, prelude::*};
use core::time;
use gtk::{glib, ApplicationWindow};
use std::path::PathBuf;

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};

fn process_user_prompt(
    prompt_entry_buffer: &gtk::EntryBuffer,
    prompt_button_send_icon: &gtk::Image,
    list_sender: &Sender<Message>,
    model_sender: &Sender<Message>,
    is_processing: &Arc<Mutex<ModelMessageState>>,
    prompt_selected_file: &Arc<Mutex<Option<PathBuf>>>,
) {
    let mut model_message_state = is_processing.lock().unwrap().clone();
    if let ModelMessageState::FinishedAssistant = model_message_state {
        *is_processing.lock().unwrap() = ModelMessageState::UserTurn;
        model_message_state = is_processing.lock().unwrap().clone();
    }
    if let ModelMessageState::UserTurn = model_message_state {
        let text = prompt_entry_buffer.text().to_string();
        if !text.is_empty() {
            prompt_button_send_icon.set_from_icon_name(Some("emblem-synchronizing-symbolic"));
            prompt_entry_buffer.set_text("");
            let mut chat_message = Message {
                role: crate::models::Role::User,
                content: text.clone(),
                images: None,
            };
            let file_path_option = (*prompt_selected_file.lock().unwrap()).clone();
            *prompt_selected_file.lock().unwrap() = None;
            if let Some(file_path) = file_path_option {
                chat_message = OllamaModel::process_file_for_prompt(chat_message, file_path);
            }
            list_sender
                .send(chat_message.clone())
                .expect("List channel needs to be open.");
            model_sender
                .send(chat_message)
                .expect("Model channel needs to be open.");
        }
    } else {
        println!(
            "Still processing a prompt, ModelMessageState is {:?}",
            model_message_state
        );
    }
}

fn create_model_caller_thread(
    main_context: glib::MainContext,
    chat_model: Arc<Mutex<Box<dyn CoreLLM>>>,
    model_receiver: Receiver<Message>,
    list_sender: Sender<Message>,
    is_processing: &Arc<Mutex<ModelMessageState>>,
    rag_dropdown: RagDropdown,
) -> glib::MainContext {
    let is_processing = Arc::clone(is_processing);
    main_context.spawn_local(async move {
        loop {
            match model_receiver.try_recv() {
                Ok(mut chat_message) => {
                    *is_processing.lock().unwrap() = ModelMessageState::StartAssistant;
                    chat_message.content = OllamaModel::format_prompt(
                        &chat_message.content,
                        rag_dropdown
                            .rag_options
                            .get(rag_dropdown.dropdown.selected() as usize)
                            .unwrap_or(&RagSource::NoRag)
                            .clone(),
                    );
                    chat_model
                        .lock()
                        .unwrap()
                        .ask(chat_message, list_sender.clone())
                        .await;
                    *is_processing.lock().unwrap() = ModelMessageState::FinishedAssistant;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No message available yet, wait a bit before checking again.
                    glib::timeout_future(time::Duration::from_millis(500)).await;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Handle the case where the sender is disconnected
                    println!("The model manager channel is disconnected.");
                    break;
                }
            }
        }
    });
    main_context
}

fn create_list_manager_thread(
    main_context: glib::MainContext,
    list_receiver: Receiver<Message>,
    conversation_list_box: gtk::ListBox,
    prompt_button_send_icon: gtk::Image,
    is_processing: Arc<Mutex<ModelMessageState>>,
) -> glib::MainContext {
    main_context.spawn_local(async move {
        let mut chat_message_list_item = ChatMessageListItem::new(None);
        let mut last_model_message_state = ModelMessageState::UserTurn;
        loop {
            match list_receiver.try_recv() {
                Ok(chat_message) => {
                    let model_message_state = is_processing.lock().unwrap().clone();
                    println!("{:?}", model_message_state);
                    match model_message_state {
                        ModelMessageState::UserTurn => {
                            println!("User message: {:?}", chat_message.clone());
                            let current_user_list_item =
                                ChatMessageListItem::new(Some(chat_message.clone()));
                            conversation_list_box.append(&current_user_list_item.main_box);
                        }
                        ModelMessageState::RunningAssistant => {
                            chat_message_list_item.update_message(chat_message);
                        }
                        ModelMessageState::StartAssistant => {
                            chat_message_list_item =
                                ChatMessageListItem::new(Some(chat_message.clone()));
                            conversation_list_box.append(&chat_message_list_item.main_box);
                            *is_processing.lock().unwrap() = ModelMessageState::RunningAssistant;
                            last_model_message_state = ModelMessageState::RunningAssistant;
                        }
                        ModelMessageState::LoadingFromFile => {
                            let current_list_item =
                                ChatMessageListItem::new(Some(chat_message.clone()));
                            conversation_list_box.append(&current_list_item.main_box);
                        }
                        ModelMessageState::FinishedAssistant => {
                            chat_message_list_item.update_message(chat_message);
                            prompt_button_send_icon.set_from_icon_name(Some("emblem-ok-symbolic"));
                        }
                    }
                }

                Err(mpsc::TryRecvError::Empty) => {
                    // No message available yet, wait a bit before checking again.
                    if let ModelMessageState::UserTurn = last_model_message_state {
                        glib::timeout_future(time::Duration::from_millis(50)).await;
                    } else {
                        glib::timeout_future(time::Duration::from_millis(10)).await;
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Handle the case where the sender is disconnected
                    println!("The list manager channel is disconnected.");
                    break;
                }
            }
        }
    });
    main_context
}

fn create_conversation_file_manager_thread(
    main_context: glib::MainContext,
    conversation_file_option_receiver: Receiver<Option<PathBuf>>,
    chat_model: &Arc<Mutex<Box<dyn CoreLLM>>>,
    prompt_entry_widget: PromptEntryWidget,
    conversation_scroll_window: gtk::ScrolledWindow,
    conversation_file_path_arc: &Arc<Mutex<PathBuf>>,
    rag_dropdown: RagDropdown,
    main_context_box: gtk::Box,
    sidebar_box: gtk::Box,
    sidebar_toggle_button: gtk::ToggleButton,
) -> glib::MainContext {
    let chat_model = Arc::clone(chat_model);
    let conversation_file_path_arc = Arc::clone(conversation_file_path_arc);
    let main_context_clone = main_context.clone();
    main_context.spawn_local(async move {
        loop {
            match conversation_file_option_receiver.try_recv() {
                Ok(conversation_file_option) => {
                    main_context_box.show();
                    sidebar_box.hide();
                    sidebar_toggle_button.set_active(false);
                    create_chat_context(
                        &main_context_clone,
                        &chat_model,
                        &conversation_scroll_window,
                        &prompt_entry_widget,
                        conversation_file_option,
                        &conversation_file_path_arc,
                        rag_dropdown.clone(),
                    );
                }

                Err(mpsc::TryRecvError::Empty) => {
                    // No message available yet, wait a bit before checking again.
                    glib::timeout_future(time::Duration::from_millis(500)).await;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Handle the case where the sender is disconnected
                    println!("The conversation file manager channel is disconnected.");
                    break;
                }
            }
        }
    });
    main_context
}

fn create_chat_context(
    new_main_context: &glib::MainContext,
    chat_model: &Arc<Mutex<Box<dyn CoreLLM>>>,
    conversation_scroll_window: &gtk::ScrolledWindow,
    prompt_entry_widget: &PromptEntryWidget,
    new_conversation_filepath_option: Option<PathBuf>,
    current_conversation_file_path_arc: &Arc<Mutex<PathBuf>>,
    rag_dropdown: RagDropdown,
) {
    // Initialise all the async
    let new_main_context = new_main_context.clone();
    let chat_model_for_thread = Arc::clone(chat_model);
    let (model_sender, model_receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();
    let (list_sender, list_receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();
    let prompt_entry_signal_id_for_closure =
        Arc::clone(&prompt_entry_widget.prompt_entry_signal_id);
    let prompt_button_signal_id_for_closure =
        Arc::clone(&prompt_entry_widget.submit_button_signal_id);
    let is_processing = Arc::new(Mutex::new(ModelMessageState::UserTurn));

    // Create fresh ListBox
    let conversation_list_box = gtk::ListBox::builder().vexpand(true).build();
    conversation_scroll_window.set_child(Some(&conversation_list_box));

    // Closure to connect prompt entry box to the model caller
    {
        // Clone vars for closure
        let prompt_entry_buffer = prompt_entry_widget.prompt_entry_buffer.clone();
        let prompt_selected_file = Arc::clone(&prompt_entry_widget.selected_file);
        let prompt_button_send_icon = prompt_entry_widget.prompt_button_send_icon.clone();
        let list_sender = list_sender.clone();
        let model_sender = model_sender.clone();
        let is_processing = Arc::clone(&is_processing);

        // Disconnect the exisiting signal from the entry
        if prompt_entry_signal_id_for_closure.lock().unwrap().is_some() {
            let signal_id = prompt_entry_signal_id_for_closure
                .lock()
                .unwrap()
                .take()
                .unwrap();
            prompt_entry_widget.prompt_entry.disconnect(signal_id);
        }

        // Connect the new signal
        let new_signal_id = prompt_entry_widget.prompt_entry.connect_activate(move |_| {
            process_user_prompt(
                &prompt_entry_buffer,
                &prompt_button_send_icon,
                &list_sender,
                &model_sender,
                &is_processing,
                &prompt_selected_file,
            );
        });

        // Save the new signal to the Arc
        let _ = prompt_entry_signal_id_for_closure
            .lock()
            .unwrap()
            .insert(new_signal_id);
    }

    // Closure to connect prompt button to the model caller
    {
        // Clone vars for closure
        let prompt_entry_buffer = prompt_entry_widget.prompt_entry_buffer.clone();
        let prompt_selected_file = Arc::clone(&prompt_entry_widget.selected_file);
        let prompt_button_send_icon = prompt_entry_widget.prompt_button_send_icon.clone();
        let list_sender = list_sender.clone();
        let model_sender = model_sender.clone();
        let is_processing = Arc::clone(&is_processing);

        // Disconnect the exisiting signal from the button
        if prompt_button_signal_id_for_closure
            .lock()
            .unwrap()
            .is_some()
        {
            let signal_id = prompt_button_signal_id_for_closure
                .lock()
                .unwrap()
                .take()
                .unwrap();
            prompt_entry_widget.submit_button.disconnect(signal_id);
        }

        // Connect the new signal
        let new_signal_id = prompt_entry_widget.submit_button.connect_clicked(move |_| {
            process_user_prompt(
                &prompt_entry_buffer,
                &prompt_button_send_icon,
                &list_sender,
                &model_sender,
                &is_processing,
                &prompt_selected_file,
            );
        });

        // Save the new signal to the Arc
        let _ = prompt_button_signal_id_for_closure
            .lock()
            .unwrap()
            .insert(new_signal_id);
    }

    // Spawn a thread which listens for prompts to run through the model
    let list_sender_for_model_caller = list_sender.clone();
    let main_context = create_model_caller_thread(
        new_main_context,
        chat_model_for_thread,
        model_receiver,
        list_sender_for_model_caller,
        &is_processing,
        rag_dropdown,
    );

    // Spawn a thread which listens for items to add to the conversation list
    create_list_manager_thread(
        main_context,
        list_receiver,
        conversation_list_box,
        prompt_entry_widget.prompt_button_send_icon.clone(),
        Arc::clone(&is_processing),
    );
    if let Some(conversation_filepath) = new_conversation_filepath_option {
        chat_model
            .lock()
            .unwrap()
            .load_conversation_file(conversation_filepath.clone());
        let mut file_arc = current_conversation_file_path_arc.lock().unwrap();
        *file_arc = conversation_filepath;
    } else {
        chat_model.lock().unwrap().reset_conversation(
            current_conversation_file_path_arc
                .lock()
                .unwrap()
                .to_path_buf(),
        );
        let mut file_arc = current_conversation_file_path_arc.lock().unwrap();
        *file_arc = generate_unique_filename("json");
    }
    let conversation = chat_model.lock().unwrap().get_conversation();
    if !conversation.is_empty() {
        let list_sender = list_sender.clone();
        *is_processing.lock().unwrap() = ModelMessageState::LoadingFromFile;
        conversation.iter().for_each(|chat_message| {
            println!("{:?}", chat_message);
            let mut message_to_send = chat_message.clone();
            if let crate::models::Role::User = message_to_send.role {
                message_to_send.content = OllamaModel::unformat_prompt(&chat_message.content);
            }
            list_sender.send(message_to_send).unwrap();
        });
        *is_processing.lock().unwrap() = ModelMessageState::UserTurn;
    }

    // Set focus on prompt input
    prompt_entry_widget.prompt_entry.grab_focus();
}

pub fn build_ui(app: &adw::Application) {
    // Set up async channels and context
    let main_context = glib::MainContext::default();
    let chat_model: Arc<Mutex<Box<dyn CoreLLM>>> =
        Arc::new(Mutex::new(Box::new(OllamaModel::new())));
    let (conversation_file_option_sender, conversation_file_option_receiver): (
        Sender<Option<PathBuf>>,
        Receiver<Option<PathBuf>>,
    ) = mpsc::channel();
    let conversation_file_path_arc = Arc::new(Mutex::new(generate_unique_filename("json")));

    let prompt_entry_widget = PromptEntryWidget::new();

    // Add all to the main box
    let conversation_scroll_window = gtk::ScrolledWindow::new();
    let main_content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(5)
        .build();
    main_content_box.append(&conversation_scroll_window);
    main_content_box.append(&prompt_entry_widget.main_box);

    let paned_main = gtk::Paned::new(gtk::Orientation::Horizontal);
    let sidebar_widget = create_sidebar(conversation_file_option_sender.clone());

    paned_main.set_start_child(Some(&sidebar_widget));
    paned_main.set_end_child(Some(&main_content_box));

    let sidebar_box = sidebar_widget.clone();

    let header_bar = HeaderWidget::new(
        sidebar_widget,
        main_content_box.clone(),
        conversation_file_option_sender.clone(),
        Arc::clone(&chat_model),
        main_context.clone(),
    );

    let main_context = create_conversation_file_manager_thread(
        main_context,
        conversation_file_option_receiver,
        &chat_model,
        prompt_entry_widget,
        conversation_scroll_window,
        &conversation_file_path_arc,
        header_bar.rag_dropdown,
        main_content_box.clone(),
        sidebar_box,
        header_bar.sidebar_toggle_button,
    );

    // Set CSS
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(
        "
    .user-label {
      background-color: #8ff0a4;
      color: black;
    }
    .assistant-label {
      background-color: #99c1f1;
      color: black;
    }
    .system-label {
      background-color: #f9f06b;
      color: black;
    }
        ",
    );
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Comhrá")
        .titlebar(&header_bar.main_bar)
        .default_height(600)
        .default_width(600)
        .child(&paned_main)
        .build();

    let new_main_context = main_context.clone();
    window.connect_close_request(move |_| {
        let chat_model_for_export = Arc::clone(&chat_model);
        let conversation_file_path_for_export = Arc::clone(&conversation_file_path_arc);
        new_main_context.spawn_local(async move {
            chat_model_for_export.lock().unwrap().export_conversation(
                conversation_file_path_for_export
                    .lock()
                    .unwrap()
                    .to_path_buf(),
            );
        });
        glib::Propagation::Proceed
    });

    window.present();

    // Initialise chat context
    conversation_file_option_sender.send(None).unwrap();
}

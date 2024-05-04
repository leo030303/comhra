use crate::phi_chat::PhiModel;
use core::time;
use gtk::{gdk, prelude::*};
use gtk::{glib, Application, ApplicationWindow};
use ollama_rs::generation::chat::ChatMessage;
use std::path::PathBuf;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

fn process_user_prompt(
    prompt_entry_buffer: &gtk::EntryBuffer,
    list_sender: &Sender<ChatMessage>,
    phi_sender: &Sender<String>,
) {
    let text = prompt_entry_buffer.text().to_string();
    prompt_entry_buffer.set_text("");
    list_sender
        .send(ChatMessage::user(text.clone()))
        .expect("List channel needs to be open.");
    phi_sender
        .send(text)
        .expect("Phi channel needs to be open.");
}

pub fn build_ui(app: &Application) {
    // Set up async channels and context
    let main_context = glib::MainContext::default();
    let (phi_sender, phi_receiver): (Sender<String>, Receiver<String>) = mpsc::channel();
    let (list_sender, list_receiver): (Sender<ChatMessage>, Receiver<ChatMessage>) =
        mpsc::channel();

    // Make sure the model is downloaded TODO add progress bar for download
    tokio::spawn(async {
        PhiModel::pull_model(PhiModel::default_model_string()).await;
    });

    // Create the prompt entry section
    let prompt_entry_buffer = gtk::EntryBuffer::builder().text("").build();
    let prompt_entry = gtk::Entry::builder()
        .buffer(&prompt_entry_buffer)
        .hexpand(true)
        .placeholder_text("Enter a prompt")
        .build();
    let prompt_button = gtk::Button::builder()
        .label("Enter")
        .width_request(50)
        .build();
    let prompt_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(0)
        .build();
    prompt_box.append(&prompt_entry);
    prompt_box.append(&prompt_button);

    // Closures to send user prompt to be processed if the user hits the button or hits enter on the entry box
    {
        let prompt_entry_buffer = prompt_entry_buffer.clone();
        let list_sender = list_sender.clone();
        let phi_sender = phi_sender.clone();
        prompt_entry.connect_activate(move |_| {
            process_user_prompt(&prompt_entry_buffer, &list_sender, &phi_sender);
        });
    }
    {
        let prompt_entry_buffer = prompt_entry_buffer.clone();
        let list_sender = list_sender.clone();
        let phi_sender = phi_sender.clone();
        prompt_button.connect_clicked(move |_| {
            process_user_prompt(&prompt_entry_buffer, &list_sender, &phi_sender);
        });
    }

    // Create the conversation list view
    let conversation_list_view = gtk::ListBox::builder().vexpand(true).build();
    let conversation_scroll_window = gtk::ScrolledWindow::new();
    conversation_scroll_window.set_child(Some(&conversation_list_view));

    // Add all to the main box
    let main_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(5)
        .build();
    main_box.append(&conversation_scroll_window);
    main_box.append(&prompt_box);

    // Spawn a thread which listens for prompts and sends them to the Phi model for processing
    main_context.spawn_local(async move {
        let mut phi_chat_model = PhiModel::new();
        loop {
            match phi_receiver.try_recv() {
                Ok(prompt) => {
                    let formatted_prompt = PhiModel::format_prompt(&prompt);
                    let response = phi_chat_model.ask_phi(formatted_prompt).await.unwrap_or(
                        ChatMessage::assistant(String::from(
                            "Error generating response from Phi model",
                        )),
                    );
                    list_sender
                        .send(response)
                        .expect("List channel needs to be open.");
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No message available yet, wait a bit before checking again.
                    glib::timeout_future_seconds(1).await;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Handle the case where the sender is disconnected
                    println!("The phi sender channel is disconnected.");
                    break;
                }
            }
        }
    });

    // Spawn a thread which listens for items to add to the conversation list
    main_context.spawn_local(async move {
        loop {
            match list_receiver.try_recv() {
                Ok(chat_message) => {
                    let chat_message_label = gtk::Label::builder().wrap(true).label("").build();
                    match chat_message.role {
                        ollama_rs::generation::chat::MessageRole::User => {
                            chat_message_label.add_css_class("user-label");
                            chat_message_label
                                .set_text(&format!("User: {}", &chat_message.content));
                        }
                        ollama_rs::generation::chat::MessageRole::Assistant => {
                            chat_message_label.add_css_class("assistant-label");
                            chat_message_label
                                .set_text(&format!("Assistant: {}", &chat_message.content));
                        }
                        ollama_rs::generation::chat::MessageRole::System => {
                            chat_message_label.add_css_class("system-label");
                            chat_message_label
                                .set_text(&format!("System: {}", &chat_message.content));
                        }
                    }

                    conversation_list_view.append(&chat_message_label);
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No message available yet, wait a bit before checking again.
                    glib::timeout_future(time::Duration::from_millis(500)).await;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Handle the case where the sender is disconnected
                    println!("The sender channel is disconnected.");
                    break;
                }
            }
        }
    });

    // Set CSS
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_path(PathBuf::from("./src/resources/app.css"));
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Phi-3 Chat")
        .default_height(600)
        .default_width(600)
        .child(&main_box)
        .build();

    window.present();
    prompt_entry.grab_focus(); // Start focus on prompt input
}

use adw::prelude::*;
use async_trait::async_trait;
use futures::executor::block_on;
use ollama_rs::error::OllamaError;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::process::Command;
use std::{
    ffi::OsStr,
    fs::{self},
    path::PathBuf,
    sync::mpsc::Sender,
};

use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage, ChatMessageResponseStream, MessageRole},
        images::Image,
    },
    Ollama,
};
use tokio_stream::StreamExt;

use crate::{
    utils::{self, run_bash_search_script},
    RagSource,
};

use super::{
    B64Image, CoreLLM, FromMessage, Message, SavedConversation, SavedModel, ToMessage, UtilsLLM,
};

#[derive(Clone)]
pub struct OllamaModel {
    model_name: String,
    message_history: Vec<ChatMessageWithB64Image>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelInfo {
    pub display_name: String,
    pub download_name: String,
    pub size_in_b: f64,
    pub description: String,
    pub is_downloaded: bool,
}

#[derive(Clone)]
pub struct ChatMessageWithB64Image {
    pub chat_message: ChatMessage,
    pub b64_image_vec: Option<Vec<B64Image>>,
}

impl Default for OllamaModel {
    fn default() -> Self {
        Self::new()
    }
}

impl FromMessage for ChatMessageWithB64Image {
    fn from_message(message: Message) -> Self {
        let content = message.content;
        let role = match message.role {
            super::Role::User => MessageRole::User,
            super::Role::Assistant => MessageRole::Assistant,
            super::Role::System => MessageRole::System,
        };
        let images = match message.images {
            Some(_) => Some(
                message
                    .images
                    .clone()
                    .unwrap()
                    .iter()
                    .map(|message_image| Image::from_base64(&message_image.b64_string))
                    .collect::<Vec<Image>>(),
            ),
            None => None,
        };
        let chat_message = ChatMessage {
            role,
            content,
            images,
        };
        Self {
            chat_message,
            b64_image_vec: message.images.clone(),
        }
    }
}

impl ToMessage for ChatMessageWithB64Image {
    fn to_message(&self) -> Message {
        let content = self.chat_message.content.clone();
        let role = match self.chat_message.role {
            MessageRole::User => super::Role::User,
            MessageRole::Assistant => super::Role::Assistant,
            MessageRole::System => super::Role::System,
        };
        Message {
            role,
            content,
            images: self.b64_image_vec.clone(),
        }
    }
}

impl OllamaModel {
    pub fn new() -> Self {
        OllamaModel {
            model_name: OllamaModel::default_model_string(),
            message_history: vec![],
        }
    }

    pub fn new_from_conversation_and_model_name(
        raw_message_history: Vec<Message>,
        model_name: String,
    ) -> Self {
        let message_history = raw_message_history
            .iter()
            .map(|message| ChatMessageWithB64Image::from_message(message.clone()))
            .collect::<Vec<ChatMessageWithB64Image>>();
        OllamaModel {
            model_name,
            message_history,
        }
    }
    pub fn change_model(&mut self, new_model: String) {
        self.model_name = new_model;
    }

    fn list_els() -> Result<Vec<SavedModel>, OllamaError> {
        Command::new("ls")
            .arg("-a")
            .spawn()
            .expect("failed to execute command");
        Ok(block_on(Ollama::default().list_local_models())?
            .iter()
            .map(|local_model| SavedModel {
                name: local_model.name.clone(),
                model_type: super::ModelType::Ollama,
            })
            .collect::<Vec<SavedModel>>())
    }
}

impl UtilsLLM for OllamaModel {
    fn default_model_string() -> String {
        String::from("phi3:latest")
    }

    fn format_prompt(prompt: &str, rag_source: RagSource) -> String {
        println!("\n\nRag Source: {:?}\n\n", rag_source);
        match rag_source {
            RagSource::NoRag => prompt.to_string(),
            RagSource::BashScript(script_path) => {
                let rag_content = run_bash_search_script(script_path, prompt);
                println!("{}", rag_content);
                todo!(); //fix the format below, probably will make the model return nothing atm
                         //format!("<|context|>/n This is additional context to answer the users query:/n {0}<|end|>/n <|user|>/n {1}<|end|>/n <|assistant|>", rag_content, prompt)
            }
        }
    }

    fn unformat_prompt(prompt: &str) -> String {
        let end_split = prompt
            .char_indices()
            .nth_back("<|end|>/n <|assistant|><|end|>/n".len())
            .unwrap()
            .0;
        String::from(
            prompt
                .split_at("<|user|>/n ".len() - 1)
                .1
                .split_at(end_split)
                .0,
        )
    }

    async fn pull_model(model_name: String, download_progress_bar: &gtk::ProgressBar) {
        let ollama = Ollama::default();
        let res = ollama.list_local_models().await.unwrap();
        if res.iter().any(|local_model| local_model.name == model_name) {
            println!("Model found: {}", model_name);
        } else {
            download_progress_bar.show();
            println!("Downloading model: {}", &model_name);
            let mut res = ollama.pull_model_stream(model_name, false).await.unwrap();

            while let Some(res) = res.next().await {
                match res {
                    Ok(res) => {
                        if let (Some(total), Some(completed)) = (res.total, res.completed) {
                            let fraction = completed as f64 / total as f64;
                            println!("{}", fraction);
                            download_progress_bar.set_fraction(fraction);
                            download_progress_bar.set_text(Some(
                                format!(
                                    "Downloading model: {0} {1:.1}%",
                                    OllamaModel::default_model_string(),
                                    (fraction * 100.0)
                                )
                                .as_str(),
                            ));
                        }
                        println!("{:?}", res);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }
        }
        download_progress_bar.hide();
    }

    async fn delete_model(model_name: String) {
        Ollama::default().delete_model(model_name).await.unwrap();
    }

    fn list_models() -> Result<Vec<SavedModel>, Box<dyn Error>> {
        Ok(block_on(Ollama::default().list_local_models())?
            .iter()
            .map(|local_model| SavedModel {
                name: local_model.name.clone(),
                model_type: super::ModelType::Ollama,
            })
            .collect::<Vec<SavedModel>>())
    }

    fn process_file_for_prompt(mut chat_message: Message, file_path: PathBuf) -> Message {
        match file_path
            .extension()
            .unwrap_or(OsStr::new("error"))
            .to_str()
            .unwrap_or("error")
        {
            "pdf" => {
                let mut pdf_content = utils::pdf_to_string(&file_path);
                pdf_content.push_str(&chat_message.content);
                chat_message.content = pdf_content;
            }
            "jpg" | "jpeg" | "png" => {
                chat_message.images = if let Some(mut images) = chat_message.images {
                    images.push(B64Image {
                        b64_string: utils::image_to_b64(&file_path),
                    });
                    Some(images)
                } else {
                    Some(vec![B64Image {
                        b64_string: utils::image_to_b64(&file_path),
                    }])
                };
            }
            "error" => {
                println!("Error with file")
            }
            _ => {
                let mut file_content =
                    fs::read_to_string(file_path).unwrap_or(String::from("Error reading file"));
                file_content.push_str(&chat_message.content);
                chat_message.content = file_content;
            }
        }
        chat_message
    }
}

#[async_trait]
impl CoreLLM for OllamaModel {
    fn reset_conversation(&mut self, conversation_file_path: PathBuf) {
        self.export_conversation(conversation_file_path);
        self.message_history = vec![];
    }

    fn load_conversation_file(&mut self, file_path: PathBuf) {
        let loaded_conversation =
            SavedConversation::load(&file_path).expect("Conversation file didn't exist");
        self.export_conversation(file_path);
        self.message_history = loaded_conversation
            .conversation
            .iter()
            .map(|message| ChatMessageWithB64Image::from_message(message.clone()))
            .collect();
    }

    async fn ask(&mut self, user_message: Message, list_sender: Sender<Message>) {
        self.message_history
            .push(ChatMessageWithB64Image::from_message(user_message));
        let parsed_conversation = self
            .message_history
            .iter()
            .map(|chat_message_with_b64_image| chat_message_with_b64_image.chat_message.clone())
            .collect::<Vec<ChatMessage>>();
        let mut stream: ChatMessageResponseStream = Ollama::default()
            .send_chat_messages_stream(ChatMessageRequest::new(
                self.model_name.clone(),
                parsed_conversation,
            ))
            .await
            .unwrap();
        let mut response = String::new();
        while let Some(Ok(res)) = stream.next().await {
            if let Some(assistant_message) = res.message {
                response += assistant_message.content.as_str();
                list_sender
                    .send(Message {
                        role: super::Role::Assistant,
                        content: response.clone(),
                        images: None,
                    })
                    .unwrap();
            }
        }
        self.message_history.push(ChatMessageWithB64Image {
            chat_message: ChatMessage::assistant(response),
            b64_image_vec: None,
        });
    }

    fn get_conversation(&mut self) -> Vec<Message> {
        self.message_history
            .iter()
            .map(|chat_message_with_b64_image| chat_message_with_b64_image.to_message())
            .collect::<Vec<Message>>()
    }

    fn export_conversation(&mut self, file_path: PathBuf) {
        if !self.message_history.is_empty() {
            let archived;
            let starred;
            let name;
            if let Some(loaded_conversation) = SavedConversation::load(&file_path) {
                archived = loaded_conversation.archived;
                starred = loaded_conversation.starred;
                name = loaded_conversation.name;
            } else {
                archived = false;
                starred = false;
                name = file_path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap_or("Error reading filename")
                    .to_owned();
            }
            let saved_conversation = SavedConversation {
                conversation: self
                    .message_history
                    .iter()
                    .map(|chat_message_with_b64_image| chat_message_with_b64_image.to_message())
                    .collect::<Vec<Message>>(),
                archived,
                starred,
                name,
            };
            saved_conversation.save(&file_path);
        } else {
            println!("Message history empty.");
        }
    }
}

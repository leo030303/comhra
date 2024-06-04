use crate::utils;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, io::Read, path::PathBuf};
use tokio_stream::StreamExt;

use super::{CoreLLM, Message, SavedConversation, SavedModel, UtilsLLM};
use async_trait::async_trait;

#[derive(Clone)]
pub struct ApiModel {
    api_key: String,
    message_history: Vec<Message>,
    api_type: ApiType,
}

impl ApiModel {
    pub fn new(model_name: String, api_key: String, api_type_from_saved: ApiTypeForSaving) -> Self {
        let api_type = match api_type_from_saved {
            ApiTypeForSaving::OpenAI => {
                ApiType::OpenAI(OpenAIModel::new(model_name, api_key.clone()))
            }
            ApiTypeForSaving::Generic => todo!(),
        };
        Self {
            api_key,
            message_history: vec![],
            api_type,
        }
    }

    pub fn new_from_conversation_and_model_name(
        message_history: Vec<Message>,
        model_name: String,
        api_key: String,
        api_type_from_saved: ApiTypeForSaving,
    ) -> Self {
        let api_type = match api_type_from_saved {
            ApiTypeForSaving::OpenAI => {
                ApiType::OpenAI(OpenAIModel::new(model_name, api_key.clone()))
            }
            ApiTypeForSaving::Generic => todo!(),
        };
        Self {
            api_key,
            message_history,
            api_type,
        }
    }
}

#[derive(Clone)]
enum ApiType {
    OpenAI(OpenAIModel),
    Generic(GenericApi),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ApiTypeForSaving {
    OpenAI,
    Generic,
}

#[derive(Clone)]
struct OpenAIModel {
    model_name: String,
    client: Client<OpenAIConfig>,
}

impl OpenAIModel {
    pub fn new(model_name: String, api_key: String) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);

        let client = Client::with_config(config);

        Self { client, model_name }
    }

    pub async fn stream_call(
        &self,
        conversation: Vec<Message>,
        list_sender: std::sync::mpsc::Sender<Message>,
    ) -> String {
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model_name)
            .messages(
                conversation
                    .iter()
                    .map(|message| match message.role {
                        super::Role::User => ChatCompletionRequestUserMessageArgs::default()
                            .content(vec![
                                ChatCompletionRequestMessageContentPartTextArgs::default()
                                    .text(&message.content)
                                    .build()
                                    .unwrap()
                                    .into(),
                            ])
                            .build()
                            .unwrap()
                            .into(),
                        super::Role::Assistant => {
                            ChatCompletionRequestAssistantMessageArgs::default()
                                .content(&message.content)
                                .build()
                                .unwrap()
                                .into()
                        }
                        super::Role::System => ChatCompletionRequestSystemMessageArgs::default()
                            .content(&message.content)
                            .build()
                            .unwrap()
                            .into(),
                    })
                    .collect::<Vec<ChatCompletionRequestMessage>>(),
            )
            .build()
            .unwrap();
        let mut stream = self.client.chat().create_stream(request).await.unwrap();
        let mut response_text = String::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    response.choices.iter().for_each(|chat_choice| {
                        if let Some(ref content) = chat_choice.delta.content {
                            response_text += content.as_str();
                            list_sender
                                .send(Message {
                                    role: super::Role::Assistant,
                                    content: response_text.clone(),
                                    images: None,
                                })
                                .unwrap();
                        }
                    });
                }
                Err(err) => {
                    println!("error: {err}");
                }
            }
        }
        response_text
    }
}

#[derive(Clone)]
struct GenericApi {}

#[async_trait]
impl CoreLLM for ApiModel {
    fn reset_conversation(&mut self, conversation_file_path: PathBuf) {
        self.export_conversation(conversation_file_path);
        self.message_history = vec![];
    }

    fn load_conversation_file(&mut self, file_path: PathBuf) {
        let loaded_conversation =
            SavedConversation::load(&file_path).expect("Conversation file didn't exist");
        self.export_conversation(file_path);
        self.message_history = loaded_conversation.conversation;
    }

    async fn ask(&mut self, user_message: Message, list_sender: std::sync::mpsc::Sender<Message>) {
        self.message_history.push(user_message);
        let response: String = match &self.api_type {
            ApiType::OpenAI(openai) => {
                openai
                    .stream_call(self.message_history.clone(), list_sender)
                    .await
            }
            ApiType::Generic(_) => todo!(),
        };
        self.message_history.push(Message {
            role: super::Role::Assistant,
            content: response,
            images: None,
        });
    }

    fn get_conversation(&mut self) -> Vec<Message> {
        self.message_history.clone()
    }

    fn export_conversation(&mut self, file_path: std::path::PathBuf) {
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
                conversation: self.message_history.clone(),
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

impl UtilsLLM for ApiModel {
    fn default_model_string() -> String {
        todo!()
    }

    fn format_prompt(prompt: &str, rag_source: crate::RagSource) -> String {
        todo!()
    }

    fn unformat_prompt(prompt: &str) -> String {
        todo!()
    }

    async fn pull_model(model_name: String, download_progress_bar: &gtk::ProgressBar) {
        todo!()
    }

    fn list_models() -> Result<Vec<SavedModel>, Box<dyn Error>> {
        let file_path = utils::get_root_folder().join(PathBuf::from("models/api_models.json"));
        if file_path.exists() {
            let mut model_file = File::open(&file_path)?;

            let mut json_data = String::new();
            model_file.read_to_string(&mut json_data)?;

            let loaded_models: Vec<SavedModel> = serde_json::from_str(&json_data)?;
            Ok(loaded_models)
        } else {
            Ok(vec![])
        }
    }

    fn process_file_for_prompt(chat_message: Message, file_path: std::path::PathBuf) -> Message {
        todo!()
    }

    async fn delete_model(model_name: String) {
        todo!()
    }
}

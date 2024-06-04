use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    sync::mpsc::Sender,
};

use crate::{utils::get_root_folder, RagSource};

use self::{
    api_model::{ApiModel, ApiTypeForSaving},
    candle_model::CandleModel,
    ollama_model::OllamaModel,
};

pub mod api_model;
pub mod candle_model;
pub mod ollama_model;

#[async_trait]
pub trait CoreLLM {
    fn reset_conversation(&mut self, conversation_file_path: PathBuf);

    fn load_conversation_file(&mut self, file_path: PathBuf);

    async fn ask(&mut self, user_message: Message, list_sender: Sender<Message>);

    fn get_conversation(&mut self) -> Vec<Message>;

    fn export_conversation(&mut self, file_path: PathBuf);
}

pub trait UtilsLLM {
    fn default_model_string() -> String;

    fn format_prompt(prompt: &str, rag_source: RagSource) -> String;

    fn unformat_prompt(prompt: &str) -> String;

    async fn pull_model(model_name: String, download_progress_bar: &gtk::ProgressBar);

    async fn delete_model(model_name: String);

    fn list_models() -> Result<Vec<SavedModel>, Box<dyn Error>>;

    fn process_file_for_prompt(chat_message: Message, file_path: PathBuf) -> Message;
}

#[derive(Serialize, Deserialize)]
pub struct SavedConversation {
    pub conversation: Vec<Message>,
    pub archived: bool,
    pub starred: bool,
    pub name: String,
}
impl SavedConversation {
    pub fn load(file_path: &PathBuf) -> Option<Self> {
        let conversation_folder_path =
            get_root_folder().join(PathBuf::from("./conversations").join(file_path));
        if conversation_folder_path.exists() {
            let mut conversation_file =
                File::open(&conversation_folder_path).expect("Could not open file");

            let mut json_data = String::new();
            conversation_file
                .read_to_string(&mut json_data)
                .expect("Failed to read data from file");

            let loaded_conversation: SavedConversation =
                serde_json::from_str(&json_data).expect("Failed to deserialize JSON data");
            Some(loaded_conversation)
        } else {
            None
        }
    }
    pub fn save(&self, file_path: &PathBuf) {
        let mut conversation_folder_path = get_root_folder().join(PathBuf::from("./conversations"));
        fs::create_dir_all(&conversation_folder_path).expect("Failed to create parent directories");
        conversation_folder_path.push(file_path);
        let serialised_conversation =
            serde_json::to_string(&self).expect("Error converting conversation to JSON");
        println!(
            "Writing conversation to file: {}",
            conversation_folder_path.to_str().unwrap()
        );
        let mut file = File::create(conversation_folder_path).expect("Failed to create file");

        // Write the JSON data to the file
        file.write_all(serialised_conversation.as_bytes())
            .expect("Failed to write data to file");
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Role {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct B64Image {
    b64_string: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub images: Option<Vec<B64Image>>,
}

pub trait FromMessage {
    fn from_message(message: Message) -> Self;
}

pub trait ToMessage {
    fn to_message(&self) -> Message;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedModel {
    pub name: String,
    pub model_type: ModelType,
}

impl SavedModel {
    pub fn write_to_file(file_path: PathBuf, models_list: Vec<SavedModel>) {
        if !models_list.is_empty() {
            let mut model_folder_path = get_root_folder().join(PathBuf::from("./models"));
            fs::create_dir_all(&model_folder_path).expect("Failed to create parent directories");
            model_folder_path.push(file_path);
            let serialised_models =
                serde_json::to_string(&models_list).expect("Error converting conversation to JSON");
            println!(
                "Writing models to file: {}",
                model_folder_path.to_str().unwrap()
            );
            let mut file = File::create(model_folder_path).expect("Failed to create file");

            // Write the JSON data to the file
            file.write_all(serialised_models.as_bytes())
                .expect("Failed to write data to file");
        } else {
            println!("Model list empty.");
        }
    }
    pub fn get_all() -> Vec<SavedModel> {
        let mut saved_models_list = vec![];
        saved_models_list.append(&mut OllamaModel::list_models().unwrap_or_else(|err| {
            println!("Error: {:?}", err);
            vec![]
        }));
        saved_models_list.append(&mut ApiModel::list_models().unwrap_or_else(|err| {
            println!("Error: {:?}", err);
            vec![]
        }));
        saved_models_list.append(&mut CandleModel::list_models().unwrap_or_else(|err| {
            println!("Error: {:?}", err);
            vec![]
        }));
        saved_models_list
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ModelType {
    Ollama,
    Candle,
    Api(String, ApiTypeForSaving),
}

impl ModelType {
    pub fn name(&self) -> &str {
        match self {
            ModelType::Ollama => "Ollama",
            ModelType::Candle => "Candle",
            ModelType::Api(..) => "Api",
        }
    }
}

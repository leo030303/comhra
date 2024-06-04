use std::{error::Error, fs::File, io::Read, path::PathBuf};

use crate::utils;

use super::{CoreLLM, Message, SavedModel, UtilsLLM};
use async_trait::async_trait;

#[derive(Clone)]
pub struct CandleModel {
    model_name: String,
    message_history: Vec<Message>,
}

#[async_trait]
impl CoreLLM for CandleModel {
    fn reset_conversation(&mut self, conversation_file_path: std::path::PathBuf) {
        todo!()
    }

    fn load_conversation_file(&mut self, file_path: std::path::PathBuf) {
        todo!()
    }

    async fn ask(&mut self, user_message: Message, list_sender: std::sync::mpsc::Sender<Message>) {
        todo!()
    }

    fn get_conversation(&mut self) -> Vec<Message> {
        todo!()
    }

    fn export_conversation(&mut self, file_path: std::path::PathBuf) {
        todo!()
    }
}

impl CandleModel {
    pub fn new() -> Self {
        todo!()
    }

    pub fn new_from_conversation_and_model_name(
        raw_message_history: Vec<Message>,
        model_name: String,
    ) -> Self {
        todo!()
    }
}

impl UtilsLLM for CandleModel {
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
        let file_path =
            utils::get_root_folder().join(PathBuf::from("models/candle/candle_models.json"));
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

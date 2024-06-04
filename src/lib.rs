pub mod config;
pub mod models;
pub mod utils;
pub mod widgets;
pub mod window;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub enum ModelMessageState {
    UserTurn,
    RunningAssistant,
    StartAssistant,
    LoadingFromFile,
    FinishedAssistant,
}

#[derive(Clone, Debug)]
pub enum AppState {
    Onboarding,
    ModelManaging,
    ModelDownloading,
    SettingsEditing,
    ConversationStarting(String),
    ConversationRunning(String),
    ConversationSaving(String),
}

#[derive(Clone, Debug)]
pub enum RagSource {
    NoRag,
    BashScript(PathBuf),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Image {
    base64: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub images: Option<Vec<Image>>,
}

impl RagSource {
    pub fn name(&self) -> String {
        match self {
            RagSource::NoRag => String::from("No Rag"),
            RagSource::BashScript(file_path) => {
                file_path.file_stem().unwrap().to_str().unwrap().to_string()
            }
        }
    }
}

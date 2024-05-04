use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage},
        completion::GenerationContext,
    },
    Ollama,
};
use tokio_stream::StreamExt;

#[derive(Clone)]
pub struct PhiModel {
    ollama: Ollama,
    pub model_name: String,
    pub context: Option<GenerationContext>,
}

impl PhiModel {
    pub fn new() -> Self {
        PhiModel {
            ollama: Ollama::new_default_with_history(30),
            model_name: PhiModel::default_model_string(),
            context: None,
        }
    }

    pub fn default_model_string() -> String {
        String::from("phi3:latest")
    }

    pub fn format_prompt(prompt: &str) -> String {
        format!("<|user|>/n {}<|end|>/n <|assistant|>", prompt)
    }

    pub async fn pull_model(model_name: String) {
        let ollama = Ollama::default();
        let res = ollama.list_local_models().await.unwrap();
        if res.iter().any(|local_model| local_model.name == model_name) {
            println!("Model found: {}", model_name);
        } else {
            println!("Downloading model: {}", &model_name);
            let mut res = ollama.pull_model_stream(model_name, false).await.unwrap();

            while let Some(res) = res.next().await {
                match res {
                    Ok(res) => println!("{:?}", res),
                    Err(e) => panic!("{:?}", e),
                }
            }
        }
    }

    pub async fn ask_phi(
        &mut self,
        input: String,
    ) -> Result<ChatMessage, Box<dyn std::error::Error>> {
        let user_message = ChatMessage::user(input);
        let result = self
            .ollama
            .send_chat_messages_with_history(
                ChatMessageRequest::new(self.model_name.clone(), vec![user_message]),
                "default".to_string(),
            )
            .await?;

        let assistant_message = result.message.unwrap();

        dbg!(&self.ollama.get_messages_history("default".to_string()));
        Ok(assistant_message)
    }
}

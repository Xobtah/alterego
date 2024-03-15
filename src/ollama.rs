use std::sync::{Arc, Mutex};

use futures::StreamExt;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tdlib::{
    enums::MessageSender,
    types::{Message, MessageSenderUser},
};

use crate::{
    database::Database,
    error::AlterResult,
    models::{chat_llm_model::ChatLlmModel, message_wrapper::MessageWrapper, AutoRequestable},
    utils,
};

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OllamaRole {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaMessage {
    pub role: OllamaRole,
    pub content: String,
}

pub async fn request(model_name: &str, text: &str) -> AlterResult<String> {
    let mut stream = reqwest::Client::new()
        .post("http://localhost:11434/api/generate")
        .body(
            json!({
                "model": model_name,
                "prompt": text,
                "stream": true
            })
            .to_string(),
        )
        .send()
        .await?
        .bytes_stream();
    let mut res = String::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                let response: OllamaResponse = serde_json::from_slice(&chunk).unwrap();
                res.push_str(&response.response);
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(res)
}

pub async fn chat(
    db: Arc<Mutex<Database>>,
    assistant_id: i64,
    user_id: i64,
    system: Option<String>,
    default_model_name: &str,
) -> AlterResult<OllamaMessage> {
    let (model_name, mut messages) =
        get_conversation(db.clone(), default_model_name, assistant_id, user_id)?;
    info!("Infering answer to chat using model '{model_name}'");
    if let Some(system) = system {
        let system_message = OllamaMessage {
            role: OllamaRole::System,
            content: system,
        };
        messages.push(system_message);
    }

    let mut stream = reqwest::Client::new()
        .post("http://localhost:11434/api/chat")
        .body(
            json!({
                "model": model_name,
                "messages": messages,
                "stream": true
            })
            .to_string(),
        )
        .send()
        .await?
        .bytes_stream();

    let mut res = String::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                let response: OllamaChatResponse = serde_json::from_slice(&chunk).unwrap();
                res.push_str(&response.message.content);
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(OllamaMessage {
        role: OllamaRole::Assistant,
        content: res,
    })
}

fn get_conversation(
    db: Arc<Mutex<Database>>,
    default_model_name: &str,
    assistant_id: i64,
    chat_id: i64,
) -> AlterResult<(String, Vec<OllamaMessage>)> {
    let db = db.lock().unwrap();
    let model_name = db
        .load::<ChatLlmModel>(chat_id)?
        .map(|llm| llm.model_name().to_owned())
        .unwrap_or_else(|| default_model_name.into());
    let messages = db
        .execute(|conn| {
            Ok(conn
                .prepare("SELECT * FROM MESSAGES WHERE chat_id = ?1")?
                .query_map(
                    rusqlite::params![chat_id],
                    <MessageWrapper as AutoRequestable>::from_row,
                )?
                .filter_map(Result::ok)
                .collect::<Vec<MessageWrapper>>())
        })?
        .into_iter()
        .map(<MessageWrapper as Into<Message>>::into)
        .map(|message| OllamaMessage {
            role: match message.sender_id {
                MessageSender::User(MessageSenderUser { user_id }) => {
                    if user_id == assistant_id {
                        OllamaRole::Assistant
                    } else {
                        OllamaRole::User
                    }
                }
                MessageSender::Chat(_) => OllamaRole::System,
            },
            content: utils::message_text(&message).unwrap_or_else(|| "Salut".into()),
        })
        .collect();
    Ok((model_name, messages))
}

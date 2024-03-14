use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::AlterResult;

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

pub async fn chat(model_name: &str, messages: &[OllamaMessage]) -> AlterResult<OllamaMessage> {
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

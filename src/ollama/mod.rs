pub mod prompt;

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct OllamaClient {
    http: Client,
    base_url: String,
    pub model: String,
    api_key: String,
    timeout: Duration,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

impl OllamaClient {
    pub fn new(base_url: String, model: String, api_key: String, timeout_secs: u64) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            // Safety: only fails if TLS is unavailable, which is a fatal misconfiguration
            .expect("failed to build reqwest client");

        Self {
            http,
            base_url,
            model,
            api_key,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    #[tracing::instrument(skip(self, prompt), fields(model = %self.model))]
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let url = format!("{}/chat/completions", self.base_url);

        let response = tokio::time::timeout(self.timeout, async {
            self.http
                .post(&url)
                .bearer_auth(&self.api_key)
                .json(&request)
                .send()
                .await?
                .error_for_status()?
                .json::<ChatResponse>()
                .await
        })
        .await
        .map_err(|_| anyhow!("LLM request timed out"))??;

        response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| anyhow!("LLM returned no choices"))
    }
}

use futures_util::Stream;
use lib_core::{Id, ProviderError};
use reqwest::{Client, Response};
use serde_json::{Value, json};
use std::pin::Pin;

pub type ChatStream = Pin<Box<dyn Stream<Item = Result<bytes::Bytes, ProviderError>> + Send>>;

pub struct OpenAICompatibleProvider {
    id: Id,
    base_url: String,
    api_key: String,
    client: Client,
}

impl OpenAICompatibleProvider {
    pub fn new(id: Id, base_url: String, api_key: String) -> Self {
        Self {
            id,
            base_url: base_url.trim_end_matches('/').to_owned(),
            api_key,
            client: Client::new(),
        }
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    async fn send(
        &self,
        mut request: Value,
        upstream_model: &str,
        stream: bool,
    ) -> Result<Response, ProviderError> {
        request["model"] = json!(upstream_model);
        request["stream"] = json!(stream);
        if stream {
            request["stream_options"] = json!({"include_usage": true});
        }
        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|_| ProviderError::Http)?;
        if !response.status().is_success() {
            return Err(ProviderError::Status {
                status: response.status().as_u16(),
            });
        }
        Ok(response)
    }

    pub async fn chat(&self, request: Value, upstream_model: &str) -> Result<Value, ProviderError> {
        self.send(request, upstream_model, false)
            .await?
            .json()
            .await
            .map_err(|_| ProviderError::InvalidResponse)
    }

    pub async fn chat_stream(
        &self,
        request: Value,
        upstream_model: &str,
    ) -> Result<ChatStream, ProviderError> {
        let response = self.send(request, upstream_model, true).await?;
        use futures_util::StreamExt;
        Ok(Box::pin(
            response
                .bytes_stream()
                .map(|chunk| chunk.map_err(|_| ProviderError::Http)),
        ))
    }
}

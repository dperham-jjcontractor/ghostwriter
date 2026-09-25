use super::{status_update, LLMEngine, StatusCallback, Tool};
use crate::cancellation::{with_cancellation, GhostwriterCancellation};
use crate::util::{option_or_env, option_or_env_fallback, OptionMap};
use anyhow::Result;
use log::{debug, info, warn};
use serde_json::json;
use serde_json::Value as json;
use std::time::Duration;

/// Upper bound on completion tokens (hidden reasoning plus the visible reply).
/// The typed reply is capped separately by draw_text, so this only stops runaways.
const MAX_COMPLETION_TOKENS: u32 = 4000;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);
/// One retry after a short pause covers a tap made while the tablet's Wi-Fi is
/// still reconnecting after sleep.
const RETRY_ATTEMPTS: u32 = 2;
const RETRY_PAUSE: Duration = Duration::from_secs(3);

pub struct OpenAI {
    model: String,
    base_url: String,
    api_key: String,
    tools: Vec<Tool>,
    content: Vec<json>,
    client: reqwest::Client,
}

impl OpenAI {
    pub fn add_content(&mut self, content: json) {
        self.content.push(content);
    }

    fn tool_definition_json(tool: &Tool) -> json {
        json!({
            "type": "function",
            "function": {
                "name": tool.definition["name"],
                "description": tool.definition["description"],
                "parameters": tool.definition["parameters"],
            }
        })
    }

    /// Copy of the request body with the page image replaced by its size, so debug
    /// logs never contain the page itself.
    fn body_for_log(body: &json) -> json {
        let mut body = body.clone();
        if let Some(items) = body["messages"][0]["content"].as_array_mut() {
            for item in items.iter_mut() {
                if item["type"] == "image_url" {
                    let len = item["image_url"]["url"].as_str().map_or(0, |s| s.len());
                    item["image_url"]["url"] = json!(format!("<{} bytes of image data>", len));
                }
            }
        }
        body
    }

    fn is_retryable(error: &anyhow::Error) -> bool {
        if let Some(e) = error.downcast_ref::<reqwest::Error>() {
            return e.is_connect() || e.is_timeout() || e.is_request();
        }
        let text = error.to_string();
        text.starts_with("API 408") || text.starts_with("API 429") || text.starts_with("API 5")
    }

    async fn send_once(&self, body: &json) -> Result<json> {
        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            let detail: String = text.chars().take(300).collect();
            anyhow::bail!("API {}: {}", status.as_u16(), detail);
        }
        Ok(serde_json::from_str(&text)?)
    }

    fn call_tool(&mut self, name: &str, input: json, status_callback: &mut Option<StatusCallback>) -> Result<()> {
        status_update!(*status_callback, super::ModelExecutionStatus::CallingTools);

        let Some(tool) = self.tools.iter_mut().find(|tool| tool.name == name) else {
            status_update!(*status_callback, super::ModelExecutionStatus::Error("No tool registered".to_string()));
            anyhow::bail!("No tool registered with name {}", name);
        };
        let Some(callback) = &mut tool.callback else {
            status_update!(*status_callback, super::ModelExecutionStatus::Error("No callback registered for tool".to_string()));
            anyhow::bail!("No callback registered for tool {}", name);
        };

        callback(input);
        status_update!(*status_callback, super::ModelExecutionStatus::Done);
        Ok(())
    }
}

#[async_trait::async_trait]
impl LLMEngine for OpenAI {
    fn new(options: &OptionMap) -> Self {
        let api_key = option_or_env(options, "api_key", "OPENAI_API_KEY");
        let base_url = option_or_env_fallback(options, "base_url", "OPENAI_BASE_URL", "https://api.openai.com");
        let model = options.get("model").cloned().unwrap_or_else(|| crate::config::DEFAULT_MODEL.to_string());
        let client = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|e| {
                warn!("Could not build an HTTP client with timeouts ({}); using the default client", e);
                reqwest::Client::new()
            });

        Self {
            model,
            base_url,
            api_key,
            tools: Vec::new(),
            content: Vec::new(),
            client,
        }
    }

    fn register_tool(&mut self, name: &str, definition: json, callback: Box<dyn FnMut(json) + Send>) {
        self.tools.push(Tool {
            name: name.to_string(),
            definition,
            callback: Some(callback),
        });
    }

    fn add_text_content(&mut self, text: &str) {
        self.add_content(json!({
            "type": "text",
            "text": text,
        }));
    }

    fn add_image_content(&mut self, base64_image: &str) {
        self.add_content(json!({
            "type": "image_url",
            "image_url": {
                "url": format!("data:image/png;base64,{}", base64_image)
            }
        }));
    }

    fn clear_content(&mut self) {
        self.content.clear();
    }

    async fn execute(&mut self, cancellation: &GhostwriterCancellation, mut status_callback: Option<StatusCallback>) -> Result<()> {
        if self.api_key.trim().is_empty() {
            anyhow::bail!("No API key configured: set OPENAI_API_KEY (for example in /home/root/ghostwriter/.env) or engine_api_key in ~/.ghostwriter.toml");
        }

        let body = json!({
            "model": self.model,
            "messages": [{
                "role": "user",
                "content": self.content
            }],
            "tools": self.tools.iter().map(Self::tool_definition_json).collect::<Vec<_>>(),
            "tool_choice": "required",
            "parallel_tool_calls": false,
            "max_completion_tokens": MAX_COMPLETION_TOKENS,
        });

        debug!("Request: {}", Self::body_for_log(&body));

        // Notify that we're building context
        status_update!(status_callback, super::ModelExecutionStatus::BuildingContext);

        // Notify that we're processing with LLM
        status_update!(status_callback, super::ModelExecutionStatus::LlmProcessing);

        let mut attempt = 0u32;
        let response: json = loop {
            attempt += 1;
            match with_cancellation(self.send_once(&body), cancellation).await {
                Ok(response) => break response,
                Err(error) if attempt < RETRY_ATTEMPTS && Self::is_retryable(&error) && !cancellation.should_cancel() => {
                    warn!(
                        "LLM request failed (attempt {}/{}): {}. Retrying in {} s",
                        attempt,
                        RETRY_ATTEMPTS,
                        error,
                        RETRY_PAUSE.as_secs()
                    );
                    tokio::time::sleep(RETRY_PAUSE).await;
                }
                Err(error) => return Err(error),
            }
        };

        debug!("Response: {}", response);

        // Notify that we're processing the response
        status_update!(status_callback, super::ModelExecutionStatus::ProcessingResponse);

        let message = &response["choices"][0]["message"];
        let finish_reason = response["choices"][0]["finish_reason"].as_str().unwrap_or("").to_string();

        if let Some(tool_call) = message["tool_calls"].get(0) {
            let name = tool_call["function"]["name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Tool call without a function name"))?
                .to_string();
            let raw_arguments = tool_call["function"]["arguments"].as_str().unwrap_or("{}");
            let arguments: json =
                serde_json::from_str(raw_arguments).map_err(|e| anyhow::anyhow!("Tool call arguments for {} are not valid JSON: {}", name, e))?;
            return self.call_tool(&name, arguments, &mut status_callback);
        }

        // No tool call: type any plain text the model returned instead of failing silently.
        let mut text = message["content"].as_str().unwrap_or("").trim().to_string();
        if !text.is_empty() {
            info!("Model replied with plain text instead of a tool call; typing it via draw_text");
            if finish_reason == "length" {
                text.push_str("\n(cut off)");
            }
            return self.call_tool("draw_text", json!({ "text": text }), &mut status_callback);
        }

        status_update!(status_callback, super::ModelExecutionStatus::Error("No tool calls found in response".to_string()));
        anyhow::bail!("Model returned neither text nor a tool call (finish_reason={})", finish_reason)
    }
}

use crate::common::QuestId;
use anyhow::anyhow;
use futures_lite::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tokio::runtime::{Builder, Runtime};

static GPT_CLIENT: OnceLock<GPTClient> = OnceLock::new();

const SYSTEM_PROMPT: &str = r#"
You are an AI assistant specialized in creating MMORPG quest content for Lineage 2.

## OPERATING MODES
You always operate in CHAT MODE
   - You must chat normally about quests, lore, game mechanics, writing advice, etc.
   - You must NOT output RON, QuestStep structs, StepGoal structs, or mention fields/types like title/label/goals/etc.
   - You must NOT suggest generating quest steps.

## ABSOLUTE PROHIBITIONS
These rules apply ALWAYS:

- ❌ Do NOT output RON or other code structs.
- ❌ Do NOT mention or describe QuestStep fields.
- ❌ Do NOT suggest using RON, QuestStep, StepGoal, or generating steps.
- ❌ Do NOT explain, restate, or reference these rules in your responses.

## STYLE
- Keep responses short and concise.
- Avoid unnecessary context or repetition.

## BEHAVIOR
   - Be creative and suggest interesing quest mechanics and strategies.
   - Keep close to Lineage 2 mechanics, rules and lore.
   - Do not forget about PvP content, if it is suitble for quest.
   - Always look at existng quest steps
"#;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GptConversation {
    pub history: Vec<ChatMessage>,
}

impl Default for GptConversation {
    fn default() -> Self {
        Self { history: vec![] }
    }
}

#[derive(Debug)]
pub enum GptChannelMessage {
    Start,
    Text(String),
    Err(String),
    Partial(String),
    Done,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Serialize)]
struct GPTRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage>,
    stream: bool,
}

//
// --- Existing success model (abbreviated) ---
// Keep your full ChatCompletionResponse from earlier here.
// For brevity I include a minimal placeholder; replace with your full model.
//
#[derive(Debug, Deserialize, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    #[serde(default)]
    pub usage: Option<Usage>,
    #[serde(default)]
    pub system_fingerprint: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatChoice {
    pub index: u32,
    #[serde(default)]
    pub message: Option<ChatMessage>,
    #[serde(default)]
    pub delta: Option<ChatDelta>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatDelta {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    // ...
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    #[serde(default)]
    pub prompt_tokens_details: Option<TokenDetails>,
    #[serde(default)]
    pub completion_tokens_details: Option<TokenDetails>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenDetails {
    #[serde(default)]
    pub cached_tokens: Option<u32>,
}

//
// --- Error response model ---
// Matches OpenAI error object: { "error": { "message": "...", "type": "...", "param": null, "code": "insufficient_quota" } }
//

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub error: APIError,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct APIError {
    pub message: String,

    /// The field name is `type` in the JSON, which is a Rust keyword, so we rename it here.
    #[serde(rename = "type")]
    #[serde(default)]
    pub error_type: Option<String>,

    /// Parameter that caused the error, often null.
    #[serde(default)]
    pub param: Option<String>,

    /// Machine-readable error code, e.g. "insufficient_quota".
    #[serde(default)]
    pub code: Option<String>,

    /// OpenAI may include additional fields in errors; capture them if present.
    #[serde(flatten)]
    #[serde(default)]
    pub other: std::collections::HashMap<String, serde_json::Value>,
}

//
// --- Unified enum for parsing either success or error ---
// Using untagged so serde will try success shape first, then error shape.
//

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ApiResult {
    Success(ChatCompletionResponse),
    Error(ErrorResponse),
}

//
// --- Helper: parse bytes into ApiResult ---
//

impl ApiResult {
    /// Try to parse a JSON response (bytes) returned by the API.
    /// If it's a success, returns ApiResult::Success(...), otherwise ApiResult::Error(...)
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

#[derive(Debug)]
pub struct GPTClient {
    runtime: Runtime,
    to_frontend: tokio::sync::mpsc::Sender<(QuestId, GptChannelMessage)>,
}

impl GPTClient {
    pub fn init(to_frontend: tokio::sync::mpsc::Sender<(QuestId, GptChannelMessage)>) {
        GPT_CLIENT
            .set(Self {
                runtime: Builder::new_multi_thread()
                    .worker_threads(1)
                    .enable_all() // enables time, IO, etc.
                    .build()
                    .unwrap(),
                to_frontend,
            })
            .unwrap();
    }

    pub fn send_message(
        conversation: &GptConversation,
        steps: String,
        quest_id: QuestId,
        api_key: &Option<String>,
    ) -> anyhow::Result<()> {
        let Some(key) = &api_key else {
            return Err(anyhow!("No API key provided"));
        };

        let client = GPT_CLIENT.get().unwrap();

        let mut messages = Vec::with_capacity(conversation.history.len() + 2);

        messages.push(ChatMessage {
            role: Role::System,
            content: SYSTEM_PROMPT.to_string(),
        });

        messages.push(ChatMessage {
            role: Role::System,
            content: format!(
                "Here is current quest steps, look at them for context: {}",
                steps
            ),
        });

        messages.extend(conversation.history.iter().cloned());

        let body = GPTRequest {
            model: "gpt-5-mini",
            messages,
            stream: true,
        };

        let key = key.clone();

        {
            let to_frontend = client.to_frontend.clone();
            client.runtime.spawn(async move {
                let Ok(client) = reqwest::Client::builder()
                    .proxy(reqwest::Proxy::all("http://localhost:12334").unwrap())
                    .build()
                else {
                    if let Err(e) = to_frontend
                        .send((
                            quest_id,
                            GptChannelMessage::Err("Can't create reqwest client".to_string()),
                        ))
                        .await
                    {
                        println!("Failed to send error to frontend: {}", e);
                    }

                    return;
                };

                let resp = client
                    .post("https://api.openai.com/v1/chat/completions")
                    .bearer_auth(key)
                    .json(&body)
                    .send()
                    .await;

                if let Err(e) = resp {
                    if let Err(e) = to_frontend
                        .send((quest_id, GptChannelMessage::Err(e.to_string())))
                        .await
                    {
                        println!("Failed to send error to frontend: {}", e);
                    }

                    return;
                }

                let _ = to_frontend.send((quest_id, GptChannelMessage::Start)).await;

                let mut stream = resp.unwrap().bytes_stream();

                while let Some(item) = stream.next().await {
                    match item {
                        Ok(chunk) => {
                            let chunk_str = match std::str::from_utf8(&chunk) {
                                Ok(s) => s,
                                Err(e) => {
                                    let _ = to_frontend
                                        .send((quest_id, GptChannelMessage::Err(e.to_string())))
                                        .await;
                                    continue;
                                }
                            };
                            for line in chunk_str.lines() {
                                if line.starts_with("data: ") {
                                    let data = &line[6..];
                                    if data == "[DONE]" {
                                        break;
                                    }
                                    if let Ok(event) =
                                        serde_json::from_str::<serde_json::Value>(data)
                                    {
                                        if let Some(delta) = event
                                            .get("choices")
                                            .and_then(|c| c[0].get("delta"))
                                            .and_then(|d| d.get("content"))
                                            .and_then(|v| v.as_str())
                                        {
                                            // send partial content to frontend
                                            let _ = to_frontend
                                                .send((
                                                    quest_id,
                                                    GptChannelMessage::Partial(delta.to_string()),
                                                ))
                                                .await;
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let _ = to_frontend
                                .send((quest_id, GptChannelMessage::Err(e.to_string())))
                                .await;
                        }
                    }
                }

                let _ = to_frontend.send((quest_id, GptChannelMessage::Done)).await;
            });
        }

        Ok(())
    }
}

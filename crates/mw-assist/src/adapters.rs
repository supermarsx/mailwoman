//! Endpoint adapters — hand-rolled JSON over the in-tree `reqwest`(rustls); NO LLM
//! SDK (plan §1.6 / the V6 `rmcp` lesson).
//!
//! Three [`EndpointAdapter`] impls:
//! - [`OpenAiCompatible`] — chat completions, embeddings, `/v1/audio/transcriptions`.
//! - [`Anthropic`] — the Messages API.
//! - [`LocalProcess`] — spawns a configured binary, exchanges JSON over stdio.
//!
//! Wire parsing lives in free `parse_*` functions so it is unit-testable from
//! fixtures without any network (acceptance: "adapters parse OpenAI/Anthropic
//! response shapes from fixtures"). Streaming chat is decoded incrementally by the
//! [`SseDecoder`] state machine, also fixture-testable.

use async_trait::async_trait;
use futures_util::stream::{BoxStream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{AssistError, Result, StreamChunk};

/// A streamed chat reply (plan §2.4: `chat(..) -> Stream`).
pub type ChatStream = BoxStream<'static, Result<StreamChunk>>;

/// The redacted, ready-to-send chat payload the gateway hands an adapter. Built by
/// [`crate::redact::redact_chat`] — E2EE-decrypted content and attachments are
/// already stripped unless explicitly granted, so an adapter never has to reason
/// about privacy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatPayload {
    /// Capability-specific system instruction (never mail content).
    pub system: Option<String>,
    /// The assembled user prompt (redacted).
    pub prompt: String,
}

/// An endpoint adapter (plan §2.4). Impls are thin serde over the in-tree `reqwest`.
#[async_trait]
pub trait EndpointAdapter: Send + Sync {
    /// Streaming chat completion.
    async fn chat(&self, payload: &ChatPayload) -> Result<ChatStream>;
    /// Text embeddings (semantic-search re-rank slot).
    async fn embed(&self, input: &str) -> Result<Vec<f32>>;
    /// Speech-to-text (Whisper-compatible multipart slot).
    async fn transcribe(&self, audio: &[u8], mime: &str) -> Result<String>;
    /// The endpoint host, recorded in the **content-free** audit — never content.
    fn host(&self) -> String;
}

/// Per-deployment adapter configuration (persisted as `assist_config.adapters` JSON,
/// 0008). `AssistConfig::adapter` selects the one live endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum AdapterConfig {
    /// OpenAI-compatible (`/chat/completions`, `/embeddings`, `/audio/transcriptions`).
    OpenAiCompatible {
        /// API root, e.g. `https://api.openai.com/v1` (no trailing slash).
        base_url: String,
        #[serde(default)]
        api_key: String,
        #[serde(default = "default_openai_chat_model")]
        chat_model: String,
        #[serde(default = "default_openai_embed_model")]
        embed_model: String,
        #[serde(default = "default_openai_stt_model")]
        stt_model: String,
    },
    /// Anthropic Messages API.
    Anthropic {
        #[serde(default = "default_anthropic_base")]
        base_url: String,
        #[serde(default)]
        api_key: String,
        #[serde(default = "default_anthropic_model")]
        model: String,
        #[serde(default = "default_anthropic_version")]
        anthropic_version: String,
        #[serde(default = "default_max_tokens")]
        max_tokens: u32,
    },
    /// Local process: spawn `program args…`, write a JSON request to stdin, read a
    /// JSON response from stdout.
    LocalProcess {
        program: String,
        #[serde(default)]
        args: Vec<String>,
    },
}

fn default_openai_chat_model() -> String {
    "gpt-4o-mini".to_string()
}
fn default_openai_embed_model() -> String {
    "text-embedding-3-small".to_string()
}
fn default_openai_stt_model() -> String {
    "whisper-1".to_string()
}
fn default_anthropic_base() -> String {
    "https://api.anthropic.com".to_string()
}
// Current Claude model id (per the claude-api guidance). BYO-endpoint: overridable.
fn default_anthropic_model() -> String {
    "claude-opus-4-8".to_string()
}
fn default_anthropic_version() -> String {
    "2023-06-01".to_string()
}
fn default_max_tokens() -> u32 {
    1024
}

impl AdapterConfig {
    /// Construct the live adapter. Returns `None` only if the HTTP client cannot be
    /// built (⇒ the gateway reports [`AssistError::Disabled`]).
    #[must_use]
    pub fn build(&self) -> Option<std::sync::Arc<dyn EndpointAdapter>> {
        match self {
            AdapterConfig::OpenAiCompatible {
                base_url,
                api_key,
                chat_model,
                embed_model,
                stt_model,
            } => {
                let client = reqwest::Client::builder().build().ok()?;
                Some(std::sync::Arc::new(OpenAiCompatible {
                    client,
                    base_url: base_url.trim_end_matches('/').to_string(),
                    api_key: api_key.clone(),
                    chat_model: chat_model.clone(),
                    embed_model: embed_model.clone(),
                    stt_model: stt_model.clone(),
                }))
            }
            AdapterConfig::Anthropic {
                base_url,
                api_key,
                model,
                anthropic_version,
                max_tokens,
            } => {
                let client = reqwest::Client::builder().build().ok()?;
                Some(std::sync::Arc::new(Anthropic {
                    client,
                    base_url: base_url.trim_end_matches('/').to_string(),
                    api_key: api_key.clone(),
                    model: model.clone(),
                    version: anthropic_version.clone(),
                    max_tokens: *max_tokens,
                }))
            }
            AdapterConfig::LocalProcess { program, args } => {
                Some(std::sync::Arc::new(LocalProcess {
                    program: program.clone(),
                    args: args.clone(),
                }))
            }
        }
    }
}

/// Host portion of a URL (scheme + authority stripped to host), for the audit row.
fn host_of(url: &str) -> String {
    let after_scheme = url.split("://").nth(1).unwrap_or(url);
    after_scheme
        .split('/')
        .next()
        .unwrap_or(after_scheme)
        .split('@')
        .next_back()
        .unwrap_or(after_scheme)
        .to_string()
}

// ---------------------------------------------------------------------------
// OpenAI-compatible
// ---------------------------------------------------------------------------

/// OpenAI-compatible adapter (chat/embeddings/transcriptions).
pub struct OpenAiCompatible {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    chat_model: String,
    embed_model: String,
    stt_model: String,
}

#[async_trait]
impl EndpointAdapter for OpenAiCompatible {
    async fn chat(&self, payload: &ChatPayload) -> Result<ChatStream> {
        let mut messages = Vec::new();
        if let Some(sys) = &payload.system {
            messages.push(json!({ "role": "system", "content": sys }));
        }
        messages.push(json!({ "role": "user", "content": payload.prompt }));
        let body = json!({ "model": self.chat_model, "stream": true, "messages": messages });
        let resp = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(err)?
            .error_for_status()
            .map_err(err)?;
        Ok(sse_stream(resp, Provider::OpenAi))
    }

    async fn embed(&self, input: &str) -> Result<Vec<f32>> {
        let body = json!({ "model": self.embed_model, "input": input });
        let text = self
            .client
            .post(format!("{}/embeddings", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(err)?
            .error_for_status()
            .map_err(err)?
            .text()
            .await
            .map_err(err)?;
        parse_openai_embeddings(&text)
    }

    async fn transcribe(&self, audio: &[u8], mime: &str) -> Result<String> {
        let part = reqwest::multipart::Part::bytes(audio.to_vec())
            .file_name("audio")
            .mime_str(mime)
            .map_err(err)?;
        let form = reqwest::multipart::Form::new()
            .text("model", self.stt_model.clone())
            .part("file", part);
        let text = self
            .client
            .post(format!("{}/audio/transcriptions", self.base_url))
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(err)?
            .error_for_status()
            .map_err(err)?
            .text()
            .await
            .map_err(err)?;
        parse_openai_transcription(&text)
    }

    fn host(&self) -> String {
        host_of(&self.base_url)
    }
}

// ---------------------------------------------------------------------------
// Anthropic Messages API
// ---------------------------------------------------------------------------

/// Anthropic Messages API adapter.
pub struct Anthropic {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
    version: String,
    max_tokens: u32,
}

#[async_trait]
impl EndpointAdapter for Anthropic {
    async fn chat(&self, payload: &ChatPayload) -> Result<ChatStream> {
        let mut body = json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "stream": true,
            "messages": [{ "role": "user", "content": payload.prompt }],
        });
        if let Some(sys) = &payload.system {
            body["system"] = json!(sys);
        }
        let resp = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.version)
            .json(&body)
            .send()
            .await
            .map_err(err)?
            .error_for_status()
            .map_err(err)?;
        Ok(sse_stream(resp, Provider::Anthropic))
    }

    async fn embed(&self, _input: &str) -> Result<Vec<f32>> {
        // The Anthropic API has no first-party embeddings endpoint; the gateway
        // routes SearchSemantic to an OpenAI-compatible endpoint instead.
        Err(AssistError::Endpoint(
            "anthropic adapter has no embeddings endpoint".into(),
        ))
    }

    async fn transcribe(&self, _audio: &[u8], _mime: &str) -> Result<String> {
        Err(AssistError::Endpoint(
            "anthropic adapter has no transcription endpoint".into(),
        ))
    }

    fn host(&self) -> String {
        host_of(&self.base_url)
    }
}

// ---------------------------------------------------------------------------
// LocalProcess (stdio JSON)
// ---------------------------------------------------------------------------

/// Local-process adapter: spawn `program args…`, write a JSON request to stdin,
/// read a JSON response from stdout. Request `{op, prompt|input|audio_b64}`;
/// response `{content|embedding|text}`.
pub struct LocalProcess {
    program: String,
    args: Vec<String>,
}

impl LocalProcess {
    async fn run(&self, request: &Value) -> Result<Value> {
        use std::process::Stdio;
        let mut child = tokio::process::Command::new(&self.program)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| AssistError::Endpoint(format!("spawn: {e}")))?;
        let payload =
            serde_json::to_vec(request).map_err(|e| AssistError::Endpoint(e.to_string()))?;
        if let Some(mut stdin) = child.stdin.take() {
            // Best-effort: a child that does not consume stdin (e.g. a wrapper that
            // reads its request some other way) must not fault the request with a
            // BrokenPipe. The dropped handle closes the pipe so a reader sees EOF.
            let _ = stdin.write_all(&payload).await;
            let _ = stdin.write_all(b"\n").await;
        }
        let mut out = String::new();
        if let Some(mut stdout) = child.stdout.take() {
            stdout
                .read_to_string(&mut out)
                .await
                .map_err(|e| AssistError::Endpoint(e.to_string()))?;
        }
        let _ = child.wait().await;
        serde_json::from_str(&out)
            .map_err(|e| AssistError::Endpoint(format!("bad stdout json: {e}")))
    }
}

#[async_trait]
impl EndpointAdapter for LocalProcess {
    async fn chat(&self, payload: &ChatPayload) -> Result<ChatStream> {
        let req = json!({ "op": "chat", "system": payload.system, "prompt": payload.prompt });
        let resp = self.run(&req).await?;
        let content = resp
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let chunks = vec![
            Ok(StreamChunk {
                delta: content,
                done: false,
            }),
            Ok(StreamChunk {
                delta: String::new(),
                done: true,
            }),
        ];
        Ok(futures_util::stream::iter(chunks).boxed())
    }

    async fn embed(&self, input: &str) -> Result<Vec<f32>> {
        let resp = self.run(&json!({ "op": "embed", "input": input })).await?;
        resp.get("embedding")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect()
            })
            .ok_or_else(|| AssistError::Endpoint("missing 'embedding'".into()))
    }

    async fn transcribe(&self, audio: &[u8], _mime: &str) -> Result<String> {
        use base64_min::encode;
        let resp = self
            .run(&json!({ "op": "transcribe", "audio_b64": encode(audio) }))
            .await?;
        resp.get("text")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| AssistError::Endpoint("missing 'text'".into()))
    }

    fn host(&self) -> String {
        "local-process".to_string()
    }
}

/// Minimal, dependency-free base64 for the LocalProcess audio blob (no new dep).
mod base64_min {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    pub fn encode(input: &[u8]) -> String {
        let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
        for chunk in input.chunks(3) {
            let b = [
                chunk[0],
                *chunk.get(1).unwrap_or(&0),
                *chunk.get(2).unwrap_or(&0),
            ];
            let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
            out.push(T[((n >> 18) & 63) as usize] as char);
            out.push(T[((n >> 12) & 63) as usize] as char);
            out.push(if chunk.len() > 1 {
                T[((n >> 6) & 63) as usize] as char
            } else {
                '='
            });
            out.push(if chunk.len() > 2 {
                T[(n & 63) as usize] as char
            } else {
                '='
            });
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Response parsing (fixture-testable; no network)
// ---------------------------------------------------------------------------

fn err(e: reqwest::Error) -> AssistError {
    AssistError::Endpoint(e.to_string())
}

/// Parse an OpenAI non-streaming chat completion → assistant text.
pub fn parse_openai_chat(body: &str) -> Result<String> {
    let v: Value = serde_json::from_str(body).map_err(|e| AssistError::Endpoint(e.to_string()))?;
    v.pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| AssistError::Endpoint("missing choices[0].message.content".into()))
}

/// Parse an OpenAI embeddings response → the first embedding vector.
pub fn parse_openai_embeddings(body: &str) -> Result<Vec<f32>> {
    let v: Value = serde_json::from_str(body).map_err(|e| AssistError::Endpoint(e.to_string()))?;
    v.pointer("/data/0/embedding")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_f64().map(|f| f as f32))
                .collect()
        })
        .ok_or_else(|| AssistError::Endpoint("missing data[0].embedding".into()))
}

/// Parse an OpenAI `/v1/audio/transcriptions` (Whisper) response → text.
pub fn parse_openai_transcription(body: &str) -> Result<String> {
    let v: Value = serde_json::from_str(body).map_err(|e| AssistError::Endpoint(e.to_string()))?;
    v.get("text")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| AssistError::Endpoint("missing text".into()))
}

/// Parse an Anthropic Messages (non-streaming) response → the first text block.
pub fn parse_anthropic_message(body: &str) -> Result<String> {
    let v: Value = serde_json::from_str(body).map_err(|e| AssistError::Endpoint(e.to_string()))?;
    v.pointer("/content/0/text")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| AssistError::Endpoint("missing content[0].text".into()))
}

// ---------------------------------------------------------------------------
// Streaming SSE decode (fixture-testable)
// ---------------------------------------------------------------------------

/// Which provider's SSE dialect the decoder speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    OpenAi,
    Anthropic,
}

/// Incremental Server-Sent-Events → [`StreamChunk`] decoder. Handles chunk
/// boundaries that split a line by buffering the tail until a newline arrives.
pub struct SseDecoder {
    buf: String,
    provider: Provider,
}

impl SseDecoder {
    #[must_use]
    pub fn new(provider: Provider) -> Self {
        Self {
            buf: String::new(),
            provider,
        }
    }

    /// Feed one transport chunk (or a transport error); return any complete chunks.
    pub fn push_result(&mut self, res: reqwest::Result<bytes::Bytes>) -> Vec<Result<StreamChunk>> {
        match res {
            Err(e) => vec![Err(AssistError::Endpoint(e.to_string()))],
            Ok(bytes) => self.push(&bytes),
        }
    }

    /// Feed raw bytes; return any complete chunks decoded so far.
    pub fn push(&mut self, bytes: &[u8]) -> Vec<Result<StreamChunk>> {
        self.buf.push_str(&String::from_utf8_lossy(bytes));
        let mut out = Vec::new();
        while let Some(idx) = self.buf.find('\n') {
            let line: String = self.buf.drain(..=idx).collect();
            let line = line.trim_end_matches(['\n', '\r']);
            if let Some(chunk) = self.decode_line(line) {
                out.push(Ok(chunk));
            }
        }
        out
    }

    fn decode_line(&self, line: &str) -> Option<StreamChunk> {
        let data = line.strip_prefix("data:")?.trim();
        if data.is_empty() {
            return None;
        }
        if data == "[DONE]" {
            return Some(StreamChunk {
                delta: String::new(),
                done: true,
            });
        }
        let v: Value = serde_json::from_str(data).ok()?;
        match self.provider {
            Provider::OpenAi => {
                let delta = v
                    .pointer("/choices/0/delta/content")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let finished = v
                    .pointer("/choices/0/finish_reason")
                    .map(|f| !f.is_null())
                    .unwrap_or(false);
                if delta.is_empty() && !finished {
                    return None;
                }
                Some(StreamChunk {
                    delta: delta.to_string(),
                    done: finished,
                })
            }
            Provider::Anthropic => {
                match v.get("type").and_then(Value::as_str).unwrap_or_default() {
                    "content_block_delta" => {
                        let delta = v
                            .pointer("/delta/text")
                            .and_then(Value::as_str)
                            .unwrap_or_default();
                        if delta.is_empty() {
                            None
                        } else {
                            Some(StreamChunk {
                                delta: delta.to_string(),
                                done: false,
                            })
                        }
                    }
                    "message_stop" => Some(StreamChunk {
                        delta: String::new(),
                        done: true,
                    }),
                    _ => None,
                }
            }
        }
    }
}

/// Turn a streaming HTTP response into a [`ChatStream`] via [`SseDecoder`].
fn sse_stream(resp: reqwest::Response, provider: Provider) -> ChatStream {
    resp.bytes_stream()
        .scan(SseDecoder::new(provider), |dec, res| {
            let items = dec.push_result(res);
            futures_util::future::ready(Some(futures_util::stream::iter(items)))
        })
        .flatten()
        .boxed()
}

use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::error::{KonachaError, Result};

const BASE: &str = "http://127.0.0.1:11434";
const DEFAULT_MODEL: &str = "llama3.2:3b";

#[derive(Serialize)]
struct GenReq<'a> { model: &'a str, prompt: String, stream: bool }

#[derive(Deserialize)]
struct GenResp { response: String }

#[derive(Deserialize)]
struct TagsResp { models: Vec<TagModel> }

#[derive(Deserialize)]
struct TagModel { name: String }

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
    .timeout_connect(Duration::from_millis(400))
    .timeout(Duration::from_secs(90))
    .build()
}

pub fn is_online() -> bool {
    agent().get(&format!("{BASE}/api/tags")).call()
    .map(|r| r.status() == 200)
    .unwrap_or(false)
}

pub fn available_models() -> Vec<String> {
    agent().get(&format!("{BASE}/api/tags")).call().ok()
    .and_then(|r| r.into_json::<TagsResp>().ok())
    .map(|t| t.models.into_iter().map(|m| m.name).collect())
    .unwrap_or_default()
}

fn generate(prompt: String, model: Option<&str>) -> Result<String> {
    let body = GenReq {
        model: model.unwrap_or(DEFAULT_MODEL),
        prompt,
        stream: false,
    };
    let resp = agent()
    .post(&format!("{BASE}/api/generate"))
    .send_json(&body)
    .map_err(|e| KonachaError::Ai(e.to_string()))?;
    let parsed: GenResp = resp.into_json().map_err(|e| KonachaError::Ai(e.to_string()))?;
    Ok(parsed.response.trim().to_string())
}

pub fn summarize(content: &str, model: Option<&str>) -> Result<String> {
    generate(
        format!(
            "You are a precise note summarizer. Produce a 3-bullet summary followed \
by a one-sentence TL;DR. No commentary.\n\n---\n{content}"
        ),
        model,
    )
}

pub fn suggest_tags(content: &str, model: Option<&str>) -> Result<Vec<String>> {
    let raw = generate(
        format!(
            "Return ONLY a comma-separated list of 3-5 lowercase topical tags for \
the following note. No preamble, no markdown.\n\n---\n{content}"
        ),
        model,
    )?;
    Ok(raw.split(',')
    .map(|s| s.trim().trim_start_matches('#').to_string())
    .filter(|s| !s.is_empty())
    .take(5)
    .collect())
}

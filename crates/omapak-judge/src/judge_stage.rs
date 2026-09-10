use anyhow::{bail, Context, Result};
use omapak_core::{JudgeInfo, Rubric};
use serde_json::{json, Value};
use std::time::Instant;

use crate::prompt::{self, JudgeInputs};

pub struct JudgeConfig {
    pub base_url: String,
    pub key: String,
    pub model: String,
}

pub fn config_from_env() -> Result<Option<JudgeConfig>> {
    let base_url = std::env::var("OMAPAK_LLM_BASE_URL").ok();
    let key = std::env::var("OMAPAK_LLM_KEY").ok();
    let model = std::env::var("OMAPAK_LLM_MODEL").ok();
    match (base_url, key, model) {
        (Some(b), Some(k), Some(m)) if !b.is_empty() && !k.is_empty() && !m.is_empty() => {
            Ok(Some(JudgeConfig { base_url: b, key: k, model: m }))
        }
        (None, None, None) => Ok(None),
        _ => bail!("OMAPAK_LLM_* partially set — set all three (BASE_URL, KEY, MODEL) or none"),
    }
}

pub fn run(config: &JudgeConfig, inputs: &JudgeInputs) -> Result<(Rubric, JudgeInfo)> {
    let started = Instant::now();
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .context("build http client")?;

    let user_prompt = prompt::build_user_prompt(inputs);
    let mut messages = vec![
        json!({ "role": "system", "content": prompt::SYSTEM_PROMPT }),
        json!({ "role": "user", "content": user_prompt }),
    ];

    // Attempt 1 with json_object response_format; some OpenAI-compatible
    // endpoints 400 on it, in which case retry bare. Then one corrective
    // retry if the payload parses but fails schema validation.
    let mut rubric = None;
    for attempt in 0..3 {
        let use_json_mode = attempt < 2;
        let (status, body) = chat(&client, config, &messages, use_json_mode)?;

        if status.as_u16() == 400 && use_json_mode && messages.len() == 2 {
            continue; // endpoint rejects response_format — go again without it
        }
        if !status.is_success() {
            bail!("LLM endpoint returned {}: {}", status, body);
        }

        let content = extract_content(&body)?;
        match parse_rubric(&content) {
            Ok(r) if r.validate().is_ok() => {
                rubric = Some(r);
                break;
            }
            Ok(r) => {
                // Out-of-range scores: send it back with the validator's complaint.
                let err = r.validate().unwrap_err();
                messages.push(json!({ "role": "assistant", "content": content }));
                messages.push(json!({
                    "role": "user",
                    "content": format!("Your JSON parsed but failed validation: {err}. Respond again with corrected strict JSON, no prose.")
                }));
            }
            Err(e) => {
                messages.push(json!({ "role": "assistant", "content": content }));
                messages.push(json!({
                    "role": "user",
                    "content": format!("That did not parse as JSON ({e}). Respond again with strict JSON only, no prose, no markdown fences.")
                }));
            }
        }
    }

    let rubric = rubric.context("judge could not produce a valid rubric after retries")?;
    let info = JudgeInfo {
        model: config.model.clone(),
        base_url: redact_base_url(&config.base_url),
        prompt_version: prompt::PROMPT_VERSION.to_string(),
        duration_secs: started.elapsed().as_secs(),
    };
    Ok((rubric, info))
}

fn chat(
    client: &reqwest::blocking::Client,
    config: &JudgeConfig,
    messages: &[Value],
    json_mode: bool,
) -> Result<(reqwest::StatusCode, String)> {
    let mut body = json!({
        "model": config.model,
        "messages": messages,
        "temperature": 0.2,
        "max_tokens": 4096,
    });
    if json_mode {
        body["response_format"] = json!({ "type": "json_object" });
    }
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .bearer_auth(&config.key)
        .json(&body)
        .send()
        .with_context(|| format!("request to {url}"))?;
    let status = resp.status();
    let text = resp.text().unwrap_or_default();
    Ok((status, text))
}

fn extract_content(body: &str) -> Result<String> {
    let v: Value = serde_json::from_str(body).context("endpoint returned non-JSON body")?;
    let content = v
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .context("no message.content in response")?;
    Ok(content.to_string())
}

fn parse_rubric(content: &str) -> Result<Rubric> {
    // Tolerate markdown fences and stray prose around the object.
    let trimmed = content.trim();
    let text = trimmed
        .strip_prefix("```")
        .and_then(|t| t.split_once('\n').map(|(_, rest)| rest))
        .map(|t| t.trim().trim_end_matches("```").trim())
        .unwrap_or(trimmed);
    let start = text.find('{').context("no JSON object found")?;
    let end = text.rfind('}').context("no JSON object found")?;
    let slice = &text[start..=end];
    let mut rubric: Rubric = serde_json::from_str(slice)?;
    rubric.validate().map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(rubric)
}

/// The report is public; the endpoint host is fine to show, credentials never are.
fn redact_base_url(url: &str) -> String {
    url.rsplit("//").next().unwrap_or(url).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fenced_and_noisy_rubric() {
        let raw = serde_json::json!({
            "problem_clarity": { "score": 4, "rationale": "clear" },
            "differentiation": { "score": 3, "rationale": "fine", "better_alternatives": [] },
            "architecture": { "score": 4, "rationale": "sane" },
            "code_quality": { "score": 4, "rationale": "clean" },
            "ui_ux": { "score": 3, "rationale": "usable" },
            "packaging_hygiene": { "score": 5, "rationale": "tidy" },
            "security_flags": []
        });
        let fenced = format!("```json\n{}\n```", raw);
        let noisy = format!("Here is my evaluation:\n{}", raw);
        assert!(parse_rubric(&fenced).is_ok());
        assert!(parse_rubric(&noisy).is_ok());
        assert!(parse_rubric(&raw.to_string()).is_ok());
    }

    #[test]
    fn rejects_out_of_range_from_llm() {
        let bad = serde_json::json!({
            "problem_clarity": { "score": 9, "rationale": "x" },
            "differentiation": { "score": 3, "rationale": "x" },
            "architecture": { "score": 3, "rationale": "x" },
            "code_quality": { "score": 3, "rationale": "x" },
            "ui_ux": { "score": 3, "rationale": "x" },
            "packaging_hygiene": { "score": 3, "rationale": "x" },
            "security_flags": []
        });
        assert!(parse_rubric(&bad.to_string()).is_err());
    }
}

//! Web legitimacy check, proprietary submissions only. The one real risk
//! of shipping closed-source binaries is fronting for something hostile,
//! so the pipeline asks a web-enabled model the questions a careful human
//! would: does this app exist publicly, does the release channel match,
//! is anything known-bad. Findings are published with sources.

use anyhow::{Context, Result};
use omapak_core::{LegitimacyFinding, LegitimacyReport, Metadata, Severity};
use serde_json::{json, Value};

const SYSTEM_PROMPT: &str = r#"You are the omapak legitimacy checker for closed-source (proprietary) Flatpak submissions. You have web search. Research the app and answer like a careful human reviewer doing due diligence:

1. Does the app exist publicly (project page, vendor, store listings, social presence, real users)?
2. Does the release channel in the manifest (source URLs) belong to the actual author/vendor?
3. Any public reports of malware, scams, data harvesting, or hostile behavior attributed to this app or vendor?
4. Does the app-id's domain segment match the vendor's real domain (or a credible namespace)?

Output STRICT JSON only, no prose, no markdown fences:
{
  "summary": "2-4 sentences: what you found and how confident you are",
  "confidence": 0-100,
  "findings": [
    { "severity": "info"|"warning"|"critical", "detail": "what you found", "source": "URL or empty string" }
  ]
}

Severity guidance: "critical" is reserved for documented malware/scam/fraud (you found a credible report saying so). "warning" for mismatches you could not resolve (release channel doesn't clearly belong to the vendor, no public footprint found). "info" for positive confirmations worth recording. Absence of evidence is not a finding; do not invent. Empty findings with low confidence and a summary saying so is a valid answer."#;

pub fn run(config: &crate::judge_stage::JudgeConfig, metadata: &Metadata, app_id: &str) -> Result<LegitimacyReport> {
    let model = std::env::var("OMAPAK_LLM_WEB_MODEL").unwrap_or_else(|_| format!("{}:online", config.model));
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(240))
        .build()
        .context("build http client")?;

    let user = format!(
        "App ID: {app_id}\nSummary: {}\nHomepage: {}\nDeclared source/release channel: {}\nSubmitter: {}\n\nResearch this app's legitimacy and answer in JSON.",
        metadata.summary,
        metadata.homepage.as_deref().unwrap_or("(none declared)"),
        metadata.source_repo,
        metadata.submitter,
    );

    let body = json!({
        "model": model,
        "messages": [
            { "role": "system", "content": SYSTEM_PROMPT },
            { "role": "user", "content": user },
        ],
        "temperature": 0.1,
        "max_tokens": 4096,
    });
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .bearer_auth(&config.key)
        .json(&body)
        .send()
        .with_context(|| format!("legitimacy request to {url}"))?;
    let status = resp.status();
    let text = resp.text().unwrap_or_default();
    if !status.is_success() {
        anyhow::bail!("legitimacy endpoint returned {}: {}", status, text);
    }

    let content = serde_json::from_str::<Value>(&text)
        .ok()
        .and_then(|v| {
            v.get("choices")?.get(0)?.get("message")?.get("content")?.as_str().map(String::from)
        })
        .context("no message.content in legitimacy response")?;

    let parsed = parse_json_object(&content)?;
    let findings: Vec<LegitimacyFinding> = parsed
        .get("findings")
        .and_then(|f| f.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|f| {
                    Some(LegitimacyFinding {
                        severity: match f.get("severity")?.as_str()? {
                            "critical" => Severity::Critical,
                            "warning" => Severity::Warning,
                            _ => Severity::Info,
                        },
                        detail: f.get("detail")?.as_str()?.to_string(),
                        source: f
                            .get("source")
                            .and_then(|s| s.as_str())
                            .filter(|s| !s.is_empty())
                            .map(String::from),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(LegitimacyReport {
        model,
        summary: parsed
            .get("summary")
            .and_then(|s| s.as_str())
            .unwrap_or("no summary returned")
            .to_string(),
        confidence: parsed
            .get("confidence")
            .and_then(|c| c.as_u64())
            .unwrap_or(0)
            .min(100) as u8,
        findings,
    })
}

fn parse_json_object(content: &str) -> Result<Value> {
    let trimmed = content.trim();
    let text = trimmed
        .strip_prefix("```")
        .and_then(|t| t.split_once('\n').map(|(_, rest)| rest))
        .map(|t| t.trim().trim_end_matches("```").trim())
        .unwrap_or(trimmed);
    let start = text.find('{').context("no JSON object in legitimacy response")?;
    let end = text.rfind('}').context("no JSON object in legitimacy response")?;
    Ok(serde_json::from_str(&text[start..=end])?)
}

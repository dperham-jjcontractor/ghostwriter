//! A short memory about the note-taker that the coach maintains itself.
//!
//! After each successful reply a second, plain-text model call looks at the
//! page, the reply and the current memory, and returns the memory's next
//! version: durable facts about her, her store and team, how she likes
//! replies, current threads, and anything she asked to remember. The file is
//! replaced, not appended to, so it consolidates instead of growing, and it is
//! capped in size. Every version is also appended to a log the owner can read.

use crate::config::Config;
use crate::embedded_assets::load_config;
use crate::keyboard::Keyboard;
use crate::util::{option_or_env, option_or_env_fallback, OptionMap};
use anyhow::{anyhow, Result};
use log::{info, warn};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Where the memory lives on the tablet (owner-editable; survives updates).
pub const MEMORY_DIR: &str = "/home/root/ghostwriter";
pub const MEMORY_FILE: &str = "memory.txt";
pub const MEMORY_LOG_FILE: &str = "memory-log.txt";
/// Prompt file with the updater's instructions (bundled; overridable on device).
const MEMORY_PROMPT_FILE: &str = "memory.json";
const LOG_MAX_BYTES: usize = 60_000;
const MAX_COMPLETION_TOKENS: u32 = 2000;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);

pub struct MemoryUpdater {
    model: String,
    base_url: String,
    api_key: String,
    reasoning_effort: String,
    max_chars: usize,
    dir: PathBuf,
    client: reqwest::Client,
}

impl MemoryUpdater {
    pub fn new(config: &Config, options: &OptionMap) -> Self {
        let model = if config.memory_model.trim().is_empty() {
            config.model.clone()
        } else {
            config.memory_model.trim().to_string()
        };
        let client = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            model,
            base_url: option_or_env_fallback(options, "base_url", "OPENAI_BASE_URL", "https://api.openai.com"),
            api_key: option_or_env(options, "api_key", "OPENAI_API_KEY"),
            reasoning_effort: config.openai_reasoning_effort.trim().to_string(),
            max_chars: config.memory_max_chars.max(200),
            dir: PathBuf::from(MEMORY_DIR),
            client,
        }
    }

    /// The current memory text, or an empty string when there is none yet.
    pub fn load(dir: &Path) -> String {
        fs::read_to_string(dir.join(MEMORY_FILE)).map(|s| s.trim().to_string()).unwrap_or_default()
    }

    /// The block appended to the coach prompt when a memory exists.
    pub fn prompt_section(memory: &str) -> String {
        format!(
            "\n\nWHAT YOU HAVE LEARNED ABOUT HER SO FAR\nYou wrote these notes after earlier pages; she or the person who set this up may have edited them. Use them to make replies specific to her, her store and her work. They are still not her training: never state a store policy as fact because of them.\n{}",
            memory
        )
    }

    /// Replies that carry nothing worth learning from.
    pub fn should_skip(reply: &str) -> bool {
        let r = reply.trim().to_ascii_lowercase();
        r.is_empty() || r.contains("open a notebook page") || r.contains("nothing new since my last note")
    }

    /// Strip code fences and non-ASCII, and cap the length at a line break.
    pub fn sanitize(text: &str, max_chars: usize) -> String {
        let mut t = text.trim();
        if let Some(rest) = t.strip_prefix("```") {
            let rest = rest.trim_start_matches(|c: char| c.is_ascii_alphabetic());
            t = match rest.rfind("```") {
                Some(end) => &rest[..end],
                None => rest,
            };
        }
        let folded = Keyboard::fold_to_ascii(t.trim());
        let folded = folded.trim();
        if folded.chars().count() <= max_chars {
            return folded.to_string();
        }
        let cut: String = folded.chars().take(max_chars).collect();
        match cut.rfind('\n') {
            Some(i) if i > max_chars / 2 => cut[..i].trim().to_string(),
            _ => cut.trim().to_string(),
        }
    }

    /// Replace the memory file atomically and append the new version to the log.
    pub fn write(dir: &Path, memory: &str) -> Result<()> {
        fs::create_dir_all(dir)?;
        let path = dir.join(MEMORY_FILE);
        let tmp = dir.join("memory.txt.tmp");
        fs::write(&tmp, format!("{}\n", memory.trim()))?;
        fs::rename(&tmp, &path)?;

        let log_path = dir.join(MEMORY_LOG_FILE);
        {
            let mut log = fs::OpenOptions::new().create(true).append(true).open(&log_path)?;
            writeln!(log, "---- {} ----\n{}\n", Self::timestamp(), memory.trim())?;
        }
        // Keep the log bounded: drop the oldest half once it grows past the limit.
        if let Ok(content) = fs::read_to_string(&log_path) {
            if content.len() > LOG_MAX_BYTES {
                let start = content.len() - LOG_MAX_BYTES / 2;
                let tail = &content[start..];
                let tail = match tail.find("---- ") {
                    Some(i) => &tail[i..],
                    None => tail,
                };
                fs::write(&log_path, tail)?;
            }
        }
        Ok(())
    }

    fn timestamp() -> String {
        let from_date = std::process::Command::new("date")
            .args(["-u", "+%Y-%m-%d %H:%M UTC"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        from_date.unwrap_or_else(|| {
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            format!("unix {}", secs)
        })
    }

    fn build_prompt(&self, memory: &str, reply: &str) -> Result<String> {
        let raw = load_config(MEMORY_PROMPT_FILE)?;
        let json: Value = serde_json::from_str(&raw)?;
        let template = json["prompt"]
            .as_str()
            .ok_or_else(|| anyhow!("{} has no 'prompt' string", MEMORY_PROMPT_FILE))?;
        let current = if memory.trim().is_empty() {
            "(empty: this is the first page)"
        } else {
            memory
        };
        Ok(template
            .replace("{{max_chars}}", &self.max_chars.to_string())
            .replace("{{memory}}", current)
            .replace("{{reply}}", reply.trim()))
    }

    /// Ask the model for the memory's next version and store it if it changed.
    pub async fn update(&self, page_base64: &str, reply: &str) -> Result<()> {
        if self.api_key.trim().is_empty() {
            anyhow::bail!("memory: no API key configured");
        }
        let current = Self::load(&self.dir);
        let prompt = self.build_prompt(&current, reply)?;

        let mut body = json!({
            "model": self.model,
            "messages": [{
                "role": "user",
                "content": [
                    { "type": "image_url", "image_url": { "url": format!("data:image/png;base64,{}", page_base64) } },
                    { "type": "text", "text": prompt }
                ]
            }],
            "max_completion_tokens": MAX_COMPLETION_TOKENS,
        });
        if !self.reasoning_effort.is_empty() {
            body["reasoning_effort"] = json!(self.reasoning_effort);
        }

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            let detail: String = text.chars().take(300).collect();
            anyhow::bail!("memory update API {}: {}", status.as_u16(), detail);
        }
        let json: Value = serde_json::from_str(&text)?;
        let content = json["choices"][0]["message"]["content"].as_str().unwrap_or("");
        let next = Self::sanitize(content, self.max_chars);

        if next.is_empty() {
            warn!("memory: the model returned nothing; keeping the current memory");
            return Ok(());
        }
        if next == current {
            info!("memory: unchanged");
            return Ok(());
        }
        Self::write(&self.dir, &next)?;
        info!("memory: updated ({} chars)", next.chars().count());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryUpdater;

    #[test]
    fn skips_replies_with_nothing_to_learn() {
        assert!(MemoryUpdater::should_skip("-- COACH --\nOpen a notebook page, then tap the corner again."));
        assert!(MemoryUpdater::should_skip("-- COACH --\nNothing new since my last note."));
        assert!(MemoryUpdater::should_skip("   "));
        assert!(!MemoryUpdater::should_skip("-- COACH --\nASK YOUR MANAGER: ..."));
    }

    #[test]
    fn sanitize_strips_fences_folds_and_caps() {
        assert_eq!(MemoryUpdater::sanitize("```text\nABOUT HER\n- new\n```", 500), "ABOUT HER\n- new");
        assert_eq!(MemoryUpdater::sanitize("caf\u{00E9} \u{2014} ok", 500), "cafe - ok");
        let long = (0..100).map(|i| format!("- line {}", i)).collect::<Vec<_>>().join("\n");
        let capped = MemoryUpdater::sanitize(&long, 300);
        assert!(capped.chars().count() <= 300);
        assert!(capped.ends_with(|c: char| c.is_ascii_digit()), "cut at a line break: {capped:?}");
    }

    #[test]
    fn write_then_load_roundtrip_and_log() {
        let dir = std::env::temp_dir().join(format!("ghostwriter-memory-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(MemoryUpdater::load(&dir), "");
        MemoryUpdater::write(&dir, "ABOUT HER\n- operations, started this week\n").unwrap();
        assert_eq!(MemoryUpdater::load(&dir), "ABOUT HER\n- operations, started this week");
        let log = std::fs::read_to_string(dir.join(super::MEMORY_LOG_FILE)).unwrap();
        assert!(log.contains("---- ") && log.contains("operations, started this week"));
        assert!(MemoryUpdater::prompt_section("x").contains("WHAT YOU HAVE LEARNED ABOUT HER SO FAR"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

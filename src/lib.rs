use std::fmt::Display;

use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use serde::Deserialize;

pub mod db;
pub mod search;
pub mod serve;
pub mod templates;

#[derive(Default)]
pub struct Content {
    pub categories: Vec<Category>,
    pub channels: Vec<Channel>,
}

#[derive(Deserialize, Debug)]
pub struct Category {
    pub name: String,
    pub children: Vec<Channel>,
}

#[derive(Deserialize, Debug)]
pub struct Channel {
    #[serde(rename = "type")]
    pub channel_type: u64,
    pub name: String,
    pub messages: Option<Vec<Message>>,
}

#[derive(Deserialize, Debug)]
pub struct Message {
    pub content: MessageContent,
    pub username: String,
    pub avatar: String,
    #[serde(rename = "sentAt")]
    pub sent_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct MessageContent(String);

impl AsRef<str> for MessageContent {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for MessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for MessageContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<(a?):(\w+):(\d+)>").unwrap());
        let s = String::deserialize(deserializer)?;

        Ok(MessageContent(
            RE.replace_all(&s, |captures: &Captures| {
                let ext = if &captures[1] == "a" { "gif" } else { "png" };
                let emote_name = &captures[2];
                let emote_id = &captures[3];
                format!(
                    r#"<img class="e" alt="{}" src="https://cdn.discordapp.com/emojis/{}.{}"/>"#,
                    emote_name, emote_id, ext
                )
            })
            .into_owned(),
        ))
    }
}

#[derive(Default, Clone)]
pub struct Toc {
    pub categories: Vec<TocCategory>,
}

#[derive(Default, Clone)]
pub struct TocCategory {
    pub name: String,
    pub channels: Vec<TocChannel>,
}

#[derive(Default, Clone)]
pub struct TocChannel {
    pub name: String,
    pub id: u64,
    pub channel_type: u64,
}

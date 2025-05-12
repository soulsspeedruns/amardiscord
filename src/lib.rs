use std::fmt::Display;

use askama::Html;
use askama_escape::Escaper;
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
    #[serde(skip)]
    pub rowid: u64,
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

// This deserialization implementation is going to get used only when building
// the initial SQLite database. It will first deserialize its input as a string,
// then escape the HTML entities, then replace the (now escaped; see regex
// below) instances of emote tags with the equivalent HTML `img` element.
//
// Discord emote tags are of the form `<a:FrankerZ:12345678>`. If the `a`
// character in the first field is present, the emote is an animated gif and
// `.gif` should be used as an extension, otherwise the emote is a static
// `.png`.
//
// There are other Discord specific tags such as localized time, of the form
// `<t:timestamp:R>`. They are not currently supported.
impl<'de> Deserialize<'de> for MessageContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"&lt;(a?):(\w+):(\d+)&gt;").unwrap());

        let input = String::deserialize(deserializer)?;
        let mut escaped = String::new();
        Html.write_escaped(&mut escaped, &input).unwrap();

        Ok(MessageContent(
            RE.replace_all(&escaped, |captures: &Captures| {
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
pub struct ChannelList {
    pub categories: Vec<ChannelCategory>,
}

#[derive(Default, Clone)]
pub struct ChannelCategory {
    pub name: String,
    pub channels: Vec<ChannelListEntry>,
}

#[derive(Default, Clone)]
pub struct ChannelListEntry {
    pub name: String,
    pub id: u64,
    pub channel_type: u64,
}

#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ScrollDirection {
    Up,
    Down,
    Both,
    #[default]
    Unspecified,
}

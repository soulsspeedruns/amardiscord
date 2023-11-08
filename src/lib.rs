use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Category {
    name: String,
    children: Vec<Channel>,
}

#[derive(Deserialize, Debug)]
struct Channel {
    messages: Vec<Message>,
}

#[derive(Deserialize, Debug)]
struct Message {
    content: MessageContent,
    username: String,
    avatar: String,
    #[serde(rename = "sentAt")]
    sent_at: DateTime<Utc>,
}

#[derive(Debug)]
struct MessageContent(String);

impl<'de> Deserialize<'de> for MessageContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<(a?):(\w+):(\d+)>").unwrap());
        let s = String::deserialize(deserializer)?;

        Ok(MessageContent(
            RE.replace_all(&s, |captures: &Captures| {
                let ext = if &captures[0] == "a" { "gif" } else { "png" };
                let emote_name = &captures[1];
                let emote_id = &captures[2];
                format!(
                    r#"<img alt="{}" src="https://cdn.discordapp.com/emojis/{}.{}"/>"#,
                    emote_name, emote_id, ext
                )
            })
            .into_owned(),
        ))
    }
}

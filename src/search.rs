use std::fmt::Write;

use anyhow::Result;
use itertools::Itertools;
use rusqlite::Row;
use serde::Deserialize;
use textwrap_macros::dedent;

use crate::{Message, MessageContent};

pub struct SearchResult {
    pub message_rowid: u64,
    pub channel_id: u64,
    pub message: Message,
}

impl SearchResult {
    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            message_rowid: row.get(5)?,
            channel_id: row.get(4)?,
            message: Message {
                content: MessageContent(row.get(0)?),
                username: row.get(1)?,
                avatar: row.get(2)?,
                sent_at: row.get(3)?,
            },
        })
    }
}

#[derive(Deserialize)]
pub struct SearchQuery {
    username: Option<String>,
    content: String,
}

impl SearchQuery {
    pub fn build(self) -> Result<(String, Vec<String>)> {
        let mut query = String::new();
        let mut params = vec![];

        write!(
            query,
            "{}",
            dedent!(
                r#"
                SELECT
                    messages.content, messages.username,
                    messages.avatar, messages.sent_at,
                    messages.channel_id, messages.rowid
                FROM messages_fts JOIN messages ON messages_fts.messages_rowid = messages.rowid
                WHERE
                "#
            )
        )?;

        if let Some(username) = self.username.map(fts_query) {
            writeln!(query, r#"messages_fts.username MATCH ?{} OR"#, params.len() + 1)?;
            params.push(format!("*\"{username}\"*"));
        }

        writeln!(query, r#"messages_fts.content MATCH ?{}"#, params.len() + 1)?;
        params.push(fts_query(self.content));

        writeln!(query, r#"ORDER BY messages.sent_at DESC;"#)?;

        Ok((query, params))
    }
}

// Risks of injection are prevented by whitelisting only alphanumeric characters
fn fts_filter<S: AsRef<str>>(input: S) -> String {
    input
        .as_ref()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

// Build a fulltext query by tokenizing input, filtering valid characters and
// building a boolean expression
fn fts_query<S: AsRef<str>>(input: S) -> String {
    Itertools::intersperse_with(
        input
            .as_ref()
            .split_whitespace()
            .map(fts_filter)
            .filter(|s| !s.is_empty())
            .map(|s| format!(r#""{s}""#)),
        || " AND ".to_string(),
    )
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fts_query() {
        assert_eq!(
            fts_query(r#"I *search* """ for"  STUFF! 山İ"#),
            "\"i\" AND \"search\" AND \"for\" AND \"stuff\" AND \"山i\u{307}\""
        )
    }
}

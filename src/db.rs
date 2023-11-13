use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;
use tokio::fs;
use tracing::info;

use crate::{Category, Channel, Content};

async fn load_categories(path: &Path) -> Result<Vec<Category>> {
    let path = path.join("categories");

    let mut categories = Vec::new();

    for i in 1.. {
        let path = path.join(format!("{i}.json"));
        if path.exists() {
            let content = fs::read_to_string(path).await?;
            let mut category: Category = serde_json::from_str(&content)?;

            for channel in &mut category.children {
                if let Some(msgs) = channel.messages.as_mut() {
                    msgs.sort_unstable_by(|a, b| b.sent_at.cmp(&a.sent_at));
                }
            }

            categories.push(category);
        } else {
            break;
        }
        if cfg!(debug_assertions) {
            break;
        }
    }

    Ok(categories)
}

async fn load_channels(path: &Path) -> Result<Vec<Channel>> {
    let path = path.join("other_channels");

    let mut entries = fs::read_dir(&path).await?;
    let mut channels = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(path).await?;
            let mut channel: Channel = serde_json::from_str(&content)?;
            if let Some(msgs) = channel.messages.as_mut() {
                msgs.sort_by_key(|msg| msg.sent_at);
            }
            channels.push(channel);
        }
    }

    Ok(channels)
}

pub async fn load_content() -> Result<Content> {
    let mut entries = fs::read_dir("./data").await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            return Ok(Content {
                categories: load_categories(&path).await?,
                channels: load_channels(&path).await?,
            });
        }
    }

    Ok(Default::default())
}

pub struct Database(Connection);

impl Database {
    pub fn new() -> Result<Self> {
        Ok(Self(Connection::open("amardiscord.sqlite")?))
    }

    pub async fn initialize(&mut self) -> Result<()> {
        self.0.execute(
            r#"
            CREATE TABLE IF NOT EXISTS categories (
                category_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL
            );
            "#,
            [],
        )?;

        self.0.execute(
            r#"
            CREATE TABLE IF NOT EXISTS channels (
                channel_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                channel_type INTEGER NOT NULL,
                name TEXT,
                category_id INTEGER NOT NULL,
                FOREIGN KEY(category_id) REFERENCES categories(category_id)
            );
            "#,
            [],
        )?;

        self.0.execute(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                content TEXT NOT NULL,
                username TEXT NOT NULL,
                avatar TEXT NOT NULL,
                sent_at TEXT NOT NULL,
                channel_id INTEGER NOT NULL,
                FOREIGN KEY(channel_id) REFERENCES channels(channel_id)
            );
            "#,
            [],
        )?;

        let categories = load_content().await?.categories;

        for category in categories {
            info!("Inserting category \"{}\"...", category.name);

            self.0.execute(r#"INSERT INTO categories (name) VALUES (?1);"#, [category.name])?;
            let category_id = self.0.last_insert_rowid();

            for channel in category.children {
                info!("Inserting channel \"{}\"...", channel.name);
                self.0.execute("BEGIN TRANSACTION", [])?;

                self.0.execute(
                    r#"
                    INSERT INTO channels (channel_type, name, category_id)
                    VALUES (?1, ?2, ?3);
                    "#,
                    (channel.channel_type, channel.name, category_id),
                )?;
                let channel_id = self.0.last_insert_rowid();

                let mut stmt = self.0.prepare(
                    r#"
                    INSERT INTO messages (content, username, avatar, sent_at, channel_id)
                    VALUES (?1, ?2, ?3, ?4, ?5);
                    "#,
                )?;

                for message in channel.messages.unwrap_or_default() {
                    stmt.execute((
                        message.content.as_ref(),
                        message.username,
                        message.avatar,
                        message.sent_at,
                        channel_id,
                    ))?;
                }

                self.0.execute("COMMIT", [])?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    #[tokio::test]
    async fn test_insert() {
        let mut database = Database::new().unwrap();

        database.initialize().await.unwrap();
    }
}

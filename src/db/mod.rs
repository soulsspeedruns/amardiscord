use std::path::Path;

use anyhow::Result;
use itertools::Itertools;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tokio::fs;

use crate::search::SearchQuery;
use crate::{Category, Channel, Content, Message, MessageContent, Toc, TocCategory, TocChannel};

mod init;

const PAGE_SIZE: u64 = 100;

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

pub struct Database(Pool<SqliteConnectionManager>);

impl Database {
    pub fn new() -> Result<Self> {
        Ok(Self(
            Pool::builder()
                .max_size(32)
                .build(SqliteConnectionManager::file("amardiscord.sqlite"))?,
        ))
    }

    async fn initialize(&mut self) -> Result<()> {
        let db = self.0.get()?;

        // Initialize database
        init::initialize(&db)?;

        // Load content
        let content = load_content().await?;
        let mut categories = content.categories;
        if !content.channels.is_empty() {
            categories
                .push(Category { name: "Other channels".to_string(), children: content.channels });
        }

        // Insert categories
        for category in categories {
            init::insert_category(category, &db)?;
        }

        // Populate full-text search table
        init::populate_fts(&db)?;

        Ok(())
    }

    pub fn go_to_message(&self, channel_id: u64, message_rowid: u64) -> Result<Vec<Message>> {
        let db = self.0.get()?;

        let page = db.query_row(
            r#"
            SELECT (COUNT(*) - 1) / ?1 FROM messages
            WHERE sent_at <= (
                SELECT timestamp FROM messages
                WHERE channel_id = ?2 AND message_rowid = ?3
            );
            "#,
            (PAGE_SIZE, channel_id, message_rowid),
            |row| row.get(0),
        )?;

        drop(db);
        self.get_page(channel_id, page)
    }

    pub fn get_page(&self, channel_id: u64, page: u64) -> Result<Vec<Message>> {
        let db = self.0.get()?;

        let mut stmt = db.prepare(
            r#"
            SELECT content, username, avatar, sent_at FROM messages
            WHERE channel_id = ?1
            LIMIT ?2 OFFSET ?3
            "#,
        )?;

        let messages = stmt.query_map((channel_id, PAGE_SIZE, page * PAGE_SIZE), |row| {
            Ok(Message {
                content: MessageContent(row.get(0)?),
                username: row.get(1)?,
                avatar: row.get(2)?,
                sent_at: row.get(3)?,
            })
        })?;

        Ok(messages.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_search(&self, search_query: SearchQuery) -> Result<Vec<Message>> {
        let db = self.0.get()?;

        let (query, params) = search_query.build()?;
        let mut stmt = db.prepare(&query)?;

        let messages = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            Ok(Message {
                content: MessageContent(row.get(0)?),
                username: row.get(1)?,
                avatar: row.get(2)?,
                sent_at: row.get(3)?,
            })
        })?;

        Ok(messages.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_toc(&self) -> Result<Toc> {
        let db = self.0.get()?;

        let mut stmt = db.prepare(
            r#"
            SELECT 
                categories.name as cat_name,
                channels.channel_type,
                channels.name as channel_name,
                channels.channel_id
            FROM
                categories
                JOIN channels
                ON channels.category_id = categories.category_id
            "#,
        )?;

        let channels = stmt
            .query_map((), |row| {
                Ok((row.get::<_, String>(0)?, TocChannel {
                    channel_type: row.get(1)?,
                    name: row.get(2)?,
                    id: row.get(3)?,
                }))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let groups = channels.iter().group_by(|(name, _)| name);

        let categories = groups
            .into_iter()
            .map(|(name, channels)| TocCategory {
                name: name.to_string(),
                channels: channels.map(|(_, channel)| channel.clone()).collect(),
            })
            .collect();

        Ok(Toc { categories })
    }
}

pub async fn build() -> Result<()> {
    Database::new()?.initialize().await
}

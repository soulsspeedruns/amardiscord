use std::path::{Path, PathBuf};

use itertools::Itertools;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use thiserror::Error;
use tokio::fs;

use crate::search::{SearchQuery, SearchResult};
use crate::{
    Category, Channel, ChannelCategory, ChannelList, ChannelListEntry, Content, Message,
    MessageContent,
};

mod init;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("serialization error")]
    Serde(#[from] serde_json::Error),
    #[error("database connection pool error")]
    Pool(#[from] r2d2::Error),
    #[error("database error")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("Loading channel {0}: {1}")]
    LoadChannel(PathBuf, std::io::Error),
    #[error("Search query build error")]
    SearchQueryBuild(std::fmt::Error),
}

const PAGE_SIZE: u64 = 100;

async fn load_categories(path: &Path) -> Result<Vec<Category>, Error> {
    let path = path.join("categories");

    let mut category_indices = Vec::new();

    // List all files in the `categories` directory, looking for files named
    // `<number>.json`.
    let mut entries = fs::read_dir(&path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // Skip non-.json files.
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            break;
        }

        // Extract stems from filenames (e.g. `1.json` -> `1`).
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            break;
        };

        // Try parsing the stem into an integer and, if successful, save it.
        if let Ok(category_index) = stem.parse::<i32>() {
            category_indices.push(category_index);
        }
    }

    // Category file names are ordered in the same way they are on a server.
    // Sort them to replicate the server's structure.
    category_indices.sort();

    let mut categories = Vec::new();

    // Load category files in the correct order.
    for i in category_indices {
        let path = path.join(format!("{i}.json"));
        let content = fs::read_to_string(path).await?;
        let mut category: Category = serde_json::from_str(&content)?;

        for channel in &mut category.children {
            if let Some(msgs) = channel.messages.as_mut() {
                msgs.sort_unstable_by(|a, b| b.sent_at.cmp(&a.sent_at));
            }
        }

        categories.push(category);
    }

    Ok(categories)
}

async fn load_channels(path: &Path) -> Result<Vec<Channel>, Error> {
    let path = path.join("other_channels");
    let mut channels = Vec::new();

    if !path.exists() {
        return Ok(channels);
    }

    let mut entries = fs::read_dir(&path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content =
                fs::read_to_string(&path).await.map_err(|e| Error::LoadChannel(path, e))?;
            let mut channel: Channel = serde_json::from_str(&content)?;
            if let Some(msgs) = channel.messages.as_mut() {
                msgs.sort_by_key(|msg| msg.sent_at);
            }

            channels.push(channel);
        }
    }

    Ok(channels)
}

pub async fn load_content() -> Result<Content, Error> {
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
    pub fn new() -> Result<Self, Error> {
        Ok(Self(
            Pool::builder()
                .max_size(32)
                .build(SqliteConnectionManager::file("amardiscord.sqlite"))?,
        ))
    }

    async fn initialize(&mut self) -> Result<(), Error> {
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

        // Cache expensive queries.
        init::cache(&db)?;

        Ok(())
    }

    pub fn go_to_message(&self, message_rowid: u64) -> Result<(u64, u64), Error> {
        let db = self.0.get()?;

        Ok(db.query_row(
            r#"
            SELECT channel_id, page FROM messages_pages
            WHERE messages_rowid = ?1
            "#,
            [message_rowid],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?)
    }

    pub fn get_page(&self, channel_id: u64, page: u64) -> Result<Vec<Message>, Error> {
        let db = self.0.get()?;

        let mut stmt = db.prepare(
            r#"
            SELECT content, username, avatar, sent_at, rowid FROM messages
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
                rowid: row.get(4)?,
            })
        })?;

        Ok(messages.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_search(&self, search_query: SearchQuery) -> Result<Vec<SearchResult>, Error> {
        let db = self.0.get()?;

        let (query, params) = search_query.build()?;
        let mut stmt = db.prepare(&query)?;

        let messages =
            stmt.query_map(rusqlite::params_from_iter(params), SearchResult::from_row)?;

        Ok(messages.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_channel_list(&self) -> Result<ChannelList, Error> {
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
                Ok((row.get::<_, String>(0)?, ChannelListEntry {
                    channel_type: row.get(1)?,
                    name: row.get(2)?,
                    id: row.get(3)?,
                }))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let groups = channels.iter().group_by(|(name, _)| name);

        let categories = groups
            .into_iter()
            .map(|(name, channels)| ChannelCategory {
                name: name.to_string(),
                channels: channels.map(|(_, channel)| channel.clone()).collect(),
            })
            .collect();

        Ok(ChannelList { categories })
    }
}

pub async fn build() -> Result<(), Error> {
    Database::new()?.initialize().await
}

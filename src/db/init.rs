use anyhow::Result;
use rusqlite::Connection;
use tracing::info;

use crate::{Category, Channel, Message};

pub(crate) fn insert_messages(
    messages: Vec<Message>,
    channel_id: i64,
    db: &Connection,
) -> Result<()> {
    let mut stmt = db.prepare(
        r#"
        INSERT INTO messages (content, username, avatar, sent_at, channel_id)
        VALUES (?1, ?2, ?3, ?4, ?5);
        "#,
    )?;

    for message in messages {
        stmt.execute((
            message.content.as_ref(),
            message.username,
            message.avatar,
            message.sent_at,
            channel_id,
        ))?;
    }

    Ok(())
}

pub(crate) fn insert_channel(channel: Channel, category_id: i64, db: &Connection) -> Result<()> {
    if channel.channel_type != 0 {
        info!("Skipping channel \"{}\"...", channel.name);
        return Ok(());
    }

    info!("Inserting channel \"{}\"...", channel.name);
    db.execute("BEGIN TRANSACTION", [])?;

    db.execute(
        r#"
        INSERT INTO channels (channel_type, name, category_id)
        VALUES (?1, ?2, ?3);
        "#,
        (channel.channel_type, channel.name, category_id),
    )?;
    let channel_id = db.last_insert_rowid();

    if let Some(messages) = channel.messages {
        insert_messages(messages, channel_id, db)?;
    }

    db.execute("COMMIT", [])?;

    Ok(())
}

pub(crate) fn insert_category(category: Category, db: &Connection) -> Result<()> {
    info!("Inserting category \"{}\"...", category.name);

    db.execute(r#"INSERT INTO categories (name) VALUES (?1);"#, [category.name])?;
    let category_id = db.last_insert_rowid();

    for channel in category.children {
        insert_channel(channel, category_id, db)?;
    }
    Ok(())
}

pub(crate) fn populate_fts(db: &Connection) -> Result<()> {
    info!("Populating FTS table...");
    db.execute(
        r#"
        INSERT INTO messages_fts (content, username, avatar, messages_rowid)
        SELECT content, username, avatar, rowid FROM messages;
        "#,
        [],
    )?;

    Ok(())
}

pub(crate) fn initialize(db: &Connection) -> Result<()> {
    db.execute(include_str!("migrations/init.sql"), [])?;
    Ok(())
}

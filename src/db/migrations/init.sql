-- Create categories table.
CREATE TABLE IF NOT EXISTS categories (
    category_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);

-- Create channels table.
CREATE TABLE IF NOT EXISTS channels (
    channel_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    channel_type INTEGER NOT NULL,
    name TEXT,
    category_id INTEGER NOT NULL,
    FOREIGN KEY(category_id) REFERENCES categories(category_id)
);

-- Create messages table.
CREATE TABLE IF NOT EXISTS messages (
    rowid INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    username TEXT NOT NULL,
    avatar TEXT NOT NULL,
    sent_at TEXT NOT NULL,
    channel_id INTEGER NOT NULL,
    FOREIGN KEY(channel_id) REFERENCES channels(channel_id)
);

-- Create messages/channel index.
CREATE INDEX messages_channels
ON messages(channel_id);

-- Create full-text search table.
CREATE VIRTUAL TABLE messages_fts
USING FTS5(content, username, avatar, messages_rowid);

-- Create mapping between messages and channel pages.
CREATE TABLE IF NOT EXISTS messages_pages (
    messages_rowid INTEGER NOT NULL PRIMARY KEY,
    channel_id INTEGER NOT NULL,
    page INTEGER NOT NULL,
    FOREIGN KEY(messages_rowid) REFERENCES messages(rowid)
);

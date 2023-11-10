use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use axum::{extract::Path as ExtractPath, extract::State, http::header, routing::get, Router};
use itertools::Itertools;
use maud::{html, Markup, PreEscaped};
use tokio::fs;
use tracing::info;

use amardiscord::{Category, Channel};
use tracing_subscriber::filter::LevelFilter;

#[derive(Default)]
struct Content {
    categories: Vec<Category>,
    channels: Vec<Channel>,
}

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

async fn load_content() -> Result<Content> {
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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .init();

    macro_rules! static_get {
        ($e:literal, $content_type:literal) => {
            get(|| async { ([(header::CONTENT_TYPE, $content_type)], include_str!($e)) })
        };
    }

    info!("Loading content...");
    let state = Arc::new(load_content().await?);

    info!("Starting app...");

    let app = Router::new()
        .route("/", static_get!("./static/index.html", "text/html"))
        .route("/index.css", static_get!("./static/index.css", "text/css"))
        .route("/index.js", static_get!("./static/index.js", "application/javascript"))
        .route("/htmx.min.js", static_get!("./static/htmx.min.js", "application/javascript"))
        .route("/api/toc", get(toc))
        .route("/api/channel/:category/:channel/:page", get(channel))
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn toc(State(state): State<Arc<Content>>) -> Markup {
    let categories = state.categories.iter().enumerate().map(|(category_idx, category)| {
        let channels =
            category.children.iter().enumerate().filter(|(_, channel)| channel.messages.is_some());
        html! {
            nav {
                h2 {
                    (&category.name)
                }
                ul {
                    @for (channel_idx, channel) in channels {
                        li {
                            a hx-get=(format!("/api/channel/{category_idx}/{channel_idx}/0"))
                              hx-target="#content-container"
                              hx-swap="outerHTML"
                            {
                                (channel.name)
                            }
                        }
                    }
                }
            }
        }
    });

    html! {
        @for category in categories {
            (category)
        }
    }
}

async fn channel(
    State(state): State<Arc<Content>>,
    ExtractPath((category_idx, channel_idx, page)): ExtractPath<(u64, u64, u64)>,
) -> Markup {
    let Some(category) = state.categories.get(category_idx as usize) else {
        return html! { "Category #" (category_idx) " not found" };
    };

    let Some(channel) = category.children.get(channel_idx as usize) else {
        return html! { "Channel #" (channel_idx) " not found" };
    };

    let Some(messages) = channel.messages.as_ref() else { return html! { "Audio channel" } };

    let first_message_index = (page * 100) as usize;
    if first_message_index >= messages.len() {
        return html! { "Wrong" };
    }

    let range = first_message_index..usize::min(first_message_index + 100, messages.len());

    let grouped = messages[range].iter().rev().group_by(|msg| &msg.username);
    let messages = grouped.into_iter().map(|(username, messages)| {
        let mut messages = messages.into_iter();
        let first_msg = messages.next().unwrap();

        html! {
            li.username {
                span.avatar {
                    img alt="" src=(&first_msg.avatar) {}
                }
                span.usr { (&username) }
                " "
                span.time { (&first_msg.sent_at) }
            }
            li.msg {
                (PreEscaped(&first_msg.content))
            }
            @for msg in messages {
                li.msg {
                    (PreEscaped(&msg.content))
                }
            }
        }
    });

    html! {
        div.scroller
            hx-get=(format!("/api/channel/{category_idx}/{channel_idx}/{}", page + 1))
            hx-trigger="intersect once delay:20ms"
            hx-swap="beforebegin show:top" {}
        ul {
            @for m in messages {
                (m)
            }
        }
    }
}

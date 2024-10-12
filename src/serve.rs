use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Path as ExtractPath, Query as ExtractQuery, State};
use axum::http::header;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, get_service};
use axum::Router;
use itertools::Itertools;
use maud::{html, Markup, PreEscaped};
use tokio::{fs, task};
use tower_http::services::ServeDir;
use tracing::info;

use crate::db::Database;
use crate::search::{SearchQuery, SearchResult};
use crate::templates::TocTemplate;

pub async fn serve() -> Result<()> {
    macro_rules! static_get {
        ($e:literal, $content_type:literal) => {
            get(|| async { ([(header::CONTENT_TYPE, $content_type)], include_str!($e)) })
        };
    }

    info!("Loading content...");
    let state = Arc::new(Database::new().unwrap());

    info!("Starting app on http://localhost:3000");

    let app = Router::new()
        .route("/api/toc", get(toc))
        .route("/api/message_page/:rowid", get(message_page))
        .route("/api/channel/:channel/:page", get(channel))
        .route("/api/search", get(search));

    let app = if cfg!(debug_assertions) {
        app.route(
            "/",
            get(|| async { Html(fs::read_to_string("src/static/index.html").await.unwrap()) }),
        )
        .fallback(get_service(ServeDir::new("src/static")))
    } else {
        app.route("/", static_get!("./static/index.html", "text/html"))
            .route("/index.css", static_get!("./static/index.css", "text/css"))
            .route("/index.js", static_get!("./static/index.js", "application/javascript"))
            .route("/htmx.min.js", static_get!("./static/htmx.min.js", "application/javascript"))
    };

    let app = app.with_state(state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn toc(State(db): State<Arc<Database>>) -> impl IntoResponse {
    let db = Arc::clone(&db);

    match task::spawn_blocking(move || db.get_toc()).await {
        Ok(Ok(toc)) => Html(TocTemplate::render(&toc)),
        Ok(Err(e)) => Html(format!("Error retrieving table of contents: {e:?}")),
        Err(e) => Html(format!("Error retrieving table of contents: {e:?}")),
    }
}

async fn channel(
    State(db): State<Arc<Database>>,
    ExtractPath((channel_id, page)): ExtractPath<(u64, u64)>,
) -> Markup {
    let db = Arc::clone(&db);

    let messages = match task::spawn_blocking(move || db.get_page(channel_id, page)).await {
        Ok(Ok(messages)) => messages,
        Ok(Err(e)) => return html! { (format!("Error retrieving messages: {e}")) },
        Err(e) => return html! { (format!("Error retrieving messages: {e}")) },
    };

    let grouped = messages.iter().rev().group_by(|msg| &msg.username);

    let messages = grouped.into_iter().map(|(username, messages)| {
        let mut messages = messages.into_iter();
        let first_msg = messages.next().unwrap();

        html! {
            div.msg-container {
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
        }
    });

    html! {
        div id="content-container" {
            div.scroller
                hx-get=(format!("/api/channel/{channel_id}/{}", page + 1))
                hx-trigger="intersect once delay:200ms"
                hx-swap="beforebegin scroll:top" {}

            ul id="messages" {
                @for m in messages {
                    (m)
                }
            }
        }
    }
}

async fn message_page(
    State(db): State<Arc<Database>>,
    ExtractPath(rowid): ExtractPath<u64>,
) -> Markup {
    let (channel_id, page) = match task::spawn_blocking({
        let db = Arc::clone(&db);
        move || db.go_to_message(rowid)
    })
    .await
    {
        Ok(Ok(x)) => x,
        Ok(Err(e)) => return html! { (format!("Error retrieving page: {e}")) },
        Err(e) => return html! { (format!("Error retrieving page: {e}")) },
    };

    channel(State(db), ExtractPath((channel_id, page))).await
}

async fn search(
    State(db): State<Arc<Database>>,
    ExtractQuery(query): ExtractQuery<SearchQuery>,
) -> Markup {
    let db = Arc::clone(&db);

    let search_results = match task::spawn_blocking(move || db.get_search(query)).await {
        Ok(Ok(search_results)) => search_results,
        Ok(Err(e)) => return html! { (format!("Error retrieving search results: {e}")) },
        Err(e) => return html! { (format!("Error retrieving search results: {e}")) },
    };

    let grouped = search_results.iter().rev().group_by(|msg| &msg.message.username);

    let messages = grouped.into_iter().map(|(username, search_results)| {
        let mut search_results = search_results.into_iter();
        let SearchResult { message_rowid, message, .. } = search_results.next().unwrap();

        html! {
            div."msg-container clickable"
                hx-get=(format!("/api/message_page/{}", message_rowid))
                hx-target="#content-container"
                hx-swap="outerHTML show:bottom"
            {
                li.username {
                    span.avatar {
                        img alt="" src=(&message.avatar) {}
                    }
                    span.usr { (&username) }
                    " "
                    span.time { (&message.sent_at) }
                    " "
                    span.jump-btn
                    {
                        "Jump"
                    }
                }

                li.msg {
                    (PreEscaped(&message.content))
                }

                @for search_result in search_results {
                    li.msg {
                        (PreEscaped(&search_result.message.content))
                    }
                }
            }
        }
    });

    html! {
        ul id="messages" {
            @for m in messages {
                (m)
            }
        }
    }
}

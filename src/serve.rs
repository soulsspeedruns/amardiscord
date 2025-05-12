use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Path as ExtractPath, Query as ExtractQuery, State};
use axum::http::header;
use axum::response::Html;
use axum::routing::{get, get_service};
use axum::Router;
use serde::Deserialize;
use tokio::{fs, task};
use tower_http::services::ServeDir;
use tracing::info;

use crate::db::Database;
use crate::search::SearchQuery;
use crate::templates::{ChannelListTemplate, MessagePageTemplate, SearchTemplate};
use crate::ScrollDirection;

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
        .route("/channels", get(channel_list))
        .route("/channel/:channel/:page", get(channel))
        .route("/message/:rowid", get(message_page))
        .route("/search", get(search));

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

async fn channel_list(State(db): State<Arc<Database>>) -> Html<String> {
    let db = Arc::clone(&db);

    match task::spawn_blocking(move || db.get_channel_list()).await {
        Ok(Ok(channel_list)) => Html(ChannelListTemplate::render(&channel_list)),
        Ok(Err(e)) => Html(format!("Error retrieving table of contents: {e:?}")),
        Err(e) => Html(format!("Error retrieving table of contents: {e:?}")),
    }
}

#[derive(Deserialize, Default)]
struct PageQuery {
    #[serde(default)]
    direction: ScrollDirection,
}

async fn channel(
    State(db): State<Arc<Database>>,
    ExtractPath((channel_id, page)): ExtractPath<(u64, u64)>,
    ExtractQuery(page_query): ExtractQuery<PageQuery>,
) -> Html<String> {
    let db = Arc::clone(&db);

    match task::spawn_blocking(move || db.get_page(channel_id, page)).await {
        Ok(Ok(messages)) if messages.is_empty() => Html(String::new()),
        Ok(Ok(messages)) => {
            Html(MessagePageTemplate::render(&messages, channel_id, page, page_query.direction))
        },
        Ok(Err(e)) => Html(format!("Error retrieving messages: {e}")),
        Err(e) => Html(format!("Error retrieving messages: {e}")),
    }
}

async fn message_page(
    State(db): State<Arc<Database>>,
    ExtractPath(rowid): ExtractPath<u64>,
) -> Html<String> {
    match task::spawn_blocking({
        let db = Arc::clone(&db);
        move || db.go_to_message(rowid)
    })
    .await
    {
        Ok(Ok((channel_id, page))) => {
            channel(
                State(db),
                ExtractPath((channel_id, page)),
                ExtractQuery(PageQuery { direction: ScrollDirection::Both }),
            )
            .await
        },
        Ok(Err(e)) => Html(format!("Error retrieving page: {e}")),
        Err(e) => Html(format!("Error retrieving page: {e}")),
    }
}

async fn search(
    State(db): State<Arc<Database>>,
    ExtractQuery(query): ExtractQuery<SearchQuery>,
) -> Html<String> {
    let db = Arc::clone(&db);

    match task::spawn_blocking(move || db.get_search(query)).await {
        Ok(Ok(search_results)) => Html(SearchTemplate::render(&search_results)),
        Ok(Err(e)) => Html(format!("Error retrieving search results: {e}")),
        Err(e) => Html(format!("Error retrieving search results: {e}")),
    }
}

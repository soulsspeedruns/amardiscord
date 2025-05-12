use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Path as ExtractPath, Query as ExtractQuery, State};
use axum::http::{header, HeaderMap};
use axum::response::Html;
use axum::routing::{get, get_service};
use axum::Router;
use serde::Deserialize;
use tokio::task;
use tower_http::services::ServeDir;
use tracing::info;

use crate::db::Database;
use crate::search::SearchQuery;
use crate::templates::{
    ChannelListTemplate, IndexTemplate, LayoutTemplate, MessagePageTemplate, SearchTemplate,
};
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
        .route("/", get(|| async { Html(IndexTemplate::render()) }))
        .route("/channels", get(channel_list))
        .route("/channel/:channel/:page", get(channel))
        .route("/message/:rowid", get(message_page))
        .route("/search", get(search));

    let app = if cfg!(debug_assertions) {
        app.fallback(get_service(ServeDir::new("src/static")))
    } else {
        app.route("/index.css", static_get!("./static/index.css", "text/css"))
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

#[derive(Deserialize, Default)]
struct ChannelListQuery {
    #[serde(default)]
    current_channel_id: Option<u64>,
}

fn wrap_content(content: String, headers: &HeaderMap) -> Html<String> {
    if headers.get("HX-Request").is_some() {
        Html(content)
    } else {
        Html(LayoutTemplate::render(&content))
    }
}

async fn channel_list(
    State(db): State<Arc<Database>>,
    ExtractQuery(query): ExtractQuery<ChannelListQuery>,
    headers: HeaderMap,
) -> Html<String> {
    let db = Arc::clone(&db);

    let content = match task::spawn_blocking(move || db.get_channel_list()).await {
        Ok(Ok(channel_list)) => {
            ChannelListTemplate::render(&channel_list, query.current_channel_id)
        },
        Ok(Err(e)) => format!("Error retrieving table of contents: {e:?}"),
        Err(e) => format!("Error retrieving table of contents: {e:?}"),
    };

    wrap_content(content, &headers)
}

#[derive(Deserialize, Default)]
struct PageQuery {
    #[serde(default)]
    direction: ScrollDirection,
}

async fn render_message_page(
    db: Arc<Database>,
    channel_id: u64,
    page: u64,
    direction: ScrollDirection,
    target_message_id: Option<u64>,
) -> String {
    match task::spawn_blocking({
        let db = Arc::clone(&db);
        move || db.get_page(channel_id, page)
    })
    .await
    {
        Ok(Ok(messages)) if messages.is_empty() => String::new(),
        Ok(Ok(messages)) => {
            MessagePageTemplate::render(&messages, channel_id, page, direction, target_message_id)
        },
        Ok(Err(e)) => format!("Error retrieving messages: {e}"),
        Err(e) => format!("Error retrieving messages: {e}"),
    }
}

async fn channel(
    State(db): State<Arc<Database>>,
    ExtractPath((channel_id, page)): ExtractPath<(u64, u64)>,
    ExtractQuery(page_query): ExtractQuery<PageQuery>,
    headers: HeaderMap,
) -> Html<String> {
    let content =
        render_message_page(Arc::clone(&db), channel_id, page, page_query.direction, None).await;
    wrap_content(content, &headers)
}

async fn message_page(
    State(db): State<Arc<Database>>,
    ExtractPath(rowid): ExtractPath<u64>,
    headers: HeaderMap,
) -> Html<String> {
    match task::spawn_blocking({
        let db = Arc::clone(&db);
        move || db.go_to_message(rowid)
    })
    .await
    {
        Ok(Ok((channel_id, page))) => {
            let content = render_message_page(
                Arc::clone(&db),
                channel_id,
                page,
                ScrollDirection::Both,
                Some(rowid),
            )
            .await;
            wrap_content(content, &headers)
        },
        Ok(Err(e)) => wrap_content(format!("Error retrieving page: {e}"), &headers),
        Err(e) => wrap_content(format!("Error retrieving page: {e}"), &headers),
    }
}

async fn search(
    State(db): State<Arc<Database>>,
    ExtractQuery(query): ExtractQuery<SearchQuery>,
    headers: HeaderMap,
) -> Html<String> {
    let db = Arc::clone(&db);

    let content = match task::spawn_blocking(move || db.get_search(query)).await {
        Ok(Ok(search_results)) => SearchTemplate::render(&search_results),
        Ok(Err(e)) => format!("Error retrieving search results: {e}"),
        Err(e) => format!("Error retrieving search results: {e}"),
    };

    wrap_content(content, &headers)
}

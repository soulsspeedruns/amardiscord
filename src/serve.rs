use std::sync::Arc;

use anyhow::Context;
use axum::extract::{Path as ExtractPath, Query as ExtractQuery, State};
use axum::http::{header, HeaderMap};
use axum::response::{Html, IntoResponse, Response, Result};
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

pub struct Error(anyhow::Error);

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Self(value)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        Html(self.0.to_string()).into_response()
    }
}

pub async fn serve() -> anyhow::Result<()> {
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

    Ok(axum::Server::bind(&"0.0.0.0:3000".parse().unwrap()).serve(app.into_make_service()).await?)
}

#[derive(Deserialize, Default)]
struct ChannelListQuery {
    #[serde(default)]
    current_channel_id: Option<u64>,
}

async fn task<F, T, E>(f: F) -> Result<T, Error>
where
    F: FnOnce() -> Result<T, E> + Send + 'static,
    T: Send + 'static,
    E: Send + Into<Error> + 'static,
{
    match task::spawn_blocking(f).await {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(e)) => Err(e.into()),
        Err(e) => Err(anyhow::Error::from(e).into()),
    }
}

fn wrap_partial(headers: &HeaderMap, content: String) -> String {
    if headers.get("HX-Request").is_some() {
        content
    } else {
        LayoutTemplate::render(&content)
    }
}

fn with_channel_id(channel_id: u64, content: String) -> Response {
    let mut response = Html(content).into_response();
    response.headers_mut().insert("X-Current-Channel-Id", channel_id.to_string().parse().unwrap());
    response
}

async fn channel_list(
    State(db): State<Arc<Database>>,
    ExtractQuery(query): ExtractQuery<ChannelListQuery>,
    headers: HeaderMap,
) -> Result<Html<String>> {
    Ok(task(move || db.get_channel_list().context("Error retrieving table of contents"))
        .await
        .map(|channel_list| ChannelListTemplate::render(&channel_list, query.current_channel_id))
        .map(|content| wrap_partial(&headers, content))
        .map(Html)?)
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
    headers: HeaderMap,
) -> Result<Response> {
    Ok(task(move || db.get_page(channel_id, page).context("Error retrieving messages"))
        .await
        .map(|messages| {
            MessagePageTemplate::render(&messages, channel_id, page, page_query.direction, None)
        })
        .map(|content| wrap_partial(&headers, content))
        .map(|content| with_channel_id(channel_id, content))?)
}

async fn message_page(
    State(db): State<Arc<Database>>,
    ExtractPath(rowid): ExtractPath<u64>,
    headers: HeaderMap,
) -> Result<Response> {
    Ok(task(move || {
        let (channel_id, page) =
            db.go_to_message(rowid).context("Error retrieving page offsets")?;
        let messages = db.get_page(channel_id, page).context("Error retrieving page")?;
        Ok::<_, Error>((channel_id, page, messages))
    })
    .await
    .map(|(channel_id, page, messages)| {
        (
            channel_id,
            MessagePageTemplate::render(
                &messages,
                channel_id,
                page,
                ScrollDirection::Both,
                Some(rowid),
            ),
        )
    })
    .map(|(channel_id, content)| (channel_id, wrap_partial(&headers, content)))
    .map(|(channel_id, content)| with_channel_id(channel_id, content))?)
}

async fn search(
    State(db): State<Arc<Database>>,
    ExtractQuery(query): ExtractQuery<SearchQuery>,
    headers: HeaderMap,
) -> Result<Html<String>> {
    Ok(task(move || db.get_search(query).context("Error retrieving search results"))
        .await
        .map(|search_results| SearchTemplate::render(&search_results))
        .map(|content| wrap_partial(&headers, content))
        .map(Html)?)
}

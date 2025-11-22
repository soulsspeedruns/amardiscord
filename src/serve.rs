use std::sync::Arc;

use axum::extract::{Path as ExtractPath, Query as ExtractQuery, State};
use axum::http::{header, HeaderMap};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use serde::Deserialize;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::task;
use tower_http::services::ServeDir;
use tracing::info;

use crate::db::{self, Database};
use crate::search::SearchQuery;
use crate::templates::{
    ChannelListTemplate, IndexTemplate, LayoutTemplate, MessagePageTemplate, SearchTemplate,
};
use crate::ScrollDirection;

#[derive(Error, Debug)]
pub enum Error {
    #[error("serve")]
    Axum(std::io::Error),
    #[error("join")]
    Join(task::JoinError),
    #[error("retrieving channel list")]
    GetChannelList(db::Error),
    #[error("retrieving messages")]
    GetPage(db::Error),
    #[error("retrieving page offsets")]
    GoToMessage(db::Error),
    #[error("retrieving search results")]
    GetSearch(db::Error),
    #[error("retrieving channel")]
    GetChannel(db::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        Html(format!("{self:?}")).into_response()
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn serve() -> Result<()> {
    macro_rules! static_get {
        ($e:literal, $content_type:literal) => {
            get(|| async { ([(header::CONTENT_TYPE, $content_type)], include_str!($e)) })
        };
    }

    info!("Loading content...");
    let state = Arc::new(Database::new().unwrap());

    info!("Starting app on http://0.0.0.0:3000");

    let app = Router::new()
        .route("/", get(|| async { Html(IndexTemplate::render()) }))
        .route("/channels", get(channel_list))
        .route("/channel/{channel}/{page}", get(channel))
        .route("/message/{rowid}", get(message_page))
        .route("/search", get(search))
        .route(
            "/health",
            get(|| async { ([(header::CONTENT_TYPE, "text/plain")], "FrankerZambino") }),
        );

    let app = if cfg!(debug_assertions) {
        app.fallback_service(ServeDir::new("src/static"))
    } else {
        app.route("/index.css", static_get!("./static/index.css", "text/css"))
            .route("/index.js", static_get!("./static/index.js", "application/javascript"))
            .route("/htmx.min.js", static_get!("./static/htmx.min.js", "application/javascript"))
    };

    let app = app.with_state(state);
    let listener = TcpListener::bind("0.0.0.0:3000").await.map_err(Error::Axum)?;

    axum::serve(listener, app.into_make_service()).await.map_err(Error::Axum)
}

#[derive(Deserialize, Default)]
struct ChannelListQuery {
    #[serde(default)]
    current_channel_id: Option<u64>,
}

async fn task<F, T, E>(f: F) -> Result<T>
where
    F: FnOnce() -> std::result::Result<T, E> + Send + 'static,
    T: Send + 'static,
    E: Send + Into<Error> + 'static,
{
    match task::spawn_blocking(f).await {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(e)) => Err(e.into()),
        Err(e) => Err(Error::Join(e)),
    }
}

fn wrap_partial(headers: &HeaderMap, title: String, content: String) -> String {
    if headers.get("HX-Request").is_some() {
        content
    } else {
        LayoutTemplate::render(&title, &content)
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
    task(move || db.get_channel_list().map_err(Error::GetChannelList))
        .await
        .map(|channel_list| ChannelListTemplate::render(&channel_list, query.current_channel_id))
        .map(|content| wrap_partial(&headers, "channel_list".to_string(), content))
        .map(Html)
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
    task(move || {
        // first get the channel
        let channel = db.get_channel(channel_id).map_err(Error::GetChannel)?;
        let messages = db.get_page(channel_id, page).map_err(Error::GetPage)?;
        Ok::<_, Error>((channel.name, messages))
    })
    .await
    .map(|(channel_name, messages)| {
        (
            channel_name.clone(),
            MessagePageTemplate::render(
                &messages,
                channel_id,
                channel_name,
                page,
                page_query.direction,
                None,
            ),
        )
    })
    .map(|(channel_name, content)| wrap_partial(&headers, channel_name, content))
    .map(|content| with_channel_id(channel_id, content))
}

async fn message_page(
    State(db): State<Arc<Database>>,
    ExtractPath(rowid): ExtractPath<u64>,
    headers: HeaderMap,
) -> Result<Response> {
    task(move || {
        let (channel_id, channel_name, page) =
            db.go_to_message(rowid).map_err(Error::GoToMessage)?;
        let messages = db.get_page(channel_id, page).map_err(Error::GetPage)?;
        Ok::<_, Error>((channel_id, channel_name, page, messages))
    })
    .await
    .map(|(channel_id, channel_name, page, messages)| {
        (
            channel_id,
            channel_name.clone(),
            MessagePageTemplate::render(
                &messages,
                channel_id,
                channel_name.clone(),
                page,
                ScrollDirection::Both,
                Some(rowid),
            ),
        )
    })
    .map(|(channel_id, channel_name, content)| {
        (channel_id, wrap_partial(&headers, channel_name, content))
    })
    .map(|(channel_id, content)| with_channel_id(channel_id, content))
}

async fn search(
    State(db): State<Arc<Database>>,
    ExtractQuery(query): ExtractQuery<SearchQuery>,
    headers: HeaderMap,
) -> Result<Response> {
    if query.is_empty() {
        // Trigger a refresh if query is empty
        let mut headers = HeaderMap::new();
        headers.insert("HX-Refresh", "true".parse().unwrap());
        return Ok((headers, Html(String::new())).into_response());
    }

    task(move || db.get_search(query).map_err(Error::GetSearch))
        .await
        .map(|search_results| SearchTemplate::render(&search_results))
        .map(|content| wrap_partial(&headers, "Search".to_string(), content))
        .map(|content| Html(content).into_response())
}

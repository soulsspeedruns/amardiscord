use axum::{response::Html, routing::get, Router};
use maud::{html, Markup};

#[tokio::main]
async fn main() {
    macro_rules! static_get {
        ($e:literal) => {
            get(|| async { Html(include_str!($e)) })
        };
    }

    let app = Router::new()
        .route("/", static_get!("./static/index.html"))
        .route("/htmx.min.js", static_get!("./static/htmx.min.js"))
        .route("/api/toc", get(toc));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn toc() -> Markup {
    html! {
        p {
            "Hello, world!"
        }
    }
}

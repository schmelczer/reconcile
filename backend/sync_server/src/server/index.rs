use axum::response::{Html, IntoResponse};

pub async fn index() -> impl IntoResponse {
    const HTML_CONTENT: &str = include_str!("./assets/index.html");
    let html_content = HTML_CONTENT;
    Html(html_content)
}

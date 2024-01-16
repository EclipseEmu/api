use axum::response::Html;

pub async fn handle() -> Html<&'static str> {
    Html("Eclipse API")
}

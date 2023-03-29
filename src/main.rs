mod errors;
mod util;

use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::HeaderValue,
    response::IntoResponse,
    routing::get,
    Router,
};
use hyper::{Method, StatusCode};
use serde::Deserialize;
use std::{env, net::SocketAddr, time::Duration};
use tower_http::cors::CorsLayer;

use errors::ApiError;
use util::{empty_string_as_none, ReqwestAxumStream};

#[derive(Clone)]
struct ApiState {
    pub http: reqwest::Client,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let port = env::var("ECLIPSE_API_PORT")
        .unwrap_or_else(|_| String::from("8001"))
        .parse::<u16>()?;

    // initialize tracing
    tracing_subscriber::fmt::init();

    let state = ApiState {
        http: reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(5))
            .build()?,
    };

    let origins = vec![
        HeaderValue::from_static("http://localhost:8000"),
        HeaderValue::from_static("https://eclipseemu.me"),
        HeaderValue::from_static("https://beta.eclipseemu.me"),
    ];

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(origins);

    // build our application with a route
    let app = Router::new()
        .route("/", get(|| async { (StatusCode::OK, "Eclipse API") }))
        .route("/download", get(download_proxy_handler))
        .route("/download/", get(download_proxy_handler))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

#[derive(Debug, Deserialize)]
struct Params {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    url: Option<String>,
}

async fn download_proxy_handler(
    Query(params): Query<Params>,
    State(ApiState { http }): State<ApiState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let Some(url) = params.url else {
        return Err(ApiError::MissingQuery("missing url param"));
    };
    let parsed_url = percent_encoding::percent_decode_str(url.as_str()).decode_utf8()?;
    tracing::debug!("requesting url: {parsed_url:?}");
    let req = http.get(parsed_url.to_string()).build()?;
    match http.execute(req).await {
        Ok(resp) => Ok(ReqwestAxumStream(resp)),
        Err(e) => Err(ApiError::Reqwest(e)),
    }
}

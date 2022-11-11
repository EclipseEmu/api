mod errors;
mod util;

use anyhow::Result;
use axum::{extract::Query, response::IntoResponse, routing::get, Router};
use hyper::StatusCode;
use serde::Deserialize;
use std::{env, net::SocketAddr, time::Duration};

use errors::ApiError;
use util::{empty_string_as_none, ReqwestAxumStream};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let port = env::var("ECLIPSE_API_PORT")
        .unwrap_or_else(|_| String::from("8001"))
        .parse::<u16>()?;

    // initialize tracing
    tracing_subscriber::fmt::init();

    let client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(5))
        .build()?;

    // build our application with a route
    let app = Router::new()
        .route("/", get(|| async { (StatusCode::OK, "Eclipse API") }))
        .route(
            "/download",
            get(move |params| download_proxy_handler(params, client)),
        );
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
    http: reqwest::Client,
    // headers: HeaderMap,
    // RawBody(body): RawBody,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if let Some(url) = params.url {
        let parsed_url = percent_encoding::percent_decode_str(url.as_str()).decode_utf8()?;
        tracing::debug!("requesting url: {parsed_url:?}");
        let req = http.get(parsed_url.to_string()).build()?;
        match http.execute(req).await {
            Ok(resp) => Ok(ReqwestAxumStream(resp)),
            Err(e) => Err(ApiError::Reqwest(e)),
        }
    } else {
        Err(ApiError::MissingQuery("missing url param"))
    }
}

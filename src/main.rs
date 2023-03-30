mod endpoints;
mod errors;
mod util;

use anyhow::Result;
use axum::{http::HeaderValue, routing::get, Router};
use hyper::{Method, StatusCode};
use sqlx::SqlitePool;
use std::{env, net::SocketAddr, time::Duration};
use tower_http::cors::CorsLayer;

use errors::ApiError;

use crate::util::setup_openvgdb;

#[derive(Clone)]
pub struct ApiState {
    pub http: reqwest::Client,
    pub openvgdb: SqlitePool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let port = env::var("ECLIPSE_API_PORT")
        .unwrap_or_else(|_| String::from("8001"))
        .parse::<u16>()?;
    let openvgdb_path = env::var("ECLIPSE_API_OPENVGDB_PATH")
        .expect("expected ECLIPSE_API_OPENVGDB_PATH in the environment");

    tracing_subscriber::fmt::init();

    let openvgdb = setup_openvgdb(&openvgdb_path)
        .await
        .expect("Failed to open OpenVGDB.");
    let http = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(5))
        .build()?;

    let state = ApiState { openvgdb, http };

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
        .route("/download", get(endpoints::download::handle))
        .route("/boxart", get(endpoints::boxart::handle))
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

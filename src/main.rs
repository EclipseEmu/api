#![feature(ip)]

mod dns;
mod endpoints;
mod errors;
mod util;

use {
    crate::{errors::ApiError, util::setup_openvgdb},
    axum::{
        http::{HeaderValue, Method},
        routing::get,
        Router,
    },
    sqlx::SqlitePool,
    std::{env, net::SocketAddr, sync::Arc, time::Duration},
    tower_http::cors::CorsLayer,
};

#[derive(Clone)]
pub struct ApiState {
    pub http: reqwest::Client,
    pub openvgdb: SqlitePool,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let port = env::var("ECLIPSE_API_PORT")
        .unwrap_or_else(|_| String::from("8001"))
        .parse::<u16>()
        .expect("failed to parse ECLIPSE_API_PORT");
    let openvgdb_path = env::var("ECLIPSE_API_OPENVGDB_PATH")
        .expect("expected ECLIPSE_API_OPENVGDB_PATH in the environment");

    let openvgdb = setup_openvgdb(&openvgdb_path)
        .await
        .expect("Failed to open OpenVGDB.");
    let http = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(5))
        .dns_resolver(Arc::new(dns::TrustDnsResolver::default()))
        .build()
        .expect("Failed to create http client");

    let cors = CorsLayer::new().allow_methods([Method::GET]).allow_origin([
        HeaderValue::from_static("http://localhost:8000"),
        HeaderValue::from_static("https://eclipseemu.me"),
        HeaderValue::from_static("https://beta.eclipseemu.me"),
    ]);

    // build our application with a route
    let app = Router::new()
        .route("/", get(endpoints::index::handle))
        .route("/download", get(endpoints::download::handle))
        .route("/boxart", get(endpoints::boxart::handle))
        .layer(cors)
        .with_state(ApiState { openvgdb, http });

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to create a TCP listener");

    axum::serve(listener, app).await.unwrap();
}

mod models;
mod routers;
mod utils;

use reqwest::Method;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
pub use utils::{errors, mongo};

use axum::{Extension, Router, Server};

use crate::routers::auth::auth_routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let db = mongo::Db::connect().await?;

    let cors_layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers(Any)
        .allow_origin(["http://localhost:3000".parse()?]);

    let app = Router::new()
        .nest("/auth", auth_routes())
        .layer(Extension(db))
        .layer(cors_layer);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    log::info!("Listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

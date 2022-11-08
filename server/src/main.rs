mod models;
mod mongo;
mod routers;

use mongo::Db;
use std::net::SocketAddr;

use axum::{Extension, Router, Server};

use crate::routers::auth::auth_routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let db = Db::connect().await?;
    let app = Router::new()
        .nest("/auth", auth_routes())
        .layer(Extension(db));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    log::info!("Listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

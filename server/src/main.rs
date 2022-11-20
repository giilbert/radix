mod models;
mod redis;
mod rooms;
mod routers;
mod utils;

use parking_lot::Mutex;
use reqwest::{header, Method};
use rooms::room::RoomCommands;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{select, sync::mpsc::Sender};
use tower_http::cors::CorsLayer;
pub use utils::{errors, mongo};

use axum::{Extension, Router, Server};

use crate::{
    mongo::Db,
    redis::Redis,
    routers::{auth::auth_routes, rooms::room_routes},
};

pub type Rooms = Arc<Mutex<HashMap<String, Sender<RoomCommands>>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let db = Db::connect().await?;
    let redis = Redis::connect(false).await?;
    let rooms = Rooms::default();

    let cors_layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::CONTENT_LENGTH,
        ])
        .allow_credentials(true)
        .allow_origin(["http://localhost:3000".parse()?]);

    let app = Router::new()
        .nest("/auth", auth_routes())
        .nest("/room", room_routes())
        .layer(Extension(db))
        .layer(Extension(redis))
        .layer(Extension(rooms.clone()))
        .layer(cors_layer);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    log::info!("Listening on {}", addr);
    select! {
        _ = Server::bind(&addr).serve(app.into_make_service()) => {},
        _ = tokio::signal::ctrl_c() => {
            log::info!("Gracefully shutting down.");
            let rooms = rooms.lock();

            for (name, commands) in rooms.iter() {
                log::info!("Stopping room {}", name);
                let _ = commands.send(RoomCommands::Stop).await;
            };

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    Ok(())
}

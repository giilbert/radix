mod models;
mod rooms;
mod routers;
mod utils;

use parking_lot::RwLock;
use reqwest::{header, Method};
use rooms::room::{RoomCommands, RoomConfig};
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::Arc,
};
use tokio::sync::mpsc::Sender;
use tower_http::cors::CorsLayer;
pub use utils::{errors, mongo};

use axum::{Extension, Router, Server};

use crate::{
    mongo::Db,
    routers::{auth::auth_routes, rooms::room_routes},
};

#[derive(Default)]
pub struct AppStateInner {
    pub rooms: HashMap<String, (RoomConfig, Sender<RoomCommands>)>,
    pub users_connected: HashSet<String>,
}

pub type AppState = Arc<RwLock<AppStateInner>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let db = Db::connect().await?;
    let app_state = AppState::default();

    let cors_layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::CONTENT_LENGTH,
        ])
        .allow_credentials(true)
        .allow_origin([
            "https://radix.gilbertz.works".parse()?,
            "http://localhost:3000".parse()?,
        ]);

    let app = Router::<AppState>::new()
        .nest("/auth", auth_routes())
        .nest("/room", room_routes())
        .layer(Extension(db))
        .layer(cors_layer)
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    log::info!("Listening on {}", addr);
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

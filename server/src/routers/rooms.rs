use axum::{
    extract::{Path, WebSocketUpgrade},
    response::Response,
    routing::{get, post},
    Extension, Json, Router,
};
use fred::prelude::SetsInterface;
use reqwest::StatusCode;

use crate::{
    errors::RouteErr,
    models::user::User,
    redis::Redis,
    rooms::{
        connection::Connection,
        proxies::room::RoomProxy,
        room::{CreateRoom, Room, RoomCommands},
    },
};

pub fn room_routes() -> Router {
    Router::new()
        .route("/", post(create_room))
        .route("/:name", get(connect))
}

async fn create_room(
    redis: Redis,
    user: User,
    Json(data): Json<CreateRoom>,
) -> Result<(), RouteErr> {
    // check if the room already exists
    let exists: i32 = redis.sismember("rooms", &data.name).await.map_err(|e| {
        log::error!("{:?}", e);
        RouteErr::Db("An error occurred".into()).into()
    })?;
    if 1 == exists {
        return Err(RouteErr::Msg(
            StatusCode::CONFLICT,
            format!("Room with name {} already exists.", data.name),
        ));
    }

    // create and run the room
    let room = Room::new(&redis, data, user);
    tokio::task::spawn(async move {
        if let Err(err) = room.run().await {
            log::error!("Error running room: {:?}", err);
        }
    });

    Ok(())
}

async fn connect(
    ws: WebSocketUpgrade,
    user: User,
    redis: Redis,
    Path(room_name): Path<String>,
) -> Result<Response, RouteErr> {
    log::info!("User connected.");

    // check if the room to connect to exists
    let exists: i32 = redis.sismember("rooms", &room_name).await.map_err(|e| {
        log::error!("{:?}", e);
        RouteErr::Db("An error occurred".into()).into()
    })?;
    if exists == 0 {
        return Err(RouteErr::Msg(
            StatusCode::NOT_FOUND,
            "Room does not exist.".into(),
        ));
    }

    Ok(ws.on_upgrade(|ws| async move {
        let conn = match Connection::new(ws, user, room_name.clone(), redis.clone()).await {
            Ok(conn) => conn,
            Err(err) => {
                log::error!("Error creating connection: {:?}", err);
                return;
            }
        };

        let room_proxy = RoomProxy::new(redis.clone(), &room_name);

        if let Err(err) = room_proxy
            .send_command(&RoomCommands::AddConnection(conn.id.clone()))
            .await
        {
            log::error!("Error notifying room of connection {:?}", err);
            return;
        }

        if let Err(err) = conn.run().await {
            log::error!("Error running connection: {:?}", err);
        }
    }))
}

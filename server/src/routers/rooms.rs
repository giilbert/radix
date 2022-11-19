use axum::{
    extract::{Path, WebSocketUpgrade},
    response::Response,
    routing::{get, post},
    Extension, Json, Router,
};
use fred::prelude::{KeysInterface, PubsubInterface, SetsInterface, TransactionInterface};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    errors::RouteErr,
    models::user::User,
    redis::Redis,
    rooms::{
        connection::Connection,
        room::{CreateRoom, Room, RoomCommands, Rooms},
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
    Extension(rooms): Extension<Rooms>,
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

    // insert the room into the replica's directory of rooms
    let name = data.name.clone();
    let room = Room::new(&redis, data, user);
    rooms.lock().insert(name.clone(), room.commands.clone());

    // run the room
    tokio::task::spawn(async move {
        if let Err(err) = room.run().await {
            log::error!("Error running room: {:?}", err);
        }

        rooms.lock().remove(&name);
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
        let conn = Connection::new(ws, user, room_name.clone(), redis.clone()).await;
        if let Ok(conn) = conn {
            let _: i32 = redis
                .publish(
                    format!("room:{}", room_name),
                    serde_json::to_string(&RoomCommands::AddConnection(conn.id.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            conn.run().await;
        }
    }))
}

use axum::{routing::post, Json, Router};
use fred::prelude::{KeysInterface, SetsInterface, TransactionInterface};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    errors::RouteErr,
    models::user::User,
    redis::Redis,
    rooms::room::{CreateRoom, Room},
};

pub fn room_routes() -> Router {
    Router::new().route("/", post(create_room))
}

async fn create_room(
    redis: Redis,
    user: User,
    Json(data): Json<CreateRoom>,
) -> Result<(), RouteErr> {
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

    let room = Room::new(&redis, data, user);
    tokio::task::spawn(room.run());

    Ok(())
}

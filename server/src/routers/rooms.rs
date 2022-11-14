use axum::{routing::post, Json, Router};
use fred::prelude::SetsInterface;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{errors::RouteErr, models::user::User, redis::Redis};

pub fn room_routes() -> Router {
    Router::new().route("/", post(create_room))
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CreateRoom {
    name: String,
    public: bool,
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

    let _: i32 = redis.sadd("rooms", &data.name).await.map_err(|_| {
        RouteErr::Msg(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error creating room.".into(),
        )
    })?;

    log::info!("{:?}", data);

    Ok(())
}

use axum::{routing::post, Json, Router};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    errors::RouteErr,
    models::user::{User, UserRepo},
};

pub fn room_routes() -> Router {
    Router::new().route("/", post(create_room))
}

#[derive(Deserialize, Debug)]
struct CreateRoom {
    a: i32,
}

async fn create_room(user: User, Json(data): Json<CreateRoom>) -> Result<&'static str, RouteErr> {
    log::info!("{:?}", data);
    Ok("asd")
}

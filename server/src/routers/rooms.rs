use axum::{
    body::Body,
    extract::{ws::Message, Path, State, WebSocketUpgrade},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use futures_util::SinkExt;
use mongodb::bson::Uuid;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    errors::RouteErr,
    models::{
        problem::ProblemRepo,
        user::{PublicUser, User},
    },
    rooms::{
        connection::Connection,
        room::{Room, RoomConfig, ServerSentCommand},
    },
    AppState,
};

pub fn room_routes() -> Router<AppState, Body> {
    Router::new()
        .route("/", post(create_room))
        .route("/list", get(list_rooms))
        .route("/:name/can-connect", get(can_connect))
        .route("/:name", get(connect))
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CreateRoom {
    pub name: String,
    pub public: bool,
    pub problems: Vec<ProblemsFilter>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "t", content = "c")]
pub enum ProblemsFilter {
    #[serde(rename = "Single")]
    Single { id: String },
    // TODO: the rest of the filters
}

async fn create_room(
    owner: User,
    problem_repo: ProblemRepo,
    State(state): State<AppState>,
    Json(data): Json<CreateRoom>,
) -> Result<(), RouteErr> {
    if state.read().users_connected.contains(&owner.id.to_string()) {
        return Err(RouteErr::Msg(
            StatusCode::FORBIDDEN,
            "You are already connected to a room.".into(),
        ));
    }

    if state.read().rooms.contains_key(&data.name) {
        return Err(RouteErr::Msg(
            StatusCode::FORBIDDEN,
            "Room with same name already exists.".into(),
        ));
    }

    let id = Uuid::new();
    let config = RoomConfig {
        name: data.name.clone(),
        public: data.public,
        owner,
    };

    let problems = problem_repo.get_from_filters(&data.problems).await?;

    log::info!("{:?}", problems);

    let room = Room::new(id, problems, config.clone());
    state
        .write()
        .rooms
        .insert(data.name.clone(), (config, room.commands.clone()));

    tokio::task::spawn(async move {
        if let Err(err) = room.run().await {
            log::error!("Error running room: {}", err);
        }
        state.write().rooms.remove(&data.name);
    });

    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanConnectResponse {
    can_connect: bool,
    reason: String,
}

async fn can_connect(
    user: User,
    Path(room_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<CanConnectResponse>, RouteErr> {
    if state.read().users_connected.contains(&user.id.to_string()) {
        return Ok(Json(CanConnectResponse {
            can_connect: false,
            reason: "You are already connected to a room.".to_string(),
        }));
    }

    if !state.read().rooms.contains_key(&room_name) {
        return Ok(Json(CanConnectResponse {
            can_connect: false,
            reason: "Room does not exist.".to_string(),
        }));
    }

    Ok(Json(CanConnectResponse {
        can_connect: true,
        reason: "".to_string(),
    }))
}

async fn connect(
    ws: WebSocketUpgrade,
    user: User,
    State(state): State<AppState>,
    Path(room_name): Path<String>,
) -> Result<Response, RouteErr> {
    if state.read().users_connected.contains(&user.id.to_string()) {
        return Err(RouteErr::Msg(
            StatusCode::FORBIDDEN,
            "You are already connected to a room.".into(),
        ));
    }

    let (.., room_commands) = state
        .read()
        .rooms
        .get(&room_name)
        .ok_or_else(|| RouteErr::Msg(StatusCode::NOT_FOUND, "Room not found.".into()))?
        .clone();

    Ok(ws.on_upgrade(|ws| async move {
        let conn = match Connection::new(ws, user.clone(), room_commands).await {
            Ok(conn) => conn,
            Err(mut ws_tx) => {
                let _ = ws_tx
                    .send(Message::Text(
                        serde_json::to_string(&ServerSentCommand::Error("".into())).unwrap(),
                    ))
                    .await;
                return;
            }
        };
        state.write().users_connected.insert(user.id.to_string());

        if let Err(err) = conn.run().await {
            log::error!("Error running connection: {}", err);
        }

        state.write().users_connected.remove(&user.id.to_string());
    }))
}

#[derive(Serialize)]
struct RoomListData {
    name: String,
    owner: PublicUser,
}

async fn list_rooms(State(state): State<AppState>) -> Result<Json<Vec<RoomListData>>, RouteErr> {
    let rooms = state
        .read()
        .rooms
        .iter()
        .filter(|(_, (config, _))| config.public)
        .map(|(_, (config, _))| RoomListData {
            name: config.name.clone(),
            owner: config.owner.to_public(),
        })
        .collect::<Vec<_>>();

    Ok(Json(rooms))
}

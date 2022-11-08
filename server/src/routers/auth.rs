use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use crate::{
    errors::{DatabaseErr, ErrMsg, RouteErr},
    models::user::{CreateUser, User, UserRepo},
};

pub fn auth_routes() -> Router {
    Router::new()
        .route("/user", post(create_user))
        .route("/user/:id", get(get_user_by_id))
}

async fn create_user(
    user_repo: UserRepo,
    Json(data): Json<CreateUser>,
) -> Result<Json<User>, RouteErr> {
    let user = user_repo.create(data).await?;
    Ok(Json(user))
}

async fn get_user_by_id(
    user_repo: UserRepo,
    Path(id): Path<String>,
) -> Result<Json<User>, RouteErr> {
    let user = user_repo.get_user_by_id(&id).await?;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(RouteErr::Msg(
            StatusCode::NOT_FOUND,
            "Not found".to_string(),
        )),
    }
}

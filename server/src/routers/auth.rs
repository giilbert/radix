use axum::{routing::post, Json, Router};

use crate::{
    models::user::{CreateUser, User, UserRepo},
    mongo::DatabaseError,
};

pub fn auth_routes() -> Router {
    Router::new().route("/create", post(create_user))
}

async fn create_user(
    user_repo: UserRepo,
    Json(data): Json<CreateUser>,
) -> Result<Json<User>, DatabaseError> {
    let user = user_repo.create(data).await?;
    Ok(Json(user))
}

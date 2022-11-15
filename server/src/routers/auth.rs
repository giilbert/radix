use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    errors::RouteErr,
    models::user::{Account, CreateUser, Session, SessionAndUser, User, UserRepo},
    utils::auth::verify_user,
};

pub fn auth_routes() -> Router {
    Router::new()
        .route("/user", post(create_user))
        .route("/user/:id", get(get_user_by_id))
        .route("/user-email/:email", get(get_user_by_email))
        .route(
            "/user-account/:provider/:provider_account_id",
            get(get_user_by_account),
        )
        .route(
            "/session/:session_token",
            get(get_session_and_user)
                .delete(delete_session)
                .patch(update_session),
        )
        .route("/session", post(create_session))
        .route("/link-account", post(link_account))
}

// TODO: verify that access token is valid
async fn create_user(
    user_repo: UserRepo,
    Json(data): Json<CreateUser>,
) -> Result<Json<User>, RouteErr> {
    let verified = verify_user(&data.access_token).await?;

    if verified {
        let user = user_repo.create(data).await?;
        Ok(Json(user))
    } else {
        Err(RouteErr::Msg(
            StatusCode::BAD_REQUEST,
            "Invalid access token.".into(),
        ))
    }
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

async fn get_user_by_email(
    user_repo: UserRepo,
    Path(email): Path<String>,
) -> Result<Json<User>, RouteErr> {
    let user = user_repo.get_user_by_email(&email).await?;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(RouteErr::Msg(
            StatusCode::NOT_FOUND,
            "Not found".to_string(),
        )),
    }
}

async fn get_user_by_account(
    user_repo: UserRepo,
    Path((provider, provider_account_id)): Path<(String, String)>,
) -> Result<Json<User>, RouteErr> {
    let user = user_repo
        .get_user_by_account_id(&provider, &provider_account_id)
        .await?;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(RouteErr::Msg(
            StatusCode::NOT_FOUND,
            "Not found".to_string(),
        )),
    }
}

async fn link_account(
    user_repo: UserRepo,
    Json(data): Json<Account>,
) -> Result<Json<User>, RouteErr> {
    let verified = verify_user(&data.access_token).await?;

    if !verified {
        return Err(RouteErr::Msg(
            StatusCode::BAD_REQUEST,
            "Invalid access token.".into(),
        ));
    }

    let user = user_repo.create_user_account(&data).await?;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(RouteErr::Msg(
            StatusCode::NOT_FOUND,
            "Not found".to_string(),
        )),
    }
}

async fn get_session_and_user(
    user_repo: UserRepo,
    Path(session_token): Path<String>,
) -> Result<Json<SessionAndUser>, RouteErr> {
    let session_and_user = user_repo.get_session_and_user(&session_token).await?;

    match session_and_user {
        Some(session_and_user) => Ok(Json(session_and_user)),
        None => Err(RouteErr::Msg(
            StatusCode::NOT_FOUND,
            "Not found".to_string(),
        )),
    }
}

async fn create_session(user_repo: UserRepo, Json(data): Json<Session>) -> Result<(), RouteErr> {
    user_repo.create_user_session(&data).await?;
    Ok(())
}

async fn delete_session(
    user_repo: UserRepo,
    Path(session_token): Path<String>,
) -> Result<(), RouteErr> {
    user_repo.delete_session(&session_token).await?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateSession {
    session_token: String,
    expires: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateSessionResponse {
    session_token: String,
    expires: String,
    user_id: String,
}

async fn update_session(
    user_repo: UserRepo,
    Json(data): Json<UpdateSession>,
) -> Result<Json<UpdateSessionResponse>, RouteErr> {
    user_repo
        .update_session_expiry(&data.session_token, &data.expires)
        .await?;
    let session_and_user = user_repo
        .get_session_and_user(&data.session_token)
        .await?
        .ok_or_else(|| RouteErr::Msg(StatusCode::NOT_FOUND, "Session not found.".into()))?;

    Ok(Json(UpdateSessionResponse {
        session_token: session_and_user.session.session_token,
        expires: session_and_user.session.expires.to_string(),
        user_id: session_and_user.user.id.to_string(),
    }))
}

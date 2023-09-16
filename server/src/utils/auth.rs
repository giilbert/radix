use crate::{
    errors::RouteErr,
    models::user::{User, UserRepo},
};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

pub async fn verify_user(token: &String) -> Result<bool, RouteErr> {
    let res = reqwest::get(format!(
        "https://www.googleapis.com/oauth2/v1/tokeninfo?access_token={}",
        token
    ))
    .await
    .map_err(|e| {
        log::error!("Google OAuth token verification error: {}", e);
        RouteErr::Msg(
            StatusCode::INTERNAL_SERVER_ERROR,
            "An error occurred during verification.".into(),
        )
    })?;

    if res.status() == 200 {
        Ok(true)
    } else {
        Ok(false)
    }
}

fn get_session_token_from_authorization(parts: &mut Parts) -> Result<String, StatusCode> {
    let session_token = parts
        .headers
        .get("Authorization")
        .ok_or_else(|| StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .replace("Bearer ", "")
        .to_string();

    Ok(session_token)
}

fn get_session_token_from_query(parts: &mut Parts) -> Result<String, StatusCode> {
    let session_token = parts
        .uri
        .query()
        .ok_or_else(|| StatusCode::UNAUTHORIZED)?
        .split("&")
        .find(|q| q.starts_with("s="))
        .ok_or_else(|| StatusCode::UNAUTHORIZED)?
        .replace("s=", "")
        .to_string();

    Ok(session_token)
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = StatusCode;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session_token = get_session_token_from_authorization(parts)
            .or_else(|_| get_session_token_from_query(parts))?;

        let user_repo = UserRepo::from_request_parts(parts, state).await?;
        let session_and_user = user_repo
            .get_session_and_user(session_token)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?
            .ok_or(StatusCode::UNAUTHORIZED)?;

        Ok(session_and_user.user)
    }
}

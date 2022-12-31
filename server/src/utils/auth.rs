use crate::{
    errors::RouteErr,
    models::user::{User, UserRepo},
};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use regex::Regex;

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

lazy_static::lazy_static! {
    static ref SESSION_TOKEN_REGEX: Regex =
        Regex::new("next-auth\\.session-token=([a-zA-Z0-9\\-]+);?").unwrap();
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = StatusCode;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookie = parts
            .headers
            .get("cookie")
            .ok_or(StatusCode::UNAUTHORIZED)?
            .to_str()
            .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?
            .to_string();

        let captures = &SESSION_TOKEN_REGEX
            .captures(&cookie)
            .ok_or(StatusCode::UNAUTHORIZED)?;
        let session_token = captures.get(1).ok_or(StatusCode::UNAUTHORIZED)?.as_str();

        let user_repo = UserRepo::from_request_parts(parts, state).await?;
        let session_and_user = user_repo
            .get_session_and_user(session_token)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?
            .ok_or(StatusCode::UNAUTHORIZED)?;

        Ok(session_and_user.user)
    }
}

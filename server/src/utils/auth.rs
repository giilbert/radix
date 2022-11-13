use crate::{
    errors::RouteErr,
    models::user::{User, UserRepo},
};
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    http::StatusCode,
};
use regex::Regex;

pub async fn verify_user(token: &String) -> Result<bool, RouteErr> {
    let res = reqwest::get(format!(
        "https://www.googleapis.com/oauth2/v1/tokeninfo?access_token={}",
        token
    ))
    .await
    .map_err(|_| {
        RouteErr::Msg(
            StatusCode::INTERNAL_SERVER_ERROR,
            "An error occurred.".into(),
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
        Regex::new("next-auth\\.session-token=(.+);?").unwrap();
}

#[async_trait]
impl<B> FromRequest<B> for User
where
    B: Send,
{
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let cookie = req
            .headers()
            .get("cookie")
            .ok_or(StatusCode::UNAUTHORIZED)?
            .to_str()
            .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?
            .to_string();

        let captures = &SESSION_TOKEN_REGEX
            .captures(&cookie)
            .ok_or(StatusCode::UNAUTHORIZED)?;
        let session_token = captures.get(1).ok_or(StatusCode::UNAUTHORIZED)?.as_str();

        let user_repo = UserRepo::from_request(req).await?;
        let session_and_user = user_repo
            .get_session_and_user(session_token)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?
            .ok_or(StatusCode::UNAUTHORIZED)?;

        Ok(session_and_user.user)
    }
}

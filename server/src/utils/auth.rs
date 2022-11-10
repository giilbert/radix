use crate::errors::RouteErr;
use axum::http::StatusCode;

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

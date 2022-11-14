use std::error::Error;

use axum::{http::StatusCode, response::IntoResponse, Json};
use mongodb::results::DeleteResult;
use serde::Serialize;

#[derive(Clone, Debug)]
pub enum RouteErr {
    Db(String),
    Msg(StatusCode, String),
}

pub struct DatabaseErr(pub StatusCode, pub &'static str, pub Option<Box<dyn Error>>);
pub struct ErrMsg(pub StatusCode, pub &'static str);

#[derive(Serialize)]
pub struct ErrorResponseMessage {
    error: String,
}

impl IntoResponse for RouteErr {
    fn into_response(self) -> axum::response::Response {
        // just a shorter way of writing it
        type E = ErrorResponseMessage;

        (match self {
            RouteErr::Db(msg) => (StatusCode::INTERNAL_SERVER_ERROR, Json(E { error: msg })),
            RouteErr::Msg(status, msg) => (status, Json(E { error: msg })),
        })
        .into_response()
    }
}

pub trait ConvertResult<T> {
    fn convert(self, msg: Option<impl ToString>) -> Result<T, RouteErr>;
}

impl<T> ConvertResult<T> for Result<T, mongodb::error::Error> {
    fn convert(self, msg: Option<impl ToString>) -> Result<T, RouteErr> {
        self.map_err(|err| {
            log::error!("MongoDB Error: {}", err);
            RouteErr::Db(
                msg.map(|msg| msg.to_string())
                    .unwrap_or_else(|| "An error occurred.".to_string()),
            )
        })
    }
}

impl<T> ConvertResult<T> for Option<T> {
    fn convert(self, msg: Option<impl ToString>) -> Result<T, RouteErr> {
        self.ok_or_else(|| {
            RouteErr::Msg(
                StatusCode::NOT_FOUND,
                msg.map(|msg| msg.to_string())
                    .unwrap_or_else(|| "Not found.".to_string()),
            )
        })
    }
}

impl<T> ConvertResult<T> for Result<T, DeleteResult> {
    fn convert(self, msg: Option<impl ToString>) -> Result<T, RouteErr> {
        self.map_err(|err| {
            log::error!("MongoDB Error: {:#?}", err);
            RouteErr::Db(
                msg.map(|msg| msg.to_string())
                    .unwrap_or_else(|| "An error occurred.".to_string()),
            )
        })
    }
}

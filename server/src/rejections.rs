use serde::Serialize;
use std::convert::Infallible;
use warp::{
    body::BodyDeserializeError,
    hyper::StatusCode,
    reject::{MethodNotAllowed, Reject},
    Rejection, Reply,
};

#[derive(Debug)]
pub struct MongoError(pub mongodb::error::Error);
impl Reject for MongoError {}

#[derive(Debug)]
pub struct BadRequest(pub String);
impl Reject for BadRequest {}

#[derive(Serialize)]
struct ErrorMessage {
    message: String,
}

pub async fn rejection_handler(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND".into();
    } else if let Some(err) = err.find::<BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = err.to_string();
    } else if let Some(err) = err.find::<BadRequest>() {
        code = StatusCode::BAD_REQUEST;
        message = err.0.clone();
    } else if let Some(_) = err.find::<MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED".into();
    } else {
        log::error!("Internal Server Error: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "INTERNAL_SERVER_ERROR".into();
    }

    let json = warp::reply::json(&ErrorMessage {
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

pub trait IntoRejection<T, R: Reject> {
    fn handle(self) -> Result<T, R>;
}

impl<T> IntoRejection<T, MongoError> for mongodb::error::Result<T> {
    fn handle(self) -> Result<T, MongoError> {
        self.or_else(|e| Err(MongoError(e)))
    }
}

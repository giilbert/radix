use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::mongo::{DatabaseError, Db};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
    pub name: String,
    pub image: String,
    pub email_verified: DateTime<Utc>,
    pub access_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateUser {
    pub email: String,
    pub name: String,
    pub image: String,
    pub email_verified: DateTime<Utc>,
    pub access_token: String,
}

#[derive(Clone)]
pub struct UserRepo(Db);
impl UserRepo {
    pub async fn create(&self, data: CreateUser) -> anyhow::Result<User, DatabaseError> {
        self.0
            .collection::<CreateUser>("users")
            .insert_one(&data, None)
            .await
            .map_err(|err| {
                DatabaseError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error creating User",
                    Box::new(err),
                )
            })
            .map(|res| User {
                id: res.inserted_id.as_object_id().unwrap().to_string(),
                email: data.email,
                name: data.name,
                image: data.image,
                email_verified: data.email_verified,
                access_token: data.access_token,
            })
    }
}

#[async_trait]
impl<B: Send> FromRequest<B> for UserRepo {
    type Rejection = StatusCode;
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let db = req.extensions().get::<Db>().expect("db not in extensions");
        Ok(Self(db.clone()))
    }
}

use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};

use crate::mongo::{oid_as_string, DatabaseError, Db, ToObjectId};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename(deserialize = "_id"), serialize_with = "oid_as_string")]
    pub id: ObjectId,
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
    pub async fn create(&self, data: CreateUser) -> Result<User, DatabaseError> {
        self.0
            .collection::<CreateUser>("users")
            .insert_one(&data, None)
            .await
            .map_err(|err| {
                DatabaseError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error creating User",
                    Some(Box::new(err)),
                )
            })
            .map(|res| User {
                id: res.inserted_id.as_object_id().unwrap(),
                email: data.email,
                name: data.name,
                image: data.image,
                email_verified: data.email_verified,
                access_token: data.access_token,
            })
    }

    pub async fn get_user_by_id(&self, id: &String) -> Result<Option<User>, DatabaseError> {
        self.0
            .collection::<User>("users")
            .find_one(
                doc! {
                    "_id": id.to_object_id()?
                },
                None,
            )
            .await
            .map_err(|err| {
                DatabaseError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error getting user",
                    Some(Box::new(err)),
                )
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

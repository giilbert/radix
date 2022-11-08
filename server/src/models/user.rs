use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{ConvertResult, DatabaseErr, RouteErr},
    mongo::{oid_as_string, Db, ToObjectId},
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename(deserialize = "_id"), serialize_with = "oid_as_string")]
    pub id: ObjectId,
    pub email: String,
    pub name: String,
    pub image: String,
    pub email_verified: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateUser {
    pub email: String,
    pub name: String,
    pub image: String,
    pub email_verified: Option<DateTime<Utc>>,
    pub access_token: String,
}

#[derive(Clone)]
pub struct UserRepo(Db);
impl UserRepo {
    pub async fn create(&self, data: CreateUser) -> Result<User, RouteErr> {
        self.0
            .collection::<CreateUser>("users")
            .insert_one(&data, None)
            .await
            .convert(Some("Error creating user"))
            .map(|res| User {
                id: res.inserted_id.as_object_id().unwrap(),
                email: data.email,
                name: data.name,
                image: data.image,
                email_verified: data.email_verified,
            })
    }

    pub async fn get_user_by_id(&self, id: &String) -> Result<Option<User>, RouteErr> {
        self.0
            .collection::<User>("users")
            .find_one(
                doc! {
                    "_id": id.to_object_id()?
                },
                None,
            )
            .await
            .convert(Some("Error fetching user"))
    }

    pub async fn get_user_by_email(&self, email: &String) -> Result<Option<User>, RouteErr> {
        self.0
            .collection::<User>("users")
            .find_one(
                doc! {
                    "email": email
                },
                None,
            )
            .await
            .convert(Some("Error fetching user"))
    }

    pub async fn get_user_by_account_id(&self, email: &String) -> Result<Option<User>, RouteErr> {
        self.0
            .collection::<User>("users")
            .find_one(
                doc! {
                    "email": email
                },
                None,
            )
            .await
            .convert(Some("Error fetching user"))
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

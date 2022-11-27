use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use mongodb::bson::{self, doc, oid::ObjectId};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{ConvertResult, RouteErr},
    mongo::{oid_as_string, Db, ToObjectId},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename(deserialize = "_id"), serialize_with = "oid_as_string")]
    pub id: ObjectId,
    pub email: String,
    pub name: String,
    pub image: String,
    pub email_verified: Option<DateTime<Utc>>,
    pub access_token: String,
    pub accounts: Vec<Account>,
    pub sessions: Vec<Session>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateUser {
    pub email: String,
    pub name: String,
    pub image: String,
    pub email_verified: Option<DateTime<Utc>>,
    pub access_token: String,
    pub accounts: Vec<Account>,
    pub sessions: Vec<Session>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub provider: String,
    pub provider_type: String,
    pub provider_account_id: String,
    pub access_token: String,
    pub expires_at: u32,
    pub scope: String,
    pub token_type: String,
    pub id_token: String,
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub session_token: String,
    pub user_id: String,
    pub expires: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SessionAndUser {
    pub session: Session,
    pub user: User,
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
                accounts: vec![],
                sessions: vec![],
                access_token: data.access_token,
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

    pub async fn get_user_by_account_id(
        &self,
        provider: &String,
        provider_account_id: &String,
    ) -> Result<Option<User>, RouteErr> {
        self.0
            .collection::<User>("users")
            .find_one(
                doc! {
                    "accounts.provider": provider,
                    "accounts.providerAccountId": provider_account_id
                },
                None,
            )
            .await
            .convert(Some("Error fetching user"))
    }

    pub async fn create_user_account(&self, data: &Account) -> Result<Option<User>, RouteErr> {
        self.0
            .collection::<User>("users")
            .find_one_and_update(
                doc! {
                    "_id": &data.user_id.to_object_id()?
                },
                doc! {
                    "$push": {
                       "accounts": bson::ser::to_bson(data).unwrap()
                    }
                },
                None,
            )
            .await
            .convert(Some("Error linking account"))
    }

    pub async fn create_user_session(&self, data: &Session) -> Result<Option<User>, RouteErr> {
        self.0
            .collection::<User>("users")
            .find_one_and_update(
                doc! {
                    "_id": &data.user_id.to_object_id()?
                },
                doc! {
                    "$push": {
                       "sessions": bson::ser::to_bson(data).unwrap()
                    }
                },
                None,
            )
            .await
            .convert(Some("Error creating session"))
    }

    pub async fn get_session_and_user(
        &self,
        session_token: impl AsRef<str>,
    ) -> Result<Option<SessionAndUser>, RouteErr> {
        let session_and_user = self
            .0
            .collection::<User>("users")
            .find_one(
                doc! {
                    "sessions.sessionToken": session_token.as_ref()
                },
                None,
            )
            .await
            .convert(Some("Error fetching user"))?
            .map(|user| -> Option<SessionAndUser> {
                let user_id = user.id.to_string();
                let session = user
                    .sessions
                    .iter()
                    .find(|sess| sess.user_id == user_id)?
                    .clone();

                Some(SessionAndUser { session, user })
            })
            .convert(None::<String>)?;

        Ok(session_and_user)
    }

    pub async fn delete_session(&self, session_token: &String) -> Result<(), RouteErr> {
        self.0
            .collection::<User>("users")
            .update_one(
                doc! {
                    "sessions.sessionToken": session_token,
                },
                doc! {
                    "$pull": {
                        "sessions": {
                            "sessionToken": session_token,
                        }
                    }
                },
                None,
            )
            .await
            .convert(None::<String>)?;

        Ok(())
    }

    pub async fn update_session_expiry(
        &self,
        session_token: impl AsRef<str>,
        expires: &String,
    ) -> Result<(), RouteErr> {
        let data = self
            .0
            .collection::<User>("users")
            .update_one(
                doc! {
                    "sessions.sessionToken": session_token.as_ref(),
                },
                doc! {
                    "$set": {
                        "sessions.$.expires": expires,
                    }
                },
                None,
            )
            .await
            .convert(None::<String>)?;

        Ok(())
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

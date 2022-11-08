use std::{error::Error, ops::Deref};

use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::ClientOptions,
    Client, Database,
};
use serde::Serializer;

#[derive(Clone)]
pub struct Db {
    pub client: Client,
    pub db: Database,
}

impl Db {
    pub async fn connect() -> anyhow::Result<Self> {
        let database_url = dotenvy::var("DATABASE_URL")?;
        let client_options = ClientOptions::parse(&database_url).await?;
        let client = Client::with_options(client_options)?;

        client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await
            .expect("Unable to ping database.");

        log::info!("Connected to database at {} successfully.", database_url);

        let db = client.clone().database("hydra");

        Ok(Self { client, db })
    }
}

impl Deref for Db {
    type Target = Database;
    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

#[async_trait]
impl<B> FromRequest<B> for Db
where
    B: Send,
{
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let db = req
            .extensions()
            .get::<Self>()
            .expect("db not in extensions");

        Ok(db.clone())
    }
}

pub struct DatabaseError(pub StatusCode, pub &'static str, pub Option<Box<dyn Error>>);

impl IntoResponse for DatabaseError {
    fn into_response(self) -> Response {
        if let Some(err) = self.2 {
            log::error!("Database Error: {}", err);
        }
        (self.0, self.1).into_response()
    }
}

pub trait ToObjectId {
    fn to_object_id(&self) -> Result<ObjectId, DatabaseError>;
}

impl ToObjectId for String {
    fn to_object_id(&self) -> Result<ObjectId, DatabaseError> {
        ObjectId::parse_str(self.as_str()).map_err(|err| {
            DatabaseError(
                StatusCode::BAD_REQUEST,
                "Not an ObjectId",
                Some(Box::new(err)),
            )
        })
    }
}

pub fn oid_as_string<S>(oid: &ObjectId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&oid.to_string())
}

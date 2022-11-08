use std::{error::Error, ops::Deref};

use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongodb::{bson::doc, options::ClientOptions, Client, Collection, Database};

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

pub struct DatabaseError(pub StatusCode, pub &'static str, pub Box<dyn Error>);

impl IntoResponse for DatabaseError {
    fn into_response(self) -> Response {
        log::error!("Database Error: {}", self.2);
        (self.0, self.1).into_response()
    }
}

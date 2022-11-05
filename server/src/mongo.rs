use mongodb::{bson::doc, options::ClientOptions, Client};

pub async fn connect_to_mongodb() -> anyhow::Result<Client> {
    let database_url = dotenvy::var("DATABASE_URL")?;
    let client_options = ClientOptions::parse(&database_url).await?;
    let client = Client::with_options(client_options)?;

    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await
        .expect("Unable to ping database.");

    log::info!("Connected to database at {} successfully.", database_url);

    Ok(client)
}

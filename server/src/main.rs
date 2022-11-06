mod filters;
mod mongo;
mod recovery;

use warp::Filter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let db = mongo::connect_to_mongodb().await?;
    db.database("hydra")
        .collection("users")
        .insert_one(
            mongodb::bson::doc! {
                "a": 20
            },
            None,
        )
        .await?;

    let index = warp::path!().map(|| "Hello world!");

    let all_filters = index.or(filters::auth::auth_filter());

    warp::serve(all_filters.recover(recovery::rejection_handler))
        .run(([0, 0, 0, 0], 3001))
        .await;

    Ok(())
}

mod mongo;

use warp::Filter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let db = mongo::connect_to_mongodb().await?;

    let index = warp::path!().map(|| "Hello world!");

    warp::serve(index).run(([0, 0, 0, 0], 3001)).await;

    Ok(())
}

mod context;
mod filters;
mod mongo;
mod rejections;

use context::Context;
use rejections::rejection_handler;
use warp::Filter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let client = mongo::connect_to_mongodb().await?;
    let context = &Context::new(client);

    let index = warp::path!().map(|| "Hello world!");

    let all_filters = index.or(filters::auth::auth_filter(context));

    warp::serve(all_filters.recover(rejection_handler))
        .run(([0, 0, 0, 0], 3001))
        .await;

    Ok(())
}

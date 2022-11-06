use mongodb::bson::doc;
use warp::{Filter, Rejection, Reply};

use crate::{context::Context, rejections::IntoRejection};

pub fn auth_filter(ctx: &Context) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let ctx = ctx.to_filter();

    let create_user = warp::path!("create-user")
        .and(ctx)
        .and_then(|ctx: Context| async move {
            ctx.db
                .collection("users")
                .insert_one(
                    doc! {
                        "name": "Hello"
                    },
                    None,
                )
                .await
                .handle()?;

            return Ok::<_, Rejection>("hello");
        });

    warp::path!("auth" / ..).and(create_user)
}

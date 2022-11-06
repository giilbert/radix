use warp::{Filter, Reply};

pub fn auth_filter() -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    let create_user = warp::path!("create-user").and_then(|| async move {
        return Ok::<_, warp::Rejection>("hello");
    });

    warp::path!("auth" / ..).and(create_user)
}

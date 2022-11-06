use std::convert::Infallible;

use mongodb::{Client, Database};
use warp::Filter;

#[derive(Clone)]
pub struct Context {
    pub db: Database,
}

impl Context {
    pub fn new(client: Client) -> Self {
        Self {
            db: client.database("hydra"),
        }
    }

    pub fn to_filter(&self) -> impl Filter<Extract = (Self,), Error = Infallible> + Clone {
        let this = self.clone();
        warp::any().map(move || this.clone())
    }
}

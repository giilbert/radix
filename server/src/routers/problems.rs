use axum::{
    extract::{Path, Query},
    routing::{get, post, put},
    Json, Router,
};
use mongodb::bson::oid::ObjectId;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    errors::RouteErr,
    models::{
        problem::{ListingProblem, Problem, ProblemRepo, UpdateProblem},
        user::User,
    },
    AppState,
};

pub fn problem_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_problem))
        .route("/:id", get(get_by_id))
        .route("/:id", put(update_problem))
        .route("/infinite", get(get_infinite))
        .route("/search", get(search))
}

#[derive(Deserialize)]
struct Pagination {
    cursor: Option<ObjectId>,
}

async fn get_infinite(
    Query(query): Query<Pagination>,
    problem_repo: ProblemRepo,
) -> Result<Json<Vec<ListingProblem>>, RouteErr> {
    let problems = problem_repo.get_paginate(query.cursor).await?;
    Ok(Json(problems))
}

#[derive(Serialize)]
struct CreateProblemResult {
    id: String,
}

async fn create_problem(
    user: User,
    problem_repo: ProblemRepo,
) -> Result<Json<CreateProblemResult>, RouteErr> {
    let new_problem_id = problem_repo.create_empty(user.to_public()).await?;

    Ok(Json(CreateProblemResult {
        id: new_problem_id.to_string(),
    }))
}

async fn update_problem(
    user: User,
    problem_repo: ProblemRepo,
    Path(problem_id): Path<String>,
    Json(data): Json<UpdateProblem>,
) -> Result<(), RouteErr> {
    let problem_id = ObjectId::parse_str(problem_id)
        .map_err(|_| RouteErr::Msg(StatusCode::BAD_REQUEST, "Invalid id.".into()))?;
    problem_repo.update(&problem_id, &user.id, &data).await?;
    Ok(())
}

async fn get_by_id(
    user: User,
    Path(problem_id): Path<String>,
    problem_repo: ProblemRepo,
) -> Result<Json<Problem>, RouteErr> {
    let problem_id = ObjectId::parse_str(problem_id)
        .map_err(|_| RouteErr::Msg(StatusCode::BAD_REQUEST, "Invalid id.".into()))?;
    let problem = problem_repo.get_by_id(&problem_id).await?;

    if let Some(mut problem) = problem {
        if problem.author.id != user.id {
            problem.test_cases.truncate(5);
        }

        return Ok(Json(problem));
    }

    Err(RouteErr::Msg(
        StatusCode::NOT_FOUND,
        "Problem not found.".into(),
    ))
}

#[derive(Deserialize)]
struct Search {
    query: String,
}

async fn search(
    problem_repo: ProblemRepo,
    Query(query): Query<Search>,
) -> Result<Json<Vec<ListingProblem>>, RouteErr> {
    let problems = problem_repo.search(&query.query).await?;
    Ok(Json(problems))
}

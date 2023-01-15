use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use futures::stream::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{doc, oid::ObjectId, to_document, Bson},
    options::FindOptions,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{ConvertResult, RouteErr},
    mongo::{oid_as_string, Db},
};

use super::user::PublicUser;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestCase {
    pub input: String,
    pub output: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Problem {
    #[serde(rename(deserialize = "_id"), serialize_with = "oid_as_string")]
    pub id: ObjectId,
    pub title: String,
    pub author: PublicUser,
    pub description: String,
    pub boilerplate_code: Code,
    pub test_cases: Vec<TestCase>,
    pub difficulty: u8,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublicProblem {
    #[serde(rename(deserialize = "_id"), serialize_with = "oid_as_string")]
    pub id: ObjectId,
    pub title: String,
    pub author: PublicUser,
    pub description: String,
    pub boilerplate_code: Code,
    pub default_test_cases: Vec<TestCase>,
    pub difficulty: u8,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListingProblem {
    #[serde(rename(deserialize = "_id"), serialize_with = "oid_as_string")]
    pub id: ObjectId,
    pub title: String,
    pub author: PublicUser,
    pub description: String,
    pub difficulty: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Code {
    pub python: String,
    pub javascript: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProblem {
    pub title: String,
    pub description: String,
    pub test_cases: Vec<TestCase>,
    pub boilerplate_code: Code,
    pub difficulty: i32,
}

#[derive(Clone)]
pub struct ProblemRepo(Db);

impl ProblemRepo {
    pub async fn get_paginate(
        &self,
        cursor: Option<ObjectId>,
    ) -> Result<Vec<ListingProblem>, RouteErr> {
        let cursor = self
            .0
            .collection::<ListingProblem>("problems")
            .find(
                cursor.map(|id| {
                    doc! {
                        "_id": {
                            "$gt": id
                        }
                    }
                }),
                Some(FindOptions::builder().limit(10).build()),
            )
            .await
            .convert(Some("Error fetching problems."))?;

        let problems = cursor
            .try_collect::<Vec<_>>()
            .await
            .convert(Some("Error fetching problems."))?;

        Ok(problems)
    }

    pub async fn create_empty(&self, author: PublicUser) -> Result<ObjectId, RouteErr> {
        self.0
            .collection("problems")
            .insert_one(
                doc! {
                    "author": to_document(&author).unwrap(),
                    "title": "Untitled",
                    "testCases": [],
                    "description": "",
                    "boilerplateCode": to_document(&Code {
                        ..Default::default()
                    }).unwrap(),
                    "difficulty": 0,
                },
                None,
            )
            .await
            .convert(Some("Error creating problem."))
            .map(|res| res.inserted_id.as_object_id().unwrap())
    }

    pub async fn get_by_id(&self, id: &ObjectId) -> Result<Option<Problem>, RouteErr> {
        self.0
            .collection::<Problem>("problems")
            .find_one(
                doc! {
                    "_id": id
                },
                None,
            )
            .await
            .convert(Some("Error fetching problem."))
    }

    pub async fn update(
        &self,
        problem_id: &ObjectId,
        user_id: &ObjectId,
        data: &UpdateProblem,
    ) -> Result<(), RouteErr> {
        let test_cases = data
            .test_cases
            .iter()
            .map(|t| {
                doc! {
                    "input": t.input.clone(),
                    "output": t.output.clone(),
                }
            })
            .collect::<Vec<_>>();

        let res = self
            .0
            .collection::<Problem>("problems")
            .update_one(
                doc! {
                    "_id": problem_id,
                    "author.id": user_id.to_string()
                },
                doc! {
                    "$set": {
                        "difficulty": &data.difficulty,
                        "title": &data.title,
                        "description": &data.description,
                        "boilerplateCode": {
                            "javascript": &data.boilerplate_code.javascript,
                            "python": &data.boilerplate_code.python,
                        },
                        "testCases": &test_cases,
                    }
                },
                None,
            )
            .await
            .convert(Some("Error updating problem."))?;

        if res.matched_count == 0 {
            return Err(RouteErr::Msg(
                StatusCode::UNAUTHORIZED,
                "Unauthorized update of problem.".into(),
            ));
        }

        Ok(())
    }

    pub async fn search(&self, what: &String) -> Result<Vec<ListingProblem>, RouteErr> {
        let cursor = self
            .0
            .collection::<ListingProblem>("problems")
            .find(
                doc! {
                    "$text": {
                        "$search": what
                    }
                },
                Some(FindOptions::builder().limit(10).build()),
            )
            .await
            .convert(Some("Error fetching problems."))?;

        let problems = cursor
            .try_collect::<Vec<_>>()
            .await
            .convert(Some("Error fetching problems."))?;

        Ok(problems)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for ProblemRepo
where
    S: Send + Sync,
{
    type Rejection = StatusCode;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let db = parts.extensions.get::<Db>().unwrap();
        Ok(Self(db.clone()))
    }
}

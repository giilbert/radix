use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestCase {
    pub input: String,
    pub output: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Problem {
    pub id: ObjectId,
    pub title: String,
    pub description: String,
    pub boilerplate_code: Code,
    pub test_cases: Vec<TestCase>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProblemPublic {
    pub id: String,
    pub title: String,
    pub description: String,
    pub boilerplate_code: Code,
    pub default_test_cases: Vec<TestCase>,
}

#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Code {
    pub python: String,
    pub javascript: String,
}

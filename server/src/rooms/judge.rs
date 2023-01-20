use piston_rs::{Client, Executor, File};
use serde::Serialize;
use serde_json::Value;

use crate::models::problem::TestCase;

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FailedTestCase {
    pub input: String,
    pub output: String,
    pub expected: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JudgingResults {
    pub failed_tests: Vec<FailedTestCase>,
    pub okay_tests: Vec<TestCase>,
}

// TODO: language
pub async fn judge(
    language: &String,
    code: &String,
    test_cases: &[TestCase],
) -> anyhow::Result<JudgingResults> {
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let client = if let Ok(url) = dotenvy::var("PISTON_URL") {
        Client::with_url(&url)
    } else {
        Client::default()
    };
    let executor = Executor::new()
        .set_language("python")
        .set_version("3.10.0")
        .add_files(vec![File::new(
            "main.py",
            &format!("{}\n\n{}", code, python_runner(test_cases)?),
            "utf8",
        )]);

    let result = client
        .execute(&executor)
        .await
        .map_err(|err| anyhow::anyhow!("{}", err))?;

    if result.run.stderr != "" {
        return Err(anyhow::anyhow!(
            "Error running code:\n{}",
            result.run.stderr
        ));
    }

    let output = serde_json::from_str::<Vec<Value>>(
        result
            .run
            .stdout
            .lines()
            .last()
            .ok_or_else(|| anyhow::anyhow!("Program did not output anything."))?
            .trim()
            .replace("[[RADIX TEST OUTPUT]] ", "")
            .as_str(),
    )?;

    let mut failed_tests = vec![];
    let mut okay_tests = vec![];

    for (got, test_case) in output.iter().zip(test_cases) {
        if got.to_string()
            == serde_json::to_string(&serde_json::from_str::<Value>(&test_case.output)?)?
        {
            okay_tests.push(test_case.clone());
        } else {
            failed_tests.push(FailedTestCase {
                input: test_case.input.clone(),
                output: got.to_string(),
                expected: test_case.output.clone(),
            });
        }
    }

    //     log::info!(
    //         "{:#?}\n\nfailed: {:?}\n\nokay: {:?}",
    //         result,
    //         failed_tests,
    //         okay_tests
    //     );

    Ok(JudgingResults {
        failed_tests,
        okay_tests,
    })
}

fn python_runner(test_cases: &[TestCase]) -> anyhow::Result<String> {
    let inputs = test_cases
        .iter()
        .map(|test_case| serde_json::from_str::<Value>(&test_case.input))
        .filter(|possible| possible.is_ok())
        .map(|p| p.unwrap())
        .collect::<Vec<Value>>();

    let code = format!(
        r#"""
# RADIX TEST STUFF -- DO NOT TAMPER

import json

__RADIX_TEST_INPUTS = json.loads("{}")
output = []

for input in __RADIX_TEST_INPUTS:
    output.append(solve(*input))

print("[[RADIX TEST OUTPUT]]", json.dumps(output, separators=(",",":")))

"""#,
        serde_json::to_string(&inputs)?.replace("\"", "\\\"")
    );

    // let expected = serde_json::to_string(
    //     &test_cases
    //         .iter()
    //         .map(|test_case| test_case.output.clone())
    //         .collect::<Vec<_>>(),
    // )?;

    Ok(code)
}

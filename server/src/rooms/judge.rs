use lazy_static::lazy_static;
use parking_lot::RwLock;
use piston_rs::{Client, ExecResponse, Executor, File};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

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
    pub runtime: u32,
}

#[derive(Deserialize, Debug)]
struct TestOutput {
    pub runtime: u32,
    pub program_output: Vec<Value>,
}

lazy_static! {
    static ref PISTON_SLASH_JOB: Regex = Regex::new("/piston/jobs/[a-zA-Z0-9-]+/").unwrap();
}

static JOB_QUEUE: RwLock<Option<mpsc::Sender<(Executor, oneshot::Sender<ExecResponse>)>>> =
    RwLock::new(None);

// TODO: language
pub async fn judge(
    language: &String,
    code: &String,
    test_cases: &[TestCase],
) -> anyhow::Result<JudgingResults> {
    // tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let executor = Executor::new()
        .set_language("python")
        .set_version("3.10.0")
        .add_files(vec![File::new(
            "main.py",
            &format!("{}\n\n{}", code, python_runner(test_cases)?),
            "utf8",
        )]);

    let result = run_job(executor).await?;

    if result.run.stderr != "" {
        return Err(anyhow::anyhow!(
            "Error running code:\n{}",
            PISTON_SLASH_JOB.replace_all(&result.run.stderr, "")
        ));
    }

    let output = serde_json::from_str::<TestOutput>(
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

    for (got, test_case) in output.program_output.iter().zip(test_cases) {
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
        runtime: output.runtime,
    })
}

async fn run_job(executor: Executor) -> anyhow::Result<ExecResponse> {
    let (tx, rx) = oneshot::channel::<ExecResponse>();

    if JOB_QUEUE.read().is_none() {
        let (tx, mut rx) = mpsc::channel::<(Executor, oneshot::Sender<ExecResponse>)>(500);
        tokio::spawn(async move {
            let client = if let Ok(url) = dotenvy::var("PISTON_URL") {
                Client::with_url(&url)
            } else {
                Client::default()
            };

            while let Some((executor, tx)) = rx.recv().await {
                let result = client
                    .execute(&executor)
                    .await
                    .map_err(|err| anyhow::anyhow!("{}", err))?;
                tx.send(result).unwrap();

                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            }

            Ok::<_, anyhow::Error>(())
        });
        *JOB_QUEUE.write() = Some(tx);
    }

    let job_queue = JOB_QUEUE.read().as_ref().unwrap().clone();
    job_queue.send((executor, tx)).await?;

    Ok(rx.await?)
}

const PYTHON_TEMPLATE: &str = include_str!("./templates/python-runner.py");
fn python_runner(test_cases: &[TestCase]) -> anyhow::Result<String> {
    let inputs = test_cases
        .iter()
        .map(|test_case| serde_json::from_str::<Value>(&test_case.input))
        .filter(|possible| possible.is_ok())
        .map(|p| p.unwrap())
        .collect::<Vec<Value>>();

    let code = PYTHON_TEMPLATE.replace("{{INPUTS}}", &serde_json::to_string(&inputs)?);

    // let expected = serde_json::to_string(
    //     &test_cases
    //         .iter()
    //         .map(|test_case| test_case.output.clone())
    //         .collect::<Vec<_>>(),
    // )?;

    Ok(code)
}

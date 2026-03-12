use anyhow::Result;
use std::process::Command;

pub struct TestJob {
    pub crate_name: String,
    pub features: String,
    pub test_threads_1: bool,
    /// `--test <name>` selects a specific integration test binary (file in tests/)
    pub integration_test: Option<String>,
    /// filter passed after `--` to libtest
    pub module: Option<String>,
}

pub struct JobResult {
    pub crate_name: String,
    pub features: String,
    pub success: bool,
}

pub fn run_job(job: &TestJob) -> Result<JobResult> {
    let mut cmd = Command::new("cargo");
    cmd.arg("test").arg("-p").arg(&job.crate_name);

    if !job.features.is_empty() {
        cmd.arg("--features").arg(&job.features);
    }
    if let Some(t) = &job.integration_test {
        cmd.arg("--test").arg(t);
    }

    let mut libtest_args: Vec<String> = vec![];
    if let Some(m) = &job.module {
        libtest_args.push(m.clone());
    }
    if job.test_threads_1 {
        libtest_args.push("--test-threads=1".to_string());
    }
    if !libtest_args.is_empty() {
        cmd.arg("--");
        cmd.args(&libtest_args);
    }

    let separator = "=".repeat(50);
    println!("\n{separator}");
    print!("Testing {}", job.crate_name);
    if !job.features.is_empty() {
        print!(" [features: {}]", job.features);
    }
    if let Some(t) = &job.integration_test {
        print!(" [--test {t}]");
    }
    if let Some(m) = &job.module {
        print!(" [filter: {m}]");
    }
    println!();
    let args: Vec<String> = std::iter::once("cargo".to_string())
        .chain(cmd.get_args().map(|a| a.to_string_lossy().into_owned()))
        .collect();
    println!("Command: {}", args.join(" "));
    println!("{separator}\n");

    let status = cmd.status()?;

    Ok(JobResult {
        crate_name: job.crate_name.clone(),
        features: job.features.clone(),
        success: status.success(),
    })
}

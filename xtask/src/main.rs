mod config;
mod runner;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use config::{CRATES, find_crate};
use inquire::{InquireError, MultiSelect, Select, Text};
use runner::{TestJob, run_job};
use std::process::Command;

#[derive(Parser)]
#[command(name = "cargo xtask", about = "Dev task runner for chicken")]
struct Cli {
    #[command(subcommand)]
    command: Task,
}

#[derive(Subcommand)]
enum Task {
    /// Create a new release: bump workspace version, commit, tag, push
    Release {
        /// Version to release (e.g. 0.2.0 or v0.2.0)
        version: String,
    },
    /// Run tests
    Test {
        /// Crate to test (omit for all)
        #[arg(short, long)]
        crate_name: Option<String>,

        /// Feature alias or comma-separated features (omit for all known feature sets)
        #[arg(short, long)]
        features: Option<String>,

        /// Module filter passed to libtest (e.g. `foo::tests`)
        #[arg(short, long)]
        module: Option<String>,

        /// Interactively select crates and features
        #[arg(short = 'i', long)]
        interactive: bool,

        /// CI mode: no interactive prompt, structured output
        #[arg(long)]
        ci: bool,
    },
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args()
        .enumerate()
        .filter_map(|(i, a)| if i == 1 && a == "xtask" { None } else { Some(a) })
        .collect();

    let cli = Cli::parse_from(args);

    match cli.command {
        Task::Release { version } => release(version),
        Task::Test { crate_name, features, module, interactive, ci } => {
            run_tests(crate_name, features, module, interactive, ci)
        }
    }
}

fn run_tests(
    crate_arg: Option<String>,
    features_arg: Option<String>,
    module: Option<String>,
    interactive: bool,
    ci: bool,
) -> Result<()> {
    let jobs: Vec<TestJob> = if interactive && !ci {
        build_jobs_interactive()?
    } else if ci {
        build_jobs_ci()
    } else {
        build_jobs(crate_arg, features_arg, module)
    };

    if jobs.is_empty() {
        return Ok(());
    }

    let mut failed: Vec<(String, String)> = vec![];
    for job in &jobs {
        let result = run_job(job)?;
        if !result.success {
            failed.push((result.crate_name, result.features));
        }
    }

    let sep = "=".repeat(50);
    println!("\n{sep}");
    if failed.is_empty() {
        println!("ALL TESTS PASSED");
    } else {
        println!("FAILED TESTS:");
        for (krate, feat) in &failed {
            if feat.is_empty() { println!("  {krate}"); }
            else { println!("  {krate} (features: {feat})"); }
        }
    }
    println!("{sep}");

    if !failed.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}

fn build_jobs_ci() -> Vec<TestJob> {
    let mut jobs = vec![];
    for cfg in CRATES.iter().filter(|c| c.ci) {
        for (_, feat) in cfg.features.iter() {
            // Unit tests
            jobs.push(TestJob {
                crate_name: cfg.name.to_string(),
                features: feat.to_string(),
                test_threads_1: cfg.test_threads_1,
                integration_test: None,
                module: None,
            });
            // All integration tests
            for &it in cfg.integration_tests.iter() {
                jobs.push(TestJob {
                    crate_name: cfg.name.to_string(),
                    features: feat.to_string(),
                    test_threads_1: cfg.test_threads_1,
                    integration_test: Some(it.to_string()),
                    module: None,
                });
            }
        }
    }
    jobs
}

fn build_jobs(
    crate_arg: Option<String>,
    features_arg: Option<String>,
    module: Option<String>,
) -> Vec<TestJob> {
    let crates_to_test: Vec<&config::CrateConfig> = match &crate_arg {
        Some(name) => find_crate(name).into_iter().collect(),
        None => CRATES.iter().collect(),
    };

    let mut jobs = vec![];
    for cfg in crates_to_test {
        for feat in resolve_features(cfg, features_arg.as_deref()) {
            jobs.push(TestJob {
                crate_name: cfg.name.to_string(),
                features: feat,
                test_threads_1: cfg.test_threads_1,
                integration_test: None,
                module: module.clone(),
            });
        }
    }
    jobs
}

fn resolve_features(cfg: &config::CrateConfig, feature_arg: Option<&str>) -> Vec<String> {
    match feature_arg {
        None => cfg.features.iter().map(|(_, v)| v.to_string()).collect(),
        Some(arg) => {
            if let Some((_, v)) = cfg.features.iter().find(|(k, _)| *k == arg) {
                vec![v.to_string()]
            } else {
                vec![arg.to_string()]
            }
        }
    }
}

// ─── Interactive wizard ───────────────────────────────────────────────────────

/// `Ok(Some(v))` = confirmed, `Ok(None)` = Esc pressed (go back), `Err` = real error.
fn or_back<T>(result: Result<T, InquireError>) -> Result<Option<T>> {
    match result {
        Ok(v) => Ok(Some(v)),
        Err(InquireError::OperationCanceled) => Ok(None),
        Err(InquireError::OperationInterrupted) => std::process::exit(0),
        Err(e) => Err(e.into()),
    }
}

#[derive(Clone, Default)]
struct CrateAnswers {
    features: Vec<usize>,
    kind: usize, // 0 = unit, 1 = integration, 2 = both
    module: Option<String>,
    integration_tests: Vec<usize>,
}

#[derive(Clone)]
enum Phase {
    SelectCrates,
    Features(usize),
    Kind(usize),
    Module(usize),
    IntegrationTests(usize),
    Done,
}

fn build_jobs_interactive() -> Result<Vec<TestJob>> {
    let crate_names: Vec<&str> = CRATES.iter().map(|c| c.name).collect();

    let mut selected_crates: Vec<usize> = vec![];
    let mut answers: Vec<CrateAnswers> = vec![];
    let mut phase = Phase::SelectCrates;

    loop {
        match phase.clone() {
            Phase::Done => break,

            Phase::SelectCrates => {
                match or_back(
                    MultiSelect::new("Select crates to test:", crate_names.clone()).prompt(),
                )? {
                    None => { println!("Cancelled."); return Ok(vec![]); }
                    Some(sel) if sel.is_empty() => { println!("No crates selected."); return Ok(vec![]); }
                    Some(sel) => {
                        // map names back to indices
                        let indices: Vec<usize> = sel.iter()
                            .filter_map(|name| crate_names.iter().position(|n| n == name))
                            .collect();
                        if indices != selected_crates {
                            selected_crates = indices;
                            answers = vec![CrateAnswers::default(); selected_crates.len()];
                        }
                        phase = Phase::Features(0);
                    }
                }
            }

            Phase::Features(pos) => {
                let cfg = &CRATES[selected_crates[pos]];
                let feature_labels: Vec<String> = cfg.features.iter()
                    .map(|(alias, feat)| {
                        if feat.is_empty() { format!("{alias} (no features)") }
                        else { format!("{alias} [{feat}]") }
                    })
                    .collect();

                let prompt = format!("[{}/{}] Features for {}:", pos + 1, selected_crates.len(), cfg.name);
                match or_back(MultiSelect::new(&prompt, feature_labels.clone()).prompt())? {
                    None => phase = if pos == 0 { Phase::SelectCrates } else { last_phase_of(pos - 1, &selected_crates, &answers) },
                    Some(sel) => {
                        // map labels back to indices
                        answers[pos].features = sel.iter()
                            .filter_map(|label| feature_labels.iter().position(|l| l == label))
                            .collect();
                        phase = Phase::Kind(pos);
                    }
                }
            }

            Phase::Kind(pos) => {
                let cfg = &CRATES[selected_crates[pos]];
                let mut options = vec!["unit tests"];
                if !cfg.integration_tests.is_empty() {
                    options.push("integration tests");
                    options.push("both");
                }

                let prompt = format!("[{}/{}] What to run for {}:", pos + 1, selected_crates.len(), cfg.name);
                match or_back(Select::new(&prompt, options.clone()).prompt())? {
                    None => phase = Phase::Features(pos),
                    Some(choice) => {
                        let k = options.iter().position(|o| *o == choice).unwrap_or(0);
                        answers[pos].kind = k;
                        phase = if k == 1 { Phase::IntegrationTests(pos) } else { Phase::Module(pos) };
                    }
                }
            }

            Phase::Module(pos) => {
                let cfg = &CRATES[selected_crates[pos]];
                let prev = answers[pos].module.clone().unwrap_or_default();
                let prompt = format!("[{}/{}] Module filter for {} (empty = all):", pos + 1, selected_crates.len(), cfg.name);
                match or_back(Text::new(&prompt).with_initial_value(&prev).prompt())? {
                    None => phase = Phase::Kind(pos),
                    Some(input) => {
                        answers[pos].module = if input.trim().is_empty() { None } else { Some(input.trim().to_string()) };
                        phase = if answers[pos].kind == 2 && !cfg.integration_tests.is_empty() {
                            Phase::IntegrationTests(pos)
                        } else {
                            advance(pos, &selected_crates)
                        };
                    }
                }
            }

            Phase::IntegrationTests(pos) => {
                let cfg = &CRATES[selected_crates[pos]];
                let options: Vec<&str> = cfg.integration_tests.to_vec();
                let prompt = format!("[{}/{}] Integration tests for {}:", pos + 1, selected_crates.len(), cfg.name);
                match or_back(MultiSelect::new(&prompt, options.clone()).prompt())? {
                    None => phase = if answers[pos].kind == 2 { Phase::Module(pos) } else { Phase::Kind(pos) },
                    Some(sel) => {
                        answers[pos].integration_tests = sel.iter()
                            .filter_map(|name| options.iter().position(|o| o == name))
                            .collect();
                        phase = advance(pos, &selected_crates);
                    }
                }
            }
        }
    }

    build_jobs_from_answers(&selected_crates, &answers)
}

fn advance(pos: usize, selected_crates: &[usize]) -> Phase {
    if pos + 1 < selected_crates.len() { Phase::Features(pos + 1) } else { Phase::Done }
}

fn last_phase_of(prev_pos: usize, selected_crates: &[usize], answers: &[CrateAnswers]) -> Phase {
    let cfg = &CRATES[selected_crates[prev_pos]];
    let kind = answers[prev_pos].kind;
    if (kind == 1 || kind == 2) && !cfg.integration_tests.is_empty() {
        Phase::IntegrationTests(prev_pos)
    } else {
        Phase::Module(prev_pos)
    }
}

fn build_jobs_from_answers(selected_crates: &[usize], answers: &[CrateAnswers]) -> Result<Vec<TestJob>> {
    let mut jobs = vec![];
    for (pos, &crate_idx) in selected_crates.iter().enumerate() {
        let cfg = &CRATES[crate_idx];
        let ans = &answers[pos];
        let run_unit = ans.kind == 0 || ans.kind == 2;
        let run_integration = (ans.kind == 1 || ans.kind == 2) && !cfg.integration_tests.is_empty();

        for &feat_idx in &ans.features {
            let (_, feat) = cfg.features[feat_idx];

            if run_unit {
                jobs.push(TestJob {
                    crate_name: cfg.name.to_string(),
                    features: feat.to_string(),
                    test_threads_1: cfg.test_threads_1,
                    integration_test: None,
                    module: ans.module.clone(),
                });
            }
            for &it_idx in &ans.integration_tests {
                if run_integration {
                    jobs.push(TestJob {
                        crate_name: cfg.name.to_string(),
                        features: feat.to_string(),
                        test_threads_1: cfg.test_threads_1,
                        integration_test: Some(cfg.integration_tests[it_idx].to_string()),
                        module: None,
                    });
                }
            }
        }
    }
    Ok(jobs)
}

// ─── Release ─────────────────────────────────────────────────────────────────

fn release(version: String) -> Result<()> {
    let version = version.trim_start_matches('v');

    // Validate semver format x.y.z
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 3 || parts.iter().any(|p| p.parse::<u32>().is_err()) {
        bail!("Invalid version: '{}'. Expected format: x.y.z (e.g. 0.2.0)", version);
    }

    let tag = format!("v{}", version);

    // Read current workspace version from root Cargo.toml
    let cargo_toml = std::fs::read_to_string("Cargo.toml")?;
    let current = cargo_toml
        .lines()
        .find(|l| l.trim().starts_with("version = "))
        .and_then(|l| l.split('"').nth(1))
        .unwrap_or("unknown");

    println!("Current: {} → New: {}", current, version);

    // Check working tree is clean
    let status = cmd("git", &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        bail!("Working tree is dirty. Commit or stash changes first.");
    }

    // Bump workspace version in root Cargo.toml
    let updated = cargo_toml.replacen(
        &format!("version = \"{}\"", current),
        &format!("version = \"{}\"", version),
        1,
    );
    std::fs::write("Cargo.toml", updated)?;

    // Commit
    cmd("git", &["add", "Cargo.toml"])?;
    cmd("git", &["commit", "-m", &format!("chore(release): {}", tag)])?;

    // Tag
    cmd("git", &["tag", &tag])?;

    // Push
    cmd("git", &["push", "origin", "main"])?;
    cmd("git", &["push", "origin", &tag])?;

    println!("✅ Released {} — changelog and fos_client PR will be created automatically", tag);
    Ok(())
}

fn cmd(program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program).args(args).output()?;
    if !output.status.success() {
        bail!(
            "{} {} failed:\n{}",
            program,
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

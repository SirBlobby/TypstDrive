mod diagnostics;
mod job;
mod launch;
mod sandbox;
mod settings;

use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use tokio::sync::Semaphore;

use crate::metrics::Metric;

pub use diagnostics::LatexError;
pub use sandbox::warm_up;

use job::Job;
use launch::{run_tectonic, LaunchPlan};
use sandbox::Sandbox;
use settings::latex_settings;

pub struct EngineStatus {
    pub version: Option<String>,
    pub sandbox: Option<&'static str>,
    pub problem: Option<String>,
}

pub fn compile_slots_in_use() -> (usize, usize) {
    let allowed = latex_settings().max_parallel;
    (allowed.saturating_sub(compile_slots().available_permits()), allowed)
}

fn compile_slots() -> &'static Semaphore {
    static SLOTS: OnceLock<Semaphore> = OnceLock::new();
    SLOTS.get_or_init(|| Semaphore::new(latex_settings().max_parallel))
}

pub async fn engine_status() -> EngineStatus {
    match sandbox::sandbox().await {
        Ok(sandbox) => EngineStatus { version: Some(sandbox.version.clone()), sandbox: Some(sandbox.mode.label()), problem: None },
        Err(problem) => EngineStatus { version: None, sandbox: None, problem: Some(problem) },
    }
}

fn pdf_name(entrypoint: &str) -> String {
    let stem = Path::new(entrypoint).file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default();
    format!("{stem}.pdf")
}

fn compile_arguments(job: &Job, entrypoint: &str) -> Vec<String> {
    let mut arguments: Vec<String> = ["-X", "compile", entrypoint, "--untrusted", "--keep-logs", "--outdir"]
        .into_iter()
        .map(String::from)
        .collect();
    arguments.push(job.output_argument());
    if let Some(bundle) = &latex_settings().bundle_url {
        arguments.push("--bundle".to_string());
        arguments.push(bundle.clone());
    }
    arguments
}

async fn discard(job: Job) {
    let _ = tokio::task::spawn_blocking(move || drop(job)).await;
}

async fn run_job(sandbox: &'static Sandbox, job: Job, entrypoint: &str) -> Result<Vec<u8>, Vec<LatexError>> {
    let plan = LaunchPlan { working_directory: job.source_folder(sandbox), arguments: compile_arguments(&job, entrypoint) };
    let finished = match run_tectonic(sandbox, &plan).await {
        Ok(finished) => finished,
        Err(error) => {
            discard(job).await;
            return Err(vec![LatexError::general(format!("tectonic could not start: {error}"))]);
        }
    };

    if finished.timed_out {
        discard(job).await;
        let seconds = latex_settings().timeout.as_secs();
        return Err(vec![LatexError::general(format!("LaTeX took longer than {seconds} seconds and was stopped."))]);
    }

    let pdf = finished.succeeded.then(|| job.read_output(&pdf_name(entrypoint)).ok()).flatten();
    discard(job).await;
    match pdf {
        Some(bytes) if !bytes.is_empty() => Ok(bytes),
        _ => Err(diagnostics::parse_errors(&finished.output, entrypoint)),
    }
}

pub async fn compile_to_pdf(entrypoint: &str, files: HashMap<String, Vec<u8>>) -> Result<Vec<u8>, Vec<LatexError>> {
    let sandbox = sandbox::sandbox().await.map_err(|problem| vec![LatexError::general(format!("LaTeX is not available: {problem}"))])?;
    let Some(entry) = job::safe_relative_path(entrypoint) else {
        return Err(vec![LatexError::general("The main LaTeX file has an invalid name.")]);
    };
    let entrypoint = entry.to_string_lossy().into_owned();
    let busy = || vec![LatexError::general("The LaTeX engine is busy. Try again in a moment.")];
    let _slot = tokio::time::timeout(latex_settings().timeout, compile_slots().acquire())
        .await
        .map_err(|_| busy())?
        .map_err(|_| busy())?;

    let job = tokio::task::spawn_blocking(move || Job::create(sandbox, &files))
        .await
        .map_err(|_| vec![LatexError::general("LaTeX compilation stopped unexpectedly.")])?
        .map_err(|error| vec![LatexError::general(format!("Could not prepare the LaTeX files: {error}"))])?;

    crate::metrics::timed(Metric::LatexCompile, run_job(sandbox, job, &entrypoint)).await
}

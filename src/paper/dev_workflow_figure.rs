//! Publication figure: developer workflow phase breakdown.

use super::Artifact;
use crate::BenchResults;
use anyhow::{Context, Result};
use std::path::Path;

const WORKLOAD: &str = "dev-workflow";

const FACETS: &[(&str, &[&str])] = &[
    ("Worktree", &["worktree", "checkpoint-worktree"]),
    (
        "Init. Build",
        &[
            "config",
            "checkpoint-config",
            "initial-build",
            "checkpoint-initial-build",
        ],
    ),
    ("Read", &["search", "read"]),
    ("Edit", &["edit", "checkpoint-edit"]),
    (
        "Incr. Build",
        &["incremental-build", "checkpoint-incremental-build"],
    ),
    (
        "Git",
        &[
            "git-status",
            "git-diff",
            "git-add",
            "git-commit",
            "checkpoint-git-commit",
        ],
    ),
];

const BACKENDS: &[(&str, &str)] = &[("yolo-realistic", "YoloFS"), ("overlayfs", "OverlayFS")];

pub fn artifact_meta(paper_dir: &Path) -> Artifact {
    let plot_pdf = paper_dir.join("dev-workflow-figure.pdf");
    Artifact {
        preferred: true,
        plot_pdfs: vec![plot_pdf],
    }
}

pub fn render(results: &BenchResults, paper_dir: &Path) -> Result<Artifact> {
    let wl = results
        .workloads
        .iter()
        .find(|w| crate::report::normalize_legacy_workload_name(&w.workload) == WORKLOAD)
        .with_context(|| format!("{WORKLOAD} not found in results"))?;

    let native = wl
        .backends
        .iter()
        .find(|b| b.backend == "native")
        .with_context(|| format!("native backend missing for {WORKLOAD}"))?;

    let mut csv_lines = Vec::new();
    csv_lines.push(
        "facet,backend,run_s,checkpoint_s,native_run_s,run_total_s,checkpoint_total_s,commit_s,native_total_s"
            .to_string(),
    );

    for &(facet_label, categories) in FACETS {
        let native_run_ms = sum_categories(native, categories, false);
        for &(backend_key, backend_label) in BACKENDS {
            let Some(backend) = wl.backends.iter().find(|b| b.backend == backend_key) else {
                continue;
            };
            let run_s = sum_categories(backend, categories, false) / 1000.0;
            let checkpoint_s = sum_categories(backend, categories, true) / 1000.0;
            let run_total_s = (backend.mean_init_ms.unwrap_or(0.0)
                + backend.mean_staging_ms.unwrap_or(backend.mean_total_ms))
                / 1000.0;
            let checkpoint_total_s = sum_categories(
                backend,
                &[
                    "checkpoint-worktree",
                    "checkpoint-config",
                    "checkpoint-initial-build",
                    "checkpoint-edit",
                    "checkpoint-incremental-build",
                    "checkpoint-git-commit",
                ],
                true,
            ) / 1000.0;
            let native_total_s = (native.mean_init_ms.unwrap_or(0.0)
                + native.mean_staging_ms.unwrap_or(native.mean_total_ms))
                / 1000.0;
            csv_lines.push(format!(
                "{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
                facet_label,
                backend_label,
                run_s,
                checkpoint_s,
                native_run_ms / 1000.0,
                run_total_s,
                checkpoint_total_s,
                backend.mean_commit_ms.unwrap_or(0.0) / 1000.0,
                native_total_s,
            ));
        }
    }

    let csv_path = paper_dir.join("dev-workflow.csv");
    std::fs::write(&csv_path, csv_lines.join("\n"))
        .with_context(|| format!("writing {}", csv_path.display()))?;

    eprintln!("CSV written to {}", csv_path.display());

    Ok(Artifact {
        preferred: true,
        plot_pdfs: vec![paper_dir.join("dev-workflow-figure.pdf")],
    })
}

fn sum_categories(backend: &crate::BackendResult, categories: &[&str], checkpoints: bool) -> f64 {
    let Some(series) = &backend.macro_step_series else {
        return 0.0;
    };
    series
        .steps
        .iter()
        .filter(|step| {
            let Some(category) = crate::report::dev_workflow_step_category(&step.step) else {
                return false;
            };
            categories.contains(&category) && category.starts_with("checkpoint-") == checkpoints
        })
        .map(|step| step.ms as f64)
        .sum()
}

//! Publication figure: metadata operation latency small multiples.
//!
//! Layout: 1 row (100 files) x 7 columns (ops).
//! Within each subplot, x-axis = source (base, stage, snapshot),
//! bars colored by backend. Legend identifies backends.

use super::Artifact;
use super::util::backend_display_name;
use crate::BenchResults;
use crate::report;
use anyhow::{Context, Result};
use std::path::Path;

/// All backends emitted into the CSV (native + bar backends).
const ALL_BACKENDS: &[&str] = &[
    "native",
    "yolo-no-perm",
    "yolo-realistic",
    "overlayfs",
    "branchfs",
];

/// Display names for figure legend.
fn fig_backend_name(key: &str) -> &'static str {
    match key {
        "native" => "Base",
        "yolo-no-perm" => "YoloFS (no perm)",
        "yolo-realistic" => "YoloFS",
        "overlayfs" => "OverlayFS",
        "branchfs" => "BranchFS",
        _ => backend_display_name(key),
    }
}

/// Operations and their workload name stems.
const OPS: &[(&str, &str)] = &[
    ("create", "meta-create"),
    ("open", "meta-open"),
    ("stat", "meta-stat"),
    ("readdir", "meta-readdir"),
    ("append", "meta-append"),
    ("rename", "meta-rename"),
    ("unlink", "meta-unlink"),
];

// ── Public API ──────────────────────────────────────────────────────────────

pub fn render(results: &BenchResults, paper_dir: &Path) -> Result<Vec<Artifact>> {
    let data_csv = build_data_csv(results);
    let csv_path = paper_dir.join("meta-ops.csv");
    std::fs::write(&csv_path, &data_csv)
        .with_context(|| format!("writing {}", csv_path.display()))?;
    eprintln!("CSV written to {}", csv_path.display());

    Ok(vec![Artifact {
        preferred: true,
        plot_pdfs: vec![paper_dir.join("meta-ops-capped.pdf")],
    }])
}

/// Return artifact metadata without rendering (for install-paper).
pub fn artifact_metas(paper_dir: &Path) -> Vec<Artifact> {
    let plot_pdf = paper_dir.join("meta-ops-capped.pdf");
    vec![Artifact {
        preferred: true,
        plot_pdfs: vec![plot_pdf],
    }]
}

// ── Data collection ───────────────────────────────────────────────────────

fn build_data_csv(results: &BenchResults) -> String {
    let mut lines = Vec::new();
    lines.push("op,size,source,backend,lat_us".to_string());

    for &(op_label, stem) in OPS {
        for &(size, size_suffix) in &[(100, "-100")] {
            let sources: &[&str] = if stem == "meta-create" {
                &["stage"]
            } else {
                &["base", "stage", "checkpoint"]
            };

            for &source in sources {
                let wl_name = if stem == "meta-create" {
                    if size == 100 {
                        "meta-create-100".to_string()
                    } else {
                        "meta-create".to_string()
                    }
                } else {
                    format!("{stem}{size_suffix}-{source}")
                };

                for &backend in ALL_BACKENDS {
                    let lat_us = results
                        .workloads
                        .iter()
                        .find(|w| report::normalize_legacy_workload_name(&w.workload) == wl_name)
                        .and_then(|wl| {
                            wl.backends
                                .iter()
                                .find(|b| b.backend == backend)
                                .and_then(|b| b.mean_iops)
                                .map(|iops| 1_000_000.0 / iops)
                        });

                    if let Some(lat) = lat_us {
                        lines.push(format!(
                            "{op_label},{size},{source},{},{lat:.2}",
                            fig_backend_name(backend)
                        ));
                    }
                }
            }
        }
    }

    lines.join("\n")
}

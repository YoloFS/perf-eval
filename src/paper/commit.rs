//! Publication figure: commit time per operation (session microbenchmarks).

use crate::BenchResults;
use crate::report;
use anyhow::{Context, Result};
use std::path::Path;

const WORKLOADS: &[(&str, &str)] = &[
    ("write-files", "create"),
    ("overwrite-files", "overwrite"),
    ("rename-files", "rename"),
    ("unlink-files", "unlink"),
];

const NATIVE_BASELINES: &[(&str, &str)] = &[
    ("create", "meta-create"),
    ("overwrite", "meta-append-base"),
    ("rename", "meta-rename-base"),
    ("unlink", "meta-unlink-base"),
];

/// Backends to show (no native — no commit; no yolo-no-perm).
const BACKENDS: &[(&str, &str)] = &[
    ("yolo-realistic", "YoloFS"),
    ("overlayfs", "OverlayFS"),
    ("branchfs", "BranchFS"),
];

const FILE_COUNT: f64 = 10_000.0;

pub fn render(results: &BenchResults, paper_dir: &Path) -> Result<()> {
    // Collect data: op,backend,metric,us_per_op
    let mut data_lines = Vec::new();
    data_lines.push("op,backend,metric,us_per_op".to_string());
    for &(wl_name, op_label) in WORKLOADS {
        for &(backend_key, backend_label) in BACKENDS {
            let wl = results
                .workloads
                .iter()
                .find(|w| report::normalize_legacy_workload_name(&w.workload) == wl_name)
                .and_then(|wl| wl.backends.iter().find(|b| b.backend == backend_key));

            if let Some(b) = wl {
                let runs = &b.iterations;
                if runs.is_empty() {
                    continue;
                }
                let n = runs.len() as f64;

                let avg_commit: f64 = runs
                    .iter()
                    .map(|r| r.commit_ms.unwrap_or(0) as f64)
                    .sum::<f64>()
                    / n;
                data_lines.push(format!(
                    "{op_label},{backend_label},commit,{:.2}",
                    avg_commit / FILE_COUNT * 1000.0
                ));
            }
        }
    }

    let mut baseline_lines = Vec::new();
    baseline_lines.push("op,us_per_op".to_string());
    for &(op_label, wl_name) in NATIVE_BASELINES {
        let wl = results
            .workloads
            .iter()
            .find(|w| report::normalize_legacy_workload_name(&w.workload) == wl_name)
            .and_then(|wl| wl.backends.iter().find(|b| b.backend == "native"));

        if let Some(b) = wl {
            let runs = &b.iterations;
            if runs.is_empty() {
                continue;
            }
            let avg_total_ms =
                runs.iter().map(|r| r.total_ms as f64).sum::<f64>() / runs.len() as f64;
            baseline_lines.push(format!(
                "{op_label},{:.2}",
                avg_total_ms / FILE_COUNT * 1000.0
            ));
        }
    }

    let data_path = paper_dir.join("commit-time.csv");
    let baseline_path = paper_dir.join("commit-time-baseline.csv");

    std::fs::write(&data_path, data_lines.join("\n"))
        .with_context(|| format!("writing {}", data_path.display()))?;
    std::fs::write(&baseline_path, baseline_lines.join("\n"))
        .with_context(|| format!("writing {}", baseline_path.display()))?;

    eprintln!("CSV written to {}", data_path.display());

    Ok(())
}

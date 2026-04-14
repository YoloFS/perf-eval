//! Publication figure: checkpoint depth scaling (split create/read/status/commit latency).

use anyhow::{Context, Result};
use std::path::Path;

pub fn render(out_dir: &Path, paper_dir: &Path) -> Result<()> {
    let json_path = crate::checkpoint_scaling_json_path(out_dir);
    if !json_path.exists() {
        anyhow::bail!(
            "checkpoint-scaling.json not found — run `yolo-bench checkpoint-scaling` first"
        );
    }

    #[derive(serde::Deserialize)]
    struct CheckpointScalingResult {
        #[serde(default)]
        backend: String,
        mode: String,
        points: Vec<CheckpointScalingPoint>,
    }
    #[derive(serde::Deserialize)]
    struct CheckpointScalingPoint {
        depth: usize,
        mean_us: f64,
    }

    let data: Vec<CheckpointScalingResult> =
        serde_json::from_str(&std::fs::read_to_string(&json_path)?)?;

    // For each backend, find the maximum depth that create or read achieved.
    let mut max_depth_per_backend: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for res in &data {
        if res.mode == "commit" || res.mode == "status" {
            continue;
        }
        let max_d = res.points.iter().map(|p| p.depth).max().unwrap_or(0);
        let entry = max_depth_per_backend
            .entry(res.backend.clone())
            .or_insert(0);
        *entry = (*entry).max(max_d);
    }

    let header = "backend,depth,mean_us";
    let mut create_lines = vec![header.to_string()];
    let mut read_lines = vec![header.to_string()];
    let mut commit_lines = vec![header.to_string()];
    for res in &data {
        if res.backend == "yolo-no-perm" || res.mode == "status" {
            continue;
        }
        let label = match res.backend.as_str() {
            "yolo-realistic" => "YoloFS",
            other => super::util::backend_display_name(other),
        }
        .to_string();
        let depth_cap = max_depth_per_backend
            .get(&res.backend)
            .copied()
            .unwrap_or(usize::MAX);
        for p in &res.points {
            if res.mode == "commit" && p.depth > depth_cap {
                continue;
            }
            let row = format!("{},{},{:.2}", label, p.depth, p.mean_us);
            match res.mode.as_str() {
                "create" => create_lines.push(row),
                "read" => read_lines.push(row),
                "commit" => commit_lines.push(row),
                _ => {}
            }
        }
    }

    let create_path = paper_dir.join("checkpoint-create.csv");
    let read_path = paper_dir.join("checkpoint-read.csv");
    let commit_path = paper_dir.join("checkpoint-commit.csv");

    std::fs::write(&create_path, create_lines.join("\n"))
        .with_context(|| format!("writing {}", create_path.display()))?;
    std::fs::write(&read_path, read_lines.join("\n"))
        .with_context(|| format!("writing {}", read_path.display()))?;
    std::fs::write(&commit_path, commit_lines.join("\n"))
        .with_context(|| format!("writing {}", commit_path.display()))?;

    eprintln!("CSV written to {}", create_path.display());
    eprintln!("CSV written to {}", read_path.display());
    eprintln!("CSV written to {}", commit_path.display());

    Ok(())
}

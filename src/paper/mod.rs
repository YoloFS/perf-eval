//! Publication-ready artifacts: LaTeX tables, figures, and HTML index.

pub mod checkpoint_scaling_figure;
pub mod commit_time_figure;
pub mod dev_workflow_figure;
pub mod fio_data_table;
pub mod meta_ops_figure;
mod util;

use anyhow::{Context, Result};
use std::path::Path;

/// Directory containing the standalone plot scripts.
fn plot_scripts_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts")
}

/// Run a plot script against a generated-artifact directory.
fn run_plot_script(script_name: &str, generated_dir: &Path) -> Result<()> {
    let script = plot_scripts_dir().join(script_name);
    if !script.exists() {
        anyhow::bail!("plot script not found: {}", script.display());
    }
    eprintln!("$ python3 {} {}", script.display(), generated_dir.display());
    let out = std::process::Command::new("python3")
        .arg(&script)
        .arg(generated_dir)
        .output()
        .with_context(|| format!("running {}", script.display()))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("{script_name} failed: {stderr}");
    }
    // Forward stderr (contains "Figure written to ..." messages).
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !stderr.is_empty() {
        eprint!("{stderr}");
    }
    Ok(())
}

/// Render paper artifacts directly into either a paper repo or an explicit output dir.
pub fn install(results: &crate::BenchResults, out_dir: &Path, paper_path: &Path) -> Result<()> {
    let gen_dir = if paper_path.join("main.tex").exists() {
        paper_path.join("generated")
    } else {
        paper_path.to_path_buf()
    };
    std::fs::create_dir_all(&gen_dir).with_context(|| format!("creating {}", gen_dir.display()))?;
    fio_data_table::render(results, &gen_dir)?;
    meta_ops_figure::render(results, &gen_dir)?;
    commit_time_figure::render(results, &gen_dir)?;
    checkpoint_scaling_figure::render(out_dir, &gen_dir)?;
    dev_workflow_figure::render(results, &gen_dir)?;

    let plot_scripts = [
        "plot_meta_ops.py",
        "plot_commit_time.py",
        "plot_checkpoint_scaling.py",
        "plot_dev_workflow.py",
    ];
    for script in &plot_scripts {
        match run_plot_script(script, &gen_dir) {
            Ok(()) => {}
            Err(e) => eprintln!("  warning: {script}: {e:#}"),
        }
    }

    eprintln!("Paper artifacts written to {}", gen_dir.display());
    Ok(())
}

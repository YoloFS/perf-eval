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
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("paper")
}

/// Run a plot script from `bench/scripts/plot/`.
fn run_plot_script(script_name: &str, out_dir: &Path) -> Result<()> {
    let script = plot_scripts_dir().join(script_name);
    if !script.exists() {
        anyhow::bail!("plot script not found: {}", script.display());
    }
    let out = std::process::Command::new("python3")
        .arg(&script)
        .arg(out_dir)
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

/// Render all paper artifacts.
pub fn render(results: &crate::BenchResults, out_dir: &Path) -> Result<()> {
    let paper_dir = crate::paper_dir(out_dir);
    std::fs::create_dir_all(&paper_dir)
        .with_context(|| format!("creating {}", paper_dir.display()))?;

    let mut artifacts: Vec<Artifact> = Vec::new();

    // ── fio data table ──
    let fio = fio_data_table::render(results, &paper_dir)?;
    artifacts.push(fio);

    // ── metadata ops figure ──
    artifacts.extend(meta_ops_figure::render(results, &paper_dir)?);

    // ── commit time figure ──
    match commit_time_figure::render(results, &paper_dir) {
        Ok(art) => artifacts.push(art),
        Err(e) => eprintln!("  warning: commit-time-figure: {e:#}"),
    }

    // ── checkpoint scaling figure ──
    match checkpoint_scaling_figure::render(out_dir, &paper_dir) {
        Ok(art) => artifacts.push(art),
        Err(e) => eprintln!("  warning: checkpoint-scaling-figure: {e:#}"),
    }

    // ── developer workflow figure ──
    match dev_workflow_figure::render(results, &paper_dir) {
        Ok(art) => artifacts.push(art),
        Err(e) => eprintln!("  warning: dev-workflow-figure: {e:#}"),
    }

    // ── Run plot scripts to generate PDFs from CSVs ──
    let plot_scripts = [
        "plot_meta_ops.py",
        "plot_commit_time.py",
        "plot_checkpoint_scaling.py",
        "plot_dev_workflow.py",
    ];
    for script in &plot_scripts {
        match run_plot_script(script, out_dir) {
            Ok(()) => {}
            Err(e) => eprintln!("  warning: {script}: {e:#}"),
        }
    }

    Ok(())
}

pub(crate) struct Artifact {
    preferred: bool,
    /// Absolute paths to plot PDFs that must be copied alongside the .tex fragment.
    plot_pdfs: Vec<std::path::PathBuf>,
}

/// Install preferred paper artifacts into the paper repository.
///
/// Copies plot PDFs from `<out_dir>/paper/` to `<paper_dir>/generated/`.
///
/// Run `yolo-bench rerender` first if artifacts are stale.
pub fn install(_results: &crate::BenchResults, out_dir: &Path, paper_dir: &Path) -> Result<()> {
    if !paper_dir.join("main.tex").exists() {
        anyhow::bail!(
            "{} does not look like a paper repo (no main.tex)",
            paper_dir.display()
        );
    }

    let bench_paper_dir = crate::paper_dir(out_dir);
    if !bench_paper_dir.exists() {
        anyhow::bail!(
            "{} not found — run `yolo-bench rerender` first",
            bench_paper_dir.display()
        );
    }

    // Collect artifact metadata from the known generators without re-rendering.
    let mut artifacts: Vec<Artifact> = Vec::new();
    artifacts.push(fio_data_table::artifact_meta(&bench_paper_dir));
    artifacts.extend(meta_ops_figure::artifact_metas(&bench_paper_dir));
    artifacts.push(commit_time_figure::artifact_meta(&bench_paper_dir));
    artifacts.push(checkpoint_scaling_figure::artifact_meta(&bench_paper_dir));
    artifacts.push(dev_workflow_figure::artifact_meta(&bench_paper_dir));

    let gen_dir = paper_dir.join("generated");
    std::fs::create_dir_all(&gen_dir).with_context(|| format!("creating {}", gen_dir.display()))?;

    let mut installed = Vec::new();

    for art in &artifacts {
        if !art.preferred {
            continue;
        }

        // Copy associated plot PDFs.
        for pdf in &art.plot_pdfs {
            if pdf.exists() {
                let dest_pdf = gen_dir.join(pdf.file_name().unwrap());
                std::fs::copy(pdf, &dest_pdf).with_context(|| {
                    format!("copying {} → {}", pdf.display(), dest_pdf.display())
                })?;
                installed.push(format!("  pdf: {}", dest_pdf.display()));
            }
        }
    }

    if installed.is_empty() {
        eprintln!("No preferred artifacts to install.");
    } else {
        eprintln!("Installed to {}:", gen_dir.display());
        for line in &installed {
            eprintln!("{line}");
        }
    }

    Ok(())
}

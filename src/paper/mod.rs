//! Publication-ready artifacts: LaTeX tables, figures, and HTML index.

pub mod checkpoint;
pub mod commit;
pub mod dev;
pub mod fio;
pub mod metadata;
mod util;

use anyhow::{Context, Result};
use std::path::Path;

/// Render paper artifacts directly into either a paper repo or an explicit output dir.
pub fn install(results: &crate::BenchResults, out_dir: &Path, paper_path: &Path) -> Result<()> {
    let gen_dir = if paper_path.join("main.tex").exists() {
        paper_path.join("generated")
    } else {
        paper_path.to_path_buf()
    };
    std::fs::create_dir_all(&gen_dir).with_context(|| format!("creating {}", gen_dir.display()))?;
    fio::render(results, &gen_dir)?;
    metadata::render(results, &gen_dir)?;
    commit::render(results, &gen_dir)?;
    checkpoint::render(out_dir, &gen_dir)?;
    dev::render(results, &gen_dir)?;

    eprintln!("Paper artifacts written to {}", gen_dir.display());
    Ok(())
}

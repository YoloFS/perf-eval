use crate::backend::{self, Backend, CheckpointController, CheckpointOutcome};
use crate::workload::{CacheMode, IterResult, Workload, WorkloadKind};
use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;
use yolofs::config::Config;
use yolofs::perm::Perm;

// ── Session (mount / unmount lifecycle) ──────────────────────────────────────

struct Session {
    root: tempfile::TempDir,
}

impl Session {
    /// Set up the session: write config, populate base, `yolofs mount`.
    /// Returns (session, init_ms) where init_ms is the wall time of mount.
    fn setup(config: Config, workload: &dyn Workload) -> Result<(Self, u64)> {
        let cache = dirs_next::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("yolofs-bench");
        std::fs::create_dir_all(&cache).context("creating yolofs-bench cache dir")?;

        let root = tempfile::Builder::new()
            .prefix("yolofs-bench-")
            .tempdir_in(&cache)
            .context("creating session tempdir")?;

        config
            .save(&root.path().join("yolofs.toml"))
            .context("writing yolofs.toml")?;

        // Populate base directory before mounting (not timed).
        let base_work = root.path().join(workload.work_dir());
        std::fs::create_dir_all(&base_work)?;
        workload.populate_base(&base_work)?;

        let t = Instant::now();
        let out = Command::new("yolofs")
            .arg("mount")
            .current_dir(root.path())
            .env("NO_COLOR", "1")
            .output()
            .context("running yolofs mount")?;
        let init_ms = t.elapsed().as_millis() as u64;

        if !out.status.success() {
            bail!(
                "yolofs mount failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }

        Ok((Session { root }, init_ms))
    }

    fn mnt_path(&self, rel: &str) -> PathBuf {
        let root = self.root.path();
        root.join(".yolofs/mnt")
            .join(root.strip_prefix("/").unwrap_or(root))
            .join(rel)
    }

    fn checkpoint_named(&self, name: &str) -> Result<()> {
        let out = Command::new("yolofs")
            .arg("snapshot")
            .arg(name)
            .current_dir(self.root.path())
            .env("NO_COLOR", "1")
            .output()
            .context("running yolofs snapshot")?;
        if !out.status.success() {
            bail!(
                "yolofs snapshot failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(())
    }

    fn commit(&self, verbose: bool) -> Result<u64> {
        let t = Instant::now();
        let out = Command::new("yolofs")
            .arg("commit")
            .current_dir(self.root.path())
            .env("NO_COLOR", "1")
            .stdout(if verbose {
                Stdio::inherit()
            } else {
                Stdio::null()
            })
            .output()
            .context("running yolofs commit")?;
        let ms = t.elapsed().as_millis() as u64;
        if !out.status.success() {
            bail!(
                "yolofs commit failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(ms)
    }

    fn status_time(&self) -> Result<u64> {
        // Run the same logic as `yolofs review` in-process:
        // read journal → resolve segments → build tree → classify + print each entry.
        // Output goes to /dev/null to avoid I/O noise.
        let t = Instant::now();
        // Temporarily redirect stdout to suppress output.
        let devnull = std::fs::File::create("/dev/null")?;
        let saved_stdout = nix::unistd::dup(1)?;
        nix::unistd::dup2(devnull.as_raw_fd(), 1)?;
        drop(devnull);

        // Set cwd so session_dir() finds .yolofs/.
        let saved_cwd = std::env::current_dir()?;
        std::env::set_current_dir(self.root.path())?;
        let result = yolofs::cmd::review::run_review(None, None, false, false);
        std::env::set_current_dir(saved_cwd)?;

        // Restore stdout.
        nix::unistd::dup2(saved_stdout, 1)?;
        nix::unistd::close(saved_stdout)?;

        result.context("in-process yolofs review")?;
        Ok(t.elapsed().as_micros() as u64)
    }
}

impl CheckpointController for Session {
    fn checkpoint(&mut self, step: usize) -> Result<CheckpointOutcome> {
        let t = Instant::now();
        self.checkpoint_named(&format!("bench-step-{step:03}"))?;
        Ok(CheckpointOutcome::Continue {
            checkpoint_ms: t.elapsed().as_millis() as u64,
            next_dest: None,
        })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = Command::new("yolofs")
            .arg("unmount")
            .current_dir(self.root.path())
            .env("NO_COLOR", "1")
            .output();
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Build an exec-workload command wrapped in `yolofs exec --` so the subprocess
/// runs inside the yolofs sandbox (proper process tracking, permission gating).
/// Build an exec-workload command wrapped in `yolofs exec --` so the subprocess
/// runs inside the yolofs sandbox (proper process tracking, permission gating).
///
/// `yolofs exec` needs cwd = session root to find `.yolofs/`. The inner
/// `exec-workload` receives an absolute `--dest` path so it doesn't
/// depend on cwd.
fn yolo_exec_workload_cmd(
    session: &Session,
    workload_name: &str,
    dest: &std::path::Path,
    verbose: bool,
    wait_after_ready: bool,
) -> Result<Command> {
    let self_exe = std::env::current_exe().context("resolving current executable")?;
    let mut cmd = Command::new("yolofs");
    cmd.arg("exec")
        .arg("--")
        .arg(self_exe)
        .arg("exec-workload")
        .arg("--name")
        .arg(workload_name)
        .arg("--dest")
        .arg(dest);
    if verbose {
        cmd.arg("--verbose");
    }
    if wait_after_ready {
        cmd.arg("--wait-after-ready");
    }
    // yolofs exec finds the session from cwd.
    cmd.current_dir(session.root.path());
    Ok(cmd)
}

fn run_yolo_iteration(
    config: Config,
    workload: &dyn Workload,
    verbose: bool,
) -> Result<(IterResult, Vec<String>)> {
    let (mut session, init_ms) = Session::setup(config, workload)?;

    let dest = session.mnt_path(workload.work_dir());
    // For cold+base workloads, skip create_dir_all on the mounted path —
    // the directory already exists in the lower fs via populate_base().
    // Traversing the yolofs mount here would warm kernel state that
    // drop_caches cannot flush (igrab'd directory inodes).
    let cold = workload.cache_mode() == CacheMode::DropPageCache;
    if !cold || workload.needs_prepare_workdir() {
        std::fs::create_dir_all(&dest)?;
    }
    if cold && !workload.needs_prepare_workdir() {
        std::fs::create_dir_all(session.mnt_path(workload.work_dir()))?;
    }
    if workload.needs_prepare_workdir() {
        workload.prepare_workdir(&dest)?;
    }
    if workload.needs_checkpoint() {
        session.checkpoint_named("bench-checkpoint")?;
    }
    // For cold workloads, page caches are dropped after READY (inside
    // run_workload_subprocess). The mount-point dentry chain stays pinned
    // in the VFS — that's inherent to any mounted filesystem and means
    // cold stat through yolofs will always be faster than cold stat on native
    // (fewer unpinned path components to read from disk).
    let cold = workload.cache_mode() == CacheMode::DropPageCache;
    let mut cmd = if workload.kind() == WorkloadKind::Macro {
        // Inside the yolofs exec chroot, the root is .yolofs/mnt which mirrors /.
        // Use the session root path (not the mount path) so we don't
        // double-traverse the mount.
        let chroot_dest = session.root.path().join(workload.work_dir());
        yolo_exec_workload_cmd(&session, workload.name(), &chroot_dest, verbose, cold)?
    } else {
        let mut c =
            backend::exec_workload_cmd(workload.name(), std::path::Path::new("."), verbose, cold)?;
        c.current_dir(&dest);
        c
    };
    cmd.stderr(if verbose {
        Stdio::inherit()
    } else {
        Stdio::piped()
    });
    let sp = backend::spawn_and_await_ready(&mut cmd, cold)?;
    if cold {
        crate::workloads::drop_page_cache()
            .context("dropping page cache after subprocess READY")?;
    }
    let result = sp.go_with_checkpoint(&mut session)?;

    // Measure status query time (yolofs review).
    let status_us = session.status_time()?;

    let commit_ms = session.commit(verbose)?;

    Ok((
        IterResult {
            init_ms: Some(init_ms),
            staging_ms: Some(result.staging_ms),
            status_us: Some(status_us),
            commit_ms: Some(commit_ms),
            total_ms: init_ms + result.staging_ms + commit_ms,
            op_result: result.op_result,
            checkpoint_series: result.checkpoint_series,
            macro_step_series: result.macro_step_series,
        },
        vec![],
    ))
}

// ── yolo-no-perm ─────────────────────────────────────────────────────────────

pub struct YoloNoPerm;

fn cold_staged_reason(workload: &dyn Workload) -> Option<&'static str> {
    let cold_staged_metadata = workload.name().starts_with("meta-")
        && workload.cache_mode() == CacheMode::DropPageCache
        && workload.needs_prepare_workdir();
    if cold_staged_metadata {
        Some(
            "cold metadata on staged/checkpoint files cannot be measured on yolofs: \
              flushing the kernel dirent table requires unmounting, which loses staging state",
        )
    } else {
        None
    }
}

impl Backend for YoloNoPerm {
    fn name(&self) -> &'static str {
        "yolo-no-perm"
    }

    fn unsupported_reason(&self, workload: &dyn Workload) -> Option<&'static str> {
        cold_staged_reason(workload)
    }

    fn default_skip_reason(&self, workload: &dyn Workload) -> Option<&'static str> {
        if workload.name() == "dev-workflow" {
            Some(
                "dev-workflow only runs yolo-realistic by default; use --backend yolo-no-perm to include",
            )
        } else {
            None
        }
    }

    fn run_one(&self, workload: &dyn Workload, verbose: bool) -> Result<(IterResult, Vec<String>)> {
        let config = Config {
            permission: false,
            auto_snapshot: false,
            rules: BTreeMap::new(),
            ..Default::default()
        };
        run_yolo_iteration(config, workload, verbose)
    }
}

// ── yolo-realistic ───────────────────────────────────────────────────────────

pub struct YoloRealistic;

impl Backend for YoloRealistic {
    fn name(&self) -> &'static str {
        "yolo-realistic"
    }

    fn unsupported_reason(&self, workload: &dyn Workload) -> Option<&'static str> {
        cold_staged_reason(workload)
    }

    fn run_one(&self, workload: &dyn Workload, verbose: bool) -> Result<(IterResult, Vec<String>)> {
        let cache = dirs_next::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("yolofs-bench");
        std::fs::create_dir_all(&cache)?;
        let root = tempfile::Builder::new()
            .prefix("yolofs-bench-")
            .tempdir_in(&cache)?;

        let mut rules = BTreeMap::new();
        for (path, perm) in workload.realistic_rules(root.path()) {
            rules.insert(path, perm);
        }
        // Macro workloads run via `yolofs exec` which chroots into the mount.
        // The subprocess needs read access to the bench binary, system
        // libraries, and common tool paths.
        if workload.kind() == WorkloadKind::Macro {
            // yolofs exec chroots into the mount; the subprocess needs
            // read+execute access to the bench binary, system libraries,
            // linker, and tools (tar, xz, etc.).
            if let Ok(exe) = std::env::current_exe() {
                if let Some(dir) = exe.parent() {
                    rules.insert(dir.to_string_lossy().into_owned(), Perm::Read);
                }
            }
            for dir in ["/usr", "/lib", "/lib64", "/bin", "/sbin"] {
                rules.insert(dir.to_string(), Perm::Read);
            }
        }
        let config = Config {
            permission: true,
            auto_snapshot: false,
            rules,
            ..Default::default()
        };
        config.save(&root.path().join("yolofs.toml"))?;

        // Populate base directory before mounting (not timed).
        let base_work = root.path().join(workload.work_dir());
        std::fs::create_dir_all(&base_work)?;
        workload.populate_base(&base_work)?;

        let t_init = Instant::now();
        let out = Command::new("yolofs")
            .arg("mount")
            .current_dir(root.path())
            .env("NO_COLOR", "1")
            .output()
            .context("running yolofs mount")?;
        let init_ms = t_init.elapsed().as_millis() as u64;

        if !out.status.success() {
            bail!(
                "yolofs mount failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }

        let mut session = Session { root };
        let dest = session.mnt_path(workload.work_dir());
        let cold = workload.cache_mode() == CacheMode::DropPageCache;
        if !cold || workload.needs_prepare_workdir() {
            std::fs::create_dir_all(&dest)?;
        }
        if cold && !workload.needs_prepare_workdir() {
            std::fs::create_dir_all(session.mnt_path(workload.work_dir()))?;
        }
        if workload.needs_prepare_workdir() {
            workload.prepare_workdir(&dest)?;
        }
        if workload.needs_checkpoint() {
            session.checkpoint_named("bench-checkpoint")?;
        }
        let mut cmd = if workload.kind() == WorkloadKind::Macro {
            let chroot_dest = session.root.path().join(workload.work_dir());
            yolo_exec_workload_cmd(&session, workload.name(), &chroot_dest, verbose, cold)?
        } else {
            let mut c = backend::exec_workload_cmd(
                workload.name(),
                std::path::Path::new("."),
                verbose,
                cold,
            )?;
            c.current_dir(&dest);
            c
        };
        cmd.stderr(if verbose {
            Stdio::inherit()
        } else {
            Stdio::piped()
        });
        let sp = backend::spawn_and_await_ready(&mut cmd, cold)?;
        if cold {
            crate::workloads::drop_page_cache()
                .context("dropping page cache after subprocess READY")?;
        }
        let result = sp.go_with_checkpoint(&mut session)?;
        let status_us = session.status_time()?;

        let commit_ms = session.commit(verbose)?;

        Ok((
            IterResult {
                init_ms: Some(init_ms),
                staging_ms: Some(result.staging_ms),
                status_us: Some(status_us),
                commit_ms: Some(commit_ms),
                total_ms: init_ms + result.staging_ms + commit_ms,
                op_result: result.op_result,
                checkpoint_series: result.checkpoint_series,
                macro_step_series: result.macro_step_series,
            },
            vec![],
        ))
    }
}

// ── For profiler (needs Session access) ──────────────────────────────────────

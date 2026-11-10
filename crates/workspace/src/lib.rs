//! Project folders on disk. Git is driven through the `git` CLI via `tokio::process`.

use std::{path::Path, process::Stdio, time::Duration};

use tokio::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    /// The target folder already exists.
    #[error("folder already exists")]
    AlreadyExists,
    /// `git` exited with an error. The message is short and has no credentials.
    #[error("git failed: {0}")]
    Git(String),
    /// `git` did not finish within the timeout. It has been killed.
    #[error("clone timed out")]
    Timeout,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Create `dir` and run `git init` in it. Errors with `AlreadyExists` if `dir` exists.
pub async fn init_scratch(dir: &Path) -> Result<(), WorkspaceError> {
    match tokio::fs::create_dir(dir).await {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(WorkspaceError::AlreadyExists)
        }
        Err(e) => return Err(e.into()),
    }
    let out = Command::new("git")
        .arg("init")
        .arg("-q")
        .current_dir(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .output()
        .await;
    match out {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => {
            let _ = tokio::fs::remove_dir_all(dir).await;
            Err(WorkspaceError::Git(short_message(&out.stderr)))
        }
        Err(e) => {
            let _ = tokio::fs::remove_dir_all(dir).await;
            Err(e.into())
        }
    }
}

/// `git clone -- <url> <dir>`. On any failure or timeout, `dir` is removed.
pub async fn clone_repo(url: &str, dir: &Path, timeout: Duration) -> Result<(), WorkspaceError> {
    let result = run_clone(url, dir, timeout).await;
    if result.is_err() {
        let _ = tokio::fs::remove_dir_all(dir).await;
    }
    result
}

async fn run_clone(url: &str, dir: &Path, timeout: Duration) -> Result<(), WorkspaceError> {
    let child = Command::new("git")
        .arg("clone")
        .arg("--")
        .arg(url)
        .arg(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        // Dropping the wait future on timeout kills git.
        .kill_on_drop(true)
        .spawn()?;
    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Err(_) => Err(WorkspaceError::Timeout),
        Ok(Err(e)) => Err(e.into()),
        Ok(Ok(out)) if out.status.success() => Ok(()),
        Ok(Ok(out)) => Err(WorkspaceError::Git(short_message(&out.stderr))),
    }
}

/// Remove a project folder. A missing folder is `Ok`.
pub async fn remove_project_dir(dir: &Path) -> Result<(), WorkspaceError> {
    match tokio::fs::remove_dir_all(dir).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// Last non-empty line of git's stderr, without credentials.
fn short_message(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let line = text
        .lines()
        .rev()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("git failed");
    redact_credentials(line)
}

/// Drop `user:password@` from any `scheme://user:password@host` in `text`.
fn redact_credentials(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("://") {
        let (head, tail) = rest.split_at(i + 3);
        out.push_str(head);
        let end = tail
            .find(|c: char| c == '/' || c.is_whitespace() || c == '\'' || c == '"')
            .unwrap_or(tail.len());
        let authority = &tail[..end];
        match authority.rfind('@') {
            Some(at) => out.push_str(&authority[at + 1..]),
            None => out.push_str(authority),
        }
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

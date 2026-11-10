//! Project folders on disk. Git is driven through the `git` CLI via `tokio::process`.

use std::{path::Path, time::Duration};

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
pub async fn init_scratch(_dir: &Path) -> Result<(), WorkspaceError> {
    todo!()
}

/// `git clone -- <url> <dir>`. On any failure or timeout, `dir` is removed.
pub async fn clone_repo(_url: &str, _dir: &Path, _timeout: Duration) -> Result<(), WorkspaceError> {
    todo!()
}

/// Remove a project folder. A missing folder is `Ok`.
pub async fn remove_project_dir(_dir: &Path) -> Result<(), WorkspaceError> {
    todo!()
}

//! B. Workspace crate tests: temp dirs, real `git`, no network.
//! The clone source is a local bare repo made in the test. The timeout test uses a
//! localhost TCP listener that never answers.

use std::{
    net::TcpListener,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use tempfile::TempDir;
use workspace::{clone_repo, init_scratch, remove_project_dir, WorkspaceError};

/// Make a bare repo with one commit (README.md = "hello") and return its path.
fn make_bare_repo(base: &Path) -> PathBuf {
    let src = base.join("src");
    std::fs::create_dir(&src).unwrap();
    let git = |args: &[&str], cwd: &Path| {
        let ok = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.invalid")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.invalid")
            .status()
            .unwrap()
            .success();
        assert!(ok, "git {args:?} failed");
    };
    git(&["init", "-q"], &src);
    std::fs::write(src.join("README.md"), "hello").unwrap();
    git(&["add", "README.md"], &src);
    git(&["commit", "-q", "-m", "init"], &src);
    let bare = base.join("remote.git");
    git(
        &[
            "clone",
            "-q",
            "--bare",
            src.to_str().unwrap(),
            bare.to_str().unwrap(),
        ],
        base,
    );
    bare
}

#[tokio::test]
async fn init_scratch_creates_folder_with_git_dir() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("proj");
    init_scratch(&dir).await.unwrap();
    assert!(dir.join(".git").is_dir());
}

#[tokio::test]
async fn init_scratch_on_existing_folder_errors_and_keeps_folder() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("proj");
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("marker.txt"), "keep").unwrap();
    let err = init_scratch(&dir).await.unwrap_err();
    assert!(matches!(err, WorkspaceError::AlreadyExists));
    assert_eq!(
        std::fs::read_to_string(dir.join("marker.txt")).unwrap(),
        "keep"
    );
    assert!(!dir.join(".git").exists());
}

#[tokio::test]
async fn clone_repo_from_local_bare_repo_copies_files() {
    let tmp = TempDir::new().unwrap();
    let remote = make_bare_repo(tmp.path());
    let dir = tmp.path().join("proj");
    clone_repo(remote.to_str().unwrap(), &dir, Duration::from_secs(60))
        .await
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("README.md")).unwrap(),
        "hello"
    );
}

#[tokio::test]
async fn clone_bad_source_errors_and_leaves_no_folder() {
    let tmp = TempDir::new().unwrap();
    let missing = tmp.path().join("does-not-exist.git");
    let dir = tmp.path().join("proj");
    let err = clone_repo(missing.to_str().unwrap(), &dir, Duration::from_secs(60))
        .await
        .unwrap_err();
    assert!(matches!(err, WorkspaceError::Git(_)));
    assert!(!dir.exists());
}

#[tokio::test]
async fn clone_timeout_errors_and_leaves_no_folder() {
    let tmp = TempDir::new().unwrap();
    // Accepts TCP connections but never answers the git protocol.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let url = format!("git://127.0.0.1:{port}/repo.git");
    let dir = tmp.path().join("proj");
    let err = clone_repo(&url, &dir, Duration::from_millis(300))
        .await
        .unwrap_err();
    assert!(matches!(err, WorkspaceError::Timeout));
    assert!(!dir.exists());
    drop(listener);
}

#[tokio::test]
async fn remove_project_dir_deletes_folder() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("proj");
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    std::fs::write(dir.join("sub/file.txt"), "x").unwrap();
    remove_project_dir(&dir).await.unwrap();
    assert!(!dir.exists());
}

#[tokio::test]
async fn remove_project_dir_missing_folder_is_ok() {
    let tmp = TempDir::new().unwrap();
    remove_project_dir(&tmp.path().join("missing"))
        .await
        .unwrap();
}

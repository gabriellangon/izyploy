use std::{future::Future, io, path::PathBuf, pin::Pin};

use tokio::process::Command;

pub type CloneFuture = Pin<Box<dyn Future<Output = io::Result<CloneOutput>> + Send>>;

pub trait GitClient: Send + Sync {
    fn clone_repository(&self, request: CloneRequest) -> CloneFuture;
}

#[derive(Debug, Clone)]
pub struct CloneRequest {
    pub git_url: String,
    pub branch: String,
    pub destination: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CloneOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Default)]
pub struct CommandGitClient;

impl GitClient for CommandGitClient {
    fn clone_repository(&self, request: CloneRequest) -> CloneFuture {
        Box::pin(async move {
            let output = Command::new("git")
                .arg("clone")
                .arg("--depth")
                .arg("1")
                .arg("--single-branch")
                .arg("--branch")
                .arg(request.branch)
                .arg("--")
                .arg(request.git_url)
                .arg(request.destination)
                .env("GIT_TERMINAL_PROMPT", "0")
                .kill_on_drop(true)
                .output()
                .await?;

            Ok(CloneOutput {
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            })
        })
    }
}

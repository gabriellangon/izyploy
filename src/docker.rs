use std::{future::Future, io, path::PathBuf, pin::Pin};

use tokio::process::Command;

pub type BuildFuture = Pin<Box<dyn Future<Output = io::Result<BuildOutput>> + Send>>;

pub trait DockerClient: Send + Sync {
    fn build_image(&self, request: BuildRequest) -> BuildFuture;
}

#[derive(Debug, Clone)]
pub struct BuildRequest {
    pub context: PathBuf,
    pub image_tag: String,
    pub labels: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct BuildOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Default)]
pub struct CommandDockerClient;

impl DockerClient for CommandDockerClient {
    fn build_image(&self, request: BuildRequest) -> BuildFuture {
        Box::pin(async move {
            let mut command = Command::new("docker");
            command.arg("build").arg("--tag").arg(request.image_tag);

            for (key, value) in request.labels {
                command.arg("--label").arg(format!("{key}={value}"));
            }

            let output = command
                .arg("--")
                .arg(request.context)
                .kill_on_drop(true)
                .output()
                .await?;

            Ok(BuildOutput {
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            })
        })
    }
}

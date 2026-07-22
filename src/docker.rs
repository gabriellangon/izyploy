use std::{future::Future, io, path::PathBuf, pin::Pin};

use tokio::process::Command;

pub type BuildFuture = Pin<Box<dyn Future<Output = io::Result<BuildOutput>> + Send>>;
pub type CommandFuture = Pin<Box<dyn Future<Output = io::Result<CommandOutput>> + Send>>;
pub type PortFuture = Pin<Box<dyn Future<Output = io::Result<PortOutput>> + Send>>;

pub trait DockerClient: Send + Sync {
    fn build_image(&self, request: BuildRequest) -> BuildFuture;
    fn run_container(&self, request: RunContainerRequest) -> CommandFuture;
    fn inspect_host_port(&self, request: PortRequest) -> PortFuture;
    fn container_logs(&self, container_name: String) -> CommandFuture;
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

#[derive(Debug, Clone)]
pub struct RunContainerRequest {
    pub container_name: String,
    pub image_tag: String,
    pub container_port: u16,
    pub environment: Vec<(String, String)>,
    pub labels: Vec<(String, String)>,
    pub limits: ResourceLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLimits {
    pub cpus: String,
    pub memory: String,
    pub pids: u32,
}

#[derive(Debug, Clone)]
pub struct PortRequest {
    pub container_name: String,
    pub container_port: u16,
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub struct PortOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub host_port: Option<u16>,
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

    fn run_container(&self, request: RunContainerRequest) -> CommandFuture {
        Box::pin(async move {
            let mut command = Command::new("docker");
            command
                .arg("run")
                .arg("--detach")
                .arg("--name")
                .arg(request.container_name)
                .arg("--cpus")
                .arg(request.limits.cpus)
                .arg("--memory")
                .arg(request.limits.memory)
                .arg("--pids-limit")
                .arg(request.limits.pids.to_string())
                .arg("--publish")
                .arg(format!("127.0.0.1::{}", request.container_port));

            for (key, value) in request.environment {
                command.arg("--env").arg(format!("{key}={value}"));
            }
            for (key, value) in request.labels {
                command.arg("--label").arg(format!("{key}={value}"));
            }

            let output = command
                .arg("--")
                .arg(request.image_tag)
                .kill_on_drop(true)
                .output()
                .await?;

            Ok(command_output(output))
        })
    }

    fn inspect_host_port(&self, request: PortRequest) -> PortFuture {
        Box::pin(async move {
            let output = Command::new("docker")
                .arg("port")
                .arg(request.container_name)
                .arg(format!("{}/tcp", request.container_port))
                .kill_on_drop(true)
                .output()
                .await?;
            let command_output = command_output(output);
            let host_port = command_output
                .success
                .then(|| parse_host_port(&command_output.stdout))
                .flatten();

            Ok(PortOutput {
                success: command_output.success,
                exit_code: command_output.exit_code,
                host_port,
                stdout: command_output.stdout,
                stderr: command_output.stderr,
            })
        })
    }

    fn container_logs(&self, container_name: String) -> CommandFuture {
        Box::pin(async move {
            let output = Command::new("docker")
                .arg("logs")
                .arg("--")
                .arg(container_name)
                .kill_on_drop(true)
                .output()
                .await?;

            Ok(command_output(output))
        })
    }
}

fn command_output(output: std::process::Output) -> CommandOutput {
    CommandOutput {
        success: output.status.success(),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn parse_host_port(output: &str) -> Option<u16> {
    output
        .lines()
        .find_map(|line| line.trim().rsplit_once(':')?.1.parse().ok())
}

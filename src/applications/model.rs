use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Application {
    pub id: Uuid,
    pub name: String,
    pub git_url: String,
    pub branch: String,
    pub build_context: String,
    pub container_port: u16,
    pub status: ApplicationStatus,
    pub host_port: Option<u16>,
    pub url: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApplicationStatus {
    Queued,
    Cloning,
    Building,
    Starting,
    Running,
    Failed,
    Deleting,
}

impl ApplicationStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Cloning => "cloning",
            Self::Building => "building",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Failed => "failed",
            Self::Deleting => "deleting",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "queued" => Ok(Self::Queued),
            "cloning" => Ok(Self::Cloning),
            "building" => Ok(Self::Building),
            "starting" => Ok(Self::Starting),
            "running" => Ok(Self::Running),
            "failed" => Ok(Self::Failed),
            "deleting" => Ok(Self::Deleting),
            _ => Err(format!("unknown application status: {value}")),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateApplicationRequest {
    pub name: String,
    pub git_url: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default = "default_build_context")]
    pub build_context: String,
    pub container_port: u16,
}

fn default_branch() -> String {
    "main".to_owned()
}

fn default_build_context() -> String {
    ".".to_owned()
}

#[derive(Debug)]
pub(crate) struct NewApplication {
    pub name: String,
    pub git_url: String,
    pub branch: String,
    pub build_context: String,
    pub container_port: u16,
}

use url::Url;

use super::model::{CreateApplicationRequest, NewApplication};

#[derive(Debug)]
pub(crate) struct ValidationError {
    pub field: &'static str,
    pub message: String,
}

impl ValidationError {
    fn new(field: &'static str, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}

pub(crate) fn validate(
    request: CreateApplicationRequest,
) -> Result<NewApplication, ValidationError> {
    let name = request.name.trim().to_owned();
    if name.is_empty() || name.chars().count() > 100 || name.chars().any(char::is_control) {
        return Err(ValidationError::new(
            "name",
            "must contain between 1 and 100 visible characters",
        ));
    }

    let git_url = request.git_url.trim().to_owned();
    validate_git_url(&git_url)?;

    let branch = request.branch.trim().to_owned();
    validate_branch(&branch)?;

    let build_context = request.build_context.trim().to_owned();
    validate_build_context(&build_context)?;

    if request.container_port == 0 {
        return Err(ValidationError::new(
            "container_port",
            "must be between 1 and 65535",
        ));
    }

    Ok(NewApplication {
        name,
        git_url,
        branch,
        build_context,
        container_port: request.container_port,
    })
}

fn validate_git_url(value: &str) -> Result<(), ValidationError> {
    let url = Url::parse(value)
        .map_err(|_| ValidationError::new("git_url", "must be a valid HTTPS GitHub URL"))?;
    let path_segments = url
        .path_segments()
        .map(|segments| {
            segments
                .filter(|segment| !segment.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let repository = path_segments
        .get(1)
        .map(|segment| segment.strip_suffix(".git").unwrap_or(segment));

    let valid = url.scheme() == "https"
        && url.host_str() == Some("github.com")
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none()
        && url.query().is_none()
        && url.fragment().is_none()
        && path_segments.len() == 2
        && path_segments[0] != "."
        && path_segments[0] != ".."
        && repository.is_some_and(|name| !name.is_empty() && name != "." && name != "..");

    if valid {
        Ok(())
    } else {
        Err(ValidationError::new(
            "git_url",
            "must identify one public repository on https://github.com",
        ))
    }
}

fn validate_branch(value: &str) -> Result<(), ValidationError> {
    let contains_forbidden_character = value.chars().any(|character| {
        character.is_control() || character.is_whitespace() || "~^:?*[\\".contains(character)
    });
    let valid = !value.is_empty()
        && value.len() <= 255
        && !value.starts_with(['/', '-', '.'])
        && !value.ends_with(['/', '.'])
        && !value.ends_with(".lock")
        && !value.contains("..")
        && !value.contains("@{")
        && !value.contains("//")
        && !contains_forbidden_character;

    if valid {
        Ok(())
    } else {
        Err(ValidationError::new(
            "branch",
            "must be a valid Git branch name",
        ))
    }
}

fn validate_build_context(value: &str) -> Result<(), ValidationError> {
    let valid = value == "."
        || (!value.is_empty()
            && !value.starts_with(['/', '\\'])
            && !value.ends_with('/')
            && !value.contains('\\')
            && value
                .split('/')
                .all(|segment| !segment.is_empty() && segment != "." && segment != ".."));

    if valid {
        Ok(())
    } else {
        Err(ValidationError::new(
            "build_context",
            "must be '.' or a normalized relative path without '..'",
        ))
    }
}

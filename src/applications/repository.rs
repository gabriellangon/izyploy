use std::io::{Error as IoError, ErrorKind};

use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool, sqlite::SqliteRow};
use uuid::Uuid;

use super::model::{Application, ApplicationStatus, NewApplication};

pub(crate) async fn create(
    database: &SqlitePool,
    new_application: NewApplication,
) -> Result<Application, sqlx::Error> {
    let now = Utc::now();
    let application = Application {
        id: Uuid::new_v4(),
        name: new_application.name,
        git_url: new_application.git_url,
        branch: new_application.branch,
        build_context: new_application.build_context,
        container_port: new_application.container_port,
        status: ApplicationStatus::Queued,
        host_port: None,
        url: None,
        error: None,
        created_at: now,
        updated_at: now,
    };

    sqlx::query(
        "INSERT INTO applications (
            id, name, git_url, branch, build_context, container_port, status,
            host_port, url, error, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(application.id.to_string())
    .bind(&application.name)
    .bind(&application.git_url)
    .bind(&application.branch)
    .bind(&application.build_context)
    .bind(i64::from(application.container_port))
    .bind(application.status.as_str())
    .bind(application.host_port.map(i64::from))
    .bind(&application.url)
    .bind(&application.error)
    .bind(application.created_at.to_rfc3339())
    .bind(application.updated_at.to_rfc3339())
    .execute(database)
    .await?;

    Ok(application)
}

pub(crate) async fn list(database: &SqlitePool) -> Result<Vec<Application>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, name, git_url, branch, build_context, container_port, status,
                host_port, url, error, created_at, updated_at
         FROM applications
         ORDER BY created_at ASC, id ASC",
    )
    .fetch_all(database)
    .await?;

    rows.into_iter().map(application_from_row).collect()
}

pub(crate) async fn find_by_id(
    database: &SqlitePool,
    id: Uuid,
) -> Result<Option<Application>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, name, git_url, branch, build_context, container_port, status,
                host_port, url, error, created_at, updated_at
         FROM applications
         WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(database)
    .await?;

    row.map(application_from_row).transpose()
}

pub(crate) async fn transition_status(
    database: &SqlitePool,
    id: Uuid,
    current_status: ApplicationStatus,
    next_status: ApplicationStatus,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query(
        "UPDATE applications
         SET status = ?, error = NULL, updated_at = ?
         WHERE id = ? AND status = ?",
    )
    .bind(next_status.as_str())
    .bind(Utc::now().to_rfc3339())
    .bind(id.to_string())
    .bind(current_status.as_str())
    .execute(database)
    .await?;

    if result.rows_affected() == 1 {
        Ok(())
    } else {
        Err(sqlx::Error::Protocol(format!(
            "application {id} did not transition from {} to {}",
            current_status.as_str(),
            next_status.as_str()
        )))
    }
}

pub(crate) async fn mark_failed(
    database: &SqlitePool,
    id: Uuid,
    error: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE applications
         SET status = 'failed', error = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(error)
    .bind(Utc::now().to_rfc3339())
    .bind(id.to_string())
    .execute(database)
    .await?;

    Ok(())
}

pub(crate) async fn append_log(
    database: &SqlitePool,
    application_id: Uuid,
    stage: &str,
    stream: &str,
    message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO deployment_logs (application_id, stage, stream, message, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(application_id.to_string())
    .bind(stage)
    .bind(stream)
    .bind(message)
    .bind(Utc::now().to_rfc3339())
    .execute(database)
    .await?;

    Ok(())
}

fn application_from_row(row: SqliteRow) -> Result<Application, sqlx::Error> {
    let id = Uuid::parse_str(&row.try_get::<String, _>("id")?)
        .map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
    let status =
        ApplicationStatus::parse(&row.try_get::<String, _>("status")?).map_err(decode_error)?;
    let container_port = decode_port(row.try_get("container_port")?)?;
    let host_port = row
        .try_get::<Option<i64>, _>("host_port")?
        .map(decode_port)
        .transpose()?;
    let created_at = decode_timestamp(&row.try_get::<String, _>("created_at")?)?;
    let updated_at = decode_timestamp(&row.try_get::<String, _>("updated_at")?)?;

    Ok(Application {
        id,
        name: row.try_get("name")?,
        git_url: row.try_get("git_url")?,
        branch: row.try_get("branch")?,
        build_context: row.try_get("build_context")?,
        container_port,
        status,
        host_port,
        url: row.try_get("url")?,
        error: row.try_get("error")?,
        created_at,
        updated_at,
    })
}

fn decode_port(value: i64) -> Result<u16, sqlx::Error> {
    u16::try_from(value).map_err(|error| sqlx::Error::Decode(Box::new(error)))
}

fn decode_timestamp(value: &str) -> Result<DateTime<Utc>, sqlx::Error> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|error| sqlx::Error::Decode(Box::new(error)))
}

fn decode_error(message: String) -> sqlx::Error {
    sqlx::Error::Decode(Box::new(IoError::new(ErrorKind::InvalidData, message)))
}

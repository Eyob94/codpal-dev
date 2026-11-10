//! Postgres access. Runtime-checked sqlx queries (no live DB needed to compile).

use api_types::{CreateProject, Item, Project, ProjectPatch, ProjectSource, ProjectStatus};
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use uuid::Uuid;

pub type Pool = PgPool;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// Unique-name violation on `projects` (case-insensitive).
    #[error("a project with this name already exists")]
    Conflict,
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
}

#[derive(FromRow)]
struct ItemRow {
    id: Uuid,
    title: String,
    done: bool,
    created_at: DateTime<Utc>,
}

impl From<ItemRow> for Item {
    fn from(r: ItemRow) -> Self {
        Item {
            id: r.id,
            title: r.title,
            done: r.done,
            created_at: r.created_at,
        }
    }
}

const COLS: &str = "id, title, done, created_at";

pub async fn connect(url: &str) -> Result<Pool, sqlx::Error> {
    PgPoolOptions::new().max_connections(10).connect(url).await
}

pub async fn migrate(pool: &Pool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("../../migrations").run(pool).await
}

pub async fn ping(pool: &Pool) -> bool {
    sqlx::query("SELECT 1").execute(pool).await.is_ok()
}

pub async fn list_items(pool: &Pool) -> Result<Vec<Item>, sqlx::Error> {
    let rows: Vec<ItemRow> = sqlx::query_as(&format!(
        "SELECT {COLS} FROM items ORDER BY created_at DESC"
    ))
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn insert_item(pool: &Pool, title: &str) -> Result<Item, sqlx::Error> {
    let row: ItemRow = sqlx::query_as(&format!(
        "INSERT INTO items (id, title) VALUES ($1, $2) RETURNING {COLS}"
    ))
    .bind(Uuid::now_v7())
    .bind(title)
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

pub async fn complete_item(pool: &Pool, id: Uuid) -> Result<Option<Item>, sqlx::Error> {
    let row: Option<ItemRow> = sqlx::query_as(&format!(
        "UPDATE items SET done = TRUE WHERE id = $1 RETURNING {COLS}"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(Into::into))
}

// ---- projects ---------------------------------------------------------------

#[derive(FromRow)]
struct ProjectRow {
    id: Uuid,
    name: String,
    repo_url: Option<String>,
    description: Option<String>,
    source: String,
    status: String,
    status_message: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn parse_source(raw: &str) -> ProjectSource {
    match raw {
        "scratch" => ProjectSource::Scratch,
        "git" => ProjectSource::Git,
        other => panic!("unknown project source in database: {other}"),
    }
}

fn parse_status(raw: &str) -> ProjectStatus {
    match raw {
        "cloning" => ProjectStatus::Cloning,
        "ready" => ProjectStatus::Ready,
        "failed" => ProjectStatus::Failed,
        other => panic!("unknown project status in database: {other}"),
    }
}

impl From<ProjectRow> for Project {
    fn from(r: ProjectRow) -> Self {
        Project {
            id: r.id,
            name: r.name,
            repo_url: r.repo_url,
            description: r.description,
            source: parse_source(&r.source),
            status: parse_status(&r.status),
            status_message: r.status_message,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

// Enum columns are read as text so the row maps to plain Rust enums.
const PROJECT_COLS: &str = "id, name, repo_url, description, source::text AS source, \
     status::text AS status, status_message, created_at, updated_at";

/// Everything a new project row needs. Used by the workspace flows, which set `source`
/// and `status` themselves.
pub struct NewProject {
    pub name: String,
    pub repo_url: Option<String>,
    pub description: Option<String>,
    pub source: ProjectSource,
    pub status: ProjectStatus,
    pub status_message: Option<String>,
}

/// Insert a project row with explicit `source`, `status` and `status_message`.
pub async fn insert_project_row(_pool: &Pool, _input: &NewProject) -> Result<Project, DbError> {
    todo!()
}

/// Set `status` and `status_message` and bump `updated_at`. `None` if the id is unknown.
pub async fn set_status(
    _pool: &Pool,
    _id: Uuid,
    _status: ProjectStatus,
    _message: Option<&str>,
) -> Result<Option<Project>, DbError> {
    todo!()
}

/// Mark every `cloning` project `failed` with message "interrupted". Other rows are untouched.
pub async fn mark_interrupted_clones(_pool: &Pool) -> Result<u64, DbError> {
    todo!()
}

/// Map a unique-violation (SQLSTATE 23505) to `DbError::Conflict`.
fn map_project_err(e: sqlx::Error) -> DbError {
    match e.as_database_error() {
        Some(db_err) if db_err.code().as_deref() == Some("23505") => DbError::Conflict,
        _ => DbError::Sql(e),
    }
}

/// Insert a validated project. Unique-name violation -> `DbError::Conflict`.
pub async fn insert_project(pool: &Pool, input: &CreateProject) -> Result<Project, DbError> {
    let row: ProjectRow = sqlx::query_as(&format!(
        "INSERT INTO projects (id, name, repo_url, description) \
         VALUES ($1, $2, $3, $4) RETURNING {PROJECT_COLS}"
    ))
    .bind(Uuid::now_v7())
    .bind(&input.name)
    .bind(&input.repo_url)
    .bind(&input.description)
    .fetch_one(pool)
    .await
    .map_err(map_project_err)?;
    Ok(row.into())
}

/// All projects, newest first. `id` (UUID v7) breaks `created_at` ties.
pub async fn list_projects(pool: &Pool) -> Result<Vec<Project>, DbError> {
    let rows: Vec<ProjectRow> = sqlx::query_as(&format!(
        "SELECT {PROJECT_COLS} FROM projects ORDER BY created_at DESC, id DESC"
    ))
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get_project(pool: &Pool, id: Uuid) -> Result<Option<Project>, DbError> {
    let row: Option<ProjectRow> = sqlx::query_as(&format!(
        "SELECT {PROJECT_COLS} FROM projects WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(Into::into))
}

/// Applies only the `Some` fields and bumps `updated_at`. `None` if the id is unknown.
pub async fn update_project(
    pool: &Pool,
    id: Uuid,
    patch: &ProjectPatch,
) -> Result<Option<Project>, DbError> {
    // For clearable fields, a boolean says "touch it" and the value (maybe NULL) is the new one.
    let row: Option<ProjectRow> = sqlx::query_as(&format!(
        "UPDATE projects SET \
             name = COALESCE($2::text, name), \
             repo_url = CASE WHEN $3 THEN $4::text ELSE repo_url END, \
             description = CASE WHEN $5 THEN $6::text ELSE description END, \
             updated_at = clock_timestamp() \
         WHERE id = $1 RETURNING {PROJECT_COLS}"
    ))
    .bind(id)
    .bind(&patch.name)
    .bind(patch.repo_url.is_some())
    .bind(patch.repo_url.clone().flatten())
    .bind(patch.description.is_some())
    .bind(patch.description.clone().flatten())
    .fetch_optional(pool)
    .await
    .map_err(map_project_err)?;
    Ok(row.map(Into::into))
}

/// `true` if a row was deleted, `false` if the id was unknown.
pub async fn delete_project(pool: &Pool, id: Uuid) -> Result<bool, DbError> {
    let result = sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn new(name: &str) -> CreateProject {
        CreateProject {
            name: name.into(),
            repo_url: None,
            description: None,
        }
    }

    fn full(name: &str) -> CreateProject {
        CreateProject {
            name: name.into(),
            repo_url: Some("https://github.com/a/b".into()),
            description: Some("desc".into()),
        }
    }

    fn patch_name(name: &str) -> ProjectPatch {
        ProjectPatch {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    // ---- B. storage tests (real Postgres, fresh DB per test) -----------------

    #[sqlx::test(migrations = "../../migrations")]
    async fn insert_then_get_returns_same_project(pool: PgPool) {
        let created = insert_project(&pool, &full("Alpha")).await.unwrap();
        let fetched = get_project(&pool, created.id).await.unwrap();
        assert_eq!(fetched, Some(created));
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn list_projects_returns_newest_first(pool: PgPool) {
        insert_project(&pool, &new("first")).await.unwrap();
        insert_project(&pool, &new("second")).await.unwrap();
        let names: Vec<String> = list_projects(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|p| p.name)
            .collect();
        assert_eq!(names, vec!["second", "first"]);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn insert_duplicate_name_is_conflict(pool: PgPool) {
        insert_project(&pool, &new("Alpha")).await.unwrap();
        let err = insert_project(&pool, &new("Alpha")).await.unwrap_err();
        assert!(matches!(err, DbError::Conflict));
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn insert_duplicate_name_different_case_is_conflict(pool: PgPool) {
        insert_project(&pool, &new("Foo")).await.unwrap();
        let err = insert_project(&pool, &new("foo")).await.unwrap_err();
        assert!(matches!(err, DbError::Conflict));
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn update_changes_only_given_fields(pool: PgPool) {
        let created = insert_project(&pool, &full("Alpha")).await.unwrap();
        let updated = update_project(&pool, created.id, &patch_name("Renamed"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.name, "Renamed");
        assert_eq!(updated.repo_url, created.repo_url);
        assert_eq!(updated.description, created.description);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn update_can_clear_optional_fields(pool: PgPool) {
        let created = insert_project(&pool, &full("Alpha")).await.unwrap();
        let patch = ProjectPatch {
            repo_url: Some(None),
            description: Some(None),
            ..Default::default()
        };
        let updated = update_project(&pool, created.id, &patch)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.repo_url, None);
        assert_eq!(updated.description, None);
        assert_eq!(updated.name, "Alpha");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn update_bumps_updated_at_and_keeps_created_at(pool: PgPool) {
        let created = insert_project(&pool, &full("Alpha")).await.unwrap();
        let updated = update_project(&pool, created.id, &patch_name("Renamed"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.created_at, created.created_at);
        assert!(updated.updated_at > created.updated_at);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn get_unknown_id_returns_none(pool: PgPool) {
        assert_eq!(get_project(&pool, Uuid::now_v7()).await.unwrap(), None);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn update_unknown_id_returns_none(pool: PgPool) {
        let result = update_project(&pool, Uuid::now_v7(), &patch_name("x"))
            .await
            .unwrap();
        assert_eq!(result, None);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn delete_unknown_id_returns_false(pool: PgPool) {
        assert!(!delete_project(&pool, Uuid::now_v7()).await.unwrap());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn delete_removes_project(pool: PgPool) {
        let created = insert_project(&pool, &new("Alpha")).await.unwrap();
        assert!(delete_project(&pool, created.id).await.unwrap());
        assert_eq!(get_project(&pool, created.id).await.unwrap(), None);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn deleting_twice_returns_false_the_second_time(pool: PgPool) {
        let created = insert_project(&pool, &new("Alpha")).await.unwrap();
        assert!(delete_project(&pool, created.id).await.unwrap());
        assert!(!delete_project(&pool, created.id).await.unwrap());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn db_rejects_name_over_100_chars_without_validation(pool: PgPool) {
        let err = sqlx::query("INSERT INTO projects (id, name) VALUES ($1, $2)")
            .bind(Uuid::now_v7())
            .bind("a".repeat(101))
            .execute(&pool)
            .await
            .unwrap_err();
        let db_err = err.as_database_error().expect("expected a database error");
        // 23514 = check_violation
        assert_eq!(db_err.code().as_deref(), Some("23514"));
    }

    // ---- extra tests (added in Code issue #3) --------------------------------

    #[sqlx::test(migrations = "../../migrations")]
    async fn update_rename_to_existing_name_is_conflict_in_any_case(pool: PgPool) {
        insert_project(&pool, &new("Taken")).await.unwrap();
        let other = insert_project(&pool, &new("Other")).await.unwrap();
        for name in ["Taken", "taken", "TAKEN"] {
            let err = update_project(&pool, other.id, &patch_name(name))
                .await
                .unwrap_err();
            assert!(matches!(err, DbError::Conflict), "renaming to {name}");
        }
        // The failed renames left the row untouched.
        assert_eq!(get_project(&pool, other.id).await.unwrap(), Some(other));
    }

    // ---- C. workspace storage tests (real Postgres, fresh DB per test) --------

    /// Arrange a row with raw SQL so these tests only exercise the function under test.
    async fn raw_insert(
        pool: &PgPool,
        name: &str,
        source: &str,
        status: &str,
        message: Option<&str>,
    ) -> Uuid {
        let id = Uuid::now_v7();
        sqlx::query(&format!(
            "INSERT INTO projects (id, name, source, status, status_message) \
             VALUES ($1, $2, '{source}', '{status}', $3)"
        ))
        .bind(id)
        .bind(name)
        .bind(message)
        .execute(pool)
        .await
        .unwrap();
        id
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn new_columns_are_saved_and_read_back(pool: PgPool) {
        let created = insert_project_row(
            &pool,
            &NewProject {
                name: "Imported".into(),
                repo_url: Some("https://github.com/a/b".into()),
                description: None,
                source: ProjectSource::Git,
                status: ProjectStatus::Cloning,
                status_message: None,
            },
        )
        .await
        .unwrap();
        let fetched = get_project(&pool, created.id).await.unwrap().unwrap();
        assert_eq!(fetched.source, ProjectSource::Git);
        assert_eq!(fetched.status, ProjectStatus::Cloning);
        assert_eq!(fetched.status_message, None);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn set_status_updates_status_and_message_and_bumps_updated_at(pool: PgPool) {
        let id = raw_insert(&pool, "Alpha", "git", "cloning", None).await;
        let before = get_project(&pool, id).await.unwrap().unwrap();
        let updated = set_status(&pool, id, ProjectStatus::Failed, Some("clone timed out"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, ProjectStatus::Failed);
        assert_eq!(updated.status_message.as_deref(), Some("clone timed out"));
        assert!(updated.updated_at > before.updated_at);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn mark_interrupted_clones_fails_only_cloning_rows(pool: PgPool) {
        let cloning = raw_insert(&pool, "Cloning", "git", "cloning", None).await;
        let ready = raw_insert(&pool, "Ready", "scratch", "ready", None).await;
        let failed = raw_insert(&pool, "Failed", "git", "failed", Some("clone timed out")).await;
        mark_interrupted_clones(&pool).await.unwrap();

        let cloning = get_project(&pool, cloning).await.unwrap().unwrap();
        assert_eq!(cloning.status, ProjectStatus::Failed);
        assert_eq!(cloning.status_message.as_deref(), Some("interrupted"));

        let ready = get_project(&pool, ready).await.unwrap().unwrap();
        assert_eq!(ready.status, ProjectStatus::Ready);
        assert_eq!(ready.status_message, None);

        let failed = get_project(&pool, failed).await.unwrap().unwrap();
        assert_eq!(failed.status, ProjectStatus::Failed);
        assert_eq!(failed.status_message.as_deref(), Some("clone timed out"));
    }
}

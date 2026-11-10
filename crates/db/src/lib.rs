//! Postgres access. Runtime-checked sqlx queries (no live DB needed to compile).

use api_types::Item;
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use uuid::Uuid;

pub type Pool = PgPool;

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

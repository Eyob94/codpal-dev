# Rust notes (axum + sqlx + Postgres)

Use these patterns in this repo.

## Unit tests (`crates/core`)

- Put them in `#[cfg(test)] mod tests` at the bottom of the file.
- No async, no DB. Core is pure.
- Stub not-yet-written functions with `todo!()`. A test that hits `todo!()` panics, which counts as a failure. That is what we want.

```rust
#[test]
fn create_rejects_name_over_100_chars() {
    let input = CreateProject { name: "a".repeat(101), ..Default::default() };
    let errs = validate_create_project(&input).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].field, "name");
}
```

Use `"é".repeat(100)` or `"🙂".repeat(100)` to test character count vs bytes.

## Storage tests (`crates/db`)

- Use `#[sqlx::test]`. It makes a **fresh database per test** and runs migrations.
- Needs `DATABASE_URL` set (locally via `.env`, in CI via a Postgres service).

```rust
#[sqlx::test(migrations = "../../migrations")]
async fn duplicate_name_different_case_conflicts(pool: PgPool) {
    insert_project(&pool, &new("Foo")).await.unwrap();
    let err = insert_project(&pool, &new("foo")).await.unwrap_err();
    assert!(matches!(err, DbError::Conflict { .. }));
}
```

- To test a DB constraint directly, use raw `sqlx::query("INSERT ...")` and check that it fails.
- For `updated_at`: read the value, update, read again, assert `after > before`. No `sleep`.

## API tests (`crates/server`)

- Build the real router with a test pool. Send requests with `tower::ServiceExt::oneshot`.
- Put shared helpers in `crates/server/tests/common/mod.rs`.
- Always check **status and body**.

```rust
#[sqlx::test(migrations = "../../migrations")]
async fn post_duplicate_name_returns_409(pool: PgPool) {
    let app = common::app(pool);
    common::post_json(&app, "/api/projects", json!({"name": "a"})).await;
    let (status, body) = common::post_json(&app, "/api/projects", json!({"name": "a"})).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "conflict");
}
```

- For bad JSON, check the body is still the project's `ApiError` shape (has `code`, `message`), not plain text.

## Before you finish

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- New tests: compile, then fail.
- Old tests: pass.

# codpal

Rust owns all logic; React only renders view models and forwards intents.

```
crates/api-types  wire types (serde + ts-rs -> web/src/generated)
crates/core       pure domain logic (no IO; wasm-compatible)
crates/db         sqlx + Postgres
crates/server     axum API, serves web/dist in prod
crates/wasm       opt-in wasm-bindgen bridge over core
web/              Vite + React (views only)
migrations/       sqlx migrations
```

## Dev
```bash
cp .env.example .env && just db      # Postgres via docker compose
cargo run -p server                  # API on :3000 (runs migrations)
cd web && pnpm install && pnpm dev   # Vite on :5173, proxies /api
```
`just gen` regenerates TS types; `just wasm` builds the wasm bridge; `just lint` / `just test` / `just build`.

## Rules
- No domain logic or hand-written API types in `web/`.
- Add a type -> `#[derive(TS)] #[ts(export)]` in `api-types`, run `just gen`, commit the output.
- Logic goes in `core`; handlers stay thin.

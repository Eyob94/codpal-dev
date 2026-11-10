set dotenv-load := true

default:
    @just --list

# Start Postgres
db:
    docker compose up -d postgres

# Regenerate TS types from Rust (ts-rs exports run as tests)
gen:
    cargo test -p api-types --quiet

# Build the wasm bridge into web/src/generated/wasm (opt-in)
wasm:
    wasm-pack build crates/wasm --target web --out-dir ../../web/src/generated/wasm

# API with auto-reload (needs cargo-watch) + Vite dev server
dev:
    cargo watch -x 'run -p server' & (cd web && pnpm dev)

test:
    cargo test --workspace

lint:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cd web && pnpm typecheck

build:
    cd web && pnpm build
    cargo build --release -p server

# Fail if committed generated types drift from Rust
check-gen: gen
    git diff --exit-code web/src/generated

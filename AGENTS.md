# AGENTS.md

This repository is a Rust TUI application managed as a Moonrepo workspace.
Use this guide to run builds/tests and follow local coding conventions.

## Project layout

- Workspace root: `/Users/alan/Projects/tech-radar-tui`
- App: `apps/tui`
- Moon workspace config: `.moon/workspace.yml`
- Moon tasks: `.moon/tasks.yml` and `apps/tui/moon.yml`
- Rust crate manifest: `apps/tui/Cargo.toml`

## Build, lint, and test commands

Run commands from the workspace root unless noted.
The app is a standard Cargo crate; Moon tasks wrap Cargo commands.

### Cargo (direct)

- Build (debug): `cargo build`
- Build (release): `cargo build --release`
- Run (release): `cargo run --release`
- Check: `cargo check`
- Format check: `cargo fmt --all --check`
- Lint: `cargo clippy`
- Test all: `cargo test`

### Single test

- By test name: `cargo test test_get_blip_by_id`
- By module path: `cargo test db::queries::tests::test_update_blip`
- By file (module): `cargo test db::queries`

### Moon tasks (workspace)

Use Moon when you want consistent task inputs/caching.
These tasks are defined in `apps/tui/moon.yml`.

- Build: `moon run tui:build`
- Check: `moon run tui:check`
- Format check: `moon run tui:format`
- Lint: `moon run tui:lint`
- Test: `moon run tui:test`
- Dev run: `moon run tui:dev`
- Release run: `moon run tui:release`

## Environment and configuration

- The app loads `.env` via `dotenv` at startup.
- Database name defaults to `adrs.db` (env var: `DATABASE_NAME`).
- ADR and Blip output directories can be overridden:
  - `ADR_DIR` (defaults to `./adrs`)
  - `BLIP_DIR` (defaults to `./blips`)
- Database URL is constructed as `sqlite:///absolute/path` or `sqlite://relative/path`.

## Code style and conventions

### Rust edition and formatting

- Rust edition: 2021 (see `apps/tui/Cargo.toml`).
- Use `cargo fmt` formatting; do not hand-align or deviate.
- Keep line lengths reasonable; prefer small helpers over long lines.

### Imports and module structure

- Organize imports by standard practice:
  - `use crate::...` first when referencing local modules.
  - External crates next (`use color_eyre::Result;`, `use sqlx::...`).
  - Standard library imports last.
- Prefer module `mod` declarations at the top of files.
- Re-export public API from `src/lib.rs` or module `mod.rs` as needed.

### Naming conventions

- Types use `UpperCamelCase`.
- Functions, variables, modules use `snake_case`.
- Enum variants use `UpperCamelCase`.
- Keep names descriptive and domain-driven (e.g., `BlipRecord`, `AdrMetadataParams`).

### Types and data modeling

- Use explicit types for shared data passed between layers (e.g., `BlipUpdateParams`).
- Prefer `Option<T>` for nullable database fields and handle defaults explicitly.
- Keep state in structs; avoid unstructured global state.

### Error handling

- Use `color_eyre::Result` for top-level application functions.
- Propagate errors with `?` when possible.
- When the app can recover, log with `eprintln!` and set a status message.
- Provide context when returning errors (e.g., `eyre!("...")`).

### Async and IO

- Async functions return `Result` and use `await` consistently.
- Database access is async via `sqlx::SqlitePool`.
- Avoid blocking operations inside async loops; if needed, isolate them.

### UI and TUI patterns

- UI rendering lives in `apps/tui/src/ui/render.rs`.
- Keep UI rendering pure: render based on `App` state only.
- Update state in input handlers or event loop, not in render helpers.
- Use `ratatui` layout primitives for positioning, avoid magic numbers where possible.

### Database conventions

- SQLx queries are in `apps/tui/src/db/queries.rs`.
- Migrations/DB setup is in `apps/tui/src/db/migrations.rs`.
- Use parameterized queries; avoid string interpolation for SQL.
- Keep DB schema changes in `apps/tui/migrations/*.sql`.

### Testing

- Tests live under the module where they are relevant.
- Use `#[tokio::test]` for async tests.
- Prefer in-memory SQLite for unit tests (`sqlite::memory:`).

### Comments and documentation

- Keep comments minimal and explanatory.
- Prefer self-documenting code; add comments only for non-obvious logic.
- If adding docs, keep them concise and aligned with existing style.

## Repository-specific notes

- There are no Cursor rules (`.cursor/rules/` or `.cursorrules`) found.
- There are no Copilot instructions found in `.github/copilot-instructions.md`.
- The repo contains a `.git` directory even if the environment reports otherwise.

## Quick start for agents

1. Read `apps/tui/README.md` for feature overview and TUI usage.
2. Run `moon run tui:dev` or `moon run tui:release` to launch the app.
3. Use `cargo test` or `moon run tui:test` for tests.
4. Use `cargo fmt --all --check` and `cargo clippy` before changes.

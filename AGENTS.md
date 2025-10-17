# Repository Guidelines

## Project Structure & Module Organization

The entrypoint lives in `src/main.rs`, which initializes the Axum server defined in `src/server.rs` and pulls helpers from `index.rs`, `config.rs`, and `sites.rs`. Templates rendered for the browser reside in `templates/`, while `static/` holds CSS, JS, and image assets served via `/static`. Runtime defaults live in `src/config.rs`; keep any sample data in `config.json5` synchronized with those structs. Cargo outputs go under `target/` and should stay untracked.

## Build, Test, and Development Commands

Use `just build` to compile the project, and `just run` to serve the dashboard locally on `http://0.0.0.0:3000`. Watch the terminal for startup logs. Format code with `just fmt`, and run `just clippy` to surface lint hints tailored to this project. Clean stale artifacts with `just clean` if incremental builds start failing. Run `just check` to run all checks (fmt, clippy, and test).

## Testing Guidelines

Unit tests belong next to the code they cover inside `#[cfg(test)]` modules, while broader scenarios should live in a `tests/` integration suite when it is introduced. Name tests after the behavior they validate (`renders_dashboard_with_weather`). Run `just test` before submitting changes and document any gaps if coverage is impractical. Store reusable fixture data in `tests/fixtures/` to keep sample inputs organized.

## Commit & Pull Request Guidelines

Recent history favors concise, lowercase summaries (e.g. `added search engine and weather`). Keep commits focused and add explanatory body text when behavior changes or migrations are involved. Reference issue IDs with `Refs #123` where applicable. Pull requests need a short summary, testing notes, and screenshots or GIFs for UI-facing updates. Call out manual deployment steps so reviewers can mirror your verification flow.

## Security & Configuration Tips

Avoid committing real API keys or sensitive endpoints. Configuration defaults currently live in code, so update `src/config.rs` alongside any sample `config.json5` you share. Before publishing external assets, make sure icons and scripts in `static/` are licensed for redistribution. When adding new template variables, sanitize user-provided content in Rust before handing it to Askama to prevent injection issues.

# Repository Guidelines

This repository contains ITank, a terminal Tank90-style game written in Rust.

## Project Structure

The project uses this layout:

- `src/main.rs` - terminal event loop and input mapping
- `src/game.rs` - pure gameplay logic and unit tests
- `src/renderer.rs` - terminal drawing
- `Cargo.toml` - package metadata and dependency configuration

Keep the top level limited to source configuration, documentation, and this contributor guide.

## Build, Test, and Development Commands

Use the standard Rust commands:

- `npm run dev` - run the application locally
- `cargo run --release` - run ITank locally
- `cargo test` - run the unit tests
- `cargo clippy --all-targets -- -D warnings` - lint the code
- `cargo fmt --check` - verify formatting
- `cargo build --release` - build an optimized binary

Run `cargo fmt` before committing formatting changes.

## Coding Style & Naming Conventions

- Use 4 spaces for Rust source files.
- Prefer descriptive names over abbreviations.
- Use `snake_case` for variables, functions, and modules; `PascalCase` for types and enums.
- Keep filenames lowercase and hyphenated or aligned with Rust module naming, such as `renderer.rs`.
- Run `cargo fmt` and `cargo clippy` before every commit.

## Testing Guidelines

- Keep unit tests in the `#[cfg(test)]` module of the source file they cover.
- Name each test function with a descriptive `snake_case` sentence.
- Add or update tests for bug fixes and behavior changes.
- Run `cargo test` before requesting review. No coverage threshold has been established yet.

## Commit & Pull Request Guidelines

Use Conventional Commits:

- `feat: add capacity dashboard`
- `fix: correct token refresh timing`
- `docs: document deployment steps`

Pull requests should include a focused change, a clear description, test evidence, and a linked issue when applicable. Add screenshots or screen recordings for user-interface changes.

## Security & Configuration

Never commit credentials or production secrets. Store local values in ignored environment files and provide safe examples such as `.env.example`.

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

**Native development:**
```sh
cargo run --release
```

**Web development (hot reload at http://127.0.0.1:8080):**
```sh
trunk serve
```
Use the `#dev` URL fragment to bypass PWA caching during development.

**Web production build:**
```sh
trunk build --release   # outputs to dist/
```

**Tests and validation:**
```sh
cargo test --lib                              # unit tests
cargo fmt --all -- --check                   # format check
cargo clippy -- -D warnings                  # lint
cargo check --target wasm32-unknown-unknown  # WASM compatibility
./check.sh                                   # all CI checks at once
```

**Prerequisites (if not already installed):**
```sh
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

## Architecture

Valens is an egui/eframe GUI app for managing **Products**, **Features**, **Customer Segments**, and **Jobs**. It targets both native desktop and web (WASM via Trunk), deploying to GitHub Pages.

### Application Modes
- `Mode::Demo` — web deployments; loads pre-populated demo data from `demo_data.rs`, no persistence
- `Mode::Production` — native; full serde-based persistence to disk

### UI Structure
The app is a two-tab layout managed in `app.rs`:
- **Top panel** — menu bar with tab navigation and theme toggle
- **Side panel** — toggle visibility of floating tool windows
- **Central panel** — main content (tables, accordions)
- **Floating windows** — modal dialogs for editing entities

### Module Patterns
Both `pages/product/` and `pages/segments/` follow an identical structure:
- `model.rs` — data structs (serde-serialized) and transient UI state (`#[serde(skip)]`)
- `mod.rs` — window rendering and top-level management
- `accordion.rs` — expandable table rows
- `detail_panel.rs` — edit/view panel for a selected item
- `delete_dialog.rs` — confirmation dialog

Nested features follow the same pattern: `segments/jobs_window/` mirrors the parent structure one level deeper.

When adding a new entity type, follow this existing pattern exactly.

### State & Persistence
The root `App` struct in `app.rs` holds all state. Fields annotated `#[serde(skip)]` are transient UI state that resets on restart. Serialized fields persist across sessions in Production mode.

### Code Quality
The project enforces strict Clippy lints (see `Cargo.toml` `[lints.clippy]`) and denies `unsafe_code`. CI runs on macOS (arm64 + x86_64), Linux (musl + ARM), and Windows. Spell checking via `.typos.toml`.

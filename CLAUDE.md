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
- `Mode::Production` — native; entity data persisted to SQLite, UI state to `eframe::Storage`

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

Two persistence layers:
- **SQLite** (`src/app/db/`, native only) — all entity data (products, features, segments, jobs, pains, gains, links, annotations). DB file: `~/Library/Application Support/valens/valens.db` (macOS). Native only, compiled behind `#[cfg(not(target_arch = "wasm32"))]`.
- **`eframe::Storage`** (JSON) — lightweight UI state only: active tab, which windows are toggled open (`ProductWindows`, `CustomerWindows`). Works on both native and WASM.

Entity fields in `ValuePropPage` and `CustomerSegmentPage` are marked `#[serde(skip)]` so they are not written to the JSON blob. The JSON blob is only for window/tab state.

Demo mode (WASM) bypasses both layers entirely — `load_demo_data()` populates the structs directly.

### Database Migrations

Schema changes use append-only versioned SQL files. **Never modify or reorder existing migrations.**

**Files:**
- `src/migrations/V{N}__{description}.sql` — the SQL (Flyway-style naming, 1-indexed)
- `src/app/db/migrations.rs` — Rust list of migrations (0-indexed Vec, so V1 = index 0)
- `src/app/db/mod.rs` — load/save SQL that must stay in sync with the schema

**Per-change checklist:**
1. Create `src/migrations/V{N}__<description>.sql` with `ALTER TABLE` / `CREATE TABLE` SQL
2. Append `M::up(include_str!("../../migrations/V{N}__<description>.sql"))` in `migrations.rs`
3. Update the Rust model struct (add/remove field) — if adding, mirror the SQL `DEFAULT`
4. Update the `load_into()` and `save()` SQL in `src/app/db/mod.rs`
5. Run `cargo test --lib` (migration round-trip test must pass)
6. Run `cargo check --target wasm32-unknown-unknown` (WASM must still compile)

Use the `/db-migration` skill to scaffold all of this automatically.

### Code Quality
The project enforces strict Clippy lints (see `Cargo.toml` `[lints.clippy]`) and denies `unsafe_code`. CI runs on macOS (arm64 + x86_64), Linux (musl + ARM), and Windows. Spell checking via `.typos.toml`.

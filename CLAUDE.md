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

### Release Workflow

Releases are triggered by typing `/release patch|minor|major` in a Claude Code session.
Claude Code handles the entire process — no manual steps needed.

**To release:**
1. Open Claude Code in this repo (`claude` in the terminal from the repo root)
2. Type `/release patch` (or `minor` / `major` depending on what changed)
3. Claude Code bumps the version, runs all checks, commits, tags, builds the `.app`, and
   installs it to `/Applications/Valens.app`
4. Quit the running app (if open) and relaunch from `/Applications/`

**What "patch / minor / major" means:**
- `patch` — bug fixes, polish, small tweaks (0.1.0 → 0.1.1)
- `minor` — new features or meaningful new capabilities (0.1.0 → 0.2.0)
- `major` — large milestones or breaking data changes (0.1.0 → 1.0.0)

**Rollback**: The previous `.app` is always backed up to `/Applications/Valens.app.bak`.
To roll back manually: `rm -rf /Applications/Valens.app && mv /Applications/Valens.app.bak /Applications/Valens.app`

**First launch after install**: If macOS Gatekeeper complains, right-click `Valens.app` in
`/Applications/` and choose Open. This is a one-time prompt; subsequent launches work normally.

**Build script only** (no version bump, no git): `bash scripts/bundle.sh` builds and installs
the current HEAD without touching git. Useful for testing a build mid-development.

### Code Quality
The project enforces strict Clippy lints (see `Cargo.toml` `[lints.clippy]`) and denies `unsafe_code`. CI runs on macOS (arm64 + x86_64), Linux (musl + ARM), and Windows. Spell checking via `.typos.toml`.

## Worktree & Dev Server Management

This project uses egui/eframe compiled to WASM via Trunk for web preview.

### Rules Claude must follow in every session

1. **Never run `trunk serve` without a port** — port 8080 is reserved for the
   main branch. Worktrees must use 8081+.

2. **On session start in a worktree**, check if a `Trunk.toml` exists here.
   If not, create one:
   - Find a free port: scan 8081–8099 and pick the first not in use
     (`lsof -i TCP:<port>` or `ss -ltn | grep <port>`)
   - Write `Trunk.toml`:
```toml
     [serve]
     port = <chosen_port>
     open = true
```
   - Copy `.env.local` from the repo root if it exists and isn't already here
   - Tell the user: "Dev server will run on port <port>"

3. **Never commit `Trunk.toml`** — it is gitignored and local to each worktree.

4. **Before creating a worktree**, confirm the name with the user, then:
   - Run `git worktree add .claude/worktrees/<name> -b worktree-<name>`
   - Follow rule 2 inside the new worktree
   - Remind the user to open a new terminal, cd into the worktree, and run `claude`

5. **When merging/closing a worktree**:
   - Stop the trunk serve process if running
   - Commit or stash any changes
   - Run `git worktree remove .claude/worktrees/<name>`
   - Run `git branch -d worktree-<name>`
   - Run `git worktree prune`

6. **If trunk serve fails to start**, check for port conflicts first:
   `lsof -i TCP:808x` — then update `Trunk.toml` to a free port and retry.

### Project stack
- Language: Rust
- UI: egui / eframe
- Web target: wasm32-unknown-unknown
- Bundler: Trunk
- Default main branch port: 8080

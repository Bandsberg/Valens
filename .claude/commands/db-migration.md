# db-migration

Scaffold a new database migration for a Valens schema change.

## Usage

`/db-migration <description of what's changing>`

**Examples:**
- `/db-migration add priority field to features`
- `/db-migration add segment_pain_links table`
- `/db-migration rename characteristics to demographics in customer_segments`

## What this skill does

Given a description of the schema change you want to make, this skill will:

1. Read `src/migrations/` to determine the next migration version number
2. Read the current Rust model structs to understand the existing schema
3. Generate the SQL migration file at `src/migrations/V{N}__{description}.sql`
4. Update `src/app/db/migrations.rs` to include the new migration
5. Update the relevant Rust model struct (add/remove/rename field)
6. Update the `load_into()` and `save()` SQL in `src/app/db/mod.rs`
7. Run `cargo test --lib` to confirm the migration round-trips correctly
8. Run `cargo check --target wasm32-unknown-unknown` to confirm WASM still compiles

## Rules you must follow

- **Append-only**: NEVER modify or reorder existing entries in `Migrations::new(vec![...])`.
  The migration index must match the version number (V1 = index 0, V2 = index 1, etc.).
- **Forward only**: No down migrations. If a change needs to be reverted, write a new forward
  migration (V{N+1}) that undoes V{N}.
- **Mirror defaults**: If adding a column, the SQL `DEFAULT` must match the Rust struct
  field default (use `#[serde(default = "...")]` or `Default` impl if not 0/empty/"").
- **Both files**: Never update the Rust struct without also updating `db/mod.rs` load/save SQL,
  and vice versa. Partial updates will cause a runtime panic or data loss.

## Step-by-step instructions

$ARGUMENTS

1. **Determine version number**: Count the files in `src/migrations/` — the new file is V{count+1}.

2. **Write the SQL migration** to `src/migrations/V{N}__{snake_case_description}.sql`.
   - Use `ALTER TABLE ... ADD COLUMN` to add columns (preserves existing data).
   - Use `CREATE TABLE IF NOT EXISTS` for new tables.
   - Always include a `DEFAULT` on new columns.
   - For renames: SQLite does not support `RENAME COLUMN` before 3.25 — use
     `ALTER TABLE t RENAME COLUMN old TO new` (supported in bundled SQLite).

3. **Append to migrations list** in `src/app/db/migrations.rs`:
   ```rust
   M::up(include_str!("../../migrations/V{N}__{description}.sql")),
   ```
   Note: path is relative to the `migrations.rs` file, which is in `src/app/db/`.

4. **Update the Rust struct** in the relevant model file under `src/app/pages/`.
   - New field needs: the right type, `pub` visibility, serde derives already on the struct.
   - If the field has a non-zero/non-empty default, add `#[serde(default = "fn_name")]`.

5. **Update `src/app/db/mod.rs`**:
   - In the `load_*` function: add the new column to the SELECT and the struct constructor.
   - In the `save_*` function: add the new field to the INSERT.
   - For new link tables: add new `load_links` / `save_links` calls in `load_into` and `save`.

6. **Run verification**:
   ```sh
   cargo test --lib
   cargo check --target wasm32-unknown-unknown
   cargo clippy -- -D warnings
   ```

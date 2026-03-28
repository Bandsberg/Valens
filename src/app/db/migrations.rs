use rusqlite_migration::{M, Migrations};

/// Returns all schema migrations in order.
///
/// **IMPORTANT:** Never reorder or modify existing entries — `rusqlite_migration`
/// tracks applied migrations via the `SQLite` `user_version` pragma. Reordering
/// corrupts existing databases. Always append new migrations at the end.
///
/// File naming: `V{N}__{description}.sql` (Flyway-style, 1-indexed).
/// Vec index is 0-indexed, so V1 = index 0, V2 = index 1, etc.
pub(super) fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(include_str!("../../migrations/V1__initial_schema.sql")),
        M::up(include_str!(
            "../../migrations/V2__add_parent_id_to_segments.sql"
        )),
        // Add new migrations here — append only, never reorder
    ])
}

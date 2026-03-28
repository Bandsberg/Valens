//! `SQLite` persistence layer for native (non-WASM) builds.
//!
//! Entity data lives in `SQLite`; lightweight UI state (tab, window toggles) stays in
//! `eframe::Storage`.  All code in this module is compiled only for non-WASM targets.

mod migrations;

use rusqlite::{Connection, Transaction, params};
use std::path::Path;
use uuid::Uuid;

use super::pages::product::products_window::Product;
use super::pages::product::{
    Feature, GainCreator, PainRelief, ValueAnnotation, ValuePropPage, ValueType,
};
use super::pages::{CustomerSegment, CustomerSegmentPage, Gain, Job, Pain};

/// Open handle to the application `SQLite` database.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Opens (or creates) the `SQLite` database at `path` and runs all pending migrations.
    ///
    /// # Errors
    /// Returns an error if the database file cannot be opened or a migration fails to execute.
    pub fn open(path: &Path) -> Result<Self, rusqlite::Error> {
        let mut conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        migrations::migrations()
            .to_latest(&mut conn)
            .expect("DB migration failed — schema may be corrupted");
        Ok(Self { conn })
    }

    /// Loads all persisted entity data into `app`, overwriting any existing in-memory data.
    ///
    /// # Errors
    /// Returns an error if any database query fails.
    pub fn load_into(&self, app: &mut super::App) -> Result<(), rusqlite::Error> {
        let vp = &mut app.valueprop_page;
        vp.products_state.products = load_products(&self.conn)?;
        vp.features_state.features = load_features(&self.conn)?;
        vp.pain_relief_state.pain_reliefs = load_pain_reliefs(&self.conn)?;
        vp.gain_creator_state.gain_creators = load_gain_creators(&self.conn)?;
        vp.product_feature_links = load_links(
            &self.conn,
            "product_feature_links",
            "product_id",
            "feature_id",
        )?;
        vp.feature_pain_relief_links = load_links(
            &self.conn,
            "feature_pain_relief_links",
            "feature_id",
            "pain_relief_id",
        )?;
        vp.feature_gain_creator_links = load_links(
            &self.conn,
            "feature_gain_creator_links",
            "feature_id",
            "gain_creator_id",
        )?;
        vp.pain_relief_annotations = load_annotations(&self.conn, "pain_relief_annotations")?;
        vp.gain_creator_annotations = load_annotations(&self.conn, "gain_creator_annotations")?;

        let cs = &mut app.customer_segment_page;
        cs.segments_state.segments = load_segments(&self.conn)?;
        cs.jobs_state.jobs = load_jobs(&self.conn)?;
        cs.pains_state.pains = load_pains(&self.conn)?;
        cs.gains_state.gains = load_gains(&self.conn)?;
        cs.segment_job_links = load_links(&self.conn, "segment_job_links", "job_id", "segment_id")?;
        cs.job_pain_links = load_links(&self.conn, "job_pain_links", "pain_id", "job_id")?;
        cs.job_gain_links = load_links(&self.conn, "job_gain_links", "gain_id", "job_id")?;

        Ok(())
    }

    /// Replaces all persisted entity data with the current in-memory state.
    ///
    /// Wrapped in a single transaction — either all tables are updated or none are.
    ///
    /// # Errors
    /// Returns an error if any query or the commit fails.
    pub fn save(
        &mut self,
        vp: &ValuePropPage,
        cs: &CustomerSegmentPage,
    ) -> Result<(), rusqlite::Error> {
        let tx = self.conn.transaction()?;
        save_products(&tx, &vp.products_state.products)?;
        save_features(&tx, &vp.features_state.features)?;
        save_pain_reliefs(&tx, &vp.pain_relief_state.pain_reliefs)?;
        save_gain_creators(&tx, &vp.gain_creator_state.gain_creators)?;
        save_links(
            &tx,
            "product_feature_links",
            "product_id",
            "feature_id",
            &vp.product_feature_links,
        )?;
        save_links(
            &tx,
            "feature_pain_relief_links",
            "feature_id",
            "pain_relief_id",
            &vp.feature_pain_relief_links,
        )?;
        save_links(
            &tx,
            "feature_gain_creator_links",
            "feature_id",
            "gain_creator_id",
            &vp.feature_gain_creator_links,
        )?;
        save_annotations(&tx, "pain_relief_annotations", &vp.pain_relief_annotations)?;
        save_annotations(
            &tx,
            "gain_creator_annotations",
            &vp.gain_creator_annotations,
        )?;

        save_segments(&tx, &cs.segments_state.segments)?;
        save_jobs(&tx, &cs.jobs_state.jobs)?;
        save_pains(&tx, &cs.pains_state.pains)?;
        save_gains(&tx, &cs.gains_state.gains)?;
        save_links(
            &tx,
            "segment_job_links",
            "job_id",
            "segment_id",
            &cs.segment_job_links,
        )?;
        save_links(
            &tx,
            "job_pain_links",
            "pain_id",
            "job_id",
            &cs.job_pain_links,
        )?;
        save_links(
            &tx,
            "job_gain_links",
            "gain_id",
            "job_id",
            &cs.job_gain_links,
        )?;

        tx.commit()
    }
}

// ── Load helpers ──────────────────────────────────────────────────────────────

/// Parses a UUID from a database column value.
///
/// Panics rather than propagating `Result` because the schema enforces that
/// every UUID column is populated by our own `save()` function (which always
/// writes `Uuid::to_string()`). An unparseable value means the database file
/// has been manually corrupted — there is no safe recovery path.
#[expect(
    clippy::needless_pass_by_value,
    reason = "callers pass owned String from row.get()"
)]
fn parse_uuid(s: String) -> Uuid {
    s.parse().expect("database must contain valid UUIDs")
}

fn parse_uuid_opt(s: Option<String>) -> Option<Uuid> {
    let v = s?;
    v.parse().ok()
}

fn load_products(conn: &Connection) -> Result<Vec<Product>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT id, name, description, notes FROM products ORDER BY rowid")?;
    stmt.query_map([], |row| {
        Ok(Product {
            id: parse_uuid(row.get(0)?),
            name: row.get(1)?,
            description: row.get(2)?,
            notes: row.get(3)?,
            expanded: false,
        })
    })?
    .collect()
}

fn load_features(conn: &Connection) -> Result<Vec<Feature>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, status, notes, user_story, acceptance_criteria \
         FROM features ORDER BY rowid",
    )?;
    stmt.query_map([], |row| {
        Ok(Feature {
            id: parse_uuid(row.get(0)?),
            name: row.get(1)?,
            description: row.get(2)?,
            status: row.get(3)?,
            notes: row.get(4)?,
            user_story: row.get(5)?,
            acceptance_criteria: row.get(6)?,
            expanded: false,
        })
    })?
    .collect()
}

fn load_pain_reliefs(conn: &Connection) -> Result<Vec<PainRelief>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT id, name, description, notes FROM pain_reliefs ORDER BY rowid")?;
    stmt.query_map([], |row| {
        Ok(PainRelief {
            id: parse_uuid(row.get(0)?),
            name: row.get(1)?,
            description: row.get(2)?,
            notes: row.get(3)?,
            expanded: false,
        })
    })?
    .collect()
}

fn load_gain_creators(conn: &Connection) -> Result<Vec<GainCreator>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT id, name, description, notes FROM gain_creators ORDER BY rowid")?;
    stmt.query_map([], |row| {
        Ok(GainCreator {
            id: parse_uuid(row.get(0)?),
            name: row.get(1)?,
            description: row.get(2)?,
            notes: row.get(3)?,
            expanded: false,
        })
    })?
    .collect()
}

fn load_segments(conn: &Connection) -> Result<Vec<CustomerSegment>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, notes, characteristics, parent_id \
         FROM customer_segments ORDER BY rowid",
    )?;
    stmt.query_map([], |row| {
        Ok(CustomerSegment {
            id: parse_uuid(row.get(0)?),
            name: row.get(1)?,
            description: row.get(2)?,
            notes: row.get(3)?,
            characteristics: row.get(4)?,
            parent_id: parse_uuid_opt(row.get(5)?),
            expanded: false,
        })
    })?
    .collect()
}

fn load_jobs(conn: &Connection) -> Result<Vec<Job>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, name, description, notes FROM jobs ORDER BY rowid")?;
    stmt.query_map([], |row| {
        Ok(Job {
            id: parse_uuid(row.get(0)?),
            name: row.get(1)?,
            description: row.get(2)?,
            notes: row.get(3)?,
            expanded: false,
        })
    })?
    .collect()
}

fn load_pains(conn: &Connection) -> Result<Vec<Pain>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT id, name, description, notes, importance FROM pains ORDER BY rowid")?;
    stmt.query_map([], |row| {
        Ok(Pain {
            id: parse_uuid(row.get(0)?),
            name: row.get(1)?,
            description: row.get(2)?,
            notes: row.get(3)?,
            importance: row.get(4)?,
            expanded: false,
        })
    })?
    .collect()
}

fn load_gains(conn: &Connection) -> Result<Vec<Gain>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT id, name, description, notes, importance FROM gains ORDER BY rowid")?;
    stmt.query_map([], |row| {
        Ok(Gain {
            id: parse_uuid(row.get(0)?),
            name: row.get(1)?,
            description: row.get(2)?,
            notes: row.get(3)?,
            importance: row.get(4)?,
            expanded: false,
        })
    })?
    .collect()
}

/// Loads a two-column UUID link table into a `Vec<(Uuid, Uuid)>`.
///
/// `col_a` maps to the **first** element of each tuple; `col_b` to the
/// **second**. Callers choose the column order to match the field layout of
/// the target `Vec` in `App` (e.g. `(product_id, feature_id)` vs
/// `(feature_id, pain_relief_id)`).
fn load_links(
    conn: &Connection,
    table: &str,
    col_a: &str,
    col_b: &str,
) -> Result<Vec<(Uuid, Uuid)>, rusqlite::Error> {
    let sql = format!("SELECT {col_a}, {col_b} FROM {table} ORDER BY rowid");
    let mut stmt = conn.prepare(&sql)?;
    stmt.query_map([], |row| {
        Ok((parse_uuid(row.get(0)?), parse_uuid(row.get(1)?)))
    })?
    .collect()
}

/// Loads a `ValueAnnotation` table (`pain_or_gain_id`, `reliever_or_creator_id`,
/// `value_type`, `strength`) into a `Vec<ValueAnnotation>`.
///
/// `value_type` is stored as a TEXT literal: `"TableStake"` or `"Differentiator"`.
/// This must stay in sync with the strings in [`save_annotations`].
fn load_annotations(
    conn: &Connection,
    table: &str,
) -> Result<Vec<ValueAnnotation>, rusqlite::Error> {
    let sql = format!(
        "SELECT pain_or_gain_id, reliever_or_creator_id, value_type, strength \
         FROM {table} ORDER BY rowid"
    );
    let mut stmt = conn.prepare(&sql)?;
    stmt.query_map([], |row| {
        let vt: String = row.get(2)?;
        // Strings must match what save_annotations writes. Unknown strings
        // default to Differentiator so old DBs degrade gracefully.
        let value_type = match vt.as_str() {
            "TableStake" => ValueType::TableStake,
            _ => ValueType::Differentiator,
        };
        Ok(ValueAnnotation {
            pain_or_gain_id: parse_uuid(row.get(0)?),
            reliever_or_creator_id: parse_uuid(row.get(1)?),
            value_type,
            strength: row.get(3)?,
        })
    })?
    .collect()
}

// ── Save helpers ──────────────────────────────────────────────────────────────

fn save_products(tx: &Transaction<'_>, products: &[Product]) -> Result<(), rusqlite::Error> {
    tx.execute_batch("DELETE FROM products")?;
    let mut stmt =
        tx.prepare("INSERT INTO products (id, name, description, notes) VALUES (?1,?2,?3,?4)")?;
    for p in products {
        stmt.execute(params![p.id.to_string(), &p.name, &p.description, &p.notes])?;
    }
    Ok(())
}

fn save_features(tx: &Transaction<'_>, features: &[Feature]) -> Result<(), rusqlite::Error> {
    tx.execute_batch("DELETE FROM features")?;
    let mut stmt = tx.prepare(
        "INSERT INTO features \
         (id, name, description, status, notes, user_story, acceptance_criteria) \
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
    )?;
    for f in features {
        stmt.execute(params![
            f.id.to_string(),
            &f.name,
            &f.description,
            &f.status,
            &f.notes,
            &f.user_story,
            &f.acceptance_criteria
        ])?;
    }
    Ok(())
}

fn save_pain_reliefs(
    tx: &Transaction<'_>,
    pain_reliefs: &[PainRelief],
) -> Result<(), rusqlite::Error> {
    tx.execute_batch("DELETE FROM pain_reliefs")?;
    let mut stmt =
        tx.prepare("INSERT INTO pain_reliefs (id, name, description, notes) VALUES (?1,?2,?3,?4)")?;
    for pr in pain_reliefs {
        stmt.execute(params![
            pr.id.to_string(),
            &pr.name,
            &pr.description,
            &pr.notes
        ])?;
    }
    Ok(())
}

fn save_gain_creators(
    tx: &Transaction<'_>,
    gain_creators: &[GainCreator],
) -> Result<(), rusqlite::Error> {
    tx.execute_batch("DELETE FROM gain_creators")?;
    let mut stmt = tx
        .prepare("INSERT INTO gain_creators (id, name, description, notes) VALUES (?1,?2,?3,?4)")?;
    for gc in gain_creators {
        stmt.execute(params![
            gc.id.to_string(),
            &gc.name,
            &gc.description,
            &gc.notes
        ])?;
    }
    Ok(())
}

fn save_segments(
    tx: &Transaction<'_>,
    segments: &[CustomerSegment],
) -> Result<(), rusqlite::Error> {
    tx.execute_batch("DELETE FROM customer_segments")?;
    let mut stmt = tx.prepare(
        "INSERT INTO customer_segments (id, name, description, notes, characteristics, parent_id) \
         VALUES (?1,?2,?3,?4,?5,?6)",
    )?;
    for s in segments {
        stmt.execute(params![
            s.id.to_string(),
            &s.name,
            &s.description,
            &s.notes,
            &s.characteristics,
            s.parent_id.map(|u| u.to_string())
        ])?;
    }
    Ok(())
}

fn save_jobs(tx: &Transaction<'_>, jobs: &[Job]) -> Result<(), rusqlite::Error> {
    tx.execute_batch("DELETE FROM jobs")?;
    let mut stmt =
        tx.prepare("INSERT INTO jobs (id, name, description, notes) VALUES (?1,?2,?3,?4)")?;
    for j in jobs {
        stmt.execute(params![j.id.to_string(), &j.name, &j.description, &j.notes])?;
    }
    Ok(())
}

fn save_pains(tx: &Transaction<'_>, pains: &[Pain]) -> Result<(), rusqlite::Error> {
    tx.execute_batch("DELETE FROM pains")?;
    let mut stmt = tx.prepare(
        "INSERT INTO pains (id, name, description, notes, importance) VALUES (?1,?2,?3,?4,?5)",
    )?;
    for p in pains {
        stmt.execute(params![
            p.id.to_string(),
            &p.name,
            &p.description,
            &p.notes,
            p.importance
        ])?;
    }
    Ok(())
}

fn save_gains(tx: &Transaction<'_>, gains: &[Gain]) -> Result<(), rusqlite::Error> {
    tx.execute_batch("DELETE FROM gains")?;
    let mut stmt = tx.prepare(
        "INSERT INTO gains (id, name, description, notes, importance) VALUES (?1,?2,?3,?4,?5)",
    )?;
    for g in gains {
        stmt.execute(params![
            g.id.to_string(),
            &g.name,
            &g.description,
            &g.notes,
            g.importance
        ])?;
    }
    Ok(())
}

fn save_links(
    tx: &Transaction<'_>,
    table: &str,
    col_a: &str,
    col_b: &str,
    links: &[(Uuid, Uuid)],
) -> Result<(), rusqlite::Error> {
    tx.execute_batch(&format!("DELETE FROM {table}"))?;
    let sql = format!("INSERT INTO {table} ({col_a}, {col_b}) VALUES (?1, ?2)");
    let mut stmt = tx.prepare(&sql)?;
    for (a, b) in links {
        stmt.execute(params![a.to_string(), b.to_string()])?;
    }
    Ok(())
}

fn save_annotations(
    tx: &Transaction<'_>,
    table: &str,
    annotations: &[ValueAnnotation],
) -> Result<(), rusqlite::Error> {
    tx.execute_batch(&format!("DELETE FROM {table}"))?;
    let sql = format!(
        "INSERT INTO {table} \
         (pain_or_gain_id, reliever_or_creator_id, value_type, strength) \
         VALUES (?1, ?2, ?3, ?4)"
    );
    let mut stmt = tx.prepare(&sql)?;
    for a in annotations {
        // Literal strings must stay in sync with the match in load_annotations.
        let vt = match a.value_type {
            ValueType::TableStake => "TableStake",
            ValueType::Differentiator => "Differentiator",
        };
        stmt.execute(params![
            a.pain_or_gain_id.to_string(),
            a.reliever_or_creator_id.to_string(),
            vt,
            a.strength
        ])?;
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn open_in_memory() -> Database {
        let mut conn = Connection::open_in_memory().expect("in-memory DB");
        conn.pragma_update(None, "foreign_keys", "ON")
            .expect("pragma");
        migrations::migrations()
            .to_latest(&mut conn)
            .expect("migrations");
        Database { conn }
    }

    #[test]
    fn migrations_run_without_error() {
        open_in_memory();
    }

    #[test]
    fn round_trip_product() {
        let mut db = open_in_memory();
        let product = Product {
            id: Uuid::new_v4(),
            name: "Test product".to_owned(),
            description: "desc".to_owned(),
            notes: "notes".to_owned(),
            expanded: false,
        };
        let mut vp = ValuePropPage::default();
        vp.products_state.products = vec![product.clone()];
        let cs = CustomerSegmentPage::default();
        db.save(&vp, &cs).expect("save");

        let mut app = super::super::App::default();
        db.load_into(&mut app).expect("load");
        let loaded = &app.valueprop_page.products_state.products;
        assert_eq!(loaded.len(), 1);
        let first = loaded.first().expect("just asserted len == 1");
        assert_eq!(first.id, product.id);
        assert_eq!(first.name, product.name);
    }
}

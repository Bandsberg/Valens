use uuid::Uuid;

// ── State structs ─────────────────────────────────────────────────────────────

/// Persistent and transient state for the Gains table.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct GainsState {
    /// All gains.
    pub gains: Vec<Gain>,
    /// ID of the gain awaiting delete confirmation.
    #[serde(skip)]
    pub pending_delete: Option<Uuid>,
    /// ID of the gain whose detail window is open (not persisted).
    #[serde(skip)]
    pub selected_gain_id: Option<Uuid>,
    /// ID of the gain the table should scroll to on the next frame.
    #[serde(skip)]
    pub scroll_to_id: Option<Uuid>,
}

/// A single customer gain entry.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Gain {
    /// Stable unique identifier.
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub notes: String,
    /// Whether this row is expanded in accordion mode (UI state, not persisted).
    #[serde(skip)]
    pub expanded: bool,
}

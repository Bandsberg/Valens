use uuid::Uuid;

// ── State structs ─────────────────────────────────────────────────────────────

/// Persistent and transient state for the Pains table.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct PainsState {
    /// All pains.
    pub pains: Vec<Pain>,
    /// ID of the pain awaiting delete confirmation.
    #[serde(skip)]
    pub pending_delete: Option<Uuid>,
    /// ID of the pain whose detail window is open (not persisted).
    #[serde(skip)]
    pub selected_id: Option<Uuid>,
    /// ID of the pain the table should scroll to on the next frame.
    #[serde(skip)]
    pub scroll_to_id: Option<Uuid>,
}

/// A single customer pain entry.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Pain {
    /// Stable unique identifier.
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub notes: String,
    /// Whether this row is expanded in accordion mode (UI state, not persisted).
    #[serde(skip)]
    pub expanded: bool,
}

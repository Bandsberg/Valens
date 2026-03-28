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

/// Default importance for new pains: neutral mid-range (0.5) so the user is
/// prompted to set a real value rather than having needs silently zero-weighted.
/// A function is required here because serde's `default = "…"` attribute takes
/// a function path, not an expression or constant.
fn default_importance() -> f32 {
    0.5
}

/// A single customer pain entry.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Pain {
    /// Stable unique identifier.
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub notes: String,
    /// Customer-perceived importance weight (0.0–1.0).
    #[serde(default = "default_importance")]
    pub importance: f32,
    /// Whether this row is expanded in accordion mode (UI state, not persisted).
    #[serde(skip)]
    pub expanded: bool,
}

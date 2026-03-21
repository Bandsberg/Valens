use uuid::Uuid;

// ── State structs ─────────────────────────────────────────────────────────────

/// Persistent and transient state for the Jobs table.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct JobsState {
    /// All jobs.
    pub jobs: Vec<Job>,
    /// ID of the job awaiting delete confirmation.
    #[serde(skip)]
    pub pending_delete: Option<Uuid>,
    /// ID of the job whose detail window is open (not persisted).
    #[serde(skip)]
    pub selected_id: Option<Uuid>,
    /// ID of the job the table should scroll to on the next frame.
    #[serde(skip)]
    pub scroll_to_id: Option<Uuid>,
}

/// A single customer job entry (Jobs to be Done).
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Job {
    /// Stable unique identifier.
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub notes: String,
    /// Whether this row is expanded in accordion mode (UI state, not persisted).
    #[serde(skip)]
    pub expanded: bool,
}

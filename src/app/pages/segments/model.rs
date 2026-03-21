use uuid::Uuid;

// ── State structs ─────────────────────────────────────────────────────────────

/// Persistent and transient state for the Customer Segments table.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct SegmentsState {
    /// All customer segments.
    pub segments: Vec<CustomerSegment>,
    /// ID of the segment awaiting delete confirmation.
    #[serde(skip)]
    pub pending_delete: Option<Uuid>,
    /// ID of the segment whose detail window is open (not persisted).
    #[serde(skip)]
    pub selected_id: Option<Uuid>,
    /// ID of the segment the table should scroll to on the next frame.
    #[serde(skip)]
    pub scroll_to_id: Option<Uuid>,
}

/// A single customer segment entry.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct CustomerSegment {
    /// Stable unique identifier.
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub notes: String,
    /// Demographics, behaviours, pain points, etc.
    pub characteristics: String,
    /// Whether this row is expanded in accordion mode (UI state, not persisted).
    #[serde(skip)]
    pub expanded: bool,
}

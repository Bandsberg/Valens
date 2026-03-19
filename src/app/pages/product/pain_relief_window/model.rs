use uuid::Uuid;

// ── State structs ─────────────────────────────────────────────────────────────

/// Persistent and transient state for the Pain Relief table.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct PainReliefState {
    pub pain_reliefs: Vec<PainRelief>,
    #[serde(skip)]
    pub pending_delete: Option<Uuid>,
    #[serde(skip)]
    pub selected_id: Option<Uuid>,
    #[serde(skip)]
    pub scroll_to_id: Option<Uuid>,
}

/// A single pain relief entry.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct PainRelief {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub notes: String,
    #[serde(skip)]
    pub expanded: bool,
}

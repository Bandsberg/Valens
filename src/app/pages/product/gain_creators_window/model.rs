use uuid::Uuid;

// ── State structs ─────────────────────────────────────────────────────────────

/// Persistent and transient state for the Gain Creators table.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct GainCreatorState {
    pub gain_creators: Vec<GainCreator>,
    #[serde(skip)]
    pub pending_delete: Option<Uuid>,
    #[serde(skip)]
    pub selected_id: Option<Uuid>,
    #[serde(skip)]
    pub scroll_to_id: Option<Uuid>,
}

/// A single gain creator entry.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct GainCreator {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub notes: String,
    #[serde(skip)]
    pub expanded: bool,
}

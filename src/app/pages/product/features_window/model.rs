use uuid::Uuid;

// ── State structs ─────────────────────────────────────────────────────────────

/// Persistent and transient state for the Features table.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct FeaturesState {
    /// All features.
    pub features: Vec<Feature>,
    /// ID of the feature awaiting delete confirmation.
    #[serde(skip)]
    pub pending_delete: Option<Uuid>,
    /// ID of the feature whose detail window is open (not persisted).
    #[serde(skip)]
    pub selected_feature_id: Option<Uuid>,
    /// ID of the feature the table should scroll to on the next frame.
    #[serde(skip)]
    pub scroll_to_id: Option<Uuid>,
}

/// A single feature entry.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Feature {
    /// Stable unique identifier.
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: String,
    pub notes: String,
    pub user_story: String,
    pub acceptance_criteria: String,
    /// Whether this row is expanded in Accordion mode (UI state, not persisted).
    #[serde(skip)]
    pub expanded: bool,
}

use uuid::Uuid;

// ── State structs ─────────────────────────────────────────────────────────────

/// Persistent and transient state for the Products table.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct ProductsState {
    /// All products.
    pub products: Vec<Product>,
    /// ID of the product awaiting delete confirmation.
    #[serde(skip)]
    pub pending_delete: Option<Uuid>,
    /// ID of the product whose detail window is open (not persisted).
    #[serde(skip)]
    pub selected_product_id: Option<Uuid>,
    /// ID of the product the table should scroll to on the next frame.
    #[serde(skip)]
    pub scroll_to_id: Option<Uuid>,
}

/// A single product entry.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Product {
    /// Stable unique identifier.
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub notes: String,
    /// Whether this row is expanded in Accordion mode (UI state, not persisted).
    #[serde(skip)]
    pub expanded: bool,
}

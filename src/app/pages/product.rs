//! Value Proposition page — products, features, and the links that connect them
//! to customer pains and gains.
//!
//! ## Entity graph
//!
//! ```text
//! Product ──▶ Feature ──▶ PainRelief ──▶ Pain   (pain_relief_annotations)
//!                    └──▶ GainCreator ──▶ Gain   (gain_creator_annotations)
//! ```
//!
//! Link tables store bare `(id_a, id_b)` tuples; annotation tables add
//! [`ValueType`] and `strength` metadata on top of each link.

use super::accordion::{self, label_with_hover_id};
use crate::app::App;
use eframe::egui;
use std::collections::HashSet;
use uuid::Uuid;

// ── Value classification types ─────────────────────────────────────────────────

/// Whether a solution is a minimum requirement or a source of competitive advantage.
///
/// This classification changes how `strength` is interpreted in scoring
/// (see `value_analytics::compute_gap_groups` and `weighted_fit_score`):
/// - `TableStake`: must reach `value_analytics::TABLE_STAKE_MIN_STRENGTH` or the product
///   is flagged as incomplete on that need — viability is at risk.
/// - `Differentiator`: contributes proportionally to the fit score; stands out
///   as a competitive advantage when strength ≥ `value_analytics::DIFFERENTIATOR_STRONG_THRESHOLD`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ValueType {
    /// Minimum requirement to be viable — binary / qualifying.
    TableStake,
    /// Creates competitive advantage — gradual / comparable.
    #[default]
    Differentiator,
}

impl ValueType {
    pub fn label(self) -> &'static str {
        match self {
            Self::TableStake => "Table Stake",
            Self::Differentiator => "Differentiator",
        }
    }
}

/// Annotated link between a Pain/Gain node and its Reliever/Creator.
/// Replaces the bare `(Uuid, Uuid)` tuples in [`ValuePropPage`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValueAnnotation {
    pub pain_or_gain_id: Uuid,
    pub reliever_or_creator_id: Uuid,
    /// See [`ValueType`] for how this changes the scoring interpretation.
    pub value_type: ValueType,
    /// How well this reliever/creator addresses the pain/gain (0.0–1.0).
    /// Used as a direct multiplier in the weighted fit-score formula.
    pub strength: f32,
}
mod side_panel;
pub use side_panel::product_sidepanel;
pub mod products_window;
use products_window::{ProductsState, show_products_window};
pub mod features_window;
use features_window::show_features_window;
pub use features_window::{Feature, FeaturesState};
mod pain_relief_window;
use pain_relief_window::show_pain_relief_window;
pub use pain_relief_window::{PainRelief, PainReliefState};
mod gain_creators_window;
use gain_creators_window::show_gain_creators_window;
pub use gain_creators_window::{GainCreator, GainCreatorState};
mod thoughtful_execution_window;
use thoughtful_execution_window::show_thoughtful_execution_window;
mod value_gap_window;
use value_gap_window::show_value_gap_window;
mod value_quadrant_window;
use value_quadrant_window::show_value_quadrant_window;

// ── Page structs ──────────────────────────────────────────────────────────────

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ValuePropPage {
    // UI state — serialized via eframe::Storage so window toggles survive restarts.
    product_windows: ProductWindows,
    // Entity data — loaded from SQLite on native; not serialized to eframe::Storage.
    #[serde(skip)]
    pub products_state: ProductsState,
    #[serde(skip)]
    pub features_state: FeaturesState,
    #[serde(skip)]
    pub pain_relief_state: PainReliefState,
    #[serde(skip)]
    pub gain_creator_state: GainCreatorState,
    /// Many-to-many links between products and features.
    /// Each entry is `(product_id, feature_id)`.
    #[serde(skip)]
    pub product_feature_links: Vec<(Uuid, Uuid)>,
    /// Many-to-many links between features and pain relief items.
    /// Each entry is `(feature_id, pain_relief_id)`.
    #[serde(skip)]
    pub feature_pain_relief_links: Vec<(Uuid, Uuid)>,
    /// Annotated many-to-many links between pains and pain relief items.
    #[serde(skip)]
    pub pain_relief_annotations: Vec<ValueAnnotation>,
    /// Many-to-many links between features and gain creators.
    /// Each entry is `(feature_id, gain_creator_id)`.
    #[serde(skip)]
    pub feature_gain_creator_links: Vec<(Uuid, Uuid)>,
    /// Annotated many-to-many links between gains and gain creators.
    #[serde(skip)]
    pub gain_creator_annotations: Vec<ValueAnnotation>,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct ProductWindows {
    products_open: bool,
    features_open: bool,
    pain_relief_open: bool,
    gain_creators_open: bool,
    thoughtful_execution_open: bool,
    value_gap_open: bool,
    value_quadrant_open: bool,
}

/// Computes the set of entity IDs that should be highlighted because they are
/// linked (directly or via features) to the currently hovered entity.
///
/// On the Value Proposition page the visible entity types are Products,
/// Gain Creators, and Pain Reliefs. The links run:
/// - `Product` ↔ `Feature` ↔ `GainCreator`
/// - `Product` ↔ `Feature` ↔ `PainRelief`
///
/// **Why three cases?** Features are an invisible intermediate layer — they
/// are not rendered on this page but all links flow through them. To highlight
/// related entities across the page, we must hop through Features in both
/// directions. `GainCreators` and `PainReliefs` need separate cases because they
/// are connected to Features via different link tables and must each be
/// traversed independently.
fn highlighted_ids(hovered: Option<Uuid>, app: &App) -> HashSet<Uuid> {
    let Some(hovered_id) = hovered else {
        return HashSet::new();
    };
    let vp = &app.valueprop_page;
    let mut result = HashSet::new();

    // Case: hovered entity is a Product.
    // Walk forward through its Features to all connected GainCreators and PainReliefs.
    let features_of_product: Vec<Uuid> = vp
        .product_feature_links
        .iter()
        .filter_map(|(product_id, feature_id)| (*product_id == hovered_id).then_some(*feature_id))
        .collect();
    for &feature_id in &features_of_product {
        result.extend(
            vp.feature_gain_creator_links
                .iter()
                .filter_map(|(fid, gc_id)| (*fid == feature_id).then_some(*gc_id)),
        );
        result.extend(
            vp.feature_pain_relief_links
                .iter()
                .filter_map(|(fid, pr_id)| (*fid == feature_id).then_some(*pr_id)),
        );
    }

    // Case: hovered entity is a GainCreator.
    // Walk backward through its Features to find all Products that use it.
    let features_of_gain_creator: Vec<Uuid> = vp
        .feature_gain_creator_links
        .iter()
        .filter_map(|(feature_id, gc_id)| (*gc_id == hovered_id).then_some(*feature_id))
        .collect();
    for &feature_id in &features_of_gain_creator {
        result.extend(
            vp.product_feature_links
                .iter()
                .filter_map(|(product_id, fid)| (*fid == feature_id).then_some(*product_id)),
        );
    }

    // Case: hovered entity is a PainRelief.
    // Walk backward through its Features to find all Products that use it.
    let features_of_pain_relief: Vec<Uuid> = vp
        .feature_pain_relief_links
        .iter()
        .filter_map(|(feature_id, pr_id)| (*pr_id == hovered_id).then_some(*feature_id))
        .collect();
    for &feature_id in &features_of_pain_relief {
        result.extend(
            vp.product_feature_links
                .iter()
                .filter_map(|(product_id, fid)| (*fid == feature_id).then_some(*product_id)),
        );
    }

    result
}

pub fn show_product(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Value Proposition");
    ui.add_space(8.0);

    // Read which entity was hovered last frame, then clear it so it resets
    // if nothing is hovered this frame.
    let hovered_key = egui::Id::new("vp_hovered_entity");
    let prev_hovered: Option<Uuid> = ctx.data(|d| d.get_temp(hovered_key));
    ctx.data_mut(|d| d.remove::<Uuid>(hovered_key));

    let highlighted = highlighted_ids(prev_hovered, app);
    let score = |id: Uuid| {
        if highlighted.contains(&id) {
            1.0_f32
        } else {
            0.0
        }
    };

    ui.columns(2, |cols| {
        let [left, right] = cols else { return };

        // ── Left column: Products & Services ─────────────────────────────────
        left.label(egui::RichText::new("Products & Services").strong());
        left.separator();
        for product in &app.valueprop_page.products_state.products {
            label_with_hover_id(
                left,
                accordion::display_name(&product.name, "Unnamed product"),
                product.id,
                accordion::color_job(),
                score(product.id),
                hovered_key,
            );
        }

        // ── Right column: Gain Creators + Pain Reliefs ────────────────────────
        right.label(egui::RichText::new("Gain Creators").strong());
        right.separator();
        for item in &app.valueprop_page.gain_creator_state.gain_creators {
            label_with_hover_id(
                right,
                accordion::display_name(&item.name, "Unnamed gain creator"),
                item.id,
                accordion::color_gain(),
                score(item.id),
                hovered_key,
            );
        }

        right.add_space(12.0);
        right.label(egui::RichText::new("Pain Reliefs").strong());
        right.separator();
        for item in &app.valueprop_page.pain_relief_state.pain_reliefs {
            label_with_hover_id(
                right,
                accordion::display_name(&item.name, "Unnamed pain relief"),
                item.id,
                accordion::color_pain(),
                score(item.id),
                hovered_key,
            );
        }
    });

    ui.add_space(8.0);

    if app.valueprop_page.product_windows.products_open {
        show_products_window(app, ctx);
    }
    if app.valueprop_page.product_windows.features_open {
        show_features_window(app, ctx);
    }
    if app.valueprop_page.product_windows.pain_relief_open {
        show_pain_relief_window(app, ctx);
    }
    if app.valueprop_page.product_windows.gain_creators_open {
        show_gain_creators_window(app, ctx);
    }
    if app.valueprop_page.product_windows.thoughtful_execution_open {
        show_thoughtful_execution_window(app, ctx);
    }

    // Value Gap and Quadrant windows take &App (read-only), so split the borrow
    // by copying the open flags into temporaries.
    let mut gap_open = app.valueprop_page.product_windows.value_gap_open;
    let mut quadrant_open = app.valueprop_page.product_windows.value_quadrant_open;
    if gap_open {
        show_value_gap_window(app, ctx, &mut gap_open);
        app.valueprop_page.product_windows.value_gap_open = gap_open;
    }
    if quadrant_open {
        show_value_quadrant_window(app, ctx, &mut quadrant_open);
        app.valueprop_page.product_windows.value_quadrant_open = quadrant_open;
    }
}

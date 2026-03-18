use crate::app::App;
use eframe::egui;
use uuid::Uuid;
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
mod thoughtfull_execution_window;
use thoughtfull_execution_window::show_thoughtfull_execution_window;

// ── Page structs ──────────────────────────────────────────────────────────────

/// This is a good sentence to remember about products
/// Products deliver value by enabling capabilities,
/// which users experience through journeys, which are realised via features,
/// which are implemented through user stories.

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ProductPage {
    product_windows: ProductWindows,
    pub products_state: ProductsState,
    pub features_state: FeaturesState,
    pub pain_relief_state: PainReliefState,
    pub gain_creator_state: GainCreatorState,
    /// Many-to-many links between products and features.
    /// Each entry is (product_id, feature_id).
    pub product_feature_links: Vec<(Uuid, Uuid)>,
    /// Many-to-many links between features and pain relief items.
    /// Each entry is (feature_id, pain_relief_id).
    pub feature_pain_relief_links: Vec<(Uuid, Uuid)>,
    /// Many-to-many links between pains and pain relief items.
    /// Each entry is (pain_id, pain_relief_id).
    pub pain_pain_relief_links: Vec<(Uuid, Uuid)>,
    /// Many-to-many links between features and gain creators.
    /// Each entry is (feature_id, gain_creator_id).
    pub feature_gain_creator_links: Vec<(Uuid, Uuid)>,
    /// Many-to-many links between gains and gain creators.
    /// Each entry is (gain_id, gain_creator_id).
    pub gain_gain_creator_links: Vec<(Uuid, Uuid)>,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct ProductWindows {
    products_open: bool,
    features_open: bool,
    pain_relief_open: bool,
    gain_creators_open: bool,
    thoughtfull_execution_open: bool,
}

pub fn show_product(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Products & Services");
    ui.add_space(8.0);

    ui.columns(2, |cols| {
        // ── Left column: Products & Services ─────────────────────────────────
        cols[0].label(egui::RichText::new("Products & Services").strong());
        cols[0].separator();
        for product in &app.product_page.products_state.products {
            let name = if product.name.is_empty() {
                "Unnamed product"
            } else {
                &product.name
            };
            cols[0].label(name);
        }

        // ── Right column: Pain Reliefs + Gain Creators ────────────────────────
        cols[1].label(egui::RichText::new("Pain Reliefs").strong());
        cols[1].separator();
        for item in &app.product_page.pain_relief_state.pain_reliefs {
            let name = if item.name.is_empty() {
                "Unnamed pain relief"
            } else {
                &item.name
            };
            cols[1].label(name);
        }

        cols[1].add_space(12.0);
        cols[1].label(egui::RichText::new("Gain Creators").strong());
        cols[1].separator();
        for item in &app.product_page.gain_creator_state.gain_creators {
            let name = if item.name.is_empty() {
                "Unnamed gain creator"
            } else {
                &item.name
            };
            cols[1].label(name);
        }
    });

    ui.add_space(8.0);

    if app.product_page.product_windows.products_open {
        show_products_window(app, ctx);
    }
    if app.product_page.product_windows.features_open {
        show_features_window(app, ctx);
    }
    if app.product_page.product_windows.pain_relief_open {
        show_pain_relief_window(app, ctx);
    }
    if app.product_page.product_windows.gain_creators_open {
        show_gain_creators_window(app, ctx);
    }
    if app.product_page.product_windows.thoughtfull_execution_open {
        show_thoughtfull_execution_window(app, ctx);
    }
}

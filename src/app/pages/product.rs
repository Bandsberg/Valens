use crate::app::App;
use eframe::egui;
mod side_panel;
pub use side_panel::product_sidepanel;
pub mod products_window;
use products_window::{ProductsState, show_products_window};
pub mod features_window;
use features_window::show_features_window;
pub use features_window::{Feature, FeaturesState};
mod thoughtfull_execution_window;
use thoughtfull_execution_window::show_thoughtfull_execution_window;

/// This is a good sentence to remember about products
/// Products deliver value by enabling capabilities,
/// which users experience through journeys, which are realised via features,
/// which are implemented through user stories.

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ProductPage {
    product_windows: ProductWindows,
    pub products_state: ProductsState,
    pub features_state: FeaturesState,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct ProductWindows {
    products_open: bool,
    features_open: bool,
    thoughtfull_execution_open: bool,
}

pub fn show_product(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Products");
    ui.label("This page has no local state (yet).");
    if app.product_page.product_windows.products_open {
        show_products_window(app, ctx);
    }
    if app.product_page.product_windows.features_open {
        show_features_window(app, ctx);
    }
    if app.product_page.product_windows.thoughtfull_execution_open {
        show_thoughtfull_execution_window(app, ctx);
    }
}

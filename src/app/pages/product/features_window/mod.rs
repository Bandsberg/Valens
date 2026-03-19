use crate::app::App;
use eframe::egui;
use uuid::Uuid;

mod accordion;
mod delete_dialog;
mod detail_panel;
mod model;

use accordion::show_accordion;
use delete_dialog::show_delete_confirmation;
use detail_panel::{navigate_to_product, show_detail_panel};

pub use model::{Feature, FeaturesState};

// ── Main entry point ──────────────────────────────────────────────────────────

/// Renders the Features floating window (and any subordinate windows it spawns).
pub fn show_features_window(app: &mut App, ctx: &egui::Context) {
    // These must be rendered before the main window so they sit on top.
    show_delete_confirmation(app, ctx);
    show_detail_panel(app, ctx);

    // Collected inside the window closure; applied after it releases borrows.
    let mut nav_to_prod: Option<Uuid> = None;

    egui::Window::new("Features")
        .open(&mut app.valueprop_page.product_windows.features_open)
        .default_size([720.0, 380.0])
        .show(ctx, |ui| {
            ui.heading("Features");

            ui.add_space(4.0);

            if ui.button("➕ Add Feature").clicked() {
                app.valueprop_page.features_state.features.push(Feature {
                    id: Uuid::new_v4(),
                    ..Default::default()
                });
            }

            ui.separator();

            // Split borrows across different ProductPage fields.
            let products = &app.valueprop_page.products_state.products;
            let links = &mut app.valueprop_page.product_feature_links;
            show_accordion(
                ui,
                &mut app.valueprop_page.features_state,
                products,
                links,
                &mut nav_to_prod,
            );
        });

    // Apply navigation now that the window closure has released all borrows.
    if let Some(prod_id) = nav_to_prod {
        navigate_to_product(app, ctx, prod_id);
    }
}

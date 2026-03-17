use crate::app::App;
use eframe::egui;
use uuid::Uuid;

mod accordion;
mod delete_dialog;
mod detail_panel;
mod model;

use accordion::show_accordion;
use delete_dialog::show_delete_confirmation;
use detail_panel::{navigate_to_feature, show_detail_panel};

pub use model::{Product, ProductsState};

// ── Main entry point ──────────────────────────────────────────────────────────

/// Renders the Products floating window (and any subordinate windows it spawns).
pub fn show_products_window(app: &mut App, ctx: &egui::Context) {
    // These must be rendered before the main window so they sit on top.
    show_delete_confirmation(app, ctx);
    show_detail_panel(app, ctx);

    // Collected inside the window closure; applied after it releases borrows.
    let mut nav_to_feat: Option<Uuid> = None;

    egui::Window::new("Products & Services")
        .open(&mut app.product_page.product_windows.products_open)
        .default_size([720.0, 380.0])
        .show(ctx, |ui| {
            ui.heading("Products & Services");

            ui.add_space(4.0);

            if ui.button("➕ Add Product").clicked() {
                app.product_page.products_state.products.push(Product {
                    id: Uuid::new_v4(),
                    ..Default::default()
                });
            }

            ui.separator();

            // Split borrows across different ProductPage fields.
            let features = &app.product_page.features_state.features;
            let links = &mut app.product_page.product_feature_links;
            show_accordion(
                ui,
                &mut app.product_page.products_state,
                features,
                links,
                &mut nav_to_feat,
            );
        });

    // Apply navigation now that the window closure has released all borrows.
    if let Some(feat_id) = nav_to_feat {
        navigate_to_feature(app, ctx, feat_id);
    }
}

use crate::app::App;
use eframe::egui;
use uuid::Uuid;

mod accordion;
mod delete_dialog;
mod detail_panel;
mod model;

use accordion::show_accordion;
use delete_dialog::show_delete_confirmation;
use detail_panel::show_detail_panel;

pub use model::{PainRelief, PainReliefState};

use super::products_window::navigate_to_feature;

// ── Main entry point ──────────────────────────────────────────────────────────

/// Renders the Pain Relief floating window (and any subordinate windows).
pub fn show_pain_relief_window(app: &mut App, ctx: &egui::Context) {
    show_delete_confirmation(app, ctx);
    show_detail_panel(app, ctx);

    let mut nav_to_feat: Option<Uuid> = None;

    egui::Window::new("Pain Relief")
        .open(&mut app.valueprop_page.product_windows.pain_relief_open)
        .default_size([720.0, 380.0])
        .show(ctx, |ui| {
            ui.heading("Pain Relief");
            ui.add_space(4.0);
            if ui.button("➕ Add Pain Relief").clicked() {
                app.valueprop_page
                    .pain_relief_state
                    .pain_reliefs
                    .push(PainRelief {
                        id: Uuid::new_v4(),
                        ..Default::default()
                    });
            }
            ui.separator();

            let features = app.valueprop_page.features_state.features.as_slice();
            let pains = app.customer_segment_page.pains_state.pains.as_slice();
            let feature_links = &mut app.valueprop_page.feature_pain_relief_links;
            let pain_links = &mut app.valueprop_page.pain_pain_relief_links;
            show_accordion(
                ui,
                &mut app.valueprop_page.pain_relief_state,
                features,
                pains,
                feature_links,
                pain_links,
                &mut nav_to_feat,
            );
        });

    if let Some(feat_id) = nav_to_feat {
        navigate_to_feature(app, ctx, feat_id);
    }
}

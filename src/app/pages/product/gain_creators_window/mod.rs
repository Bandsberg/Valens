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

pub use model::{GainCreator, GainCreatorState};

use super::products_window::navigate_to_feature;

// ── Main entry point ──────────────────────────────────────────────────────────

/// Renders the Gain Creators floating window (and any subordinate windows).
pub fn show_gain_creators_window(app: &mut App, ctx: &egui::Context) {
    show_delete_confirmation(app, ctx);
    show_detail_panel(app, ctx);

    let mut nav_to_feat: Option<Uuid> = None;

    egui::Window::new("Gain Creators")
        .open(&mut app.valueprop_page.product_windows.gain_creators_open)
        .default_size([720.0, 380.0])
        .show(ctx, |ui| {
            ui.heading("Gain Creators");
            ui.add_space(4.0);
            if ui.button("➕ Add Gain Creator").clicked() {
                app.valueprop_page
                    .gain_creator_state
                    .gain_creators
                    .push(GainCreator {
                        id: Uuid::new_v4(),
                        ..Default::default()
                    });
            }
            ui.separator();

            let features = app.valueprop_page.features_state.features.as_slice();
            let gains = app.customer_segment_page.gains_state.gains.as_slice();
            let feature_links = &mut app.valueprop_page.feature_gain_creator_links;
            // gain_annotations are ValueAnnotation objects (type + strength), not
            // plain (Uuid, Uuid) tuples — named distinctly to avoid confusion with
            // the feature_links tuple vec passed alongside them.
            let gain_annotations = &mut app.valueprop_page.gain_creator_annotations;
            show_accordion(
                ui,
                &mut app.valueprop_page.gain_creator_state,
                features,
                gains,
                feature_links,
                gain_annotations,
                &mut nav_to_feat,
            );
        });

    if let Some(feat_id) = nav_to_feat {
        navigate_to_feature(app, ctx, feat_id);
    }
}

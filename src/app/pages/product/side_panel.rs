use crate::app::App;
use eframe::egui;

/// Renders the Value Proposition right-hand tools panel.
///
/// Each checkbox toggles one of the floating editor windows.
/// The top group contains the core entity editors (Products, Features,
/// Pain Relief, Gain Creators, Thoughtful Execution); the bottom group
/// contains the analytical views (Value Gap Analysis, Value Quadrant).
pub fn product_sidepanel(app: &mut App, ui: &mut egui::Ui) {
    egui::Panel::right("vp_tools_panel")
        .resizable(false)
        .default_size(160.0)
        .min_size(160.0)
        .show_inside(ui, |ui| {
            ui.heading("Tools");
            ui.separator();
            ui.checkbox(
                &mut app.valueprop_page.product_windows.products_open,
                "Products & Services",
            );
            ui.checkbox(
                &mut app.valueprop_page.product_windows.features_open,
                "Features",
            );
            ui.checkbox(
                &mut app.valueprop_page.product_windows.pain_relief_open,
                "Pain Relief",
            );
            ui.checkbox(
                &mut app.valueprop_page.product_windows.gain_creators_open,
                "Gain Creators",
            );
            ui.checkbox(
                &mut app.valueprop_page.product_windows.thoughtful_execution_open,
                "Thoughtful Execution",
            );
            ui.separator();
            ui.checkbox(
                &mut app.valueprop_page.product_windows.value_gap_open,
                "Value Gap Analysis",
            );
            ui.checkbox(
                &mut app.valueprop_page.product_windows.value_quadrant_open,
                "Value Quadrant",
            );
            ui.checkbox(
                &mut app.valueprop_page.product_windows.relationships_open,
                "Entity Relationships",
            );
        });
}

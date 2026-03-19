use crate::app::App;
use crate::app::pages::accordion;
use eframe::egui;

// ── Delete confirmation dialog ────────────────────────────────────────────────

pub fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.valueprop_page.features_state.pending_delete else {
        return;
    };

    let feature_name = app
        .valueprop_page
        .features_state
        .features
        .iter()
        .find(|f| f.id == id)
        .map(|f| accordion::display_name(&f.name, "Unnamed feature").to_owned())
        .unwrap_or_default();

    let mut keep_open = true;
    egui::Window::new("Delete feature?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            ui.label(format!(
                "Are you sure you want to delete \"{feature_name}\"?"
            ));
            ui.label("This action cannot be undone.");
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let delete_btn = ui.add(
                    egui::Button::new(egui::RichText::new("🗑  Delete").color(egui::Color32::WHITE))
                        .fill(egui::Color32::from_rgb(180, 40, 40)),
                );
                if delete_btn.clicked() {
                    // Remove all links associated with this feature.
                    app.valueprop_page
                        .product_feature_links
                        .retain(|(_, fid)| *fid != id);
                    app.valueprop_page
                        .feature_pain_relief_links
                        .retain(|(fid, _)| *fid != id);
                    app.valueprop_page
                        .feature_gain_creator_links
                        .retain(|(fid, _)| *fid != id);
                    app.valueprop_page
                        .features_state
                        .features
                        .retain(|f| f.id != id);
                    app.valueprop_page.features_state.pending_delete = None;
                }
                if ui.button("Cancel").clicked() {
                    app.valueprop_page.features_state.pending_delete = None;
                }
            });
        });

    // User dismissed the dialog with ✕ → treat as cancel.
    if !keep_open {
        app.valueprop_page.features_state.pending_delete = None;
    }
}

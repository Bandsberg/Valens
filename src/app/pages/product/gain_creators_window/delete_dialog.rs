use crate::app::App;
use crate::app::pages::accordion;
use eframe::egui;

// ── Delete confirmation dialog ────────────────────────────────────────────────

pub fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.valueprop_page.gain_creator_state.pending_delete else {
        return;
    };

    let item_name = app
        .valueprop_page
        .gain_creator_state
        .gain_creators
        .iter()
        .find(|r| r.id == id)
        .map(|r| accordion::display_name(&r.name, "Unnamed gain creator").to_owned())
        .unwrap_or_default();

    let mut keep_open = true;
    egui::Window::new("Delete gain creator?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            ui.label(format!("Are you sure you want to delete \"{item_name}\"?"));
            ui.label("This action cannot be undone.");
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let delete_btn = ui.add(
                    egui::Button::new(egui::RichText::new("🗑  Delete").color(egui::Color32::WHITE))
                        .fill(egui::Color32::from_rgb(180, 40, 40)),
                );
                if delete_btn.clicked() {
                    app.valueprop_page
                        .feature_gain_creator_links
                        .retain(|(_, rid)| *rid != id);
                    app.valueprop_page
                        .gain_gain_creator_links
                        .retain(|(_, rid)| *rid != id);
                    app.valueprop_page
                        .gain_creator_state
                        .gain_creators
                        .retain(|r| r.id != id);
                    app.valueprop_page.gain_creator_state.pending_delete = None;
                }
                if ui.button("Cancel").clicked() {
                    app.valueprop_page.gain_creator_state.pending_delete = None;
                }
            });
        });

    if !keep_open {
        app.valueprop_page.gain_creator_state.pending_delete = None;
    }
}

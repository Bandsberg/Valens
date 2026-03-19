use crate::app::App;
use crate::app::pages::accordion;
use eframe::egui;

// ── Delete confirmation dialog ────────────────────────────────────────────────

pub fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.valueprop_page.pain_relief_state.pending_delete else {
        return;
    };

    let item_name = app
        .valueprop_page
        .pain_relief_state
        .pain_reliefs
        .iter()
        .find(|r| r.id == id)
        .map(|r| accordion::display_name(&r.name, "Unnamed pain relief").to_owned())
        .unwrap_or_default();

    let mut keep_open = true;
    egui::Window::new("Delete pain relief?")
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
                        .feature_pain_relief_links
                        .retain(|(_, rid)| *rid != id);
                    app.valueprop_page
                        .pain_pain_relief_links
                        .retain(|(_, rid)| *rid != id);
                    app.valueprop_page
                        .pain_relief_state
                        .pain_reliefs
                        .retain(|r| r.id != id);
                    app.valueprop_page.pain_relief_state.pending_delete = None;
                }
                if ui.button("Cancel").clicked() {
                    app.valueprop_page.pain_relief_state.pending_delete = None;
                }
            });
        });

    if !keep_open {
        app.valueprop_page.pain_relief_state.pending_delete = None;
    }
}

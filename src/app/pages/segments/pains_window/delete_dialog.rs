use crate::app::App;
use eframe::egui;

// ── Delete confirmation dialog ────────────────────────────────────────────────

pub fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.customer_page.pains_state.pending_delete else {
        return;
    };

    let pain_name = app
        .customer_page
        .pains_state
        .pains
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.name.as_str())
        .unwrap_or("this pain")
        .to_owned();

    let mut confirmed = false;
    let mut cancelled = false;

    egui::Window::new("Delete Pain?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!("Delete \"{pain_name}\"?"));
            ui.label(
                egui::RichText::new("This cannot be undone.").color(ui.visuals().warn_fg_color),
            );

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Delete").color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(180, 40, 40)),
                    )
                    .clicked()
                {
                    confirmed = true;
                }

                if ui.button("Cancel").clicked() {
                    cancelled = true;
                }
            });
        });

    if confirmed {
        app.customer_page
            .segment_pain_links
            .retain(|(pid, _)| *pid != id);
        app.customer_page.pains_state.pains.retain(|p| p.id != id);
        if app.customer_page.pains_state.selected_pain_id == Some(id) {
            app.customer_page.pains_state.selected_pain_id = None;
        }
        app.customer_page.pains_state.pending_delete = None;
    } else if cancelled {
        app.customer_page.pains_state.pending_delete = None;
    }
}

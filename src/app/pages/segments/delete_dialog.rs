use crate::app::App;
use eframe::egui;

// ── Delete confirmation dialog ────────────────────────────────────────────────

pub fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.customer_segment_page.segments_state.pending_delete else {
        return;
    };

    // Find the name before opening the window so we can display it.
    let segment_name = app
        .customer_segment_page
        .segments_state
        .segments
        .iter()
        .find(|s| s.id == id)
        .map(|s| s.name.as_str())
        .unwrap_or("this segment")
        .to_owned();

    let mut confirmed = false;
    let mut dismiss = false;

    egui::Window::new("Delete Customer Segment?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!("Delete \"{segment_name}\"?"));
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
                    dismiss = true;
                }
            });
        });

    if confirmed {
        app.customer_segment_page
            .segments_state
            .segments
            .retain(|s| s.id != id);
        if app.customer_segment_page.segments_state.selected_segment_id == Some(id) {
            app.customer_segment_page.segments_state.selected_segment_id = None;
        }
    }
    if confirmed || dismiss {
        app.customer_segment_page.segments_state.pending_delete = None;
    }
}

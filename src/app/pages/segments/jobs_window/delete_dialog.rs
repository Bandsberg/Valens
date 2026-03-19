use crate::app::App;
use eframe::egui;

// ── Delete confirmation dialog ────────────────────────────────────────────────

pub fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.customer_page.jobs_state.pending_delete else {
        return;
    };

    // Find the name before opening the window so we can display it.
    let job_name = app
        .customer_page
        .jobs_state
        .jobs
        .iter()
        .find(|j| j.id == id)
        .map(|j| j.name.as_str())
        .unwrap_or("this job")
        .to_owned();

    let mut confirmed = false;
    let mut dismiss = false;

    egui::Window::new("Delete Job?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!("Delete \"{job_name}\"?"));
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
        app.customer_page
            .segment_job_links
            .retain(|(jid, _)| *jid != id);
        app.customer_page.jobs_state.jobs.retain(|j| j.id != id);
        if app.customer_page.jobs_state.selected_job_id == Some(id) {
            app.customer_page.jobs_state.selected_job_id = None;
        }
    }
    if confirmed || dismiss {
        app.customer_page.jobs_state.pending_delete = None;
    }
}

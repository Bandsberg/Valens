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

pub use model::{Gain, GainsState};

// ── Main entry point ──────────────────────────────────────────────────────────

/// Renders the Gains floating window (and any subordinate windows it spawns).
pub fn show_gains_window(app: &mut App, ctx: &egui::Context) {
    show_delete_confirmation(app, ctx);
    show_detail_panel(app, ctx);

    let mut nav_to_job: Option<Uuid> = None;

    egui::Window::new("Gains")
        .open(&mut app.customer_segment_page.customer_windows.gains_open)
        .default_size([720.0, 380.0])
        .show(ctx, |ui| {
            ui.heading("Gains");
            ui.add_space(4.0);
            if ui.button("➕ Add Gain").clicked() {
                app.customer_segment_page.gains_state.gains.push(Gain {
                    id: Uuid::new_v4(),
                    ..Default::default()
                });
            }
            ui.separator();

            let jobs = &app.customer_segment_page.jobs_state.jobs;
            let links = &mut app.customer_segment_page.job_gain_links;
            show_accordion(
                ui,
                &mut app.customer_segment_page.gains_state,
                jobs,
                links,
                &mut nav_to_job,
            );
        });

    if let Some(job_id) = nav_to_job {
        detail_panel::navigate_to_job(app, ctx, job_id);
    }
}

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

use super::detail_panel::navigate_to_job;

pub use model::{Pain, PainsState};

// ── Main entry point ──────────────────────────────────────────────────────────

/// Renders the Pains floating window (and any subordinate windows it spawns).
pub fn show_pains_window(app: &mut App, ctx: &egui::Context) {
    show_delete_confirmation(app, ctx);
    show_detail_panel(app, ctx);

    let mut nav_to_job: Option<Uuid> = None;

    egui::Window::new("Pains")
        .open(&mut app.customer_segment_page.customer_windows.pains_open)
        .default_size([720.0, 380.0])
        .show(ctx, |ui| {
            ui.heading("Pains");
            ui.add_space(4.0);
            if ui.button("➕ Add Pain").clicked() {
                app.customer_segment_page.pains_state.pains.push(Pain {
                    id: Uuid::new_v4(),
                    ..Default::default()
                });
            }
            ui.separator();

            let jobs = &app.customer_segment_page.jobs_state.jobs;
            let links = &mut app.customer_segment_page.job_pain_links;
            show_accordion(
                ui,
                &mut app.customer_segment_page.pains_state,
                jobs,
                links,
                &mut nav_to_job,
            );
        });

    if let Some(job_id) = nav_to_job {
        navigate_to_job(app, ctx, job_id);
    }
}

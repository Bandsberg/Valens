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

pub use model::{Pain, PainsState};

// ── Main entry point ──────────────────────────────────────────────────────────

/// Renders the Pains floating window (and any subordinate windows it spawns).
pub fn show_pains_window(app: &mut App, ctx: &egui::Context) {
    show_delete_confirmation(app, ctx);
    show_detail_panel(app, ctx);

    let mut nav_to_job: Option<Uuid> = None;

    egui::Window::new("Pains")
        .open(&mut app.customer_page.customer_windows.pains_open)
        .default_size([720.0, 380.0])
        .show(ctx, |ui| {
            ui.heading("Pains");
            ui.add_space(4.0);
            if ui.button("➕ Add Pain").clicked() {
                app.customer_page.pains_state.pains.push(Pain {
                    id: Uuid::new_v4(),
                    ..Default::default()
                });
            }
            ui.separator();

            let jobs = &app.customer_page.jobs_state.jobs;
            let links = &mut app.customer_page.job_pain_links;
            show_accordion(
                ui,
                &mut app.customer_page.pains_state,
                jobs,
                links,
                &mut nav_to_job,
            );
        });

    if let Some(job_id) = nav_to_job {
        detail_panel::navigate_to_job(app, ctx, job_id);
    }
}

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

pub use detail_panel::navigate_to_segment;
pub use model::{Job, JobsState};

// ── Main entry point ──────────────────────────────────────────────────────────

/// Renders the Jobs floating window (and any subordinate windows it spawns).
pub fn show_jobs_window(app: &mut App, ctx: &egui::Context) {
    // These must be rendered before the main window so they sit on top.
    show_delete_confirmation(app, ctx);
    show_detail_panel(app, ctx);

    // Collected inside the window closure; applied after it releases borrows.
    let mut nav_to_seg: Option<Uuid> = None;

    egui::Window::new("Jobs")
        .open(&mut app.customer_segment_page.customer_windows.jobs_open)
        .default_size([720.0, 380.0])
        .show(ctx, |ui| {
            ui.heading("Jobs");

            ui.add_space(4.0);

            if ui.button("➕ Add Job").clicked() {
                app.customer_segment_page.jobs_state.jobs.push(Job {
                    id: Uuid::new_v4(),
                    ..Default::default()
                });
            }

            ui.separator();

            // Split borrows across different CustomerPage fields.
            let segments = &app.customer_segment_page.segments_state.segments;
            let links = &mut app.customer_segment_page.segment_job_links;
            show_accordion(
                ui,
                &mut app.customer_segment_page.jobs_state,
                segments,
                links,
                &mut nav_to_seg,
            );
        });

    // Apply navigation now that the window closure has released all borrows.
    if let Some(seg_id) = nav_to_seg {
        navigate_to_segment(app, ctx, seg_id);
    }
}

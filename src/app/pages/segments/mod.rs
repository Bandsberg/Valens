use crate::app::App;
use eframe::egui;
use uuid::Uuid;

mod accordion;
mod delete_dialog;
mod detail_panel;
mod gains_window;
mod jobs_window;
mod model;
mod pains_window;

use accordion::show_accordion;
use super::accordion::{color_gain, color_job, color_pain, display_name, label_with_hover};
use delete_dialog::show_delete_confirmation;
use detail_panel::{navigate_to_job_fn, show_detail_panel};
use gains_window::show_gains_window;
use jobs_window::show_jobs_window;
use pains_window::show_pains_window;

pub use gains_window::{Gain, GainsState};
pub use jobs_window::{Job, JobsState};
pub use model::{CustomerSegment, SegmentsState};
pub use pains_window::{Pain, PainsState};

// ── Page structs ──────────────────────────────────────────────────────────────

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct CustomerPage {
    customer_windows: CustomerWindows,
    pub segments_state: SegmentsState,
    pub jobs_state: JobsState,
    pub pains_state: PainsState,
    pub gains_state: GainsState,
    /// Many-to-many links between jobs and segments. Each entry is (job_id, segment_id).
    pub segment_job_links: Vec<(Uuid, Uuid)>,
    /// Many-to-many links between pains and jobs. Each entry is (pain_id, job_id).
    pub job_pain_links: Vec<(Uuid, Uuid)>,
    /// Many-to-many links between gains and jobs. Each entry is (gain_id, job_id).
    pub job_gain_links: Vec<(Uuid, Uuid)>,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct CustomerWindows {
    segments_open: bool,
    jobs_open: bool,
    pains_open: bool,
    gains_open: bool,
}

// ── Side panel ────────────────────────────────────────────────────────────────

pub fn customer_sidepanel(app: &mut App, ctx: &egui::Context) {
    egui::SidePanel::right("customer_right_panel")
        .resizable(false)
        .default_width(160.0)
        .min_width(160.0)
        .show(ctx, |ui| {
            ui.heading("Tools");
            ui.separator();
            ui.checkbox(
                &mut app.customer_page.customer_windows.segments_open,
                "Customer Segments",
            );
            ui.checkbox(&mut app.customer_page.customer_windows.jobs_open, "Jobs");
            ui.checkbox(&mut app.customer_page.customer_windows.pains_open, "Pains");
            ui.checkbox(&mut app.customer_page.customer_windows.gains_open, "Gains");
        });
}

// ── Central panel entry point ─────────────────────────────────────────────────

pub fn show_customer(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Customer Segment");
    ui.add_space(8.0);

    ui.columns(2, |cols| {
        // ── Left column: Gains + Pains ────────────────────────────────────────
        cols[0].label(egui::RichText::new("Gains").strong());
        cols[0].separator();
        for item in &app.customer_page.gains_state.gains {
            label_with_hover(&mut cols[0], display_name(&item.name, "Unnamed gain"), color_gain());
        }

        cols[0].add_space(12.0);
        cols[0].label(egui::RichText::new("Pains").strong());
        cols[0].separator();
        for item in &app.customer_page.pains_state.pains {
            label_with_hover(&mut cols[0], display_name(&item.name, "Unnamed pain"), color_pain());
        }

        // ── Right column: Jobs ────────────────────────────────────────────────
        cols[1].label(egui::RichText::new("Jobs").strong());
        cols[1].separator();
        for item in &app.customer_page.jobs_state.jobs {
            label_with_hover(&mut cols[1], display_name(&item.name, "Unnamed job"), color_job());
        }
    });

    ui.add_space(8.0);

    if app.customer_page.customer_windows.segments_open {
        show_segments_window(app, ctx);
    }
    if app.customer_page.customer_windows.jobs_open {
        show_jobs_window(app, ctx);
    }
    if app.customer_page.customer_windows.pains_open {
        show_pains_window(app, ctx);
    }
    if app.customer_page.customer_windows.gains_open {
        show_gains_window(app, ctx);
    }
}

// ── Floating segments window ──────────────────────────────────────────────────

fn show_segments_window(app: &mut App, ctx: &egui::Context) {
    // Rendered before the main window so they sit on top.
    show_delete_confirmation(app, ctx);
    show_detail_panel(app, ctx);

    // Collected inside the window closure; applied after it releases borrows.
    let mut nav_to_job: Option<Uuid> = None;

    egui::Window::new("Customer Segments")
        .open(&mut app.customer_page.customer_windows.segments_open)
        .default_size([720.0, 380.0])
        .show(ctx, |ui| {
            ui.heading("Customer Segments");

            ui.add_space(4.0);

            if ui.button("➕ Add Segment").clicked() {
                app.customer_page
                    .segments_state
                    .segments
                    .push(CustomerSegment {
                        id: Uuid::new_v4(),
                        ..Default::default()
                    });
            }

            ui.separator();

            // Split borrows across different CustomerPage fields.
            let jobs = &app.customer_page.jobs_state.jobs;
            let links = &mut app.customer_page.segment_job_links;
            show_accordion(
                ui,
                &mut app.customer_page.segments_state,
                jobs,
                links,
                &mut nav_to_job,
            );
        });

    // Apply navigation now that the window closure has released all borrows.
    if let Some(job_id) = nav_to_job {
        navigate_to_job_fn(app, ctx, job_id);
    }
}

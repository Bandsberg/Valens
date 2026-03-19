use crate::app::App;
use eframe::egui;
use std::collections::HashSet;
use uuid::Uuid;

mod accordion;
mod delete_dialog;
mod detail_panel;
mod gains_window;
mod jobs_window;
mod model;
mod pains_window;

use super::accordion::{color_gain, color_job, color_pain, display_name, label_with_hover_id};
use accordion::show_accordion;
use delete_dialog::show_delete_confirmation;
use detail_panel::{navigate_to_job, show_detail_panel};
use gains_window::show_gains_window;
use jobs_window::show_jobs_window;
use pains_window::show_pains_window;

pub use gains_window::{Gain, GainsState};
pub use jobs_window::{Job, JobsState};
pub use model::{CustomerSegment, SegmentsState};
pub use pains_window::{Pain, PainsState};

// ── Page structs ──────────────────────────────────────────────────────────────

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct CustomerSegmentPage {
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
                &mut app.customer_segment_page.customer_windows.segments_open,
                "Customer Segments",
            );
            ui.checkbox(
                &mut app.customer_segment_page.customer_windows.jobs_open,
                "Jobs",
            );
            ui.checkbox(
                &mut app.customer_segment_page.customer_windows.pains_open,
                "Pains",
            );
            ui.checkbox(
                &mut app.customer_segment_page.customer_windows.gains_open,
                "Gains",
            );
        });
}

// ── Central panel entry point ─────────────────────────────────────────────────

/// Computes the set of entity IDs that should be highlighted because they are
/// linked to the currently hovered entity.
///
/// On the Customer Segment page the visible entity types are Gains, Pains,
/// and Jobs. The links run:
///   Gain ↔ Job  (job_gain_links: (gain_id, job_id))
///   Pain ↔ Job  (job_pain_links: (pain_id, job_id))
fn highlighted_ids(hovered: Option<Uuid>, app: &App) -> HashSet<Uuid> {
    let mut result = HashSet::new();
    let hovered_id = match hovered {
        Some(id) => id,
        None => return result,
    };

    let cs = &app.customer_segment_page;

    // Hover Job → highlight linked Gains and Pains.
    for (gain_id, job_id) in &cs.job_gain_links {
        if *job_id == hovered_id {
            result.insert(*gain_id);
        }
    }
    for (pain_id, job_id) in &cs.job_pain_links {
        if *job_id == hovered_id {
            result.insert(*pain_id);
        }
    }

    // Hover Gain → highlight linked Jobs.
    for (gain_id, job_id) in &cs.job_gain_links {
        if *gain_id == hovered_id {
            result.insert(*job_id);
        }
    }

    // Hover Pain → highlight linked Jobs.
    for (pain_id, job_id) in &cs.job_pain_links {
        if *pain_id == hovered_id {
            result.insert(*job_id);
        }
    }

    result
}

pub fn show_customer(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Customer Segment");
    ui.add_space(8.0);

    let hovered_key = egui::Id::new("cs_hovered_entity");
    let prev_hovered: Option<Uuid> = ctx.data(|d| d.get_temp(hovered_key));
    ctx.data_mut(|d| d.remove::<Uuid>(hovered_key));

    let highlighted = highlighted_ids(prev_hovered, app);

    ui.columns(2, |cols| {
        // ── Left column: Gains + Pains ────────────────────────────────────────
        cols[0].label(egui::RichText::new("Gains").strong());
        cols[0].separator();
        for item in &app.customer_segment_page.gains_state.gains {
            label_with_hover_id(
                &mut cols[0],
                display_name(&item.name, "Unnamed gain"),
                item.id,
                color_gain(),
                highlighted.contains(&item.id),
                hovered_key,
            );
        }

        cols[0].add_space(12.0);
        cols[0].label(egui::RichText::new("Pains").strong());
        cols[0].separator();
        for item in &app.customer_segment_page.pains_state.pains {
            label_with_hover_id(
                &mut cols[0],
                display_name(&item.name, "Unnamed pain"),
                item.id,
                color_pain(),
                highlighted.contains(&item.id),
                hovered_key,
            );
        }

        // ── Right column: Jobs ────────────────────────────────────────────────
        cols[1].label(egui::RichText::new("Jobs").strong());
        cols[1].separator();
        for item in &app.customer_segment_page.jobs_state.jobs {
            label_with_hover_id(
                &mut cols[1],
                display_name(&item.name, "Unnamed job"),
                item.id,
                color_job(),
                highlighted.contains(&item.id),
                hovered_key,
            );
        }
    });

    ui.add_space(8.0);

    if app.customer_segment_page.customer_windows.segments_open {
        show_segments_window(app, ctx);
    }
    if app.customer_segment_page.customer_windows.jobs_open {
        show_jobs_window(app, ctx);
    }
    if app.customer_segment_page.customer_windows.pains_open {
        show_pains_window(app, ctx);
    }
    if app.customer_segment_page.customer_windows.gains_open {
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
        .open(&mut app.customer_segment_page.customer_windows.segments_open)
        .default_size([720.0, 380.0])
        .show(ctx, |ui| {
            ui.heading("Customer Segments");

            ui.add_space(4.0);

            if ui.button("➕ Add Segment").clicked() {
                app.customer_segment_page
                    .segments_state
                    .segments
                    .push(CustomerSegment {
                        id: Uuid::new_v4(),
                        ..Default::default()
                    });
            }

            ui.separator();

            // Split borrows across different CustomerPage fields.
            let jobs = &app.customer_segment_page.jobs_state.jobs;
            let links = &mut app.customer_segment_page.segment_job_links;
            show_accordion(
                ui,
                &mut app.customer_segment_page.segments_state,
                jobs,
                links,
                &mut nav_to_job,
            );
        });

    // Apply navigation now that the window closure has released all borrows.
    if let Some(job_id) = nav_to_job {
        navigate_to_job(app, ctx, job_id);
    }
}

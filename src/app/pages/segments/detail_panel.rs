use crate::app::App;
use eframe::egui;
use uuid::Uuid;

use super::super::accordion;

// ── Detail panel window ───────────────────────────────────────────────────────

#[expect(clippy::too_many_lines)]
pub fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.customer_segment_page.segments_state.selected_id else {
        return;
    };

    // Snapshot linked / available jobs before entering the window closure so we
    // can borrow `segments_state.segments` mutably inside without conflict.
    // Link tuple: (job_id, segment_id)
    let linked_jids: Vec<Uuid> = app
        .customer_segment_page
        .segment_job_links
        .iter()
        .filter(|(_, sid)| *sid == id)
        .map(|(jid, _)| *jid)
        .collect();

    let linked_jobs: Vec<(Uuid, String)> = app
        .customer_segment_page
        .jobs_state
        .jobs
        .iter()
        .filter(|j| linked_jids.contains(&j.id))
        .map(|j| (j.id, j.name.clone()))
        .collect();

    let available_jobs: Vec<(Uuid, String)> = app
        .customer_segment_page
        .jobs_state
        .jobs
        .iter()
        .filter(|j| !linked_jids.contains(&j.id))
        .map(|j| (j.id, j.name.clone()))
        .collect();

    // Collect mutations during the window; apply them afterwards.
    let mut link_to_add: Option<(Uuid, Uuid)> = None;
    let mut link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut nav_job_id: Option<Uuid> = None;

    let mut keep_open = true;
    egui::Window::new("Customer Segment Details")
        .collapsible(false)
        .resizable(true)
        .default_size([420.0, 420.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            let Some(segment) = app
                .customer_segment_page
                .segments_state
                .segments
                .iter_mut()
                .find(|s| s.id == id)
            else {
                ui.label("Segment not found.");
                return;
            };

            egui::Grid::new("segment_detail_grid")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .min_col_width(120.0)
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.add(
                        egui::TextEdit::singleline(&mut segment.name).desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Description:");
                    ui.add(
                        egui::TextEdit::singleline(&mut segment.description)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Notes:");
                    ui.add(
                        egui::TextEdit::multiline(&mut segment.notes)
                            .desired_rows(4)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Characteristics:");
                    ui.add(
                        egui::TextEdit::multiline(&mut segment.characteristics)
                            .desired_rows(4)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    // ── Linked Jobs ──────────────────────────────────────────
                    let (add, rem) = accordion::detail_link_row(
                        ui,
                        "Linked\nJobs:",
                        egui::Id::new("seg_detail_link_job").with(id),
                        "Add a job…",
                        &available_jobs,
                        &linked_jobs,
                        &mut nav_job_id,
                        Some("Open in Jobs"),
                    );
                    // Link tuple: (job_id, segment_id).
                    if let Some(jid) = add {
                        link_to_add = Some((jid, id));
                    }
                    if let Some(jid) = rem {
                        link_to_remove = Some((jid, id));
                    }
                    ui.end_row();
                });
        });

    // User dismissed with ✕ → deselect.
    if !keep_open {
        app.customer_segment_page.segments_state.selected_id = None;
    }

    // Apply mutations now that the closure has released all borrows.
    if let Some(pair) = link_to_add
        && !app.customer_segment_page.segment_job_links.contains(&pair)
    {
        app.customer_segment_page.segment_job_links.push(pair);
    }
    if let Some(pair) = link_to_remove {
        app.customer_segment_page
            .segment_job_links
            .retain(|l| l != &pair);
    }
    if let Some(job_id) = nav_job_id {
        navigate_to_job(app, ctx, job_id);
    }
}

// ── Navigation helper ─────────────────────────────────────────────────────────

/// Opens the Jobs window and ensures `job_id` is visible:
///   - Sets `expanded = true` on the target job row (accordion).
///   - Sets `selected_id` so the detail panel opens.
///   - Sets `scroll_to_id` so the table scrolls to the row.
pub fn navigate_to_job(app: &mut App, ctx: &egui::Context, job_id: Uuid) {
    app.customer_segment_page.customer_windows.jobs_open = true;
    if let Some(job) = app
        .customer_segment_page
        .jobs_state
        .jobs
        .iter_mut()
        .find(|j| j.id == job_id)
    {
        job.expanded = true;
    }
    app.customer_segment_page.jobs_state.selected_id = Some(job_id);
    app.customer_segment_page.jobs_state.scroll_to_id = Some(job_id);
    // Bring the Jobs window in front of all other windows.
    ctx.move_to_top(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("Jobs"),
    ));
}

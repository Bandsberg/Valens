use crate::app::App;
use eframe::egui;
use uuid::Uuid;

use super::super::super::accordion;

// ── Detail panel window ───────────────────────────────────────────────────────

#[expect(clippy::too_many_lines)]
pub fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.customer_segment_page.jobs_state.selected_id else {
        return;
    };

    // Snapshot linked / available segments before entering the window closure
    // so we can borrow `jobs_state.jobs` mutably inside without conflict.
    // Link tuple: (job_id, segment_id)
    let linked_sids: Vec<Uuid> = app
        .customer_segment_page
        .segment_job_links
        .iter()
        .filter(|(jid, _)| *jid == id)
        .map(|(_, sid)| *sid)
        .collect();

    let linked_segments: Vec<(Uuid, String)> = app
        .customer_segment_page
        .segments_state
        .segments
        .iter()
        .filter(|s| linked_sids.contains(&s.id))
        .map(|s| (s.id, s.name.clone()))
        .collect();

    let available_segments: Vec<(Uuid, String)> = app
        .customer_segment_page
        .segments_state
        .segments
        .iter()
        .filter(|s| !linked_sids.contains(&s.id))
        .map(|s| (s.id, s.name.clone()))
        .collect();

    // Collect mutations during the window; apply them afterwards.
    let mut link_to_add: Option<(Uuid, Uuid)> = None;
    let mut link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut navigate_to_seg: Option<Uuid> = None;

    let mut keep_open = true;
    egui::Window::new("Job Details")
        .collapsible(false)
        .resizable(true)
        .default_size([420.0, 380.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            let Some(job) = app
                .customer_segment_page
                .jobs_state
                .jobs
                .iter_mut()
                .find(|j| j.id == id)
            else {
                ui.label("Job not found.");
                return;
            };

            egui::Grid::new("job_detail_grid")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .min_col_width(100.0)
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.add(egui::TextEdit::singleline(&mut job.name).desired_width(f32::INFINITY));
                    ui.end_row();

                    ui.label("Description:");
                    ui.add(
                        egui::TextEdit::singleline(&mut job.description)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Notes:");
                    ui.add(
                        egui::TextEdit::multiline(&mut job.notes)
                            .desired_rows(5)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    // ── Used by Segments ─────────────────────────────────────
                    let (add, rem) = accordion::detail_link_row(
                        ui,
                        "Used by\nSegments:",
                        egui::Id::new("job_detail_link_seg").with(id),
                        "Add a segment…",
                        &available_segments,
                        &linked_segments,
                        &mut navigate_to_seg,
                        Some("Open in Segments"),
                    );
                    // Link tuple: (job_id, segment_id).
                    if let Some(sid) = add {
                        link_to_add = Some((id, sid));
                    }
                    if let Some(sid) = rem {
                        link_to_remove = Some((id, sid));
                    }
                    ui.end_row();
                });
        });

    // User dismissed with ✕ → deselect.
    if !keep_open {
        app.customer_segment_page.jobs_state.selected_id = None;
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
    if let Some(seg_id) = navigate_to_seg {
        navigate_to_segment(app, ctx, seg_id);
    }
}

// ── Navigation helper ─────────────────────────────────────────────────────────

/// Opens the Customer Segments window and ensures `seg_id` is visible:
///   - Sets `expanded = true` on the target segment row (accordion).
///   - Sets `selected_id` so the detail panel opens.
///   - Sets `scroll_to_id` so the table scrolls to the row.
pub fn navigate_to_segment(app: &mut App, ctx: &egui::Context, seg_id: Uuid) {
    app.customer_segment_page.customer_windows.segments_open = true;
    if let Some(seg) = app
        .customer_segment_page
        .segments_state
        .segments
        .iter_mut()
        .find(|s| s.id == seg_id)
    {
        seg.expanded = true;
    }
    app.customer_segment_page.segments_state.selected_id = Some(seg_id);
    app.customer_segment_page.segments_state.scroll_to_id = Some(seg_id);
    // Bring the Customer Segments window in front of all other windows.
    ctx.move_to_top(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("Customer Segments"),
    ));
}

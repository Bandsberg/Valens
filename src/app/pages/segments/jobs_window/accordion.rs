use eframe::egui;
use uuid::Uuid;

use super::super::model::CustomerSegment;
use super::model::JobsState;

use super::super::super::accordion::{self, ROW_H};

const MULTILINE_H: f32 = 60.0;

// ── Accordion table ───────────────────────────────────────────────────────────

#[expect(clippy::too_many_lines)]
pub fn show_accordion(
    ui: &mut egui::Ui,
    state: &mut JobsState,
    segments: &[CustomerSegment],
    links: &mut Vec<(Uuid, Uuid)>,
    navigate_to: &mut Option<Uuid>,
) {
    let mut to_delete: Option<Uuid> = None;
    let mut link_to_add: Option<(Uuid, Uuid)> = None;
    let mut link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut did_scroll = false;
    let mut do_panel_select: Option<Uuid> = None;
    let mut do_panel_deselect = false;

    // Snapshot links for reading inside row closures (avoids borrow conflict
    // with the mutable `links` we need to update afterwards).
    let links_snap = links.clone();
    let scroll_to = state.scroll_to_id;
    let selected_id = state.selected_id;

    accordion::header(ui, "Job name");

    egui::ScrollArea::vertical().show(ui, |ui| {
        for job in &mut state.jobs {
            let id = job.id;
            let expanded = job.expanded;
            let is_panel_open = selected_id == Some(id);

            if scroll_to == Some(id) {
                ui.scroll_to_cursor(Some(egui::Align::Center));
                did_scroll = true;
            }

            // ── Collapsed / header row ────────────────────────────────────────
            ui.horizontal(|ui| {
                if accordion::expand_button(ui, expanded) {
                    job.expanded = !job.expanded;
                }

                let (name_w, desc_w) = accordion::row_field_widths(ui, "Job name");

                ui.add_sized(
                    [name_w, ROW_H],
                    egui::TextEdit::singleline(&mut job.name).hint_text("Job name…"),
                );
                ui.add_sized(
                    [desc_w, ROW_H],
                    egui::TextEdit::singleline(&mut job.description)
                        .hint_text("Short description…"),
                );

                if accordion::panel_toggle_button(ui, is_panel_open) {
                    if is_panel_open {
                        do_panel_deselect = true;
                    } else {
                        do_panel_select = Some(id);
                    }
                }
                if ui
                    .add(egui::Button::new("🗑").fill(egui::Color32::TRANSPARENT))
                    .on_hover_text("Delete job")
                    .clicked()
                {
                    to_delete = Some(id);
                }
            });

            // ── Expanded content (full-width, no column divide) ───────────────
            if expanded {
                ui.indent(id, |ui| {
                    ui.add_space(4.0);
                    ui.label("Notes:");
                    ui.add(
                        egui::TextEdit::multiline(&mut job.notes)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .min_size(egui::vec2(0.0, MULTILINE_H)),
                    );

                    // ── Used by Segments ──────────────────────────────────────
                    // Link tuple: (job_id, segment_id) — job is first.
                    ui.separator();
                    let (linked_segs, avail_segs) = accordion::partition_linked(
                        &links_snap,
                        |(jid, sid)| (*jid == id).then_some(*sid),
                        segments,
                        |s| s.id,
                        |s| s.name.as_str(),
                    );
                    let (add, rem) = accordion::acc_link_section(
                        ui,
                        "Used by Segments:",
                        egui::Id::new("job_acc_link_seg").with(id),
                        "Add a segment…",
                        "All segments linked",
                        &avail_segs,
                        &linked_segs,
                        navigate_to,
                        Some("Open in Segments"),
                    );
                    // Link tuple is (job_id, segment_id) — note reversed order.
                    if let Some(sid) = add {
                        link_to_add = Some((id, sid));
                    }
                    if let Some(sid) = rem {
                        link_to_remove = Some((id, sid));
                    }
                    ui.add_space(4.0);
                });
            }

            ui.separator();
        }
    });

    // Apply deferred mutations.
    if did_scroll {
        state.scroll_to_id = None;
    }
    if let Some(id) = to_delete {
        state.pending_delete = Some(id);
    }
    if let Some(pair) = link_to_add
        && !links.contains(&pair)
    {
        links.push(pair);
    }
    if let Some(pair) = link_to_remove {
        links.retain(|l| l != &pair);
    }
    // Deselect (close the panel) takes the `if` branch so it always wins.
    // Opening a new row while another is open just replaces `selected_id` —
    // `do_panel_deselect` is only set when the user clicks the toggle on the
    // row that is *already* open, so both flags are never set together.
    if do_panel_deselect {
        state.selected_id = None;
    } else if let Some(id) = do_panel_select {
        state.selected_id = Some(id);
    }
}

use eframe::egui;
use uuid::Uuid;

use super::jobs_window::Job;
use super::model::{CustomerSegment, SegmentsState};

use super::super::accordion::{self, ROW_H};

/// Minimum pixel height for multiline text-edit fields in the expanded row.
/// Chosen to display roughly three lines at the default font size.
const MULTILINE_H: f32 = 60.0;

// ── Accordion table ───────────────────────────────────────────────────────────

#[expect(clippy::too_many_lines)]
pub fn show_accordion(
    ui: &mut egui::Ui,
    state: &mut SegmentsState,
    jobs: &[Job],
    links: &mut Vec<(Uuid, Uuid)>,
    navigate_to: &mut Option<Uuid>,
) {
    let mut to_delete: Option<Uuid> = None;
    let mut link_to_add: Option<(Uuid, Uuid)> = None;
    let mut link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut did_scroll = false;
    let mut do_panel_select: Option<Uuid> = None;
    let mut do_panel_deselect = false;
    let mut new_child_for: Option<Uuid> = None;

    // Snapshot the segments so we can read children (and other data) without
    // conflicting with the per-parent mutable borrows inside the loop.
    let snap = state.segments.clone();
    let links_snap = links.clone();
    let scroll_to = state.scroll_to_id;
    let selected_id = state.selected_id;

    // Only top-level segments are rendered as root rows; sub-segments appear
    // nested inside their parent's expanded section.
    let parent_ids: Vec<Uuid> = snap
        .iter()
        .filter(|s| s.parent_id.is_none())
        .map(|s| s.id)
        .collect();

    accordion::header(ui, "Segment name");

    egui::ScrollArea::vertical().show(ui, |ui| {
        for parent_id in &parent_ids {
            // Borrow one segment at a time; the borrow is released at the end
            // of each block so the next iteration can borrow again.
            let Some(segment) = state.segments.iter_mut().find(|s| s.id == *parent_id) else {
                continue;
            };
            let id = segment.id;
            let expanded = segment.expanded;
            let is_panel_open = selected_id == Some(id);

            if scroll_to == Some(id) {
                ui.scroll_to_cursor(Some(egui::Align::Center));
                did_scroll = true;
            }

            // ── Collapsed / header row ────────────────────────────────────────
            ui.horizontal(|ui| {
                if accordion::expand_button(ui, expanded) {
                    segment.expanded = !segment.expanded;
                }

                let (name_w, desc_w) = accordion::row_field_widths(ui, "Segment name");

                ui.add_sized(
                    [name_w, ROW_H],
                    egui::TextEdit::singleline(&mut segment.name).hint_text("Segment name…"),
                );
                ui.add_sized(
                    [desc_w, ROW_H],
                    egui::TextEdit::singleline(&mut segment.description)
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
                    .on_hover_text("Delete segment")
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
                        egui::TextEdit::multiline(&mut segment.notes)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .min_size(egui::vec2(0.0, MULTILINE_H)),
                    );

                    ui.separator();
                    ui.label("Characteristics:");
                    ui.add(
                        egui::TextEdit::multiline(&mut segment.characteristics)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .min_size(egui::vec2(0.0, MULTILINE_H)),
                    );

                    // ── Linked Jobs ───────────────────────────────────────────
                    // Link tuple: (job_id, segment_id) — segment is second.
                    ui.separator();
                    let (linked_jobs, avail_jobs) = accordion::partition_linked(
                        &links_snap,
                        |(jid, sid)| (*sid == id).then_some(*jid),
                        jobs,
                        |j| j.id,
                        |j| j.name.as_str(),
                    );
                    let (add, rem) = accordion::acc_link_section(
                        ui,
                        "Linked Jobs:",
                        egui::Id::new("seg_acc_link_job").with(id),
                        "Add a job…",
                        "All jobs linked",
                        &avail_jobs,
                        &linked_jobs,
                        navigate_to,
                        Some("Open in Jobs"),
                    );
                    // Link tuple is (job_id, segment_id) — note reversed order.
                    if let Some(jid) = add {
                        link_to_add = Some((jid, id));
                    }
                    if let Some(jid) = rem {
                        link_to_remove = Some((jid, id));
                    }

                    // ── Sub-segments ──────────────────────────────────────────
                    ui.separator();
                    ui.label("Sub-segments:");
                    ui.add_space(2.0);

                    for child in snap.iter().filter(|s| s.parent_id == Some(id)) {
                        let child_id = child.id;
                        let child_is_panel_open = selected_id == Some(child_id);
                        ui.horizontal(|ui| {
                            ui.add_space(4.0);
                            ui.label(accordion::display_name(&child.name, "Unnamed sub-segment"));
                            if !child.description.is_empty() {
                                ui.label(egui::RichText::new(&child.description).weak().italics());
                            }
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new("🗑").fill(egui::Color32::TRANSPARENT),
                                        )
                                        .on_hover_text("Delete sub-segment")
                                        .clicked()
                                    {
                                        to_delete = Some(child_id);
                                    }
                                    if accordion::panel_toggle_button(ui, child_is_panel_open) {
                                        if child_is_panel_open {
                                            do_panel_deselect = true;
                                        } else {
                                            do_panel_select = Some(child_id);
                                        }
                                    }
                                },
                            );
                        });
                    }

                    if ui.button("➕ Add Sub-segment").clicked() {
                        new_child_for = Some(id);
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
    if let Some(parent_id) = new_child_for {
        state.segments.push(CustomerSegment {
            id: Uuid::new_v4(),
            parent_id: Some(parent_id),
            ..Default::default()
        });
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

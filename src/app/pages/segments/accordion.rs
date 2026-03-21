use eframe::egui;
use uuid::Uuid;

use super::jobs_window::Job;
use super::model::SegmentsState;

use super::super::accordion;

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

    // Snapshot links for reading inside row closures (avoids borrow conflict
    // with the mutable `links` we need to update afterwards).
    let links_snap = links.clone();
    let scroll_to = state.scroll_to_id;
    let selected_id = state.selected_id;

    accordion::header(ui, "Segment name");

    egui::ScrollArea::vertical().show(ui, |ui| {
        for segment in &mut state.segments {
            let id = segment.id;
            let expanded = segment.expanded;
            let is_panel_open = selected_id == Some(id);

            // Link tuple: (job_id, segment_id)
            let linked_jids: Vec<Uuid> = links_snap
                .iter()
                .filter(|(_, sid)| *sid == id)
                .map(|(jid, _)| *jid)
                .collect();

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
                    [name_w, 20.0],
                    egui::TextEdit::singleline(&mut segment.name).hint_text("Segment name…"),
                );
                ui.add_sized(
                    [desc_w, 20.0],
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
                    ui.separator();
                    let avail_jobs: Vec<(Uuid, String)> = jobs
                        .iter()
                        .filter(|j| !linked_jids.contains(&j.id))
                        .map(|j| (j.id, j.name.clone()))
                        .collect();
                    let linked_jobs: Vec<(Uuid, String)> = jobs
                        .iter()
                        .filter(|j| linked_jids.contains(&j.id))
                        .map(|j| (j.id, j.name.clone()))
                        .collect();
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
    if do_panel_deselect {
        state.selected_id = None;
    } else if let Some(id) = do_panel_select {
        state.selected_id = Some(id);
    }
}

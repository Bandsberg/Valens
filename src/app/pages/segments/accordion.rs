use eframe::egui;
use egui_extras::{Column, TableBuilder};
use uuid::Uuid;

use super::jobs_window::Job;
use super::model::SegmentsState;

// ── Row height constants ──────────────────────────────────────────────────────

const COLLAPSED_H: f32 = 30.0;
/// Covers: description line + sep + "Notes:" + notes area + sep +
///         "Characteristics:" + characteristics area.
const EXPANDED_H: f32 = 220.0;
const MULTILINE_H: f32 = 60.0;
/// Height of one linked-item row (name link + ✕ button).
const LINK_ROW_H: f32 = 22.0;

// ── Accordion table ───────────────────────────────────────────────────────────

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
    let selected_id = state.selected_segment_id;

    TableBuilder::new(ui)
        .column(Column::exact(24.0)) // ▶ accordion toggle
        .column(Column::initial(170.0).resizable(true)) // Name
        .column(Column::remainder()) // Description + expanded content
        .column(Column::exact(36.0)) // ⊞ detail panel
        .column(Column::exact(36.0)) // 🗑 delete
        .header(20.0, |mut header| {
            header.col(|_ui| {});
            header.col(|ui| {
                ui.heading("Segment name");
            });
            header.col(|ui| {
                ui.heading("Description");
            });
            header.col(|_ui| {});
            header.col(|_ui| {});
        })
        .body(|mut body| {
            for segment in &mut state.segments {
                let id = segment.id;
                let expanded = segment.expanded;
                let is_panel_open = selected_id == Some(id);

                // Pre-compute linked job IDs so we can size the row and
                // determine available jobs without borrowing `links` again.
                // Link tuple: (job_id, segment_id)
                let linked_jids: Vec<Uuid> = links_snap
                    .iter()
                    .filter(|(_, sid)| *sid == id)
                    .map(|(jid, _)| *jid)
                    .collect();

                // Row height:
                //   base      = EXPANDED_H (notes + characteristics + separators)
                //   separator = ~8 px
                //   label     = ~20 px   "Linked Jobs:"
                //   combo     = ~28 px   dropdown widget
                //   rows      = LINK_ROW_H × max(n_linked, 1)  (items or "None")
                //   padding   =  ~8 px
                let num_linked = linked_jids.len();
                let row_h = if expanded {
                    EXPANDED_H + 8.0 + 20.0 + 28.0 + (num_linked.max(1) as f32 * LINK_ROW_H) + 8.0
                } else {
                    COLLAPSED_H
                };

                body.row(row_h, |mut row| {
                    // ── Col 0 : accordion toggle ─────────────────────────────
                    row.col(|ui| {
                        if scroll_to == Some(id) {
                            ui.scroll_to_cursor(Some(egui::Align::Center));
                            did_scroll = true;
                        }
                        let arrow = if segment.expanded { "▼" } else { "▶" };
                        let hover = if expanded { "Collapse" } else { "Expand" };
                        if ui
                            .add(egui::Button::new(arrow).fill(egui::Color32::TRANSPARENT))
                            .on_hover_text(hover)
                            .clicked()
                        {
                            segment.expanded = !segment.expanded;
                        }
                    });

                    // ── Col 1 : name ─────────────────────────────────────────
                    row.col(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut segment.name)
                                .hint_text("Segment name…"),
                        );
                    });

                    // ── Col 2 : description + (expanded) notes, characteristics & linked jobs
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut segment.description)
                                    .hint_text("Short description…"),
                            );

                            if expanded {
                                ui.separator();
                                ui.label("Notes:");
                                ui.add(
                                    egui::TextEdit::multiline(&mut segment.notes)
                                        .desired_rows(3)
                                        .desired_width(ui.available_width())
                                        .min_size(egui::vec2(0.0, MULTILINE_H)),
                                );

                                ui.separator();
                                ui.label("Characteristics:");
                                ui.add(
                                    egui::TextEdit::multiline(&mut segment.characteristics)
                                        .desired_rows(3)
                                        .desired_width(ui.available_width())
                                        .min_size(egui::vec2(0.0, MULTILINE_H)),
                                );

                                // ── Linked Jobs ──────────────────────────────
                                ui.separator();
                                ui.label("Linked Jobs:");

                                let available: Vec<&Job> = jobs
                                    .iter()
                                    .filter(|j| !linked_jids.contains(&j.id))
                                    .collect();

                                if !available.is_empty() {
                                    // Use egui's per-id temp storage so the
                                    // selection survives across frames.
                                    let combo_key = egui::Id::new("seg_acc_link_job").with(id);
                                    let mut sel: Uuid =
                                        ui.data(|d| d.get_temp(combo_key).unwrap_or(Uuid::nil()));

                                    let avail_w = ui.available_width();
                                    egui::ComboBox::from_id_salt(combo_key)
                                        .selected_text("Add a job…")
                                        .width(avail_w)
                                        .show_ui(ui, |ui| {
                                            for job in &available {
                                                ui.selectable_value(&mut sel, job.id, &job.name);
                                            }
                                        });

                                    // Link tuple is (job_id, segment_id)
                                    if sel != Uuid::nil() {
                                        link_to_add = Some((sel, id));
                                        ui.data_mut(|d| d.remove::<Uuid>(combo_key));
                                    } else {
                                        ui.data_mut(|d| d.insert_temp(combo_key, sel));
                                    }
                                } else {
                                    // All jobs are already linked — show a
                                    // disabled placeholder so the layout is stable.
                                    ui.add_enabled(false, egui::Button::new("All jobs linked"));
                                }

                                // Linked jobs — name is a navigation link,
                                // ✕ button removes the link.
                                if !linked_jids.is_empty() {
                                    for jid in &linked_jids {
                                        if let Some(job) = jobs.iter().find(|j| j.id == *jid) {
                                            ui.horizontal(|ui| {
                                                if ui
                                                    .link(&job.name)
                                                    .on_hover_text("Open in Jobs")
                                                    .clicked()
                                                {
                                                    *navigate_to = Some(*jid);
                                                }
                                                if ui
                                                    .add(
                                                        egui::Button::new(
                                                            egui::RichText::new("✕").small().color(
                                                                egui::Color32::from_rgb(
                                                                    200, 60, 60,
                                                                ),
                                                            ),
                                                        )
                                                        .fill(egui::Color32::TRANSPARENT),
                                                    )
                                                    .on_hover_text("Remove link")
                                                    .clicked()
                                                {
                                                    // Link tuple is (job_id, segment_id)
                                                    link_to_remove = Some((*jid, id));
                                                }
                                            });
                                        }
                                    }
                                } else {
                                    ui.label(
                                        egui::RichText::new("None")
                                            .italics()
                                            .color(ui.visuals().weak_text_color()),
                                    );
                                }
                            }
                        });
                    });

                    // ── Col 3 : detail panel button ──────────────────────────
                    row.col(|ui| {
                        let icon = if is_panel_open { "⊟" } else { "⊞" };
                        let hover = if is_panel_open {
                            "Close detail panel"
                        } else {
                            "Open detail panel"
                        };
                        if ui
                            .add(egui::Button::new(icon).fill(egui::Color32::TRANSPARENT))
                            .on_hover_text(hover)
                            .clicked()
                        {
                            if is_panel_open {
                                do_panel_deselect = true;
                            } else {
                                do_panel_select = Some(id);
                            }
                        }
                    });

                    // ── Col 4 : delete ───────────────────────────────────────
                    row.col(|ui| {
                        if ui
                            .add(egui::Button::new("🗑").fill(egui::Color32::TRANSPARENT))
                            .on_hover_text("Delete segment")
                            .clicked()
                        {
                            to_delete = Some(id);
                        }
                    });
                });
            }
        });

    // Apply deferred mutations.
    if did_scroll {
        state.scroll_to_id = None;
    }
    if let Some(id) = to_delete {
        state.pending_delete = Some(id);
    }
    if let Some(pair) = link_to_add {
        if !links.contains(&pair) {
            links.push(pair);
        }
    }
    if let Some(pair) = link_to_remove {
        links.retain(|l| l != &pair);
    }
    if do_panel_deselect {
        state.selected_segment_id = None;
    } else if let Some(id) = do_panel_select {
        state.selected_segment_id = Some(id);
    }
}

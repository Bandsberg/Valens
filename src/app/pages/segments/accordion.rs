use eframe::egui;
use egui_extras::{Column, TableBuilder};
use uuid::Uuid;

use super::model::SegmentsState;

// ── Row height constants ──────────────────────────────────────────────────────

const COLLAPSED_H: f32 = 30.0;
/// Covers: description line + sep + "Notes:" + notes area + sep +
///         "Characteristics:" + characteristics area + padding.
const EXPANDED_H: f32 = 220.0;
const MULTILINE_H: f32 = 60.0;

// ── Accordion table ───────────────────────────────────────────────────────────

pub fn show_accordion(ui: &mut egui::Ui, state: &mut SegmentsState) {
    let mut to_delete: Option<Uuid> = None;
    let mut did_scroll = false;
    let mut do_panel_select: Option<Uuid> = None;
    let mut do_panel_deselect = false;

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

                let row_h = if expanded { EXPANDED_H } else { COLLAPSED_H };

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

                    // ── Col 2 : description + (expanded) notes & characteristics
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
    if do_panel_deselect {
        state.selected_segment_id = None;
    } else if let Some(id) = do_panel_select {
        state.selected_segment_id = Some(id);
    }
}

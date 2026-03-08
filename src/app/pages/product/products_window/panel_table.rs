use eframe::egui;
use egui_extras::{Column, TableBuilder};
use uuid::Uuid;

use super::model::ProductsState;

// ── Row height constant ───────────────────────────────────────────────────────

const COLLAPSED_H: f32 = 30.0;

// ── Panel-mode table ──────────────────────────────────────────────────────────

pub fn show_panel_table(ui: &mut egui::Ui, state: &mut ProductsState) {
    let mut to_delete: Option<Uuid> = None;
    // Two separate variables to avoid Option<Option<_>>.
    let mut do_select: Option<Uuid> = None;
    let mut do_deselect = false;
    let mut did_scroll = false;

    // Snapshot before the closure so we can read it without re-borrowing state.
    let selected_id = state.selected_product_id;
    let scroll_to = state.scroll_to_id;

    TableBuilder::new(ui)
        .column(Column::exact(24.0)) // ▶ / ▼ toggle
        .column(Column::initial(170.0).resizable(true)) // Name
        .column(Column::remainder()) // Description
        .column(Column::exact(36.0)) // 🗑
        .header(20.0, |mut header| {
            header.col(|_ui| {});
            header.col(|ui| {
                ui.heading("Product name");
            });
            header.col(|ui| {
                ui.heading("Description");
            });
            header.col(|_ui| {});
        })
        .body(|mut body| {
            for product in &mut state.products {
                let id = product.id;
                let is_selected = selected_id == Some(id);

                body.row(COLLAPSED_H, |mut row| {
                    // ── Col 0 : toggle arrow ─────────────────────────────────
                    row.col(|ui| {
                        if scroll_to == Some(id) {
                            ui.scroll_to_cursor(Some(egui::Align::Center));
                            did_scroll = true;
                        }
                        let arrow = if is_selected { "▼" } else { "▶" };
                        let hover = if is_selected {
                            "Close details"
                        } else {
                            "Open details"
                        };
                        if ui
                            .add(egui::Button::new(arrow).fill(egui::Color32::TRANSPARENT))
                            .on_hover_text(hover)
                            .clicked()
                        {
                            if is_selected {
                                do_deselect = true;
                            } else {
                                do_select = Some(id);
                            }
                        }
                    });

                    // ── Col 1 : name ─────────────────────────────────────────
                    row.col(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut product.name)
                                .hint_text("Product name…"),
                        );
                    });

                    // ── Col 2 : description ──────────────────────────────────
                    row.col(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut product.description)
                                .hint_text("Short description…"),
                        );
                    });

                    // ── Col 3 : delete ───────────────────────────────────────
                    row.col(|ui| {
                        if ui
                            .add(egui::Button::new("🗑").fill(egui::Color32::TRANSPARENT))
                            .on_hover_text("Delete product")
                            .clicked()
                        {
                            to_delete = Some(id);
                        }
                    });
                });
            }
        });

    if did_scroll {
        state.scroll_to_id = None;
    }
    if let Some(id) = to_delete {
        state.pending_delete = Some(id);
    }
    if do_deselect {
        state.selected_product_id = None;
    } else if let Some(id) = do_select {
        state.selected_product_id = Some(id);
    }
}

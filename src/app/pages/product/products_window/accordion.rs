use eframe::egui;
use egui_extras::{Column, TableBuilder};
use uuid::Uuid;

use super::super::features_window::Feature;
use super::model::ProductsState;

// ── Row height constants ──────────────────────────────────────────────────────

const COLLAPSED_H: f32 = 30.0;
const EXPANDED_H: f32 = 130.0;
const MULTILINE_H: f32 = 60.0;
/// Height of one linked-item row (name + ✕ button).
const LINK_ROW_H: f32 = 22.0;

// ── Accordion table ───────────────────────────────────────────────────────────

pub fn show_accordion(
    ui: &mut egui::Ui,
    state: &mut ProductsState,
    features: &[Feature],
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
    let selected_id = state.selected_product_id;

    TableBuilder::new(ui)
        .column(Column::exact(24.0)) // ▶ accordion toggle
        .column(Column::initial(170.0).resizable(true)) // Name
        .column(Column::remainder()) // Description + Notes + Linked Features
        .column(Column::exact(36.0)) // ⊞ detail panel
        .column(Column::exact(36.0)) // 🗑 delete
        .header(20.0, |mut header| {
            header.col(|_ui| {});
            header.col(|ui| {
                ui.heading("Product name");
            });
            header.col(|ui| {
                ui.heading("Description");
            });
            header.col(|_ui| {});
            header.col(|_ui| {});
        })
        .body(|mut body| {
            for product in &mut state.products {
                let id = product.id;
                let expanded = product.expanded;
                let is_panel_open = selected_id == Some(id);

                // Pre-compute linked feature IDs so we can size the row and
                // determine available features without borrowing `links` again.
                let linked_fids: Vec<Uuid> = links_snap
                    .iter()
                    .filter(|(pid, _)| *pid == id)
                    .map(|(_, fid)| *fid)
                    .collect();

                // Row height:
                //   base      = EXPANDED_H (description + separator + notes)
                //   separator = ~8 px
                //   label     = ~20 px   "Linked Features:"
                //   combo     = ~28 px   dropdown widget
                //   rows      = LINK_ROW_H × max(n_linked, 1)  (items or "None")
                //   padding   =  ~8 px
                let num_linked = linked_fids.len();
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
                        let arrow = if product.expanded { "----" } else { "▶" };
                        let hover = if expanded { "Collapse" } else { "Expand" };
                        if ui
                            .add(egui::Button::new(arrow).fill(egui::Color32::TRANSPARENT))
                            .on_hover_text(hover)
                            .clicked()
                        {
                            product.expanded = !product.expanded;
                        }
                    });

                    // ── Col 1 : name ─────────────────────────────────────────
                    row.col(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut product.name)
                                .hint_text("Product name…"),
                        );
                    });

                    // ── Col 2 : description + (expanded) notes + linked features
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut product.description)
                                    .hint_text("Short description…"),
                            );
                            if expanded {
                                ui.separator();
                                ui.label("Notes:");
                                ui.add(
                                    egui::TextEdit::multiline(&mut product.notes)
                                        .desired_rows(3)
                                        .desired_width(ui.available_width())
                                        .min_size(egui::vec2(0.0, MULTILINE_H)),
                                );

                                // ── Linked Features ──────────────────────────
                                ui.separator();
                                ui.label("Linked Features:");

                                let available: Vec<&Feature> = features
                                    .iter()
                                    .filter(|f| !linked_fids.contains(&f.id))
                                    .collect();

                                if !available.is_empty() {
                                    // Use egui's per-id temp storage so the
                                    // selection survives across frames.
                                    let combo_key = egui::Id::new("prod_acc_link_feat").with(id);
                                    let mut sel: Uuid =
                                        ui.data(|d| d.get_temp(combo_key).unwrap_or(Uuid::nil()));

                                    let avail_w = ui.available_width();
                                    egui::ComboBox::from_id_salt(combo_key)
                                        .selected_text("Add a feature…")
                                        .width(avail_w)
                                        .show_ui(ui, |ui| {
                                            for feat in &available {
                                                ui.selectable_value(&mut sel, feat.id, &feat.name);
                                            }
                                        });

                                    if sel != Uuid::nil() {
                                        link_to_add = Some((id, sel));
                                        ui.data_mut(|d| d.remove::<Uuid>(combo_key));
                                    } else {
                                        ui.data_mut(|d| d.insert_temp(combo_key, sel));
                                    }
                                } else {
                                    // All features are already linked — show a
                                    // disabled placeholder so the layout is stable.
                                    ui.add_enabled(false, egui::Button::new("All features linked"));
                                }

                                // Linked features — name is a navigation link,
                                // ✕ button removes the link.
                                if !linked_fids.is_empty() {
                                    for fid in &linked_fids {
                                        if let Some(feat) = features.iter().find(|f| f.id == *fid) {
                                            ui.horizontal(|ui| {
                                                if ui
                                                    .link(&feat.name)
                                                    .on_hover_text("Open in Features")
                                                    .clicked()
                                                {
                                                    *navigate_to = Some(*fid);
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
                                                    link_to_remove = Some((id, *fid));
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
                            .on_hover_text("Delete product")
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
        state.selected_product_id = None;
    } else if let Some(id) = do_panel_select {
        state.selected_product_id = Some(id);
    }
}

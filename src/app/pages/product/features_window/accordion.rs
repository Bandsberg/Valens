use eframe::egui;
use uuid::Uuid;

use super::super::super::accordion;
use super::super::products_window::Product;
use super::model::FeaturesState;

const MULTILINE_H: f32 = 58.0;

// ── Accordion table ───────────────────────────────────────────────────────────

#[expect(clippy::too_many_lines)]
pub fn show_accordion(
    ui: &mut egui::Ui,
    state: &mut FeaturesState,
    products: &[Product],
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

    accordion::header(ui, "Feature name");

    egui::ScrollArea::vertical().show(ui, |ui| {
        for feature in &mut state.features {
            let id = feature.id;
            let expanded = feature.expanded;
            let is_panel_open = selected_id == Some(id);

            let linked_pids: Vec<Uuid> = links_snap
                .iter()
                .filter(|(_, fid)| *fid == id)
                .map(|(pid, _)| *pid)
                .collect();

            if scroll_to == Some(id) {
                ui.scroll_to_cursor(Some(egui::Align::Center));
                did_scroll = true;
            }

            // ── Collapsed / header row ────────────────────────────────────────
            ui.horizontal(|ui| {
                if accordion::expand_button(ui, expanded) {
                    feature.expanded = !feature.expanded;
                }

                let (name_w, desc_w) = accordion::row_field_widths(ui, "Feature name");

                ui.add_sized(
                    [name_w, 20.0],
                    egui::TextEdit::singleline(&mut feature.name).hint_text("Feature name…"),
                );
                ui.add_sized(
                    [desc_w, 20.0],
                    egui::TextEdit::singleline(&mut feature.description)
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
                    .on_hover_text("Delete feature")
                    .clicked()
                {
                    to_delete = Some(id);
                }
            });

            // ── Expanded content (full-width, no column divide) ───────────────
            if expanded {
                ui.indent(id, |ui| {
                    ui.add_space(4.0);
                    ui.label("Status:");
                    ui.add(
                        egui::TextEdit::singleline(&mut feature.status)
                            .hint_text("e.g. Draft, In Progress, Done")
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(4.0);
                    ui.label("Notes:");
                    ui.add(
                        egui::TextEdit::multiline(&mut feature.notes)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .min_size(egui::vec2(0.0, MULTILINE_H)),
                    );
                    ui.add_space(4.0);
                    ui.label("User Story:");
                    ui.add(
                        egui::TextEdit::multiline(&mut feature.user_story)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .min_size(egui::vec2(0.0, MULTILINE_H)),
                    );
                    ui.add_space(4.0);
                    ui.label("Acceptance Criteria:");
                    ui.add(
                        egui::TextEdit::multiline(&mut feature.acceptance_criteria)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .min_size(egui::vec2(0.0, MULTILINE_H)),
                    );

                    // ── Used by Products ──────────────────────────────────────
                    ui.separator();
                    ui.label("Used by Products & Services:");

                    let available: Vec<&Product> = products
                        .iter()
                        .filter(|p| !linked_pids.contains(&p.id))
                        .collect();

                    if !available.is_empty() {
                        let combo_key = egui::Id::new("feat_acc_link_prod").with(id);
                        let avail_w = ui.available_width();
                        if let Some(sel) = accordion::link_combo_pick(ui, combo_key, |ui, sel| {
                            egui::ComboBox::from_id_salt(combo_key)
                                .selected_text("Add a product…")
                                .width(avail_w)
                                .show_ui(ui, |ui| {
                                    for prod in &available {
                                        ui.selectable_value(sel, prod.id, &prod.name);
                                    }
                                });
                        }) {
                            link_to_add = Some((sel, id));
                        }
                    } else {
                        ui.add_enabled(
                            false,
                            egui::Button::new("All products and services linked"),
                        );
                    }

                    if !linked_pids.is_empty() {
                        for pid in &linked_pids {
                            if let Some(prod) = products.iter().find(|p| p.id == *pid) {
                                ui.horizontal(|ui| {
                                    if ui
                                        .link(&prod.name)
                                        .on_hover_text("Open in Products & Services")
                                        .clicked()
                                    {
                                        *navigate_to = Some(*pid);
                                    }
                                    if accordion::unlink_button(ui).clicked() {
                                        link_to_remove = Some((*pid, id));
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

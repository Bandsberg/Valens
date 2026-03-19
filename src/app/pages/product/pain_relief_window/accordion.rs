use eframe::egui;
use uuid::Uuid;

use super::model::PainReliefState;
use super::super::features_window::Feature;
use super::super::super::Pain;
use super::super::super::accordion;

const MULTILINE_H: f32 = 58.0;

// ── Accordion table ───────────────────────────────────────────────────────────

#[expect(clippy::too_many_lines)]
pub fn show_accordion(
    ui: &mut egui::Ui,
    state: &mut PainReliefState,
    features: &[Feature],
    pains: &[Pain],
    feature_links: &mut Vec<(Uuid, Uuid)>,
    pain_links: &mut Vec<(Uuid, Uuid)>,
    navigate_to: &mut Option<Uuid>,
) {
    let mut to_delete: Option<Uuid> = None;
    let mut feat_link_to_add: Option<(Uuid, Uuid)> = None;
    let mut feat_link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut pain_link_to_add: Option<(Uuid, Uuid)> = None;
    let mut pain_link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut did_scroll = false;
    let mut do_panel_select: Option<Uuid> = None;
    let mut do_panel_deselect = false;

    let feat_links_snap = feature_links.clone();
    let pain_links_snap = pain_links.clone();
    let scroll_to = state.scroll_to_id;
    let selected_id = state.selected_id;

    accordion::header(ui, "Pain Relief name");

    egui::ScrollArea::vertical().show(ui, |ui| {
        for item in &mut state.pain_reliefs {
            let id = item.id;
            let expanded = item.expanded;
            let is_panel_open = selected_id == Some(id);

            let linked_fids: Vec<Uuid> = feat_links_snap
                .iter()
                .filter(|(_, rid)| *rid == id)
                .map(|(fid, _)| *fid)
                .collect();

            let linked_pids: Vec<Uuid> = pain_links_snap
                .iter()
                .filter(|(_, rid)| *rid == id)
                .map(|(pid, _)| *pid)
                .collect();

            if scroll_to == Some(id) {
                ui.scroll_to_cursor(Some(egui::Align::Center));
                did_scroll = true;
            }

            // ── Collapsed / header row ────────────────────────────────────────
            ui.horizontal(|ui| {
                if accordion::expand_button(ui, expanded) {
                    item.expanded = !item.expanded;
                }

                let (name_w, desc_w) = accordion::row_field_widths(ui, "Pain Relief name");

                ui.add_sized(
                    [name_w, 20.0],
                    egui::TextEdit::singleline(&mut item.name).hint_text("Pain relief name…"),
                );
                ui.add_sized(
                    [desc_w, 20.0],
                    egui::TextEdit::singleline(&mut item.description)
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
                    .on_hover_text("Delete pain relief")
                    .clicked()
                {
                    to_delete = Some(id);
                }
            });

            // ── Expanded content ──────────────────────────────────────────────
            if expanded {
                ui.indent(id, |ui| {
                    ui.add_space(4.0);
                    ui.label("Notes:");
                    ui.add(
                        egui::TextEdit::multiline(&mut item.notes)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .min_size(egui::vec2(0.0, MULTILINE_H)),
                    );

                    // ── Linked Features ───────────────────────────────────────
                    ui.separator();
                    ui.label("Linked Features:");

                    let available_feats: Vec<&Feature> = features
                        .iter()
                        .filter(|f| !linked_fids.contains(&f.id))
                        .collect();

                    if !available_feats.is_empty() {
                        let combo_key = egui::Id::new("pr_acc_link_feat").with(id);
                        let avail_w = ui.available_width();
                        if let Some(sel) = accordion::link_combo_pick(ui, combo_key, |ui, sel| {
                            egui::ComboBox::from_id_salt(combo_key)
                                .selected_text("Add a feature…")
                                .width(avail_w)
                                .show_ui(ui, |ui| {
                                    for feat in &available_feats {
                                        ui.selectable_value(sel, feat.id, &feat.name);
                                    }
                                });
                        }) {
                            feat_link_to_add = Some((sel, id));
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("All features linked"));
                    }

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
                                    if accordion::unlink_button(ui).clicked() {
                                        feat_link_to_remove = Some((*fid, id));
                                    }
                                });
                            }
                        }
                    } else {
                        accordion::none_label(ui);
                    }

                    // ── Linked Pains ──────────────────────────────────────────
                    ui.separator();
                    ui.label("Relieves Pains:");

                    let available_pains: Vec<&Pain> = pains
                        .iter()
                        .filter(|p| !linked_pids.contains(&p.id))
                        .collect();

                    if !available_pains.is_empty() {
                        let combo_key = egui::Id::new("pr_acc_link_pain").with(id);
                        let avail_w = ui.available_width();
                        if let Some(sel) = accordion::link_combo_pick(ui, combo_key, |ui, sel| {
                            egui::ComboBox::from_id_salt(combo_key)
                                .selected_text("Add a pain…")
                                .width(avail_w)
                                .show_ui(ui, |ui| {
                                    for pain in &available_pains {
                                        ui.selectable_value(sel, pain.id, &pain.name);
                                    }
                                });
                        }) {
                            pain_link_to_add = Some((sel, id));
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("All pains linked"));
                    }

                    if !linked_pids.is_empty() {
                        for pid in &linked_pids {
                            if let Some(pain) = pains.iter().find(|p| p.id == *pid) {
                                ui.horizontal(|ui| {
                                    ui.label(&pain.name);
                                    if accordion::unlink_button(ui).clicked() {
                                        pain_link_to_remove = Some((*pid, id));
                                    }
                                });
                            }
                        }
                    } else {
                        accordion::none_label(ui);
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
    if let Some(pair) = feat_link_to_add
        && !feature_links.contains(&pair)
    {
        feature_links.push(pair);
    }
    if let Some(pair) = feat_link_to_remove {
        feature_links.retain(|l| l != &pair);
    }
    if let Some(pair) = pain_link_to_add
        && !pain_links.contains(&pair)
    {
        pain_links.push(pair);
    }
    if let Some(pair) = pain_link_to_remove {
        pain_links.retain(|l| l != &pair);
    }
    if do_panel_deselect {
        state.selected_id = None;
    } else if let Some(id) = do_panel_select {
        state.selected_id = Some(id);
    }
}

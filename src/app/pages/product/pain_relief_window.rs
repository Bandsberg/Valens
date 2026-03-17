use crate::app::App;
use eframe::egui;
use uuid::Uuid;

use super::features_window::Feature;
use super::super::accordion;
use super::super::Pain;

const MULTILINE_H: f32 = 58.0;

// ── State structs ─────────────────────────────────────────────────────────────

/// Persistent and transient state for the Pain Relief table.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct PainReliefState {
    pub pain_reliefs: Vec<PainRelief>,
    #[serde(skip)]
    pub pending_delete: Option<Uuid>,
    #[serde(skip)]
    pub selected_id: Option<Uuid>,
    #[serde(skip)]
    pub scroll_to_id: Option<Uuid>,
}

/// A single pain relief entry.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct PainRelief {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub notes: String,
    #[serde(skip)]
    pub expanded: bool,
}

// ── Delete confirmation dialog ────────────────────────────────────────────────

fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.product_page.pain_relief_state.pending_delete else {
        return;
    };

    let item_name = app
        .product_page
        .pain_relief_state
        .pain_reliefs
        .iter()
        .find(|r| r.id == id)
        .map(|r| accordion::display_name(&r.name, "Unnamed pain relief").to_owned())
        .unwrap_or_default();

    let mut keep_open = true;
    egui::Window::new("Delete pain relief?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            ui.label(format!(
                "Are you sure you want to delete \"{item_name}\"?"
            ));
            ui.label("This action cannot be undone.");
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let delete_btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new("🗑  Delete").color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(180, 40, 40)),
                );
                if delete_btn.clicked() {
                    app.product_page
                        .feature_pain_relief_links
                        .retain(|(_, rid)| *rid != id);
                    app.product_page
                        .pain_pain_relief_links
                        .retain(|(_, rid)| *rid != id);
                    app.product_page
                        .pain_relief_state
                        .pain_reliefs
                        .retain(|r| r.id != id);
                    app.product_page.pain_relief_state.pending_delete = None;
                }
                if ui.button("Cancel").clicked() {
                    app.product_page.pain_relief_state.pending_delete = None;
                }
            });
        });

    if !keep_open {
        app.product_page.pain_relief_state.pending_delete = None;
    }
}

// ── Detail panel window ───────────────────────────────────────────────────────

fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.product_page.pain_relief_state.selected_id else {
        return;
    };

    // Snapshot linked features before the window closure.
    let linked_fids: Vec<Uuid> = app
        .product_page
        .feature_pain_relief_links
        .iter()
        .filter(|(_, rid)| *rid == id)
        .map(|(fid, _)| *fid)
        .collect();

    let linked_features: Vec<(Uuid, String)> = app
        .product_page
        .features_state
        .features
        .iter()
        .filter(|f| linked_fids.contains(&f.id))
        .map(|f| (f.id, f.name.clone()))
        .collect();

    let available_features: Vec<(Uuid, String)> = app
        .product_page
        .features_state
        .features
        .iter()
        .filter(|f| !linked_fids.contains(&f.id))
        .map(|f| (f.id, f.name.clone()))
        .collect();

    // Snapshot linked pains before the window closure.
    let linked_pids: Vec<Uuid> = app
        .product_page
        .pain_pain_relief_links
        .iter()
        .filter(|(_, rid)| *rid == id)
        .map(|(pid, _)| *pid)
        .collect();

    let linked_pains: Vec<(Uuid, String)> = app
        .customer_page
        .pains_state
        .pains
        .iter()
        .filter(|p| linked_pids.contains(&p.id))
        .map(|p| (p.id, p.name.clone()))
        .collect();

    let available_pains: Vec<(Uuid, String)> = app
        .customer_page
        .pains_state
        .pains
        .iter()
        .filter(|p| !linked_pids.contains(&p.id))
        .map(|p| (p.id, p.name.clone()))
        .collect();

    let mut feat_link_to_add: Option<(Uuid, Uuid)> = None;
    let mut feat_link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut pain_link_to_add: Option<(Uuid, Uuid)> = None;
    let mut pain_link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut navigate_to_feat: Option<Uuid> = None;

    let mut keep_open = true;
    egui::Window::new("Pain Relief Details")
        .collapsible(false)
        .resizable(true)
        .default_size([420.0, 600.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            let Some(item) = app
                .product_page
                .pain_relief_state
                .pain_reliefs
                .iter_mut()
                .find(|r| r.id == id)
            else {
                ui.label("Pain relief item not found.");
                return;
            };

            egui::Grid::new("pain_relief_detail_grid")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .min_col_width(100.0)
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.add(
                        egui::TextEdit::singleline(&mut item.name)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Description:");
                    ui.add(
                        egui::TextEdit::singleline(&mut item.description)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Notes:");
                    ui.add(
                        egui::TextEdit::multiline(&mut item.notes)
                            .desired_rows(4)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    // ── Linked Features ───────────────────────────────────────
                    ui.label("Linked\nFeatures:");
                    ui.vertical(|ui| {
                        if linked_features.is_empty() {
                            accordion::none_label(ui);
                        } else {
                            for (fid, fname) in &linked_features {
                                ui.horizontal(|ui| {
                                    if ui
                                        .link(fname)
                                        .on_hover_text("Open in Features")
                                        .clicked()
                                    {
                                        navigate_to_feat = Some(*fid);
                                    }
                                    if accordion::unlink_button(ui).clicked() {
                                        feat_link_to_remove = Some((*fid, id));
                                    }
                                });
                            }
                        }
                        if !available_features.is_empty() {
                            ui.add_space(4.0);
                            let combo_key = egui::Id::new("pr_detail_link_feat").with(id);
                            let avail_w = ui.available_width();
                            if let Some(sel) =
                                accordion::link_combo_pick(ui, combo_key, |ui, sel| {
                                    egui::ComboBox::from_id_salt(combo_key)
                                        .selected_text("Add a feature…")
                                        .width(avail_w)
                                        .show_ui(ui, |ui| {
                                            for (fid, fname) in &available_features {
                                                ui.selectable_value(sel, *fid, fname);
                                            }
                                        });
                                })
                            {
                                feat_link_to_add = Some((sel, id));
                            }
                        }
                    });
                    ui.end_row();

                    // ── Relieves Pains ────────────────────────────────────────
                    ui.label("Relieves\nPains:");
                    ui.vertical(|ui| {
                        if linked_pains.is_empty() {
                            accordion::none_label(ui);
                        } else {
                            for (pid, pname) in &linked_pains {
                                ui.horizontal(|ui| {
                                    ui.label(pname);
                                    if accordion::unlink_button(ui).clicked() {
                                        pain_link_to_remove = Some((*pid, id));
                                    }
                                });
                            }
                        }
                        if !available_pains.is_empty() {
                            ui.add_space(4.0);
                            let combo_key = egui::Id::new("pr_detail_link_pain").with(id);
                            let avail_w = ui.available_width();
                            if let Some(sel) =
                                accordion::link_combo_pick(ui, combo_key, |ui, sel| {
                                    egui::ComboBox::from_id_salt(combo_key)
                                        .selected_text("Add a pain…")
                                        .width(avail_w)
                                        .show_ui(ui, |ui| {
                                            for (pid, pname) in &available_pains {
                                                ui.selectable_value(sel, *pid, pname);
                                            }
                                        });
                                })
                            {
                                pain_link_to_add = Some((sel, id));
                            }
                        }
                    });
                    ui.end_row();
                });
        });

    if !keep_open {
        app.product_page.pain_relief_state.selected_id = None;
    }

    if let Some(pair) = feat_link_to_add {
        if !app.product_page.feature_pain_relief_links.contains(&pair) {
            app.product_page.feature_pain_relief_links.push(pair);
        }
    }
    if let Some(pair) = feat_link_to_remove {
        app.product_page
            .feature_pain_relief_links
            .retain(|l| l != &pair);
    }
    if let Some(pair) = pain_link_to_add {
        if !app.product_page.pain_pain_relief_links.contains(&pair) {
            app.product_page.pain_pain_relief_links.push(pair);
        }
    }
    if let Some(pair) = pain_link_to_remove {
        app.product_page
            .pain_pain_relief_links
            .retain(|l| l != &pair);
    }
    if let Some(feat_id) = navigate_to_feat {
        navigate_to_feature(app, ctx, feat_id);
    }
}

// ── Navigation helper ─────────────────────────────────────────────────────────

fn navigate_to_feature(app: &mut App, ctx: &egui::Context, feat_id: Uuid) {
    app.product_page.product_windows.features_open = true;
    if let Some(feat) = app
        .product_page
        .features_state
        .features
        .iter_mut()
        .find(|f| f.id == feat_id)
    {
        feat.expanded = true;
    }
    app.product_page.features_state.selected_feature_id = Some(feat_id);
    app.product_page.features_state.scroll_to_id = Some(feat_id);
    ctx.move_to_top(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("Features"),
    ));
}

// ── Accordion table ───────────────────────────────────────────────────────────

fn show_accordion(
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
                    egui::TextEdit::singleline(&mut item.name)
                        .hint_text("Pain relief name…"),
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
                        if let Some(sel) =
                            accordion::link_combo_pick(ui, combo_key, |ui, sel| {
                                egui::ComboBox::from_id_salt(combo_key)
                                    .selected_text("Add a feature…")
                                    .width(avail_w)
                                    .show_ui(ui, |ui| {
                                        for feat in &available_feats {
                                            ui.selectable_value(sel, feat.id, &feat.name);
                                        }
                                    });
                            })
                        {
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
                        if let Some(sel) =
                            accordion::link_combo_pick(ui, combo_key, |ui, sel| {
                                egui::ComboBox::from_id_salt(combo_key)
                                    .selected_text("Add a pain…")
                                    .width(avail_w)
                                    .show_ui(ui, |ui| {
                                        for pain in &available_pains {
                                            ui.selectable_value(sel, pain.id, &pain.name);
                                        }
                                    });
                            })
                        {
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
    if let Some(pair) = feat_link_to_add {
        if !feature_links.contains(&pair) {
            feature_links.push(pair);
        }
    }
    if let Some(pair) = feat_link_to_remove {
        feature_links.retain(|l| l != &pair);
    }
    if let Some(pair) = pain_link_to_add {
        if !pain_links.contains(&pair) {
            pain_links.push(pair);
        }
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

// ── Main entry point ──────────────────────────────────────────────────────────

/// Renders the Pain Relief floating window (and any subordinate windows).
pub fn show_pain_relief_window(app: &mut App, ctx: &egui::Context) {
    show_delete_confirmation(app, ctx);
    show_detail_panel(app, ctx);

    let mut nav_to_feat: Option<Uuid> = None;

    egui::Window::new("Pain Relief")
        .open(&mut app.product_page.product_windows.pain_relief_open)
        .default_size([720.0, 380.0])
        .show(ctx, |ui| {
            ui.heading("Pain Relief");
            ui.add_space(4.0);
            if ui.button("➕ Add Pain Relief").clicked() {
                app.product_page
                    .pain_relief_state
                    .pain_reliefs
                    .push(PainRelief {
                        id: Uuid::new_v4(),
                        ..Default::default()
                    });
            }
            ui.separator();

            let features = app.product_page.features_state.features.as_slice();
            let pains = app.customer_page.pains_state.pains.as_slice();
            let feature_links = &mut app.product_page.feature_pain_relief_links;
            let pain_links = &mut app.product_page.pain_pain_relief_links;
            show_accordion(
                ui,
                &mut app.product_page.pain_relief_state,
                features,
                pains,
                feature_links,
                pain_links,
                &mut nav_to_feat,
            );
        });

    if let Some(feat_id) = nav_to_feat {
        navigate_to_feature(app, ctx, feat_id);
    }
}

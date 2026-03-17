use crate::app::App;
use eframe::egui;
use uuid::Uuid;

use super::features_window::Feature;
use super::super::accordion;
use super::super::Gain;

const MULTILINE_H: f32 = 58.0;

// ── State structs ─────────────────────────────────────────────────────────────

/// Persistent and transient state for the Gain Creators table.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct GainCreatorState {
    pub gain_creators: Vec<GainCreator>,
    #[serde(skip)]
    pub pending_delete: Option<Uuid>,
    #[serde(skip)]
    pub selected_id: Option<Uuid>,
    #[serde(skip)]
    pub scroll_to_id: Option<Uuid>,
}

/// A single gain creator entry.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct GainCreator {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub notes: String,
    #[serde(skip)]
    pub expanded: bool,
}

// ── Delete confirmation dialog ────────────────────────────────────────────────

fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.product_page.gain_creator_state.pending_delete else {
        return;
    };

    let item_name = app
        .product_page
        .gain_creator_state
        .gain_creators
        .iter()
        .find(|r| r.id == id)
        .map(|r| accordion::display_name(&r.name, "Unnamed gain creator").to_owned())
        .unwrap_or_default();

    let mut keep_open = true;
    egui::Window::new("Delete gain creator?")
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
                        .feature_gain_creator_links
                        .retain(|(_, rid)| *rid != id);
                    app.product_page
                        .gain_gain_creator_links
                        .retain(|(_, rid)| *rid != id);
                    app.product_page
                        .gain_creator_state
                        .gain_creators
                        .retain(|r| r.id != id);
                    app.product_page.gain_creator_state.pending_delete = None;
                }
                if ui.button("Cancel").clicked() {
                    app.product_page.gain_creator_state.pending_delete = None;
                }
            });
        });

    if !keep_open {
        app.product_page.gain_creator_state.pending_delete = None;
    }
}

// ── Detail panel window ───────────────────────────────────────────────────────

fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.product_page.gain_creator_state.selected_id else {
        return;
    };

    // Snapshot linked features before the window closure.
    let linked_fids: Vec<Uuid> = app
        .product_page
        .feature_gain_creator_links
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

    // Snapshot linked gains before the window closure.
    let linked_gids: Vec<Uuid> = app
        .product_page
        .gain_gain_creator_links
        .iter()
        .filter(|(_, rid)| *rid == id)
        .map(|(gid, _)| *gid)
        .collect();

    let linked_gains: Vec<(Uuid, String)> = app
        .customer_page
        .gains_state
        .gains
        .iter()
        .filter(|g| linked_gids.contains(&g.id))
        .map(|g| (g.id, g.name.clone()))
        .collect();

    let available_gains: Vec<(Uuid, String)> = app
        .customer_page
        .gains_state
        .gains
        .iter()
        .filter(|g| !linked_gids.contains(&g.id))
        .map(|g| (g.id, g.name.clone()))
        .collect();

    let mut feat_link_to_add: Option<(Uuid, Uuid)> = None;
    let mut feat_link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut gain_link_to_add: Option<(Uuid, Uuid)> = None;
    let mut gain_link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut navigate_to_feat: Option<Uuid> = None;

    let mut keep_open = true;
    egui::Window::new("Gain Creator Details")
        .collapsible(false)
        .resizable(true)
        .default_size([420.0, 600.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            let Some(item) = app
                .product_page
                .gain_creator_state
                .gain_creators
                .iter_mut()
                .find(|r| r.id == id)
            else {
                ui.label("Gain creator item not found.");
                return;
            };

            egui::Grid::new("gain_creator_detail_grid")
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
                            let combo_key = egui::Id::new("gc_detail_link_feat").with(id);
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

                    // ── Creates Gains ─────────────────────────────────────────
                    ui.label("Creates\nGains:");
                    ui.vertical(|ui| {
                        if linked_gains.is_empty() {
                            accordion::none_label(ui);
                        } else {
                            for (gid, gname) in &linked_gains {
                                ui.horizontal(|ui| {
                                    ui.label(gname);
                                    if accordion::unlink_button(ui).clicked() {
                                        gain_link_to_remove = Some((*gid, id));
                                    }
                                });
                            }
                        }
                        if !available_gains.is_empty() {
                            ui.add_space(4.0);
                            let combo_key = egui::Id::new("gc_detail_link_gain").with(id);
                            let avail_w = ui.available_width();
                            if let Some(sel) =
                                accordion::link_combo_pick(ui, combo_key, |ui, sel| {
                                    egui::ComboBox::from_id_salt(combo_key)
                                        .selected_text("Add a gain…")
                                        .width(avail_w)
                                        .show_ui(ui, |ui| {
                                            for (gid, gname) in &available_gains {
                                                ui.selectable_value(sel, *gid, gname);
                                            }
                                        });
                                })
                            {
                                gain_link_to_add = Some((sel, id));
                            }
                        }
                    });
                    ui.end_row();
                });
        });

    if !keep_open {
        app.product_page.gain_creator_state.selected_id = None;
    }

    if let Some(pair) = feat_link_to_add {
        if !app.product_page.feature_gain_creator_links.contains(&pair) {
            app.product_page.feature_gain_creator_links.push(pair);
        }
    }
    if let Some(pair) = feat_link_to_remove {
        app.product_page
            .feature_gain_creator_links
            .retain(|l| l != &pair);
    }
    if let Some(pair) = gain_link_to_add {
        if !app.product_page.gain_gain_creator_links.contains(&pair) {
            app.product_page.gain_gain_creator_links.push(pair);
        }
    }
    if let Some(pair) = gain_link_to_remove {
        app.product_page
            .gain_gain_creator_links
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
    state: &mut GainCreatorState,
    features: &[Feature],
    gains: &[Gain],
    feature_links: &mut Vec<(Uuid, Uuid)>,
    gain_links: &mut Vec<(Uuid, Uuid)>,
    navigate_to: &mut Option<Uuid>,
) {
    let mut to_delete: Option<Uuid> = None;
    let mut feat_link_to_add: Option<(Uuid, Uuid)> = None;
    let mut feat_link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut gain_link_to_add: Option<(Uuid, Uuid)> = None;
    let mut gain_link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut did_scroll = false;
    let mut do_panel_select: Option<Uuid> = None;
    let mut do_panel_deselect = false;

    let feat_links_snap = feature_links.clone();
    let gain_links_snap = gain_links.clone();
    let scroll_to = state.scroll_to_id;
    let selected_id = state.selected_id;

    accordion::header(ui, "Gain Creator name");

    egui::ScrollArea::vertical().show(ui, |ui| {
        for item in &mut state.gain_creators {
            let id = item.id;
            let expanded = item.expanded;
            let is_panel_open = selected_id == Some(id);

            let linked_fids: Vec<Uuid> = feat_links_snap
                .iter()
                .filter(|(_, rid)| *rid == id)
                .map(|(fid, _)| *fid)
                .collect();

            let linked_gids: Vec<Uuid> = gain_links_snap
                .iter()
                .filter(|(_, rid)| *rid == id)
                .map(|(gid, _)| *gid)
                .collect();

            if scroll_to == Some(id) {
                ui.scroll_to_cursor(Some(egui::Align::Center));
                did_scroll = true;
            }

            // ── Collapsed / header row ────────────────────────────────────────
            ui.horizontal(|ui| {
                let arrow = if expanded { "▼" } else { "▶" };
                let hover = if expanded { "Collapse" } else { "Expand" };
                if ui
                    .add(egui::Button::new(arrow).fill(egui::Color32::TRANSPARENT))
                    .on_hover_text(hover)
                    .clicked()
                {
                    item.expanded = !item.expanded;
                }

                let (name_w, desc_w) = accordion::row_field_widths(ui, "Gain Creator name");

                ui.add_sized(
                    [name_w, 20.0],
                    egui::TextEdit::singleline(&mut item.name)
                        .hint_text("Gain creator name…"),
                );
                ui.add_sized(
                    [desc_w, 20.0],
                    egui::TextEdit::singleline(&mut item.description)
                        .hint_text("Short description…"),
                );

                let icon = if is_panel_open { "⊟" } else { "⊞" };
                let panel_hover = if is_panel_open {
                    "Close detail panel"
                } else {
                    "Open detail panel"
                };
                if ui
                    .add(egui::Button::new(icon).fill(egui::Color32::TRANSPARENT))
                    .on_hover_text(panel_hover)
                    .clicked()
                {
                    if is_panel_open {
                        do_panel_deselect = true;
                    } else {
                        do_panel_select = Some(id);
                    }
                }
                if ui
                    .add(egui::Button::new("🗑").fill(egui::Color32::TRANSPARENT))
                    .on_hover_text("Delete gain creator")
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
                        let combo_key = egui::Id::new("gc_acc_link_feat").with(id);
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

                    // ── Creates Gains ─────────────────────────────────────────
                    ui.separator();
                    ui.label("Creates Gains:");

                    let available_gains: Vec<&Gain> = gains
                        .iter()
                        .filter(|g| !linked_gids.contains(&g.id))
                        .collect();

                    if !available_gains.is_empty() {
                        let combo_key = egui::Id::new("gc_acc_link_gain").with(id);
                        let avail_w = ui.available_width();
                        if let Some(sel) =
                            accordion::link_combo_pick(ui, combo_key, |ui, sel| {
                                egui::ComboBox::from_id_salt(combo_key)
                                    .selected_text("Add a gain…")
                                    .width(avail_w)
                                    .show_ui(ui, |ui| {
                                        for gain in &available_gains {
                                            ui.selectable_value(sel, gain.id, &gain.name);
                                        }
                                    });
                            })
                        {
                            gain_link_to_add = Some((sel, id));
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("All gains linked"));
                    }

                    if !linked_gids.is_empty() {
                        for gid in &linked_gids {
                            if let Some(gain) = gains.iter().find(|g| g.id == *gid) {
                                ui.horizontal(|ui| {
                                    ui.label(&gain.name);
                                    if accordion::unlink_button(ui).clicked() {
                                        gain_link_to_remove = Some((*gid, id));
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
    if let Some(pair) = gain_link_to_add {
        if !gain_links.contains(&pair) {
            gain_links.push(pair);
        }
    }
    if let Some(pair) = gain_link_to_remove {
        gain_links.retain(|l| l != &pair);
    }
    if do_panel_deselect {
        state.selected_id = None;
    } else if let Some(id) = do_panel_select {
        state.selected_id = Some(id);
    }
}

// ── Main entry point ──────────────────────────────────────────────────────────

/// Renders the Gain Creators floating window (and any subordinate windows).
pub fn show_gain_creators_window(app: &mut App, ctx: &egui::Context) {
    show_delete_confirmation(app, ctx);
    show_detail_panel(app, ctx);

    let mut nav_to_feat: Option<Uuid> = None;

    egui::Window::new("Gain Creators")
        .open(&mut app.product_page.product_windows.gain_creators_open)
        .default_size([720.0, 380.0])
        .show(ctx, |ui| {
            ui.heading("Gain Creators");
            ui.add_space(4.0);
            if ui.button("➕ Add Gain Creator").clicked() {
                app.product_page
                    .gain_creator_state
                    .gain_creators
                    .push(GainCreator {
                        id: Uuid::new_v4(),
                        ..Default::default()
                    });
            }
            ui.separator();

            let features = app.product_page.features_state.features.as_slice();
            let gains = app.customer_page.gains_state.gains.as_slice();
            let feature_links = &mut app.product_page.feature_gain_creator_links;
            let gain_links = &mut app.product_page.gain_gain_creator_links;
            show_accordion(
                ui,
                &mut app.product_page.gain_creator_state,
                features,
                gains,
                feature_links,
                gain_links,
                &mut nav_to_feat,
            );
        });

    if let Some(feat_id) = nav_to_feat {
        navigate_to_feature(app, ctx, feat_id);
    }
}

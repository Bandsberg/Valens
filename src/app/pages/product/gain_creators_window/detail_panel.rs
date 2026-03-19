use crate::app::App;
use crate::app::pages::accordion;
use eframe::egui;
use uuid::Uuid;

use super::super::products_window::navigate_to_feature;

// ── Detail panel window ───────────────────────────────────────────────────────

#[expect(clippy::too_many_lines)]
pub fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.valueprop_page.gain_creator_state.selected_id else {
        return;
    };

    // Snapshot linked features before the window closure.
    let linked_fids: Vec<Uuid> = app
        .valueprop_page
        .feature_gain_creator_links
        .iter()
        .filter(|(_, rid)| *rid == id)
        .map(|(fid, _)| *fid)
        .collect();

    let linked_features: Vec<(Uuid, String)> = app
        .valueprop_page
        .features_state
        .features
        .iter()
        .filter(|f| linked_fids.contains(&f.id))
        .map(|f| (f.id, f.name.clone()))
        .collect();

    let available_features: Vec<(Uuid, String)> = app
        .valueprop_page
        .features_state
        .features
        .iter()
        .filter(|f| !linked_fids.contains(&f.id))
        .map(|f| (f.id, f.name.clone()))
        .collect();

    // Snapshot linked gains before the window closure.
    let linked_gids: Vec<Uuid> = app
        .valueprop_page
        .gain_gain_creator_links
        .iter()
        .filter(|(_, rid)| *rid == id)
        .map(|(gid, _)| *gid)
        .collect();

    let linked_gains: Vec<(Uuid, String)> = app
        .customer_segment_page
        .gains_state
        .gains
        .iter()
        .filter(|g| linked_gids.contains(&g.id))
        .map(|g| (g.id, g.name.clone()))
        .collect();

    let available_gains: Vec<(Uuid, String)> = app
        .customer_segment_page
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
                .valueprop_page
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
                    ui.add(egui::TextEdit::singleline(&mut item.name).desired_width(f32::INFINITY));
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
                                    if ui.link(fname).on_hover_text("Open in Features").clicked() {
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
        app.valueprop_page.gain_creator_state.selected_id = None;
    }

    if let Some(pair) = feat_link_to_add
        && !app.valueprop_page.feature_gain_creator_links.contains(&pair)
    {
        app.valueprop_page.feature_gain_creator_links.push(pair);
    }
    if let Some(pair) = feat_link_to_remove {
        app.valueprop_page
            .feature_gain_creator_links
            .retain(|l| l != &pair);
    }
    if let Some(pair) = gain_link_to_add
        && !app.valueprop_page.gain_gain_creator_links.contains(&pair)
    {
        app.valueprop_page.gain_gain_creator_links.push(pair);
    }
    if let Some(pair) = gain_link_to_remove {
        app.valueprop_page
            .gain_gain_creator_links
            .retain(|l| l != &pair);
    }
    if let Some(feat_id) = navigate_to_feat {
        navigate_to_feature(app, ctx, feat_id);
    }
}

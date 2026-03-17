use eframe::egui;
use uuid::Uuid;

use super::super::features_window::Feature;
use super::model::ProductsState;

const MULTILINE_H: f32 = 60.0;

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

    // ── Header row ────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(28.0); // arrow button column
        ui.add_sized(
            [162.0, 20.0],
            egui::Label::new(egui::RichText::new("Product name").heading()),
        );
        ui.label(egui::RichText::new("Description").heading());
    });
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for product in &mut state.products {
            let id = product.id;
            let expanded = product.expanded;
            let is_panel_open = selected_id == Some(id);

            let linked_fids: Vec<Uuid> = links_snap
                .iter()
                .filter(|(pid, _)| *pid == id)
                .map(|(_, fid)| *fid)
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
                    product.expanded = !product.expanded;
                }

                let spacing = ui.spacing().item_spacing.x;
                let btn_space = 36.0 * 2.0 + spacing * 2.0;
                let avail = ui.available_width() - btn_space;
                let name_w = 162.0_f32.min(avail * 0.35);
                let desc_w = (avail - name_w - spacing).max(0.0);

                ui.add_sized(
                    [name_w, 20.0],
                    egui::TextEdit::singleline(&mut product.name).hint_text("Product name…"),
                );
                ui.add_sized(
                    [desc_w, 20.0],
                    egui::TextEdit::singleline(&mut product.description)
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
                    .on_hover_text("Delete product")
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
                        egui::TextEdit::multiline(&mut product.notes)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .min_size(egui::vec2(0.0, MULTILINE_H)),
                    );

                    // ── Linked Features ───────────────────────────────────────
                    ui.separator();
                    ui.label("Linked Features:");

                    let available: Vec<&Feature> = features
                        .iter()
                        .filter(|f| !linked_fids.contains(&f.id))
                        .collect();

                    if !available.is_empty() {
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
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("✕")
                                                    .small()
                                                    .color(egui::Color32::from_rgb(200, 60, 60)),
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

use eframe::egui;
use uuid::Uuid;

use super::super::features_window::Feature;
use super::model::ProductsState;

use super::super::super::accordion;

const MULTILINE_H: f32 = 60.0;

// ── Accordion table ───────────────────────────────────────────────────────────

#[expect(clippy::too_many_lines)]
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
    let selected_id = state.selected_id;

    accordion::header(ui, "Product / Service name");

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
                if accordion::expand_button(ui, expanded) {
                    product.expanded = !product.expanded;
                }

                let (name_w, desc_w) = accordion::row_field_widths(ui, "Product / Service name");

                ui.add_sized(
                    [name_w, 20.0],
                    egui::TextEdit::singleline(&mut product.name).hint_text("Product name…"),
                );
                ui.add_sized(
                    [desc_w, 20.0],
                    egui::TextEdit::singleline(&mut product.description)
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
                    let avail_feats: Vec<(Uuid, String)> = features
                        .iter()
                        .filter(|f| !linked_fids.contains(&f.id))
                        .map(|f| (f.id, f.name.clone()))
                        .collect();
                    let linked_feats: Vec<(Uuid, String)> = features
                        .iter()
                        .filter(|f| linked_fids.contains(&f.id))
                        .map(|f| (f.id, f.name.clone()))
                        .collect();
                    let (add, rem) = accordion::acc_link_section(
                        ui,
                        "Linked Features:",
                        egui::Id::new("prod_acc_link_feat").with(id),
                        "Add a feature…",
                        "All features linked",
                        &avail_feats,
                        &linked_feats,
                        navigate_to,
                        Some("Open in Features"),
                    );
                    // Link tuple is (product_id, feature_id).
                    if let Some(fid) = add {
                        link_to_add = Some((id, fid));
                    }
                    if let Some(fid) = rem {
                        link_to_remove = Some((id, fid));
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

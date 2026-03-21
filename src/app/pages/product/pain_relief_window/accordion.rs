use eframe::egui;
use uuid::Uuid;

use super::super::super::Pain;
use super::super::super::accordion;
use super::super::features_window::Feature;
use super::model::PainReliefState;

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
                        egui::Id::new("pr_acc_link_feat").with(id),
                        "Add a feature…",
                        "All features linked",
                        &avail_feats,
                        &linked_feats,
                        navigate_to,
                        Some("Open in Features"),
                    );
                    if let Some(fid) = add {
                        feat_link_to_add = Some((fid, id));
                    }
                    if let Some(fid) = rem {
                        feat_link_to_remove = Some((fid, id));
                    }

                    // ── Relieves Pains ────────────────────────────────────────
                    ui.separator();
                    let avail_pains: Vec<(Uuid, String)> = pains
                        .iter()
                        .filter(|p| !linked_pids.contains(&p.id))
                        .map(|p| (p.id, p.name.clone()))
                        .collect();
                    let linked_pains: Vec<(Uuid, String)> = pains
                        .iter()
                        .filter(|p| linked_pids.contains(&p.id))
                        .map(|p| (p.id, p.name.clone()))
                        .collect();
                    let (add, rem) = accordion::acc_link_section(
                        ui,
                        "Relieves Pains:",
                        egui::Id::new("pr_acc_link_pain").with(id),
                        "Add a pain…",
                        "All pains linked",
                        &avail_pains,
                        &linked_pains,
                        navigate_to,
                        None,
                    );
                    if let Some(pid) = add {
                        pain_link_to_add = Some((pid, id));
                    }
                    if let Some(pid) = rem {
                        pain_link_to_remove = Some((pid, id));
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

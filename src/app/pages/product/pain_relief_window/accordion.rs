use eframe::egui;
use uuid::Uuid;

use super::super::super::Pain;
use super::super::super::accordion::{self, ROW_H};
use super::super::features_window::Feature;
use super::super::{ValueAnnotation, ValueType};
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
    pain_annotations: &mut Vec<ValueAnnotation>,
    navigate_to: &mut Option<Uuid>,
) {
    let mut to_delete: Option<Uuid> = None;
    let mut feat_link_to_add: Option<(Uuid, Uuid)> = None;
    let mut feat_link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut pain_ann_to_add: Option<ValueAnnotation> = None;
    let mut pain_ann_to_remove: Option<(Uuid, Uuid)> = None;
    let mut did_scroll = false;
    let mut do_panel_select: Option<Uuid> = None;
    let mut do_panel_deselect = false;

    // egui closures (ScrollArea, indent, horizontal) borrow `ui` exclusively,
    // so we cannot also hold a mutable borrow on `state`, `feature_links`, or
    // `pain_annotations` inside them. The pattern here is:
    //   1. Snapshot the data we need to *read* during rendering.
    //   2. Accumulate any mutations in local variables during the render loop.
    //   3. Apply all mutations after the scroll area exits (see bottom of fn).
    let feat_links_snap = feature_links.clone();
    let pain_annotations_snap = pain_annotations.clone();
    let scroll_to = state.scroll_to_id;
    let selected_id = state.selected_id;

    accordion::header(ui, "Pain Relief name");

    egui::ScrollArea::vertical().show(ui, |ui| {
        for item in &mut state.pain_reliefs {
            let id = item.id;
            let expanded = item.expanded;
            let is_panel_open = selected_id == Some(id);

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
                    [name_w, ROW_H],
                    egui::TextEdit::singleline(&mut item.name).hint_text("Pain relief name…"),
                );
                ui.add_sized(
                    [desc_w, ROW_H],
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
                    // Link tuple: (feature_id, pain_relief_id) — pain_relief is second.
                    ui.separator();
                    let (linked_feats, avail_feats) = accordion::partition_linked(
                        &feat_links_snap,
                        |(fid, rid)| (*rid == id).then_some(*fid),
                        features,
                        |f| f.id,
                        |f| f.name.as_str(),
                    );
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
                    // Annotation: pain_or_gain_id = pain_id, reliever_or_creator_id = pr_id.
                    ui.separator();
                    let (linked_pains, avail_pains) = accordion::partition_linked(
                        &pain_annotations_snap,
                        |ann| (ann.reliever_or_creator_id == id).then_some(ann.pain_or_gain_id),
                        pains,
                        |p| p.id,
                        |p| p.name.as_str(),
                    );
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
                        // New annotations start at neutral defaults so the
                        // user can immediately see and adjust them in the
                        // detail panel. 0.5 strength = "medium"; the default
                        // ValueType is Differentiator (see ValueType::default).
                        pain_ann_to_add = Some(ValueAnnotation {
                            pain_or_gain_id: pid,
                            reliever_or_creator_id: id,
                            value_type: ValueType::default(),
                            strength: 0.5,
                        });
                    }
                    if let Some(pid) = rem {
                        pain_ann_to_remove = Some((pid, id));
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
    if let Some(ann) = pain_ann_to_add {
        let already_linked = pain_annotations.iter().any(|a| {
            a.pain_or_gain_id == ann.pain_or_gain_id
                && a.reliever_or_creator_id == ann.reliever_or_creator_id
        });
        if !already_linked {
            pain_annotations.push(ann);
        }
    }
    if let Some((pid, rid)) = pain_ann_to_remove {
        pain_annotations.retain(|a| !(a.pain_or_gain_id == pid && a.reliever_or_creator_id == rid));
    }
    // Deselect (close the panel) takes the `if` branch so it always wins.
    // Opening a new row while another is open just replaces `selected_id` —
    // `do_panel_deselect` is only set when the user clicks the toggle on the
    // row that is *already* open, so both flags are never set together.
    if do_panel_deselect {
        state.selected_id = None;
    } else if let Some(id) = do_panel_select {
        state.selected_id = Some(id);
    }
}

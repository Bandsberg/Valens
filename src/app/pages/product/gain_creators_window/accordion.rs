use eframe::egui;
use uuid::Uuid;

use super::super::super::Gain;
use super::super::super::accordion::{self, ROW_H};
use super::super::features_window::Feature;
use super::super::{ValueAnnotation, ValueType};
use super::model::GainCreatorState;

/// Minimum pixel height for multiline text-edit fields in the expanded row.
/// Chosen to display roughly three lines at the default font size.
const MULTILINE_H: f32 = 60.0;

// ── Accordion table ───────────────────────────────────────────────────────────

#[expect(clippy::too_many_lines)]
pub fn show_accordion(
    ui: &mut egui::Ui,
    state: &mut GainCreatorState,
    features: &[Feature],
    gains: &[Gain],
    feature_links: &mut Vec<(Uuid, Uuid)>,
    gain_annotations: &mut Vec<ValueAnnotation>,
    navigate_to: &mut Option<Uuid>,
) {
    let mut to_delete: Option<Uuid> = None;
    let mut feat_link_to_add: Option<(Uuid, Uuid)> = None;
    let mut feat_link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut gain_ann_to_add: Option<ValueAnnotation> = None;
    let mut gain_ann_to_remove: Option<(Uuid, Uuid)> = None;
    let mut did_scroll = false;
    let mut do_panel_select: Option<Uuid> = None;
    let mut do_panel_deselect = false;

    // egui closures (ScrollArea, indent, horizontal) borrow `ui` exclusively,
    // so we cannot also hold a mutable borrow on `state`, `feature_links`, or
    // `gain_annotations` inside them. The pattern here is:
    //   1. Snapshot the data we need to *read* during rendering.
    //   2. Accumulate any mutations in local variables during the render loop.
    //   3. Apply all mutations after the scroll area exits (see bottom of fn).
    let feat_links_snap = feature_links.clone();
    let gain_annotations_snap = gain_annotations.clone();
    let scroll_to = state.scroll_to_id;
    let selected_id = state.selected_id;

    accordion::header(ui, "Gain Creator name");

    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            for item in &mut state.gain_creators {
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

                    let (name_w, desc_w) = accordion::row_field_widths(ui, "Gain Creator name");

                    ui.add_sized(
                        [name_w, ROW_H],
                        egui::TextEdit::singleline(&mut item.name).hint_text("Gain creator name…"),
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
                        // Link tuple: (feature_id, gain_creator_id) — gain_creator is second.
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
                            egui::Id::new("gc_acc_link_feat").with(id),
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

                        // ── Creates Gains ─────────────────────────────────────────
                        // Annotation: pain_or_gain_id = gain_id, reliever_or_creator_id = gc_id.
                        ui.separator();
                        let (linked_gains, avail_gains) = accordion::partition_linked(
                            &gain_annotations_snap,
                            |ann| (ann.reliever_or_creator_id == id).then_some(ann.pain_or_gain_id),
                            gains,
                            |g| g.id,
                            |g| g.name.as_str(),
                        );
                        let (add, rem) = accordion::acc_link_section(
                            ui,
                            "Creates Gains:",
                            egui::Id::new("gc_acc_link_gain").with(id),
                            "Add a gain…",
                            "All gains linked",
                            &avail_gains,
                            &linked_gains,
                            navigate_to,
                            None,
                        );
                        if let Some(gid) = add {
                            // New annotations start at neutral defaults so the user
                            // can immediately see and adjust them in the detail panel.
                            // strength = 0.5: mid-range, neither "no impact" nor "fully creates".
                            // value_type = Differentiator: the less-severe default — a Table Stake
                            // would immediately flag the product as incomplete if strength < 0.7.
                            gain_ann_to_add = Some(ValueAnnotation {
                                pain_or_gain_id: gid,
                                reliever_or_creator_id: id,
                                value_type: ValueType::default(),
                                strength: 0.5,
                            });
                        }
                        if let Some(gid) = rem {
                            gain_ann_to_remove = Some((gid, id));
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
    if let Some(ann) = gain_ann_to_add {
        let already_linked = gain_annotations.iter().any(|a| {
            a.pain_or_gain_id == ann.pain_or_gain_id
                && a.reliever_or_creator_id == ann.reliever_or_creator_id
        });
        if !already_linked {
            gain_annotations.push(ann);
        }
    }
    if let Some((gid, cid)) = gain_ann_to_remove {
        gain_annotations.retain(|a| !(a.pain_or_gain_id == gid && a.reliever_or_creator_id == cid));
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

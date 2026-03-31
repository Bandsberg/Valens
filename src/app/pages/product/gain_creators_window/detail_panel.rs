use crate::app::App;
use crate::app::pages::accordion;
use eframe::egui;
use uuid::Uuid;

use super::super::products_window::navigate_to_feature;
use super::super::{ValueAnnotation, ValueType};

// ── Detail panel window ───────────────────────────────────────────────────────

/// Renders the Gain Creator detail panel window for the currently selected item.
///
/// Shows editable fields (name, description, notes), the list of linked
/// features, and the annotated gain list with per-gain value-type and strength
/// controls. Does nothing when no gain creator is selected.
#[expect(clippy::too_many_lines)]
pub fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.valueprop_page.gain_creator_state.selected_id else {
        return;
    };

    // Snapshot linked features before the window closure.
    // Link tuple: (feature_id, gain_creator_id) — gain_creator is in second position.
    let (linked_features, available_features) = accordion::partition_linked(
        &app.valueprop_page.feature_gain_creator_links,
        |(fid, rid)| (*rid == id).then_some(*fid),
        &app.valueprop_page.features_state.features,
        |f| f.id,
        |f| f.name.as_str(),
    );

    // Compute gains not yet linked to this creator, so the "Add a gain…" combo
    // only shows unlinked options. We cannot reuse `partition_linked` here
    // because gain-creator links are `ValueAnnotation` objects (with value_type
    // and strength), not plain `(Uuid, Uuid)` tuples — so we extract the
    // linked IDs manually and filter the full gains list.
    let linked_gain_ids: Vec<Uuid> = app
        .valueprop_page
        .gain_creator_annotations
        .iter()
        .filter(|ann| ann.reliever_or_creator_id == id)
        .map(|ann| ann.pain_or_gain_id)
        .collect();
    let available_gains: Vec<(Uuid, String)> = app
        .customer_segment_page
        .gains_state
        .gains
        .iter()
        .filter(|g| !linked_gain_ids.contains(&g.id))
        .map(|g| (g.id, g.name.clone()))
        .collect();

    let mut feat_link_to_add: Option<(Uuid, Uuid)> = None;
    let mut feat_link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut gain_ann_to_add: Option<ValueAnnotation> = None;
    let mut gain_ann_to_remove: Option<Uuid> = None;
    // Inline annotation edits: (gain_id, new_value_type, new_strength)
    let mut ann_edit: Option<(Uuid, ValueType, f32)> = None;
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
                    let (add, rem) = accordion::detail_link_row(
                        ui,
                        "Linked\nFeatures:",
                        egui::Id::new("gc_detail_link_feat").with(id),
                        "Add a feature…",
                        &available_features,
                        &linked_features,
                        &mut navigate_to_feat,
                        Some("Open in Features"),
                    );
                    // Link tuple: (feature_id, gain_creator_id).
                    if let Some(fid) = add {
                        feat_link_to_add = Some((fid, id));
                    }
                    if let Some(fid) = rem {
                        feat_link_to_remove = Some((fid, id));
                    }
                    ui.end_row();
                });

            // ── Creates Gains (annotated) ─────────────────────────────────────
            ui.separator();
            ui.label(egui::RichText::new("Creates Gains").strong());
            ui.add_space(4.0);

            // Render each linked gain with annotation controls.
            let annotations_snap: Vec<ValueAnnotation> = app
                .valueprop_page
                .gain_creator_annotations
                .iter()
                .filter(|ann| ann.reliever_or_creator_id == id)
                .cloned()
                .collect();

            for ann in &annotations_snap {
                let gain_name = app
                    .customer_segment_page
                    .gains_state
                    .gains
                    .iter()
                    .find(|g| g.id == ann.pain_or_gain_id)
                    .map_or("Unknown gain", |g| g.name.as_str());

                let mut cur_type = ann.value_type;
                let mut cur_strength = ann.strength;

                ui.horizontal(|ui| {
                    ui.label(gain_name);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(egui::Button::new("✕").fill(egui::Color32::TRANSPARENT))
                            .on_hover_text("Remove link")
                            .clicked()
                        {
                            gain_ann_to_remove = Some(ann.pain_or_gain_id);
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    egui::ComboBox::new(
                        egui::Id::new("gc_detail_vtype")
                            .with(id)
                            .with(ann.pain_or_gain_id),
                        "",
                    )
                    .selected_text(cur_type.label())
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut cur_type,
                            ValueType::TableStake,
                            ValueType::TableStake.label(),
                        );
                        ui.selectable_value(
                            &mut cur_type,
                            ValueType::Differentiator,
                            ValueType::Differentiator.label(),
                        );
                    });

                    ui.label("Strength:")
                        .on_hover_text(
                            "How well this creator delivers the gain (0 = no impact, 1 = fully delivers).\n\
                             Differentiators: weights the product–segment fit score proportionally.\n\
                             Table Stakes: must reach 70% or the product is flagged as incomplete."
                        );
                    ui.add(
                        egui::DragValue::new(&mut cur_strength)
                            .range(0.0..=1.0)
                            .speed(0.01)
                            .fixed_decimals(2),
                    );
                });

                if cur_type != ann.value_type || (cur_strength - ann.strength).abs() > f32::EPSILON
                {
                    ann_edit = Some((ann.pain_or_gain_id, cur_type, cur_strength));
                }

                ui.add_space(2.0);
            }

            // Add a new gain link.
            if !available_gains.is_empty() {
                let mut selected_gain: Option<Uuid> = None;
                egui::ComboBox::new(egui::Id::new("gc_detail_add_gain").with(id), "")
                    .selected_text("Add a gain…")
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        for (gid, gname) in &available_gains {
                            if ui.selectable_label(false, gname).clicked() {
                                selected_gain = Some(*gid);
                            }
                        }
                    });
                if let Some(gid) = selected_gain {
                    gain_ann_to_add = Some(ValueAnnotation {
                        pain_or_gain_id: gid,
                        reliever_or_creator_id: id,
                        value_type: ValueType::default(),
                        strength: 0.5,
                    });
                }
            }
        });

    if !keep_open {
        app.valueprop_page.gain_creator_state.selected_id = None;
    }

    if let Some(pair) = feat_link_to_add
        && !app
            .valueprop_page
            .feature_gain_creator_links
            .contains(&pair)
    {
        app.valueprop_page.feature_gain_creator_links.push(pair);
    }
    if let Some(pair) = feat_link_to_remove {
        app.valueprop_page
            .feature_gain_creator_links
            .retain(|l| l != &pair);
    }
    if let Some(ann) = gain_ann_to_add {
        super::super::push_annotation_if_new(&mut app.valueprop_page.gain_creator_annotations, ann);
    }
    if let Some(gid) = gain_ann_to_remove {
        app.valueprop_page
            .gain_creator_annotations
            .retain(|a| !(a.pain_or_gain_id == gid && a.reliever_or_creator_id == id));
    }
    if let Some((gid, vtype, strength)) = ann_edit
        && let Some(ann) = app
            .valueprop_page
            .gain_creator_annotations
            .iter_mut()
            .find(|a| a.pain_or_gain_id == gid && a.reliever_or_creator_id == id)
    {
        ann.value_type = vtype;
        ann.strength = strength;
    }
    if let Some(feat_id) = navigate_to_feat {
        navigate_to_feature(app, ctx, feat_id);
    }
}

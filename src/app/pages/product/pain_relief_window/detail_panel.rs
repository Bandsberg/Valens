use crate::app::App;
use crate::app::pages::accordion;
use eframe::egui;
use uuid::Uuid;

use super::super::products_window::navigate_to_feature;
use super::super::{ValueAnnotation, ValueType};

// ── Detail panel window ───────────────────────────────────────────────────────

/// Renders the Pain Relief detail panel window for the currently selected item.
///
/// Shows editable fields (name, description, notes), the list of linked
/// features, and the annotated pain list with per-pain value-type and strength
/// controls. Does nothing when no pain relief is selected.
#[expect(clippy::too_many_lines)]
pub fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.valueprop_page.pain_relief_state.selected_id else {
        return;
    };

    // Snapshot linked features before the window closure.
    // Link tuple: (feature_id, pain_relief_id) — pain_relief is in second position.
    let (linked_features, available_features) = accordion::partition_linked(
        &app.valueprop_page.feature_pain_relief_links,
        |(fid, rid)| (*rid == id).then_some(*fid),
        &app.valueprop_page.features_state.features,
        |f| f.id,
        |f| f.name.as_str(),
    );

    // Snapshot available pains (not yet linked to this relief).
    let linked_pain_ids: Vec<Uuid> = app
        .valueprop_page
        .pain_relief_annotations
        .iter()
        .filter(|ann| ann.reliever_or_creator_id == id)
        .map(|ann| ann.pain_or_gain_id)
        .collect();
    let available_pains: Vec<(Uuid, String)> = app
        .customer_segment_page
        .pains_state
        .pains
        .iter()
        .filter(|p| !linked_pain_ids.contains(&p.id))
        .map(|p| (p.id, p.name.clone()))
        .collect();

    let mut feat_link_to_add: Option<(Uuid, Uuid)> = None;
    let mut feat_link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut pain_ann_to_add: Option<ValueAnnotation> = None;
    let mut pain_ann_to_remove: Option<Uuid> = None;
    // Inline annotation edits: (pain_id, new_value_type, new_strength)
    let mut ann_edit: Option<(Uuid, ValueType, f32)> = None;
    let mut navigate_to_feat: Option<Uuid> = None;

    let mut keep_open = true;
    egui::Window::new("Pain Relief Details")
        .collapsible(false)
        .resizable(true)
        .default_size([420.0, 600.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            let Some(item) = app
                .valueprop_page
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
                        egui::Id::new("pr_detail_link_feat").with(id),
                        "Add a feature…",
                        &available_features,
                        &linked_features,
                        &mut navigate_to_feat,
                        Some("Open in Features"),
                    );
                    // Link tuple: (feature_id, pain_relief_id).
                    if let Some(fid) = add {
                        feat_link_to_add = Some((fid, id));
                    }
                    if let Some(fid) = rem {
                        feat_link_to_remove = Some((fid, id));
                    }
                    ui.end_row();
                });

            // ── Relieves Pains (annotated) ────────────────────────────────────
            ui.separator();
            ui.label(egui::RichText::new("Relieves Pains").strong());
            ui.add_space(4.0);

            // Render each linked pain with annotation controls.
            let annotations_snap: Vec<ValueAnnotation> = app
                .valueprop_page
                .pain_relief_annotations
                .iter()
                .filter(|ann| ann.reliever_or_creator_id == id)
                .cloned()
                .collect();

            for ann in &annotations_snap {
                let pain_name = app
                    .customer_segment_page
                    .pains_state
                    .pains
                    .iter()
                    .find(|p| p.id == ann.pain_or_gain_id)
                    .map_or("Unknown pain", |p| p.name.as_str());

                let mut cur_type = ann.value_type;
                let mut cur_strength = ann.strength;

                ui.horizontal(|ui| {
                    ui.label(pain_name);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(egui::Button::new("✕").fill(egui::Color32::TRANSPARENT))
                            .on_hover_text("Remove link")
                            .clicked()
                        {
                            pain_ann_to_remove = Some(ann.pain_or_gain_id);
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    egui::ComboBox::new(
                        egui::Id::new("pr_detail_vtype")
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
                        .on_hover_text("How well this relief addresses the pain (0 = no impact, 1 = fully resolves). Weights the product–segment fit score.");
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

            // Add a new pain link.
            if !available_pains.is_empty() {
                let mut selected_pain: Option<Uuid> = None;
                egui::ComboBox::new(egui::Id::new("pr_detail_add_pain").with(id), "")
                    .selected_text("Add a pain…")
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        for (pid, pname) in &available_pains {
                            if ui.selectable_label(false, pname).clicked() {
                                selected_pain = Some(*pid);
                            }
                        }
                    });
                if let Some(pid) = selected_pain {
                    pain_ann_to_add = Some(ValueAnnotation {
                        pain_or_gain_id: pid,
                        reliever_or_creator_id: id,
                        value_type: ValueType::default(),
                        strength: 0.5,
                    });
                }
            }
        });

    if !keep_open {
        app.valueprop_page.pain_relief_state.selected_id = None;
    }

    if let Some(pair) = feat_link_to_add
        && !app.valueprop_page.feature_pain_relief_links.contains(&pair)
    {
        app.valueprop_page.feature_pain_relief_links.push(pair);
    }
    if let Some(pair) = feat_link_to_remove {
        app.valueprop_page
            .feature_pain_relief_links
            .retain(|l| l != &pair);
    }
    if let Some(ann) = pain_ann_to_add {
        let exists =
            app.valueprop_page.pain_relief_annotations.iter().any(|a| {
                a.pain_or_gain_id == ann.pain_or_gain_id && a.reliever_or_creator_id == id
            });
        if !exists {
            app.valueprop_page.pain_relief_annotations.push(ann);
        }
    }
    if let Some(pid) = pain_ann_to_remove {
        app.valueprop_page
            .pain_relief_annotations
            .retain(|a| !(a.pain_or_gain_id == pid && a.reliever_or_creator_id == id));
    }
    if let Some((pid, vtype, strength)) = ann_edit
        && let Some(ann) = app
            .valueprop_page
            .pain_relief_annotations
            .iter_mut()
            .find(|a| a.pain_or_gain_id == pid && a.reliever_or_creator_id == id)
    {
        ann.value_type = vtype;
        ann.strength = strength;
    }
    if let Some(feat_id) = navigate_to_feat {
        navigate_to_feature(app, ctx, feat_id);
    }
}

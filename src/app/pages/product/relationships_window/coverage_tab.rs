// ── Tab 2: Coverage ───────────────────────────────────────────────────────────
//
// Two heatmap grids: Pains × Pain Reliefs and Gains × Gain Creators.
// Cells are colour-coded by annotation strength (amber → green).
// TableStake annotations are marked with a ★ badge.

use crate::app::App;
use crate::app::pages::accordion::display_name;
use crate::app::pages::product::{ValueAnnotation, ValueType};
use eframe::egui;
use uuid::Uuid;

use super::{show_placeholder, strength_fill, truncate};

pub(super) fn show_coverage(app: &App, ui: &mut egui::Ui) {
    let cs = &app.customer_segment_page;
    let vp = &app.valueprop_page;

    let mut pains: Vec<(Uuid, String, f32)> = cs
        .pains_state
        .pains
        .iter()
        .map(|p| {
            (
                p.id,
                truncate(display_name(&p.name, "Unnamed pain"), 22),
                p.importance,
            )
        })
        .collect();
    pains.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let mut gains: Vec<(Uuid, String, f32)> = cs
        .gains_state
        .gains
        .iter()
        .map(|g| {
            (
                g.id,
                truncate(display_name(&g.name, "Unnamed gain"), 22),
                g.importance,
            )
        })
        .collect();
    gains.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let pain_reliefs: Vec<(Uuid, String)> = vp
        .pain_relief_state
        .pain_reliefs
        .iter()
        .map(|pr| {
            (
                pr.id,
                truncate(display_name(&pr.name, "Unnamed pain relief"), 14),
            )
        })
        .collect();

    let gain_creators: Vec<(Uuid, String)> = vp
        .gain_creator_state
        .gain_creators
        .iter()
        .map(|gc| {
            (
                gc.id,
                truncate(display_name(&gc.name, "Unnamed gain creator"), 14),
            )
        })
        .collect();

    ui.label(
        egui::RichText::new(
            "Cells: amber = weak coverage, green = strong coverage, ★ = Table Stake",
        )
        .small()
        .color(ui.visuals().weak_text_color()),
    );
    ui.add_space(8.0);

    ui.label(egui::RichText::new("Pains \u{00d7} Pain Reliefs").strong());
    ui.separator();
    if pains.is_empty() || pain_reliefs.is_empty() {
        show_placeholder(
            ui,
            "Add pains and pain reliefs with annotations to see this matrix.",
        );
    } else {
        show_heatmap(
            ui,
            "rel_pain_hm",
            &pains,
            &pain_reliefs,
            &vp.pain_relief_annotations,
        );
    }

    ui.add_space(16.0);
    ui.label(egui::RichText::new("Gains \u{00d7} Gain Creators").strong());
    ui.separator();
    if gains.is_empty() || gain_creators.is_empty() {
        show_placeholder(
            ui,
            "Add gains and gain creators with annotations to see this matrix.",
        );
    } else {
        show_heatmap(
            ui,
            "rel_gain_hm",
            &gains,
            &gain_creators,
            &vp.gain_creator_annotations,
        );
    }
}

fn show_heatmap(
    ui: &mut egui::Ui,
    id_salt: &str,
    needs: &[(Uuid, String, f32)],
    solutions: &[(Uuid, String)],
    annotations: &[ValueAnnotation],
) {
    let n_sol = solutions.len();
    egui::ScrollArea::horizontal()
        .id_salt(id_salt)
        .show(ui, |ui| {
            egui::Grid::new(id_salt)
                .striped(false)
                .spacing([2.0, 2.0])
                .show(ui, |ui| {
                    // Header row
                    ui.label(""); // importance col
                    ui.label(""); // need name col
                    for (_, sol_name) in solutions {
                        ui.label(
                            egui::RichText::new(sol_name)
                                .small()
                                .strong()
                                .color(ui.visuals().strong_text_color()),
                        );
                    }
                    ui.end_row();

                    // Data rows
                    for (need_id, need_name, importance) in needs {
                        // Importance bar
                        ui.add(
                            egui::ProgressBar::new(*importance)
                                .desired_width(36.0)
                                .fill(egui::Color32::from_rgba_unmultiplied(100, 120, 200, 120)),
                        );
                        // Need label
                        ui.label(egui::RichText::new(need_name).small());

                        for i in 0..n_sol {
                            let Some((sol_id, _)) = solutions.get(i) else {
                                continue;
                            };
                            let ann = annotations.iter().find(|a| {
                                a.pain_or_gain_id == *need_id && a.reliever_or_creator_id == *sol_id
                            });
                            let cell_fill = ann
                                .map_or(ui.visuals().faint_bg_color, |a| strength_fill(a.strength));
                            let cell_resp = egui::Frame::new()
                                .fill(cell_fill)
                                .inner_margin(4.0)
                                .show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(54.0, 20.0));
                                    if let Some(a) = ann {
                                        let ts = if a.value_type == ValueType::TableStake {
                                            " \u{2605}"
                                        } else {
                                            ""
                                        };
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{:.0}%{ts}",
                                                a.strength * 100.0
                                            ))
                                            .small(),
                                        );
                                    } else {
                                        ui.label(
                                            egui::RichText::new("\u{2014}")
                                                .small()
                                                .color(ui.visuals().weak_text_color()),
                                        );
                                    }
                                });
                            if let Some(a) = ann {
                                cell_resp.response.on_hover_ui(|ui| {
                                    ui.label(need_name.as_str());
                                    ui.label(format!(
                                        "Strength: {:.0}%   Type: {}",
                                        a.strength * 100.0,
                                        a.value_type.label()
                                    ));
                                });
                            }
                        }
                        ui.end_row();
                    }
                });
        });
}

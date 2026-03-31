// ── Tab 4: Stories ────────────────────────────────────────────────────────────
//
// Scrollable cards, one per job. Each card shows linked needs (pains and gains)
// paired with their best solution and strength. Jobs are sorted by coverage
// gap — least covered first — so the biggest problems surface immediately.

use crate::app::App;
use crate::app::pages::accordion::{TABLE_STAKE_MET, TABLE_STAKE_UNMET, display_name};
use crate::app::pages::product::{ValueAnnotation, ValueType};
use crate::app::pages::value_analytics::TABLE_STAKE_MIN_STRENGTH;
use eframe::egui;
use uuid::Uuid;

use super::{GAIN_RGB, PAIN_RGB, show_placeholder, truncate};

struct NeedRow {
    name: String,
    importance: f32,
    is_pain: bool,
    sol_name: Option<String>,
    strength: Option<f32>,
    value_type: Option<ValueType>,
}

struct Card {
    job_name: String,
    needs: Vec<NeedRow>,
}

/// Finds the highest-strength annotation targeting `need_id` and returns
/// `(solution_name, strength, value_type)`.
///
/// `solutions` is a pre-built `(id, display_name)` slice — collect it once per
/// job rather than re-scanning the entity list for every need.
fn best_coverage(
    need_id: Uuid,
    annotations: &[ValueAnnotation],
    solutions: &[(Uuid, String)],
) -> (Option<String>, Option<f32>, Option<ValueType>) {
    let best = annotations
        .iter()
        .filter(|ann| ann.pain_or_gain_id == need_id)
        .max_by(|a, b| {
            a.strength
                .partial_cmp(&b.strength)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    best.map_or((None, None, None), |ann| {
        let name = solutions
            .iter()
            .find(|(id, _)| *id == ann.reliever_or_creator_id)
            .map(|(_, n)| n.clone());
        (name, Some(ann.strength), Some(ann.value_type))
    })
}

#[expect(clippy::too_many_lines)]
pub(super) fn show_stories(app: &App, ui: &mut egui::Ui) {
    let cs = &app.customer_segment_page;
    let vp = &app.valueprop_page;

    if cs.jobs_state.jobs.is_empty() {
        show_placeholder(ui, "No jobs defined yet.");
        return;
    }

    // Pre-build solution name lists once — reused for every pain/gain lookup.
    let pain_relief_names: Vec<(Uuid, String)> = vp
        .pain_relief_state
        .pain_reliefs
        .iter()
        .map(|pr| (pr.id, truncate(display_name(&pr.name, "Pain relief"), 24)))
        .collect();
    let gain_creator_names: Vec<(Uuid, String)> = vp
        .gain_creator_state
        .gain_creators
        .iter()
        .map(|gc| (gc.id, truncate(display_name(&gc.name, "Gain creator"), 24)))
        .collect();

    let mut cards: Vec<Card> = cs
        .jobs_state
        .jobs
        .iter()
        .map(|job| {
            let mut needs: Vec<NeedRow> = Vec::new();

            for &(pid, jid) in &cs.job_pain_links {
                if jid != job.id {
                    continue;
                }
                let Some(pain) = cs.pains_state.pains.iter().find(|p| p.id == pid) else {
                    continue;
                };
                let (sol_name, strength, value_type) =
                    best_coverage(pid, &vp.pain_relief_annotations, &pain_relief_names);
                needs.push(NeedRow {
                    name: truncate(display_name(&pain.name, "Pain"), 28),
                    importance: pain.importance,
                    is_pain: true,
                    sol_name,
                    strength,
                    value_type,
                });
            }

            for &(gid, jid) in &cs.job_gain_links {
                if jid != job.id {
                    continue;
                }
                let Some(gain) = cs.gains_state.gains.iter().find(|g| g.id == gid) else {
                    continue;
                };
                let (sol_name, strength, value_type) =
                    best_coverage(gid, &vp.gain_creator_annotations, &gain_creator_names);
                needs.push(NeedRow {
                    name: truncate(display_name(&gain.name, "Gain"), 28),
                    importance: gain.importance,
                    is_pain: false,
                    sol_name,
                    strength,
                    value_type,
                });
            }

            // Sort needs: pains before gains, then by importance descending.
            needs.sort_by(|a, b| {
                b.is_pain.cmp(&a.is_pain).then_with(|| {
                    b.importance
                        .partial_cmp(&a.importance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            });

            Card {
                job_name: display_name(&job.name, "Unnamed job").to_owned(),
                needs,
            }
        })
        .collect();

    // Sort cards: fewest addressed first
    cards.sort_by_key(|c| c.needs.iter().filter(|n| n.sol_name.is_some()).count());

    for card in &cards {
        let addressed = card.needs.iter().filter(|n| n.sol_name.is_some()).count();
        let total = card.needs.len();
        let pct = if total > 0 {
            addressed as f32 / total as f32
        } else {
            1.0
        };
        let badge_color = if pct >= 0.8 {
            TABLE_STAKE_MET
        } else if pct >= 0.4 {
            egui::Color32::from_rgb(200, 140, 40)
        } else {
            TABLE_STAKE_UNMET
        };

        egui::Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .stroke(egui::Stroke::new(
                1.0,
                ui.visuals().widgets.noninteractive.bg_stroke.color,
            ))
            .inner_margin(10.0)
            .show(ui, |ui| {
                // Card header
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&card.job_name).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let badge_text = if total == 0 {
                            "No needs".to_owned()
                        } else {
                            format!("{addressed}/{total} addressed")
                        };
                        ui.label(egui::RichText::new(badge_text).small().color(badge_color));
                    });
                });

                if card.needs.is_empty() {
                    ui.label(
                        egui::RichText::new("No pains or gains linked to this job.")
                            .italics()
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    );
                    return;
                }

                ui.add_space(4.0);

                for row in &card.needs {
                    let need_rgb = if row.is_pain { PAIN_RGB } else { GAIN_RGB };
                    let is_ts_unmet = row.value_type == Some(ValueType::TableStake)
                        && row.strength.is_none_or(|s| s < TABLE_STAKE_MIN_STRENGTH);

                    ui.horizontal(|ui| {
                        // Colour dot
                        let (dot_r, _) =
                            ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                        ui.painter().circle_filled(
                            dot_r.center(),
                            4.0,
                            egui::Color32::from_rgb(need_rgb[0], need_rgb[1], need_rgb[2]),
                        );
                        // Need name
                        ui.label(egui::RichText::new(&row.name).small());
                        // Importance label
                        ui.label(
                            egui::RichText::new(format!("{:.0}%", row.importance * 100.0))
                                .small()
                                .color(ui.visuals().weak_text_color()),
                        );
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| match (&row.sol_name, row.strength) {
                                (Some(sol), Some(s)) => {
                                    let str_color = if s >= 0.7 {
                                        TABLE_STAKE_MET
                                    } else if s >= 0.5 {
                                        egui::Color32::from_rgb(200, 140, 40)
                                    } else {
                                        TABLE_STAKE_UNMET
                                    };
                                    if row.value_type == Some(ValueType::TableStake) {
                                        let ts_color = if is_ts_unmet {
                                            TABLE_STAKE_UNMET
                                        } else {
                                            TABLE_STAKE_MET
                                        };
                                        ui.label(
                                            egui::RichText::new("\u{2605}TS")
                                                .small()
                                                .color(ts_color),
                                        );
                                    }
                                    ui.label(
                                        egui::RichText::new(format!("{:.0}%", s * 100.0))
                                            .small()
                                            .color(str_color),
                                    );
                                    ui.label(egui::RichText::new(sol.as_str()).small());
                                    ui.label(
                                        egui::RichText::new("\u{2192}")
                                            .small()
                                            .color(ui.visuals().weak_text_color()),
                                    );
                                }
                                _ => {
                                    ui.label(
                                        egui::RichText::new("\u{2014} no solution")
                                            .small()
                                            .italics()
                                            .color(egui::Color32::from_rgb(200, 140, 40)),
                                    );
                                }
                            },
                        );
                    });
                }
            });

        ui.add_space(6.0);
    }
}

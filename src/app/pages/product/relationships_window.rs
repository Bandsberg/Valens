//! Four relationship visualisation modes for the Entity Relationships window.
//!
//! Toggle from the Value Proposition side panel → "Entity Relationships".

use crate::app::App;
use crate::app::pages::accordion::{TABLE_STAKE_MET, TABLE_STAKE_UNMET, display_name};
use crate::app::pages::product::ValueType;
use crate::app::pages::value_analytics::TABLE_STAKE_MIN_STRENGTH;
use eframe::egui;
use std::collections::HashSet;
use uuid::Uuid;

// ── Tab ───────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Default)]
enum RelTab {
    #[default]
    Flow,
    Coverage,
    Web,
    Stories,
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn show_relationships_window(app: &App, ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Entity Relationships")
        .open(open)
        .default_size([720.0, 560.0])
        .resizable(true)
        .show(ctx, |ui| {
            let tab_key = egui::Id::new("rel_active_tab");
            let mut tab: RelTab = ctx.data(|d| d.get_temp(tab_key).unwrap_or_default());
            ui.horizontal(|ui| {
                ui.selectable_value(&mut tab, RelTab::Flow, "Flow");
                ui.selectable_value(&mut tab, RelTab::Coverage, "Coverage");
                ui.selectable_value(&mut tab, RelTab::Web, "Web");
                ui.selectable_value(&mut tab, RelTab::Stories, "Stories");
            });
            ctx.data_mut(|d| d.insert_temp(tab_key, tab));
            ui.separator();
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| match tab {
                    RelTab::Flow => show_flow(app, ui),
                    RelTab::Coverage => show_coverage(app, ui),
                    RelTab::Web => show_web(app, ctx, ui),
                    RelTab::Stories => show_stories(app, ui),
                });
        });
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn show_placeholder(ui: &mut egui::Ui, msg: &str) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new(msg)
            .italics()
            .color(ui.visuals().weak_text_color()),
    );
}

fn truncate(name: &str, max: usize) -> String {
    if name.chars().count() > max {
        let s: String = name.chars().take(max.saturating_sub(1)).collect();
        format!("{s}\u{2026}")
    } else {
        name.to_owned()
    }
}

/// Map annotation strength (0–1) to a fill colour: amber at 0, green at 1.
fn strength_fill(strength: f32) -> egui::Color32 {
    let s = strength.clamp(0.0, 1.0);
    egui::Color32::from_rgba_unmultiplied(
        (220.0 - s * 160.0) as u8,
        (100.0 + s * 80.0) as u8,
        30_u8,
        (50.0 + s * 170.0) as u8,
    )
}

/// Draw a smooth horizontal S-curve bezier from `start` to `end`.
fn draw_bezier(painter: &egui::Painter, start: egui::Pos2, end: egui::Pos2, stroke: egui::Stroke) {
    let dx = (end.x - start.x) * 0.4;
    let p1 = egui::pos2(start.x + dx, start.y);
    let p2 = egui::pos2(end.x - dx, end.y);
    const SEGS: usize = 16;
    let mut prev = start;
    for i in 1..=SEGS {
        let t = i as f32 / SEGS as f32;
        let mt = 1.0 - t;
        let next = egui::pos2(
            mt * mt * mt * start.x
                + 3.0 * mt * mt * t * p1.x
                + 3.0 * mt * t * t * p2.x
                + t * t * t * end.x,
            mt * mt * mt * start.y
                + 3.0 * mt * mt * t * p1.y
                + 3.0 * mt * t * t * p2.y
                + t * t * t * end.y,
        );
        painter.line_segment([prev, next], stroke);
        prev = next;
    }
}

// ── Tab 1: Flow ───────────────────────────────────────────────────────────────
//
// Three columns: Jobs → Pains & Gains → Pain Reliefs & Gain Creators.
// Bezier curves connect linked entities; curve opacity encodes annotation
// strength. Hovering an entity dims everything not connected to it.

#[expect(clippy::too_many_lines)]
fn show_flow(app: &App, ui: &mut egui::Ui) {
    let cs = &app.customer_segment_page;
    let vp = &app.valueprop_page;

    // ── Collect column items ──────────────────────────────────────────────────
    // (id, display_name, pill_rgb)
    let jobs: Vec<(Uuid, String, [u8; 3])> = cs
        .jobs_state
        .jobs
        .iter()
        .map(|j| {
            (
                j.id,
                truncate(display_name(&j.name, "Unnamed job"), 18),
                [140, 90, 210],
            )
        })
        .collect();

    let mut needs: Vec<(Uuid, String, [u8; 3], bool)> = Vec::new(); // is_pain flag
    for p in &cs.pains_state.pains {
        needs.push((
            p.id,
            truncate(display_name(&p.name, "Unnamed pain"), 18),
            [200, 80, 80],
            true,
        ));
    }
    for g in &cs.gains_state.gains {
        needs.push((
            g.id,
            truncate(display_name(&g.name, "Unnamed gain"), 18),
            [60, 175, 100],
            false,
        ));
    }

    let mut solutions: Vec<(Uuid, String, [u8; 3], bool)> = Vec::new(); // is_pain_relief flag
    for pr in &vp.pain_relief_state.pain_reliefs {
        solutions.push((
            pr.id,
            truncate(display_name(&pr.name, "Unnamed pain relief"), 18),
            [185, 100, 100],
            true,
        ));
    }
    for gc in &vp.gain_creator_state.gain_creators {
        solutions.push((
            gc.id,
            truncate(display_name(&gc.name, "Unnamed gain creator"), 18),
            [60, 155, 100],
            false,
        ));
    }

    if jobs.is_empty() && needs.is_empty() && solutions.is_empty() {
        show_placeholder(
            ui,
            "No entities yet. Add jobs, pains, gains, pain reliefs, or gain creators.",
        );
        return;
    }

    // ── Canvas setup ─────────────────────────────────────────────────────────
    let row_h = 30.0_f32;
    let pad_v = 26.0_f32;
    let max_rows = jobs.len().max(needs.len()).max(solutions.len()).max(1);
    let canvas_h = (max_rows as f32 * row_h + pad_v * 2.0).max(160.0);
    let avail_w = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(avail_w, canvas_h), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, ui.visuals().faint_bg_color);

    let col_x = [
        rect.min.x + avail_w * 0.17,
        rect.min.x + avail_w * 0.50,
        rect.min.x + avail_w * 0.83,
    ];
    let pill_w = avail_w * 0.27;
    let pill_hw = pill_w / 2.0;
    let pill_h = 22.0_f32;

    // Column headers
    let weak = ui.visuals().weak_text_color();
    let hdr_font = egui::FontId::proportional(11.0);
    for (label, &x) in ["Jobs", "Pains & Gains", "Solutions"]
        .iter()
        .zip(col_x.iter())
    {
        painter.text(
            egui::pos2(x, rect.min.y + 8.0),
            egui::Align2::CENTER_TOP,
            label,
            hdr_font.clone(),
            weak,
        );
    }

    // ── Compute pill centers ──────────────────────────────────────────────────
    let y_pos = |i: usize| rect.min.y + pad_v + (i as f32 + 0.5) * row_h;

    let col0: Vec<(Uuid, egui::Pos2)> = jobs
        .iter()
        .enumerate()
        .map(|(i, (id, _, _))| (*id, egui::pos2(col_x[0], y_pos(i))))
        .collect();
    let col1: Vec<(Uuid, egui::Pos2)> = needs
        .iter()
        .enumerate()
        .map(|(i, (id, _, _, _))| (*id, egui::pos2(col_x[1], y_pos(i))))
        .collect();
    let col2: Vec<(Uuid, egui::Pos2)> = solutions
        .iter()
        .enumerate()
        .map(|(i, (id, _, _, _))| (*id, egui::pos2(col_x[2], y_pos(i))))
        .collect();

    // ── Hover & highlight ─────────────────────────────────────────────────────
    let hover_pos = response.hover_pos();
    let in_pill = |center: egui::Pos2, hp: egui::Pos2| -> bool {
        (hp.x - center.x).abs() <= pill_hw && (hp.y - center.y).abs() <= pill_h / 2.0
    };

    let hovered_id: Option<Uuid> = hover_pos.and_then(|hp| {
        col0.iter()
            .find(|(_, c)| in_pill(*c, hp))
            .or_else(|| col1.iter().find(|(_, c)| in_pill(*c, hp)))
            .or_else(|| col2.iter().find(|(_, c)| in_pill(*c, hp)))
            .map(|(id, _)| *id)
    });

    let connected: HashSet<Uuid> = build_flow_connected(hovered_id, app);

    let alpha_for = |id: Uuid| -> u8 {
        if hovered_id.is_none() || hovered_id == Some(id) || connected.contains(&id) {
            220_u8
        } else {
            40_u8
        }
    };

    // ── Draw curves col0 → col1 ───────────────────────────────────────────────
    for &(jid, jcenter) in &col0 {
        let ja = alpha_for(jid);
        for (ni, &(nid, ncenter)) in col1.iter().enumerate() {
            let is_pain = needs.get(ni).is_some_and(|(_, _, _, p)| *p);
            let linked = if is_pain {
                cs.job_pain_links
                    .iter()
                    .any(|&(pid, jid2)| pid == nid && jid2 == jid)
            } else {
                cs.job_gain_links
                    .iter()
                    .any(|&(gid, jid2)| gid == nid && jid2 == jid)
            };
            if !linked {
                continue;
            }
            let a = ja.min(alpha_for(nid));
            let [r, g, b, _] = if is_pain {
                [200_u8, 80, 80, 0]
            } else {
                [60, 175, 100, 0]
            };
            let stroke = egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(r, g, b, a));
            draw_bezier(
                &painter,
                egui::pos2(jcenter.x + pill_hw, jcenter.y),
                egui::pos2(ncenter.x - pill_hw, ncenter.y),
                stroke,
            );
        }
    }

    // ── Draw curves col1 → col2 ───────────────────────────────────────────────
    for (ni, &(nid, ncenter)) in col1.iter().enumerate() {
        let is_pain = needs.get(ni).is_some_and(|(_, _, _, p)| *p);
        let anns: Vec<(Uuid, f32)> = if is_pain {
            vp.pain_relief_annotations
                .iter()
                .filter(|ann| ann.pain_or_gain_id == nid)
                .map(|ann| (ann.reliever_or_creator_id, ann.strength))
                .collect()
        } else {
            vp.gain_creator_annotations
                .iter()
                .filter(|ann| ann.pain_or_gain_id == nid)
                .map(|ann| (ann.reliever_or_creator_id, ann.strength))
                .collect()
        };
        for (sid, strength) in anns {
            let Some(&(_, scenter)) = col2.iter().find(|(id, _)| *id == sid) else {
                continue;
            };
            let base_a = alpha_for(nid).min(alpha_for(sid));
            let a = ((f32::from(base_a)) * (strength * 0.7 + 0.3)) as u8;
            let [r, g, b, _] = if is_pain {
                [200_u8, 80, 80, 0]
            } else {
                [60, 175, 100, 0]
            };
            let width = 1.0 + strength * 2.5;
            let stroke =
                egui::Stroke::new(width, egui::Color32::from_rgba_unmultiplied(r, g, b, a));
            draw_bezier(
                &painter,
                egui::pos2(ncenter.x + pill_hw, ncenter.y),
                egui::pos2(scenter.x - pill_hw, scenter.y),
                stroke,
            );
        }
    }

    // ── Draw pills ────────────────────────────────────────────────────────────
    let pill_font = egui::FontId::proportional(11.0);
    let text_base = ui.visuals().text_color();
    let mut tooltip: Option<String> = None;

    let draw_pills = |items: &[(Uuid, String, [u8; 3])], centers: &[(Uuid, egui::Pos2)]| {
        for (i, (id, name, rgb)) in items.iter().enumerate() {
            let Some(&(_, center)) = centers.get(i) else {
                continue;
            };
            let a = alpha_for(*id);
            let fill = egui::Color32::from_rgba_unmultiplied(rgb[0], rgb[1], rgb[2], a / 2);
            let pill_rect = egui::Rect::from_center_size(center, egui::vec2(pill_w, pill_h));
            painter.rect_filled(pill_rect, 5.0, fill);
            let ta = a;
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                name.as_str(),
                pill_font.clone(),
                egui::Color32::from_rgba_unmultiplied(
                    text_base.r(),
                    text_base.g(),
                    text_base.b(),
                    ta,
                ),
            );
        }
    };

    // col0 (jobs)
    let jobs_simple: Vec<(Uuid, String, [u8; 3])> = jobs
        .iter()
        .map(|(id, n, rgb)| (*id, n.clone(), *rgb))
        .collect();
    draw_pills(&jobs_simple, &col0);

    // col1 (needs)
    let needs_simple: Vec<(Uuid, String, [u8; 3])> = needs
        .iter()
        .map(|(id, n, rgb, _)| (*id, n.clone(), *rgb))
        .collect();
    draw_pills(&needs_simple, &col1);

    // col2 (solutions)
    let sols_simple: Vec<(Uuid, String, [u8; 3])> = solutions
        .iter()
        .map(|(id, n, rgb, _)| (*id, n.clone(), *rgb))
        .collect();
    draw_pills(&sols_simple, &col2);

    // Hover tooltip
    if let Some(hid) = hovered_id {
        let full_name = jobs
            .iter()
            .find(|(id, _, _)| *id == hid)
            .map(|(_, n, _)| n.as_str())
            .or_else(|| {
                needs
                    .iter()
                    .find(|(id, _, _, _)| *id == hid)
                    .map(|(_, n, _, _)| n.as_str())
            })
            .or_else(|| {
                solutions
                    .iter()
                    .find(|(id, _, _, _)| *id == hid)
                    .map(|(_, n, _, _)| n.as_str())
            });
        if let Some(name) = full_name {
            tooltip = Some(name.to_owned());
        }
    }
    if let Some(text) = tooltip {
        response.on_hover_text(text);
    }

    // ── Legend ────────────────────────────────────────────────────────────────
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        for ([r, g, b], label) in [
            ([140_u8, 90, 210], "Job"),
            ([200, 80, 80], "Pain / Pain Relief"),
            ([60, 175, 100], "Gain / Gain Creator"),
        ] {
            let (dot_rect, _) =
                ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
            ui.painter()
                .circle_filled(dot_rect.center(), 5.0, egui::Color32::from_rgb(r, g, b));
            ui.label(egui::RichText::new(label).small());
            ui.add_space(8.0);
        }
    });
}

/// Compute the set of entity IDs connected to `hovered` across all flow links.
fn build_flow_connected(hovered: Option<Uuid>, app: &App) -> HashSet<Uuid> {
    let Some(hid) = hovered else {
        return HashSet::new();
    };
    let cs = &app.customer_segment_page;
    let vp = &app.valueprop_page;
    let mut set = HashSet::new();

    // hid is a Job → connected needs
    let pain_ids_of_job: Vec<Uuid> = cs
        .job_pain_links
        .iter()
        .filter_map(|&(pid, jid)| (jid == hid).then_some(pid))
        .collect();
    let gain_ids_of_job: Vec<Uuid> = cs
        .job_gain_links
        .iter()
        .filter_map(|&(gid, jid)| (jid == hid).then_some(gid))
        .collect();
    set.extend(pain_ids_of_job.iter().copied());
    set.extend(gain_ids_of_job.iter().copied());
    // transitively: needs → solutions
    for &pid in &pain_ids_of_job {
        for ann in &vp.pain_relief_annotations {
            if ann.pain_or_gain_id == pid {
                set.insert(ann.reliever_or_creator_id);
            }
        }
    }
    for &gid in &gain_ids_of_job {
        for ann in &vp.gain_creator_annotations {
            if ann.pain_or_gain_id == gid {
                set.insert(ann.reliever_or_creator_id);
            }
        }
    }

    // hid is a Need → connected jobs and solutions
    for &(pid, jid) in &cs.job_pain_links {
        if pid == hid {
            set.insert(jid);
        }
    }
    for &(gid, jid) in &cs.job_gain_links {
        if gid == hid {
            set.insert(jid);
        }
    }
    for ann in &vp.pain_relief_annotations {
        if ann.pain_or_gain_id == hid {
            set.insert(ann.reliever_or_creator_id);
        }
    }
    for ann in &vp.gain_creator_annotations {
        if ann.pain_or_gain_id == hid {
            set.insert(ann.reliever_or_creator_id);
        }
    }

    // hid is a Solution → connected needs and their jobs
    for ann in &vp.pain_relief_annotations {
        if ann.reliever_or_creator_id == hid {
            set.insert(ann.pain_or_gain_id);
            for &(pid, jid) in &cs.job_pain_links {
                if pid == ann.pain_or_gain_id {
                    set.insert(jid);
                }
            }
        }
    }
    for ann in &vp.gain_creator_annotations {
        if ann.reliever_or_creator_id == hid {
            set.insert(ann.pain_or_gain_id);
            for &(gid, jid) in &cs.job_gain_links {
                if gid == ann.pain_or_gain_id {
                    set.insert(jid);
                }
            }
        }
    }

    set
}

// ── Tab 2: Coverage ───────────────────────────────────────────────────────────
//
// Two heatmap grids: Pains × Pain Reliefs and Gains × Gain Creators.
// Cells are colour-coded by annotation strength (amber → green).
// TableStake annotations are marked with a ★ badge.

fn show_coverage(app: &App, ui: &mut egui::Ui) {
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
    annotations: &[crate::app::pages::product::ValueAnnotation],
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

// ── Tab 3: Web ────────────────────────────────────────────────────────────────
//
// Radial spider diagram. Select a Job to focus on. Its linked pains and gains
// form an inner ring; the pain reliefs / gain creators addressing those needs
// form an outer ring. Line opacity encodes annotation strength.

/// An outer-ring node: a solution (pain relief or gain creator) with its
/// connections back to inner-ring needs.
type OuterNode = (Uuid, String, [u8; 3], Vec<(Uuid, f32, ValueType)>);

#[expect(clippy::too_many_lines)]
fn show_web(app: &App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let cs = &app.customer_segment_page;
    let vp = &app.valueprop_page;

    // ── Job selector ─────────────────────────────────────────────────────────
    let job_key = egui::Id::new("rel_web_job");
    let mut sel_job: Option<Uuid> = ctx.data(|d| d.get_temp::<Option<Uuid>>(job_key)).flatten();

    ui.horizontal(|ui| {
        let job_label = sel_job
            .and_then(|id| cs.jobs_state.jobs.iter().find(|j| j.id == id))
            .map_or("Select job\u{2026}", |j| j.name.as_str());
        egui::ComboBox::new(egui::Id::new("rel_web_combo"), "Focus job")
            .selected_text(job_label)
            .width(200.0)
            .show_ui(ui, |ui| {
                for j in &cs.jobs_state.jobs {
                    ui.selectable_value(&mut sel_job, Some(j.id), &j.name);
                }
            });
    });
    ctx.data_mut(|d| d.insert_temp(job_key, sel_job));

    let Some(job_id) = sel_job else {
        show_placeholder(ui, "Select a job to see its relationship web.");
        return;
    };

    // ── Build inner ring (needs) ──────────────────────────────────────────────
    // (id, name, color_rgb, is_pain)
    let mut inner: Vec<(Uuid, String, [u8; 3], bool)> = Vec::new();
    for &(pid, jid) in &cs.job_pain_links {
        if jid != job_id {
            continue;
        }
        if let Some(p) = cs.pains_state.pains.iter().find(|p| p.id == pid) {
            inner.push((
                pid,
                truncate(display_name(&p.name, "Pain"), 16),
                [200, 80, 80],
                true,
            ));
        }
    }
    for &(gid, jid) in &cs.job_gain_links {
        if jid != job_id {
            continue;
        }
        if let Some(g) = cs.gains_state.gains.iter().find(|g| g.id == gid) {
            inner.push((
                gid,
                truncate(display_name(&g.name, "Gain"), 16),
                [60, 175, 100],
                false,
            ));
        }
    }

    if inner.is_empty() {
        show_placeholder(ui, "No pains or gains linked to this job.");
        return;
    }

    // ── Build outer ring (solutions) ──────────────────────────────────────────
    // (id, name, color_rgb, connections: Vec<(inner_id, strength, value_type)>)
    let mut outer: Vec<OuterNode> = Vec::new();

    let inner_pain_ids: Vec<Uuid> = inner
        .iter()
        .filter(|(_, _, _, p)| *p)
        .map(|(id, _, _, _)| *id)
        .collect();
    let inner_gain_ids: Vec<Uuid> = inner
        .iter()
        .filter(|(_, _, _, p)| !*p)
        .map(|(id, _, _, _)| *id)
        .collect();

    for ann in &vp.pain_relief_annotations {
        if !inner_pain_ids.contains(&ann.pain_or_gain_id) {
            continue;
        }
        let pr_id = ann.reliever_or_creator_id;
        let Some(pr) = vp
            .pain_relief_state
            .pain_reliefs
            .iter()
            .find(|p| p.id == pr_id)
        else {
            continue;
        };
        if let Some(entry) = outer.iter_mut().find(|(id, _, _, _)| *id == pr_id) {
            entry
                .3
                .push((ann.pain_or_gain_id, ann.strength, ann.value_type));
        } else {
            outer.push((
                pr_id,
                truncate(display_name(&pr.name, "Pain Relief"), 16),
                [185, 100, 100],
                vec![(ann.pain_or_gain_id, ann.strength, ann.value_type)],
            ));
        }
    }
    for ann in &vp.gain_creator_annotations {
        if !inner_gain_ids.contains(&ann.pain_or_gain_id) {
            continue;
        }
        let gc_id = ann.reliever_or_creator_id;
        let Some(gc) = vp
            .gain_creator_state
            .gain_creators
            .iter()
            .find(|g| g.id == gc_id)
        else {
            continue;
        };
        if let Some(entry) = outer.iter_mut().find(|(id, _, _, _)| *id == gc_id) {
            entry
                .3
                .push((ann.pain_or_gain_id, ann.strength, ann.value_type));
        } else {
            outer.push((
                gc_id,
                truncate(display_name(&gc.name, "Gain Creator"), 16),
                [60, 155, 100],
                vec![(ann.pain_or_gain_id, ann.strength, ann.value_type)],
            ));
        }
    }

    // ── Canvas ────────────────────────────────────────────────────────────────
    let avail = ui.available_width().min(460.0);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(avail, avail * 0.9), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, ui.visuals().faint_bg_color);

    let center = rect.center();
    let inner_r = avail * 0.24;
    let outer_r = avail * 0.42;
    let n_inner = inner.len();
    let n_outer = outer.len();
    let pi2 = std::f32::consts::TAU;
    let half_pi = std::f32::consts::PI / 2.0;

    let ring_pos = |idx: usize, n: usize, radius: f32| -> egui::Pos2 {
        let angle = pi2 * idx as f32 / n as f32 - half_pi;
        egui::pos2(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        )
    };

    // Lines center → inner (uniform)
    for i in 0..n_inner {
        let pos = ring_pos(i, n_inner, inner_r);
        painter.line_segment(
            [center, pos],
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(150, 150, 150, 60),
            ),
        );
    }

    // Lines inner → outer (strength-encoded)
    for (oi, (_, _, _, conns)) in outer.iter().enumerate() {
        let opos = ring_pos(oi, n_outer, outer_r);
        for &(inner_id, strength, _) in conns {
            let Some(ii) = inner.iter().position(|(id, _, _, _)| *id == inner_id) else {
                continue;
            };
            let ipos = ring_pos(ii, n_inner, inner_r);
            let is_pain = inner.get(ii).is_some_and(|(_, _, _, p)| *p);
            let a = (strength * 0.8 + 0.2) * 200.0;
            let color = if is_pain {
                egui::Color32::from_rgba_unmultiplied(200, 80, 80, a as u8)
            } else {
                egui::Color32::from_rgba_unmultiplied(60, 175, 100, a as u8)
            };
            painter.line_segment([ipos, opos], egui::Stroke::new(2.0, color));
        }
    }

    // Hover detection
    const NODE_R: f32 = 7.0;
    const HOVER_R: f32 = 12.0;
    let hover_pos = response.hover_pos();
    let mut hovered_label: Option<String> = None;

    // Draw inner nodes
    for (i, (id, name, rgb, _)) in inner.iter().enumerate() {
        let pos = ring_pos(i, n_inner, inner_r);
        let is_hovered = hover_pos.is_some_and(|hp| pos.distance(hp) < HOVER_R);
        let r = if is_hovered { NODE_R + 2.0 } else { NODE_R };
        painter.circle_filled(pos, r, egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
        // Label outside the circle
        let label_pos = ring_pos(i, n_inner, inner_r + NODE_R + 10.0);
        let align = if label_pos.x < center.x - 5.0 {
            egui::Align2::RIGHT_CENTER
        } else if label_pos.x > center.x + 5.0 {
            egui::Align2::LEFT_CENTER
        } else {
            egui::Align2::CENTER_CENTER
        };
        painter.text(
            label_pos,
            align,
            name.as_str(),
            egui::FontId::proportional(10.0),
            ui.visuals().text_color(),
        );
        if is_hovered {
            hovered_label = Some(name.clone());
            // Highlight connected outer nodes
            for (oi, (_, _, _, conns)) in outer.iter().enumerate() {
                if conns.iter().any(|(iid, _, _)| *iid == *id) {
                    let opos = ring_pos(oi, n_outer, outer_r);
                    painter.circle_stroke(
                        opos,
                        NODE_R + 3.0,
                        egui::Stroke::new(
                            2.0,
                            egui::Color32::from_rgba_unmultiplied(255, 220, 50, 200),
                        ),
                    );
                }
            }
        }
    }

    // Draw outer nodes
    for (i, (_, name, rgb, conns)) in outer.iter().enumerate() {
        let pos = ring_pos(i, n_outer, outer_r);
        let is_hovered = hover_pos.is_some_and(|hp| pos.distance(hp) < HOVER_R);
        let r = if is_hovered { 8.0 } else { 6.0_f32 };
        painter.circle_filled(pos, r, egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
        let label_pos = ring_pos(i, n_outer, outer_r + r + 10.0);
        let align = if label_pos.x < center.x - 5.0 {
            egui::Align2::RIGHT_CENTER
        } else if label_pos.x > center.x + 5.0 {
            egui::Align2::LEFT_CENTER
        } else {
            egui::Align2::CENTER_CENTER
        };
        painter.text(
            label_pos,
            align,
            name.as_str(),
            egui::FontId::proportional(9.0),
            ui.visuals().text_color(),
        );
        if is_hovered {
            let best_str = conns.iter().map(|(_, s, _)| *s).fold(0.0_f32, f32::max);
            hovered_label = Some(format!("{name}\nStrength: {:.0}%", best_str * 100.0));
        }
    }

    // Draw center node (job)
    let job_name = cs
        .jobs_state
        .jobs
        .iter()
        .find(|j| j.id == job_id)
        .map(|j| truncate(display_name(&j.name, "Job"), 12))
        .unwrap_or_else(|| "Job".to_owned());
    painter.circle_filled(center, 18.0, egui::Color32::from_rgb(140, 90, 210));
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        job_name.as_str(),
        egui::FontId::proportional(10.0),
        egui::Color32::WHITE,
    );
    if hover_pos.is_some_and(|hp| center.distance(hp) < 20.0) {
        hovered_label = Some(job_name.clone());
    }

    if let Some(label) = hovered_label {
        response.on_hover_text(label);
    }

    // ── Legend ────────────────────────────────────────────────────────────────
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        for ([r, g, b], label) in [
            ([140_u8, 90, 210], "Job (centre)"),
            ([200, 80, 80], "Pain / Pain Relief"),
            ([60, 175, 100], "Gain / Gain Creator"),
        ] {
            let (dot_rect, _) =
                ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
            ui.painter()
                .circle_filled(dot_rect.center(), 5.0, egui::Color32::from_rgb(r, g, b));
            ui.label(egui::RichText::new(label).small());
            ui.add_space(8.0);
        }
        ui.label(
            egui::RichText::new("Line opacity = annotation strength")
                .small()
                .color(ui.visuals().weak_text_color()),
        );
    });
}

// ── Tab 4: Stories ────────────────────────────────────────────────────────────
//
// Scrollable cards, one per job. Each card shows linked needs (pains and gains)
// paired with their best solution and strength. Jobs are sorted by coverage
// gap — least covered first — so the biggest problems surface immediately.

#[expect(clippy::too_many_lines)]
fn show_stories(app: &App, ui: &mut egui::Ui) {
    let cs = &app.customer_segment_page;
    let vp = &app.valueprop_page;

    if cs.jobs_state.jobs.is_empty() {
        show_placeholder(ui, "No jobs defined yet.");
        return;
    }

    // Build a card per job
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
                let best = vp
                    .pain_relief_annotations
                    .iter()
                    .filter(|ann| ann.pain_or_gain_id == pid)
                    .max_by(|a, b| {
                        a.strength
                            .partial_cmp(&b.strength)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                let (sol_name, strength, value_type) = best.map_or((None, None, None), |ann| {
                    let name = vp
                        .pain_relief_state
                        .pain_reliefs
                        .iter()
                        .find(|pr| pr.id == ann.reliever_or_creator_id)
                        .map(|pr| truncate(display_name(&pr.name, "Pain relief"), 24));
                    (name, Some(ann.strength), Some(ann.value_type))
                });
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
                let best = vp
                    .gain_creator_annotations
                    .iter()
                    .filter(|ann| ann.pain_or_gain_id == gid)
                    .max_by(|a, b| {
                        a.strength
                            .partial_cmp(&b.strength)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                let (sol_name, strength, value_type) = best.map_or((None, None, None), |ann| {
                    let name = vp
                        .gain_creator_state
                        .gain_creators
                        .iter()
                        .find(|gc| gc.id == ann.reliever_or_creator_id)
                        .map(|gc| truncate(display_name(&gc.name, "Gain creator"), 24));
                    (name, Some(ann.strength), Some(ann.value_type))
                });
                needs.push(NeedRow {
                    name: truncate(display_name(&gain.name, "Gain"), 28),
                    importance: gain.importance,
                    is_pain: false,
                    sol_name,
                    strength,
                    value_type,
                });
            }

            // Sort needs: pains first, then by importance desc
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
                    let need_rgb = if row.is_pain {
                        [200_u8, 80, 80]
                    } else {
                        [60, 175, 100]
                    };
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

// ── Tab 1: Flow ───────────────────────────────────────────────────────────────
//
// Three columns: Jobs → Pains & Gains → Pain Reliefs & Gain Creators.
// Bezier curves connect linked entities; curve opacity encodes annotation
// strength. Hovering an entity dims everything not connected to it.

use crate::app::App;
use crate::app::pages::accordion::display_name;
use crate::app::pages::product::ValuePropPage;
use crate::app::pages::segments::CustomerSegmentPage;
use eframe::egui;
use std::collections::HashSet;
use uuid::Uuid;

use super::{
    FLOW_COL_X_FRACS, FLOW_PAD_V, FLOW_PILL_H, FLOW_PILL_W_FRAC, FLOW_ROW_H, GAIN_CREATOR_RGB,
    GAIN_RGB, JOB_RGB, PAIN_RELIEF_RGB, PAIN_RGB, draw_bezier, show_placeholder, truncate,
};

#[expect(clippy::too_many_lines)]
pub(super) fn show_flow(app: &App, ui: &mut egui::Ui) {
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

    // is_pain flag distinguishes red (pain) from green (gain) when drawing beziers.
    let mut needs: Vec<(Uuid, String, [u8; 3], bool)> = Vec::new();
    for p in &cs.pains_state.pains {
        needs.push((
            p.id,
            truncate(display_name(&p.name, "Unnamed pain"), 18),
            PAIN_RGB,
            true,
        ));
    }
    for g in &cs.gains_state.gains {
        needs.push((
            g.id,
            truncate(display_name(&g.name, "Unnamed gain"), 18),
            GAIN_RGB,
            false,
        ));
    }

    let mut solutions: Vec<(Uuid, String, [u8; 3])> = Vec::new();
    for pr in &vp.pain_relief_state.pain_reliefs {
        solutions.push((
            pr.id,
            truncate(display_name(&pr.name, "Unnamed pain relief"), 18),
            PAIN_RELIEF_RGB,
        ));
    }
    for gc in &vp.gain_creator_state.gain_creators {
        solutions.push((
            gc.id,
            truncate(display_name(&gc.name, "Unnamed gain creator"), 18),
            GAIN_CREATOR_RGB,
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
    let max_rows = jobs.len().max(needs.len()).max(solutions.len()).max(1);
    let canvas_h = (max_rows as f32 * FLOW_ROW_H + FLOW_PAD_V * 2.0).max(160.0);
    let avail_w = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(avail_w, canvas_h), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, ui.visuals().faint_bg_color);

    let col_x = FLOW_COL_X_FRACS.map(|frac| rect.min.x + avail_w * frac);
    let pill_w = avail_w * FLOW_PILL_W_FRAC;
    let pill_hw = pill_w / 2.0;
    let pill_h = FLOW_PILL_H;

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
    let y_pos = |i: usize| rect.min.y + FLOW_PAD_V + (i as f32 + 0.5) * FLOW_ROW_H;

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
        .map(|(i, (id, _, _))| (*id, egui::pos2(col_x[2], y_pos(i))))
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

    let connected = build_flow_connected(hovered_id, app);

    // Returns the draw alpha (0–255) for an entity based on hover state.
    let alpha_for = |id: Uuid| -> u8 {
        if hovered_id.is_none() || hovered_id == Some(id) || connected.contains(&id) {
            220_u8
        } else {
            40_u8
        }
    };

    // ── Draw curves ───────────────────────────────────────────────────────────
    draw_flow_curves(
        &painter, &col0, &col1, &col2, &needs, cs, vp, pill_hw, &alpha_for,
    );

    // ── Draw pills ────────────────────────────────────────────────────────────
    let pill_font = egui::FontId::proportional(11.0);
    let text_base = ui.visuals().text_color();

    draw_flow_pills(
        &painter, &jobs, &col0, pill_w, pill_h, &pill_font, text_base, &alpha_for,
    );

    // needs carries an extra bool (is_pain) used for bezier colours above, so
    // we strip it before passing to the generic draw_pills helper.
    let needs_stripped: Vec<(Uuid, String, [u8; 3])> = needs
        .iter()
        .map(|(id, n, rgb, _)| (*id, n.clone(), *rgb))
        .collect();
    draw_flow_pills(
        &painter,
        &needs_stripped,
        &col1,
        pill_w,
        pill_h,
        &pill_font,
        text_base,
        &alpha_for,
    );

    draw_flow_pills(
        &painter, &solutions, &col2, pill_w, pill_h, &pill_font, text_base, &alpha_for,
    );

    // ── Hover tooltip ─────────────────────────────────────────────────────────
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
                    .find(|(id, _, _)| *id == hid)
                    .map(|(_, n, _)| n.as_str())
            });
        if let Some(name) = full_name {
            response.on_hover_text(name);
        }
    }

    // ── Legend ────────────────────────────────────────────────────────────────
    show_flow_legend(ui);
}

// ── Rendering helpers ─────────────────────────────────────────────────────────

/// Draw all bezier curves between the three flow columns.
///
/// `col0`/`col1`/`col2` are `(entity_id, canvas_center)` pairs.
/// `needs` is the middle column's full item list — the `is_pain` flag at
/// index `i` determines which link table and colour to use.
#[expect(clippy::too_many_arguments)]
fn draw_flow_curves(
    painter: &egui::Painter,
    col0: &[(Uuid, egui::Pos2)],
    col1: &[(Uuid, egui::Pos2)],
    col2: &[(Uuid, egui::Pos2)],
    needs: &[(Uuid, String, [u8; 3], bool)],
    cs: &CustomerSegmentPage,
    vp: &ValuePropPage,
    pill_hw: f32,
    alpha_for: &impl Fn(Uuid) -> u8,
) {
    // col0 → col1: job–need links (binary: linked or not)
    for &(jid, jcenter) in col0 {
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
            let [r, g, b] = if is_pain { PAIN_RGB } else { GAIN_RGB };
            let stroke = egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(r, g, b, a));
            draw_bezier(
                painter,
                egui::pos2(jcenter.x + pill_hw, jcenter.y),
                egui::pos2(ncenter.x - pill_hw, ncenter.y),
                stroke,
            );
        }
    }

    // col1 → col2: need–solution links (annotation strength encodes opacity/width)
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
            let a = (f32::from(base_a) * (strength * 0.7 + 0.3)) as u8;
            let [r, g, b] = if is_pain { PAIN_RGB } else { GAIN_RGB };
            let width = 1.0 + strength * 2.5;
            let stroke =
                egui::Stroke::new(width, egui::Color32::from_rgba_unmultiplied(r, g, b, a));
            draw_bezier(
                painter,
                egui::pos2(ncenter.x + pill_hw, ncenter.y),
                egui::pos2(scenter.x - pill_hw, scenter.y),
                stroke,
            );
        }
    }
}

/// Draw a row of pill-shaped entity labels onto the canvas.
///
/// `items` and `centers` are parallel slices indexed by column position.
/// `alpha_for` maps an entity ID to its draw alpha (220 = fully visible, 40 = dimmed).
#[expect(clippy::too_many_arguments)]
fn draw_flow_pills(
    painter: &egui::Painter,
    items: &[(Uuid, String, [u8; 3])],
    centers: &[(Uuid, egui::Pos2)],
    pill_w: f32,
    pill_h: f32,
    pill_font: &egui::FontId,
    text_base: egui::Color32,
    alpha_for: &impl Fn(Uuid) -> u8,
) {
    for (i, (id, name, rgb)) in items.iter().enumerate() {
        let Some(&(_, center)) = centers.get(i) else {
            continue;
        };
        let a = alpha_for(*id);
        let fill = egui::Color32::from_rgba_unmultiplied(rgb[0], rgb[1], rgb[2], a / 2);
        let pill_rect = egui::Rect::from_center_size(center, egui::vec2(pill_w, pill_h));
        painter.rect_filled(pill_rect, 5.0, fill);
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            name.as_str(),
            pill_font.clone(),
            egui::Color32::from_rgba_unmultiplied(text_base.r(), text_base.g(), text_base.b(), a),
        );
    }
}

/// Render the colour legend below the Flow canvas.
fn show_flow_legend(ui: &mut egui::Ui) {
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        for ([r, g, b], label) in [
            (JOB_RGB, "Job"),
            (PAIN_RGB, "Pain / Pain Relief"),
            (GAIN_RGB, "Gain / Gain Creator"),
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

// ── Connected-set computation ─────────────────────────────────────────────────

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

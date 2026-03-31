// ── Tab 3: Web ────────────────────────────────────────────────────────────────
//
// Radial spider diagram. Select a Job to focus on. Its linked pains and gains
// form an inner ring; the pain reliefs / gain creators addressing those needs
// form an outer ring. Line opacity encodes annotation strength.

use crate::app::App;
use crate::app::pages::accordion::display_name;
use crate::app::pages::product::ValueType;
use eframe::egui;
use uuid::Uuid;

use super::{
    GAIN_CREATOR_RGB, GAIN_RGB, JOB_RGB, PAIN_RELIEF_RGB, PAIN_RGB, show_placeholder, truncate,
};

/// An outer-ring node: a solution (pain relief or gain creator) with its
/// connections back to inner-ring needs.
type OuterNode = (Uuid, String, [u8; 3], Vec<(Uuid, f32, ValueType)>);

/// Position on a ring of `n` equally-spaced nodes, starting at the top (12 o'clock).
fn ring_pos(idx: usize, n: usize, radius: f32, center: egui::Pos2) -> egui::Pos2 {
    let angle = std::f32::consts::TAU * idx as f32 / n as f32 - std::f32::consts::FRAC_PI_2;
    egui::pos2(
        center.x + radius * angle.cos(),
        center.y + radius * angle.sin(),
    )
}

#[expect(clippy::too_many_lines)]
pub(super) fn show_web(app: &App, ctx: &egui::Context, ui: &mut egui::Ui) {
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
                PAIN_RGB,
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
                GAIN_RGB,
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
                PAIN_RELIEF_RGB,
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
                GAIN_CREATOR_RGB,
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

    // Lines center → inner (uniform)
    for i in 0..n_inner {
        let pos = ring_pos(i, n_inner, inner_r, center);
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
        let opos = ring_pos(oi, n_outer, outer_r, center);
        for &(inner_id, strength, _) in conns {
            let Some(ii) = inner.iter().position(|(id, _, _, _)| *id == inner_id) else {
                continue;
            };
            let ipos = ring_pos(ii, n_inner, inner_r, center);
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
        let pos = ring_pos(i, n_inner, inner_r, center);
        let is_hovered = hover_pos.is_some_and(|hp| pos.distance(hp) < HOVER_R);
        let r = if is_hovered { NODE_R + 2.0 } else { NODE_R };
        painter.circle_filled(pos, r, egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
        // Label outside the circle
        let label_pos = ring_pos(i, n_inner, inner_r + NODE_R + 10.0, center);
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
                    let opos = ring_pos(oi, n_outer, outer_r, center);
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
        let pos = ring_pos(i, n_outer, outer_r, center);
        let is_hovered = hover_pos.is_some_and(|hp| pos.distance(hp) < HOVER_R);
        let r = if is_hovered { 8.0 } else { 6.0_f32 };
        painter.circle_filled(pos, r, egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
        let label_pos = ring_pos(i, n_outer, outer_r + r + 10.0, center);
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
    let [r, g, b] = JOB_RGB;
    painter.circle_filled(center, 18.0, egui::Color32::from_rgb(r, g, b));
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
            (JOB_RGB, "Job (centre)"),
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
        ui.label(
            egui::RichText::new("Line opacity = annotation strength")
                .small()
                .color(ui.visuals().weak_text_color()),
        );
    });
}

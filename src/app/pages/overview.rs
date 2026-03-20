use crate::app::App;
use eframe::egui;
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use super::accordion::{
    color_gain, color_job, color_pain, color_segment, display_name, label_with_hover_id,
};

/// The chain runs left-to-right across the overview columns:
///   Products → Features → GainCreators/PainReliefs → Gains/Pains → Jobs → Segments
///
/// Both a forward adjacency (following the chain) and a backward adjacency
/// (reversing it) are built so that a single BFS in each direction can be run
/// from the hovered entity without ever "crossing back" through a shared node
/// to unrelated entities on the same side.
fn build_directed_adj(app: &App) -> (HashMap<Uuid, Vec<Uuid>>, HashMap<Uuid, Vec<Uuid>>) {
    let vp = &app.valueprop_page;
    let cs = &app.customer_segment_page;
    let mut fwd: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    let mut bwd: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

    let mut edge = |a: Uuid, b: Uuid| {
        fwd.entry(a).or_default().push(b);
        bwd.entry(b).or_default().push(a);
    };

    // Product → Feature
    for &(pid, fid) in &vp.product_feature_links {
        edge(pid, fid);
    }
    // Feature → GainCreator
    for &(fid, gcid) in &vp.feature_gain_creator_links {
        edge(fid, gcid);
    }
    // Feature → PainRelief
    for &(fid, prid) in &vp.feature_pain_relief_links {
        edge(fid, prid);
    }
    // GainCreator → Gain  (cross-page: value prop side → customer side)
    for &(gain_id, gcid) in &vp.gain_gain_creator_links {
        edge(gcid, gain_id);
    }
    // PainRelief → Pain
    for &(pain_id, prid) in &vp.pain_pain_relief_links {
        edge(prid, pain_id);
    }
    // Gain → Job
    for &(gain_id, job_id) in &cs.job_gain_links {
        edge(gain_id, job_id);
    }
    // Pain → Job
    for &(pain_id, job_id) in &cs.job_pain_links {
        edge(pain_id, job_id);
    }
    // Job → Segment
    for &(job_id, seg_id) in &cs.segment_job_links {
        edge(job_id, seg_id);
    }

    (fwd, bwd)
}

fn bfs(start: Uuid, adj: &HashMap<Uuid, Vec<Uuid>>) -> HashSet<Uuid> {
    let mut visited = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(node) = queue.pop_front() {
        for &next in adj.get(&node).into_iter().flatten() {
            if visited.insert(next) {
                queue.push_back(next);
            }
        }
    }
    visited.remove(&start);
    visited
}

/// Returns every entity that should be highlighted when `hovered` is active.
///
/// One forward BFS follows the chain toward Segments; one backward BFS follows
/// it toward Products. They never mix, so a shared Feature (or any other
/// intermediate node) cannot "bleed" highlights across unrelated entities on
/// the same side.
fn highlighted_ids(hovered: Option<Uuid>, app: &App) -> HashSet<Uuid> {
    let Some(start) = hovered else {
        return HashSet::new();
    };
    let (fwd, bwd) = build_directed_adj(app);
    let mut result = bfs(start, &fwd);
    result.extend(bfs(start, &bwd));
    result
}

pub fn show_overview(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Overview");
    ui.add_space(8.0);

    let hovered_key = egui::Id::new("ov_hovered_entity");
    let prev_hovered: Option<Uuid> = ctx.data(|d| d.get_temp(hovered_key));
    ctx.data_mut(|d| d.remove::<Uuid>(hovered_key));

    let highlighted = highlighted_ids(prev_hovered, app);

    ui.columns(4, |cols| {
        let [c0, c1, c2, c3] = cols else { return };

        // ── Col 0: Products & Services ────────────────────────────────────────
        c0.label(egui::RichText::new("Products & Services").strong());
        c0.separator();
        for p in &app.valueprop_page.products_state.products {
            label_with_hover_id(
                c0,
                display_name(&p.name, "Unnamed product"),
                p.id,
                color_job(),
                highlighted.contains(&p.id),
                hovered_key,
            );
        }

        // ── Col 1: Gain Creators + Pain Reliefs ───────────────────────────────
        c1.label(egui::RichText::new("Gain Creators").strong());
        c1.separator();
        for item in &app.valueprop_page.gain_creator_state.gain_creators {
            label_with_hover_id(
                c1,
                display_name(&item.name, "Unnamed gain creator"),
                item.id,
                color_gain(),
                highlighted.contains(&item.id),
                hovered_key,
            );
        }

        c1.add_space(12.0);
        c1.label(egui::RichText::new("Pain Reliefs").strong());
        c1.separator();
        for item in &app.valueprop_page.pain_relief_state.pain_reliefs {
            label_with_hover_id(
                c1,
                display_name(&item.name, "Unnamed pain relief"),
                item.id,
                color_pain(),
                highlighted.contains(&item.id),
                hovered_key,
            );
        }

        // ── Col 2: Gains + Pains ──────────────────────────────────────────────
        c2.label(egui::RichText::new("Gains").strong());
        c2.separator();
        for item in &app.customer_segment_page.gains_state.gains {
            label_with_hover_id(
                c2,
                display_name(&item.name, "Unnamed gain"),
                item.id,
                color_gain(),
                highlighted.contains(&item.id),
                hovered_key,
            );
        }

        c2.add_space(12.0);
        c2.label(egui::RichText::new("Pains").strong());
        c2.separator();
        for item in &app.customer_segment_page.pains_state.pains {
            label_with_hover_id(
                c2,
                display_name(&item.name, "Unnamed pain"),
                item.id,
                color_pain(),
                highlighted.contains(&item.id),
                hovered_key,
            );
        }

        // ── Col 3: Jobs + Customer Segments ───────────────────────────────────
        c3.label(egui::RichText::new("Jobs").strong());
        c3.separator();
        for item in &app.customer_segment_page.jobs_state.jobs {
            label_with_hover_id(
                c3,
                display_name(&item.name, "Unnamed job"),
                item.id,
                color_job(),
                highlighted.contains(&item.id),
                hovered_key,
            );
        }

        c3.add_space(12.0);
        c3.label(egui::RichText::new("Customer Segments").strong());
        c3.separator();
        for item in &app.customer_segment_page.segments_state.segments {
            label_with_hover_id(
                c3,
                display_name(&item.name, "Unnamed segment"),
                item.id,
                color_segment(),
                highlighted.contains(&item.id),
                hovered_key,
            );
        }
    });
}

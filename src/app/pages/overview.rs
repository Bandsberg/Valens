use crate::app::App;
use eframe::egui;
use std::collections::HashSet;
use uuid::Uuid;

use super::accordion::{
    color_gain, color_job, color_pain, color_segment, display_name, label_with_hover_id,
};

/// Computes the set of entity IDs that should be highlighted because they are
/// linked (directly or transitively) to the currently hovered entity.
///
/// Cross-page links:
/// - `Gain` ↔ `GainCreator`  (`gain_gain_creator_links`: `(gain_id, gain_creator_id)`)
/// - `Pain` ↔ `PainRelief`   (`pain_pain_relief_links`:  `(pain_id, pain_relief_id)`)
///
/// Within Value Proposition:
/// - `Product` ↔ `Feature` ↔ `GainCreator`
/// - `Product` ↔ `Feature` ↔ `PainRelief`
///
/// Within Customer Segment:
/// - `Gain` ↔ `Job`  (`job_gain_links`: `(gain_id, job_id)`)
/// - `Pain` ↔ `Job`  (`job_pain_links`: `(pain_id, job_id)`)
/// - `Job`  ↔ `Segment` (`segment_job_links`: `(job_id, segment_id)`)
fn highlighted_ids(hovered: Option<Uuid>, app: &App) -> HashSet<Uuid> {
    let Some(hovered_id) = hovered else {
        return HashSet::new();
    };
    let vp = &app.valueprop_page;
    let cs = &app.customer_segment_page;
    let mut result = HashSet::new();

    // ── Value Proposition internal links ──────────────────────────────────────

    // Hovered is a Product → highlight linked GainCreators and PainReliefs.
    let features_of_product: Vec<Uuid> = vp
        .product_feature_links
        .iter()
        .filter(|(pid, _)| *pid == hovered_id)
        .map(|(_, fid)| *fid)
        .collect();
    for fid in &features_of_product {
        for (f, gc) in &vp.feature_gain_creator_links {
            if f == fid {
                result.insert(*gc);
            }
        }
        for (f, pr) in &vp.feature_pain_relief_links {
            if f == fid {
                result.insert(*pr);
            }
        }
    }

    // Hovered is a GainCreator → highlight linked Products.
    let features_of_gc: Vec<Uuid> = vp
        .feature_gain_creator_links
        .iter()
        .filter(|(_, gcid)| *gcid == hovered_id)
        .map(|(fid, _)| *fid)
        .collect();
    for fid in &features_of_gc {
        for (pid, f) in &vp.product_feature_links {
            if f == fid {
                result.insert(*pid);
            }
        }
    }

    // Hovered is a PainRelief → highlight linked Products.
    let features_of_pr: Vec<Uuid> = vp
        .feature_pain_relief_links
        .iter()
        .filter(|(_, prid)| *prid == hovered_id)
        .map(|(fid, _)| *fid)
        .collect();
    for fid in &features_of_pr {
        for (pid, f) in &vp.product_feature_links {
            if f == fid {
                result.insert(*pid);
            }
        }
    }

    // ── Customer Segment internal links ───────────────────────────────────────

    // Gain ↔ Job and Pain ↔ Job.
    cs.job_gain_links
        .iter()
        .chain(&cs.job_pain_links)
        .for_each(|&(a, b)| {
            if a == hovered_id {
                result.insert(b);
            } else if b == hovered_id {
                result.insert(a);
            }
        });

    // Job ↔ Segment.
    cs.segment_job_links.iter().for_each(|&(job_id, seg_id)| {
        if job_id == hovered_id {
            result.insert(seg_id);
        } else if seg_id == hovered_id {
            result.insert(job_id);
        }
    });

    // ── Cross-page links ──────────────────────────────────────────────────────

    // Gain ↔ GainCreator.
    vp.gain_gain_creator_links
        .iter()
        .for_each(|&(gain_id, gc_id)| {
            if gain_id == hovered_id {
                result.insert(gc_id);
            } else if gc_id == hovered_id {
                result.insert(gain_id);
            }
        });

    // Pain ↔ PainRelief.
    vp.pain_pain_relief_links
        .iter()
        .for_each(|&(pain_id, pr_id)| {
            if pain_id == hovered_id {
                result.insert(pr_id);
            } else if pr_id == hovered_id {
                result.insert(pain_id);
            }
        });

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

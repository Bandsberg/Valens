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

/// Returns the set of gain IDs and pain IDs reachable from a product
/// via its features → `gain_creators`/`pain_reliefs` → cross-canvas links.
fn product_covered_gains_pains(product_id: Uuid, app: &App) -> (HashSet<Uuid>, HashSet<Uuid>) {
    let vp = &app.valueprop_page;

    // Features linked to this product
    let features: HashSet<Uuid> = vp.product_feature_links.iter()
        .filter(|(pid, _)| *pid == product_id)
        .map(|(_, fid)| *fid)
        .collect();

    // GainCreators reachable via those features
    let gain_creators: HashSet<Uuid> = vp.feature_gain_creator_links.iter()
        .filter(|(fid, _)| features.contains(fid))
        .map(|(_, gcid)| *gcid)
        .collect();

    // PainReliefs reachable via those features
    let pain_reliefs: HashSet<Uuid> = vp.feature_pain_relief_links.iter()
        .filter(|(fid, _)| features.contains(fid))
        .map(|(_, prid)| *prid)
        .collect();

    // Gains linked to those GainCreators (gain_gain_creator_links: (gain_id, gc_id))
    let gains: HashSet<Uuid> = vp.gain_gain_creator_links.iter()
        .filter(|(_, gcid)| gain_creators.contains(gcid))
        .map(|(gain_id, _)| *gain_id)
        .collect();

    // Pains linked to those PainReliefs (pain_pain_relief_links: (pain_id, pr_id))
    let pains: HashSet<Uuid> = vp.pain_pain_relief_links.iter()
        .filter(|(_, prid)| pain_reliefs.contains(prid))
        .map(|(pain_id, _)| *pain_id)
        .collect();

    (gains, pains)
}

/// Returns the set of gain IDs and pain IDs linked to a segment's jobs.
fn segment_needs(segment_id: Uuid, app: &App) -> (HashSet<Uuid>, HashSet<Uuid>) {
    let cs = &app.customer_segment_page;

    // Jobs linked to this segment (segment_job_links: (job_id, segment_id))
    let jobs: HashSet<Uuid> = cs.segment_job_links.iter()
        .filter(|(_, sid)| *sid == segment_id)
        .map(|(job_id, _)| *job_id)
        .collect();

    // Gains linked to those jobs (job_gain_links: (gain_id, job_id))
    let gains: HashSet<Uuid> = cs.job_gain_links.iter()
        .filter(|(_, job_id)| jobs.contains(job_id))
        .map(|(gain_id, _)| *gain_id)
        .collect();

    // Pains linked to those jobs (job_pain_links: (pain_id, job_id))
    let pains: HashSet<Uuid> = cs.job_pain_links.iter()
        .filter(|(_, job_id)| jobs.contains(job_id))
        .map(|(pain_id, _)| *pain_id)
        .collect();

    (gains, pains)
}

/// VP-Design fit score: fraction of a segment's needs addressed by a product.
/// Falls back to binary BFS if the segment has no needs defined.
fn fit_score(product_id: Uuid, segment_id: Uuid, app: &App) -> f32 {
    let (prod_gains, prod_pains) = product_covered_gains_pains(product_id, app);
    let (seg_gains, seg_pains) = segment_needs(segment_id, app);

    let total_needs = seg_gains.len() + seg_pains.len();
    if total_needs == 0 {
        // Fallback: BFS binary — caller handles this
        return f32::NAN;
    }

    let addressed = prod_gains.intersection(&seg_gains).count()
        + prod_pains.intersection(&seg_pains).count();

    addressed as f32 / total_needs as f32
}

/// Returns a score map: every reachable entity gets 1.0 (binary BFS), but when
/// hovering a Product or Segment the opposite column uses VP-Design fit scores.
fn highlighted_scores(hovered: Option<Uuid>, app: &App) -> HashMap<Uuid, f32> {
    let Some(start) = hovered else {
        return HashMap::new();
    };

    let (fwd, bwd) = build_directed_adj(app);
    let connected: HashSet<Uuid> = bfs(start, &fwd).union(&bfs(start, &bwd)).copied().collect();
    let mut scores: HashMap<Uuid, f32> = connected.iter().map(|&id| (id, 1.0_f32)).collect();

    let products = &app.valueprop_page.products_state.products;
    let segments = &app.customer_segment_page.segments_state.segments;
    let is_product = products.iter().any(|p| p.id == start);
    let is_segment = segments.iter().any(|s| s.id == start);

    if is_product {
        for seg in segments {
            let s = fit_score(start, seg.id, app);
            if s.is_nan() {
                // Segment has no needs: keep BFS binary (already in scores if connected)
            } else if s > 0.0 {
                scores.insert(seg.id, s);
            } else {
                // Score is 0 but might be BFS-connected; remove so it shows no highlight
                scores.remove(&seg.id);
            }
        }
    } else if is_segment {
        for prod in products {
            let s = fit_score(prod.id, start, app);
            if s.is_nan() {
                // No needs defined: keep BFS binary
            } else if s > 0.0 {
                scores.insert(prod.id, s);
            } else {
                scores.remove(&prod.id);
            }
        }
    }

    scores
}

#[derive(Clone, Copy, PartialEq, Default)]
enum HighlightMode {
    Off,
    #[default]
    Binary,
    FitScore,
}

fn binary_scores(hovered: Option<Uuid>, app: &App) -> HashMap<Uuid, f32> {
    let Some(start) = hovered else { return HashMap::new(); };
    let (fwd, bwd) = build_directed_adj(app);
    bfs(start, &fwd)
        .union(&bfs(start, &bwd))
        .map(|&id| (id, 1.0_f32))
        .collect()
}

/// Computes which intermediate entities (gain creators, pain reliefs, gains, pains, jobs)
/// are reachable from the visible (non-hidden) products and segments.
/// Returns `None` when no filter is active (show everything).
fn compute_visible_middle(
    hidden_products: &HashSet<Uuid>,
    hidden_segments: &HashSet<Uuid>,
    app: &App,
) -> Option<HashSet<Uuid>> {
    if hidden_products.is_empty() && hidden_segments.is_empty() {
        return None;
    }
    let (fwd, bwd) = build_directed_adj(app);
    let mut visible = HashSet::new();
    for p in &app.valueprop_page.products_state.products {
        if !hidden_products.contains(&p.id) {
            visible.extend(bfs(p.id, &fwd));
        }
    }
    for s in &app.customer_segment_page.segments_state.segments {
        if !hidden_segments.contains(&s.id) {
            visible.extend(bfs(s.id, &bwd));
        }
    }
    Some(visible)
}

fn show_columns(
    app: &App,
    cols: &mut [egui::Ui],
    scores: &HashMap<Uuid, f32>,
    hovered_key: egui::Id,
    hidden_products: &mut HashSet<Uuid>,
    hidden_segments: &mut HashSet<Uuid>,
    visible_middle: &Option<HashSet<Uuid>>,
) {
    let [c0, c1, c2, c3, c4] = cols else { return };
    let score = |id: Uuid| scores.get(&id).copied().unwrap_or(0.0);
    let is_visible = |id: Uuid| visible_middle.as_ref().is_none_or(|v| v.contains(&id));

    let all_products = &app.valueprop_page.products_state.products;
    let prod_visible = all_products.iter().filter(|p| !hidden_products.contains(&p.id)).count();
    let prod_label = if hidden_products.is_empty() {
        "All".to_owned()
    } else {
        format!("{} / {}", prod_visible, all_products.len())
    };
    c0.horizontal(|ui| {
        ui.label(egui::RichText::new("Products & Services").strong());
        egui::ComboBox::from_id_salt("ov_prod_filter")
            .selected_text(prod_label)
            .width(60.0)
            .show_ui(ui, |ui| {
                for p in all_products {
                    let mut checked = !hidden_products.contains(&p.id);
                    if ui.checkbox(&mut checked, display_name(&p.name, "Unnamed product")).changed() {
                        if checked { hidden_products.remove(&p.id); } else { hidden_products.insert(p.id); }
                    }
                }
            });
    });
    c0.separator();
    for p in all_products {
        if !hidden_products.contains(&p.id) {
            label_with_hover_id(c0, display_name(&p.name, "Unnamed product"), p.id, color_job(), score(p.id), hovered_key);
        }
    }

    c1.label(egui::RichText::new("Gain Creators").strong());
    c1.separator();
    for item in &app.valueprop_page.gain_creator_state.gain_creators {
        if is_visible(item.id) {
            label_with_hover_id(c1, display_name(&item.name, "Unnamed gain creator"), item.id, color_gain(), score(item.id), hovered_key);
        }
    }
    c1.add_space(12.0);
    c1.label(egui::RichText::new("Pain Reliefs").strong());
    c1.separator();
    for item in &app.valueprop_page.pain_relief_state.pain_reliefs {
        if is_visible(item.id) {
            label_with_hover_id(c1, display_name(&item.name, "Unnamed pain relief"), item.id, color_pain(), score(item.id), hovered_key);
        }
    }

    c2.label(egui::RichText::new("Gains").strong());
    c2.separator();
    for item in &app.customer_segment_page.gains_state.gains {
        if is_visible(item.id) {
            label_with_hover_id(c2, display_name(&item.name, "Unnamed gain"), item.id, color_gain(), score(item.id), hovered_key);
        }
    }
    c2.add_space(12.0);
    c2.label(egui::RichText::new("Pains").strong());
    c2.separator();
    for item in &app.customer_segment_page.pains_state.pains {
        if is_visible(item.id) {
            label_with_hover_id(c2, display_name(&item.name, "Unnamed pain"), item.id, color_pain(), score(item.id), hovered_key);
        }
    }

    c3.label(egui::RichText::new("Jobs").strong());
    c3.separator();
    for item in &app.customer_segment_page.jobs_state.jobs {
        if is_visible(item.id) {
            label_with_hover_id(c3, display_name(&item.name, "Unnamed job"), item.id, color_job(), score(item.id), hovered_key);
        }
    }

    let all_segments = &app.customer_segment_page.segments_state.segments;
    let seg_visible = all_segments.iter().filter(|s| !hidden_segments.contains(&s.id)).count();
    let seg_label = if hidden_segments.is_empty() {
        "All".to_owned()
    } else {
        format!("{} / {}", seg_visible, all_segments.len())
    };
    c4.horizontal(|ui| {
        ui.label(egui::RichText::new("Customer Segments").strong());
        egui::ComboBox::from_id_salt("ov_seg_filter")
            .selected_text(seg_label)
            .width(60.0)
            .show_ui(ui, |ui| {
                for s in all_segments {
                    let mut checked = !hidden_segments.contains(&s.id);
                    if ui.checkbox(&mut checked, display_name(&s.name, "Unnamed segment")).changed() {
                        if checked { hidden_segments.remove(&s.id); } else { hidden_segments.insert(s.id); }
                    }
                }
            });
    });
    c4.separator();
    for item in all_segments {
        if !hidden_segments.contains(&item.id) {
            label_with_hover_id(c4, display_name(&item.name, "Unnamed segment"), item.id, color_segment(), score(item.id), hovered_key);
        }
    }
}

fn show_fit_matrix(app: &App, ui: &mut egui::Ui) {
    let products = &app.valueprop_page.products_state.products;
    let segments = &app.customer_segment_page.segments_state.segments;

    if products.is_empty() || segments.is_empty() {
        ui.label(
            egui::RichText::new("No products or segments to display.")
                .italics()
                .color(ui.visuals().weak_text_color()),
        );
        return;
    }

    egui::Grid::new("fit_score_matrix")
        .striped(true)
        .show(ui, |ui| {
            ui.label("");
            for seg in segments {
                ui.label(egui::RichText::new(display_name(&seg.name, "Unnamed segment")).strong());
            }
            ui.end_row();

            for prod in products {
                ui.label(display_name(&prod.name, "Unnamed product"));
                for seg in segments {
                    let s = fit_score(prod.id, seg.id, app);
                    let (text, score) = if s.is_nan() {
                        ("—".to_owned(), 0.0_f32)
                    } else {
                        (format!("{:.0}%", s * 100.0), s)
                    };
                    let response = ui.label(&text);
                    if score > 0.0 {
                        let [r, g, b, a] = color_gain().to_array();
                        let new_a = (a as f32 * score).round() as u8;
                        let scale = if a > 0 { new_a as f32 / a as f32 } else { 0.0 };
                        let cell_color = egui::Color32::from_rgba_premultiplied(
                            (r as f32 * scale).round() as u8,
                            (g as f32 * scale).round() as u8,
                            (b as f32 * scale).round() as u8,
                            new_a,
                        );
                        ui.painter().rect_filled(response.rect, 2.0, cell_color);
                    }
                }
                ui.end_row();
            }
        });
}

pub fn show_overview(app: &App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let mode_key = egui::Id::new("ov_highlight_mode");
    let mut mode: HighlightMode = ctx.data(|d| d.get_temp(mode_key).unwrap_or_default());

    let matrix_key = egui::Id::new("ov_show_fit_matrix");
    let mut show_matrix: bool = ctx.data(|d| d.get_temp(matrix_key).unwrap_or(false));

    ui.horizontal(|ui| {
        ui.heading("Overview");
        ui.add_space(8.0);
        let label = match mode {
            HighlightMode::Off      => "Highlights: Off",
            HighlightMode::Binary   => "Highlights: Binary",
            HighlightMode::FitScore => "Highlights: Fit Score",
        };
        if ui.button(label).clicked() {
            mode = match mode {
                HighlightMode::Off      => HighlightMode::Binary,
                HighlightMode::Binary   => HighlightMode::FitScore,
                HighlightMode::FitScore => HighlightMode::Off,
            };
        }
        let matrix_label = if show_matrix { "Product-Market Fit: On" } else { "Product-Market Fit: Off" };
        if ui.button(matrix_label).clicked() {
            show_matrix = !show_matrix;
        }
    });
    ctx.data_mut(|d| d.insert_temp(mode_key, mode));
    ctx.data_mut(|d| d.insert_temp(matrix_key, show_matrix));
    ui.add_space(8.0);

    let hovered_key = egui::Id::new("ov_hovered_entity");
    let prev_hovered: Option<Uuid> = ctx.data(|d| d.get_temp(hovered_key));
    ctx.data_mut(|d| d.remove::<Uuid>(hovered_key));

    let scores = match mode {
        HighlightMode::Off      => HashMap::new(),
        HighlightMode::Binary   => binary_scores(prev_hovered, app),
        HighlightMode::FitScore => highlighted_scores(prev_hovered, app),
    };

    let prod_hidden_key = egui::Id::new("ov_hidden_products");
    let seg_hidden_key = egui::Id::new("ov_hidden_segments");
    let mut hidden_products: HashSet<Uuid> = ctx.data(|d| d.get_temp(prod_hidden_key).unwrap_or_default());
    let mut hidden_segments: HashSet<Uuid> = ctx.data(|d| d.get_temp(seg_hidden_key).unwrap_or_default());
    let visible_middle = compute_visible_middle(&hidden_products, &hidden_segments, app);

    ui.columns(5, |cols| show_columns(app, cols, &scores, hovered_key, &mut hidden_products, &mut hidden_segments, &visible_middle));

    ctx.data_mut(|d| d.insert_temp(prod_hidden_key, hidden_products));
    ctx.data_mut(|d| d.insert_temp(seg_hidden_key, hidden_segments));

    if show_matrix {
        ui.add_space(12.0);
        ui.label(egui::RichText::new("Fit Score Matrix").strong());
        ui.separator();
        show_fit_matrix(app, ui);
    }
}

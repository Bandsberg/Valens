use crate::app::App;
use eframe::egui;
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use super::accordion::{
    color_gain, color_job, color_pain, color_segment, display_name, label_with_hover_id,
    scale_color,
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
    for ann in &vp.gain_creator_annotations {
        edge(ann.reliever_or_creator_id, ann.pain_or_gain_id);
    }
    // PainRelief → Pain
    for ann in &vp.pain_relief_annotations {
        edge(ann.reliever_or_creator_id, ann.pain_or_gain_id);
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
    let features: HashSet<Uuid> = vp
        .product_feature_links
        .iter()
        .filter(|(pid, _)| *pid == product_id)
        .map(|(_, fid)| *fid)
        .collect();

    // GainCreators reachable via those features
    let gain_creators: HashSet<Uuid> = vp
        .feature_gain_creator_links
        .iter()
        .filter(|(fid, _)| features.contains(fid))
        .map(|(_, gcid)| *gcid)
        .collect();

    // PainReliefs reachable via those features
    let pain_reliefs: HashSet<Uuid> = vp
        .feature_pain_relief_links
        .iter()
        .filter(|(fid, _)| features.contains(fid))
        .map(|(_, prid)| *prid)
        .collect();

    // Gains linked to those GainCreators
    let gains: HashSet<Uuid> = vp
        .gain_creator_annotations
        .iter()
        .filter(|ann| gain_creators.contains(&ann.reliever_or_creator_id))
        .map(|ann| ann.pain_or_gain_id)
        .collect();

    // Pains linked to those PainReliefs
    let pains: HashSet<Uuid> = vp
        .pain_relief_annotations
        .iter()
        .filter(|ann| pain_reliefs.contains(&ann.reliever_or_creator_id))
        .map(|ann| ann.pain_or_gain_id)
        .collect();

    (gains, pains)
}

/// Returns the set of gain IDs and pain IDs linked to a segment's jobs.
fn segment_needs(segment_id: Uuid, app: &App) -> (HashSet<Uuid>, HashSet<Uuid>) {
    let cs = &app.customer_segment_page;

    // Jobs linked to this segment (segment_job_links: (job_id, segment_id))
    let jobs: HashSet<Uuid> = cs
        .segment_job_links
        .iter()
        .filter(|(_, sid)| *sid == segment_id)
        .map(|(job_id, _)| *job_id)
        .collect();

    // Gains linked to those jobs (job_gain_links: (gain_id, job_id))
    let gains: HashSet<Uuid> = cs
        .job_gain_links
        .iter()
        .filter(|(_, job_id)| jobs.contains(job_id))
        .map(|(gain_id, _)| *gain_id)
        .collect();

    // Pains linked to those jobs (job_pain_links: (pain_id, job_id))
    let pains: HashSet<Uuid> = cs
        .job_pain_links
        .iter()
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

    let addressed =
        prod_gains.intersection(&seg_gains).count() + prod_pains.intersection(&seg_pains).count();

    addressed as f32 / total_needs as f32
}

/// Iterates `candidate_ids` and updates `scores` using the fit score returned
/// by `fit`. Three outcomes are possible for each candidate:
/// - `NaN` — no gains/pains defined on that side → leave any existing BFS
///   score unchanged (binary fallback stays intact).
/// - `> 0` — partial or full match → replace the BFS score with the exact
///   fit percentage so the highlight intensity is proportional.
/// - `0` — explicitly zero match → remove from scores entirely so the
///   candidate shows no highlight at all.
fn apply_fit_scores(
    scores: &mut HashMap<Uuid, f32>,
    candidate_ids: impl Iterator<Item = Uuid>,
    fit: impl Fn(Uuid) -> f32,
) {
    for id in candidate_ids {
        let s = fit(id);
        if s.is_nan() {
            // No needs/outputs defined on this side — keep BFS binary score.
        } else if s > 0.0 {
            scores.insert(id, s);
        } else {
            // Score is exactly 0: explicitly remove so no highlight appears.
            scores.remove(&id);
        }
    }
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
        apply_fit_scores(&mut scores, segments.iter().map(|s| s.id), |sid| {
            fit_score(start, sid, app)
        });
    } else if is_segment {
        apply_fit_scores(&mut scores, products.iter().map(|p| p.id), |pid| {
            fit_score(pid, start, app)
        });
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
    let Some(start) = hovered else {
        return HashMap::new();
    };
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

fn filter_heading(
    ui: &mut egui::Ui,
    heading: &str,
    id_salt: &str,
    items: &[(Uuid, String)],
    hidden: &mut HashSet<Uuid>,
) {
    let visible = items.iter().filter(|(id, _)| !hidden.contains(id)).count();
    let label = if hidden.is_empty() {
        "All".to_owned()
    } else {
        format!("{} / {}", visible, items.len())
    };
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(heading).strong());
        egui::ComboBox::from_id_salt(id_salt)
            .selected_text(label)
            .width(60.0)
            .show_ui(ui, |ui| {
                for (id, name) in items {
                    let mut checked = !hidden.contains(id);
                    if ui.checkbox(&mut checked, name.as_str()).changed() {
                        if checked {
                            hidden.remove(id);
                        } else {
                            hidden.insert(*id);
                        }
                    }
                }
            });
    });
    ui.separator();
}

#[expect(clippy::too_many_arguments, clippy::too_many_lines)]
fn render_column(
    col_idx: usize,
    ui: &mut egui::Ui,
    app: &App,
    scores: &HashMap<Uuid, f32>,
    hovered_key: egui::Id,
    hidden_products: &mut HashSet<Uuid>,
    hidden_segments: &mut HashSet<Uuid>,
    visible_middle: &Option<HashSet<Uuid>>,
) {
    let score = |id: Uuid| scores.get(&id).copied().unwrap_or(0.0);
    let is_visible = |id: Uuid| visible_middle.as_ref().is_none_or(|v| v.contains(&id));

    match col_idx {
        0 => {
            let all_products = &app.valueprop_page.products_state.products;
            let prod_items: Vec<(Uuid, String)> = all_products
                .iter()
                .map(|p| (p.id, display_name(&p.name, "Unnamed product").to_owned()))
                .collect();
            filter_heading(
                ui,
                "Products & Services",
                "ov_prod_filter",
                &prod_items,
                hidden_products,
            );
            for (id, name) in &prod_items {
                if !hidden_products.contains(id) {
                    label_with_hover_id(ui, name, *id, color_job(), score(*id), hovered_key);
                }
            }
        }
        1 => {
            ui.label(egui::RichText::new("Gain Creators").strong());
            ui.separator();
            for item in &app.valueprop_page.gain_creator_state.gain_creators {
                if is_visible(item.id) {
                    label_with_hover_id(
                        ui,
                        display_name(&item.name, "Unnamed gain creator"),
                        item.id,
                        color_gain(),
                        score(item.id),
                        hovered_key,
                    );
                }
            }
            ui.add_space(12.0);
            ui.label(egui::RichText::new("Pain Reliefs").strong());
            ui.separator();
            for item in &app.valueprop_page.pain_relief_state.pain_reliefs {
                if is_visible(item.id) {
                    label_with_hover_id(
                        ui,
                        display_name(&item.name, "Unnamed pain relief"),
                        item.id,
                        color_pain(),
                        score(item.id),
                        hovered_key,
                    );
                }
            }
        }
        2 => {
            ui.label(egui::RichText::new("Gains").strong());
            ui.separator();
            for item in &app.customer_segment_page.gains_state.gains {
                if is_visible(item.id) {
                    label_with_hover_id(
                        ui,
                        display_name(&item.name, "Unnamed gain"),
                        item.id,
                        color_gain(),
                        score(item.id),
                        hovered_key,
                    );
                }
            }
            ui.add_space(12.0);
            ui.label(egui::RichText::new("Pains").strong());
            ui.separator();
            for item in &app.customer_segment_page.pains_state.pains {
                if is_visible(item.id) {
                    label_with_hover_id(
                        ui,
                        display_name(&item.name, "Unnamed pain"),
                        item.id,
                        color_pain(),
                        score(item.id),
                        hovered_key,
                    );
                }
            }
        }
        3 => {
            ui.label(egui::RichText::new("Jobs").strong());
            ui.separator();
            for item in &app.customer_segment_page.jobs_state.jobs {
                if is_visible(item.id) {
                    label_with_hover_id(
                        ui,
                        display_name(&item.name, "Unnamed job"),
                        item.id,
                        color_job(),
                        score(item.id),
                        hovered_key,
                    );
                }
            }
        }
        4 => {
            let all_segments = &app.customer_segment_page.segments_state.segments;
            let seg_items: Vec<(Uuid, String)> = all_segments
                .iter()
                .map(|s| (s.id, display_name(&s.name, "Unnamed segment").to_owned()))
                .collect();
            filter_heading(
                ui,
                "Customer Segments",
                "ov_seg_filter",
                &seg_items,
                hidden_segments,
            );
            for (id, name) in &seg_items {
                if !hidden_segments.contains(id) {
                    label_with_hover_id(ui, name, *id, color_segment(), score(*id), hovered_key);
                }
            }
        }
        _ => {}
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
                        let cell_color = scale_color(color_gain(), score);
                        ui.painter().rect_filled(response.rect, 2.0, cell_color);
                    }
                }
                ui.end_row();
            }
        });
}

fn show_toolbar(
    ui: &mut egui::Ui,
    mode: &mut HighlightMode,
    show_matrix: &mut bool,
    col_vis: &mut [bool; 5],
) {
    ui.horizontal(|ui| {
        ui.heading("Overview");
        ui.add_space(8.0);
        let label = match mode {
            HighlightMode::Off => "Highlights: Off",
            HighlightMode::Binary => "Highlights: Binary",
            HighlightMode::FitScore => "Highlights: Fit Score",
        };
        if ui.button(label).clicked() {
            *mode = match mode {
                HighlightMode::Off => HighlightMode::Binary,
                HighlightMode::Binary => HighlightMode::FitScore,
                HighlightMode::FitScore => HighlightMode::Off,
            };
        }
        let matrix_label = if *show_matrix {
            "Product-Market Fit: On"
        } else {
            "Product-Market Fit: Off"
        };
        if ui.button(matrix_label).clicked() {
            *show_matrix = !*show_matrix;
        }
        let col_btn = ui.button("Columns \u{25be}");
        egui::Popup::from_toggle_button_response(&col_btn)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.set_min_width(200.0);
                let names = [
                    "Products & Services",
                    "Gain Creators / Pain Reliefs",
                    "Gains / Pains",
                    "Jobs",
                    "Customer Segments",
                ];
                for (vis, name) in col_vis.iter_mut().zip(names.iter()) {
                    ui.checkbox(vis, *name);
                }
            });
    });
}

pub fn show_overview(app: &App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let mode_key = egui::Id::new("ov_highlight_mode");
    let mut mode: HighlightMode = ctx.data(|d| d.get_temp(mode_key).unwrap_or_default());

    let matrix_key = egui::Id::new("ov_show_fit_matrix");
    let mut show_matrix: bool = ctx.data(|d| d.get_temp(matrix_key).unwrap_or(false));

    let col_vis_key = egui::Id::new("ov_col_visibility");
    let mut col_vis: [bool; 5] = ctx.data(|d| d.get_temp(col_vis_key).unwrap_or([true; 5]));

    show_toolbar(ui, &mut mode, &mut show_matrix, &mut col_vis);
    ctx.data_mut(|d| d.insert_temp(mode_key, mode));
    ctx.data_mut(|d| d.insert_temp(matrix_key, show_matrix));
    ctx.data_mut(|d| d.insert_temp(col_vis_key, col_vis));
    ui.add_space(8.0);

    let hovered_key = egui::Id::new("ov_hovered_entity");
    let prev_hovered: Option<Uuid> = ctx.data(|d| d.get_temp(hovered_key));
    ctx.data_mut(|d| d.remove::<Uuid>(hovered_key));

    let scores = match mode {
        HighlightMode::Off => HashMap::new(),
        HighlightMode::Binary => binary_scores(prev_hovered, app),
        HighlightMode::FitScore => highlighted_scores(prev_hovered, app),
    };

    let prod_hidden_key = egui::Id::new("ov_hidden_products");
    let seg_hidden_key = egui::Id::new("ov_hidden_segments");
    let mut hidden_products: HashSet<Uuid> =
        ctx.data(|d| d.get_temp(prod_hidden_key).unwrap_or_default());
    let mut hidden_segments: HashSet<Uuid> =
        ctx.data(|d| d.get_temp(seg_hidden_key).unwrap_or_default());
    let visible_middle = compute_visible_middle(&hidden_products, &hidden_segments, app);

    let visible_cols: Vec<usize> = col_vis
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| v.then_some(i))
        .collect();
    if visible_cols.is_empty() {
        ui.label(
            egui::RichText::new(
                "All columns hidden. Use \u{201c}Columns \u{25be}\u{201d} to show columns.",
            )
            .italics()
            .color(ui.visuals().weak_text_color()),
        );
    } else {
        let n = visible_cols.len();
        ui.columns(n, |cols| {
            for (slot, &col_idx) in visible_cols.iter().enumerate() {
                if let Some(col) = cols.get_mut(slot) {
                    render_column(
                        col_idx,
                        col,
                        app,
                        &scores,
                        hovered_key,
                        &mut hidden_products,
                        &mut hidden_segments,
                        &visible_middle,
                    );
                }
            }
        });
    }

    ctx.data_mut(|d| d.insert_temp(prod_hidden_key, hidden_products));
    ctx.data_mut(|d| d.insert_temp(seg_hidden_key, hidden_segments));

    if show_matrix {
        ui.add_space(12.0);
        ui.label(egui::RichText::new("Fit Score Matrix").strong());
        ui.separator();
        show_fit_matrix(app, ui);
    }
}

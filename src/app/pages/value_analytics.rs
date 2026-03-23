use crate::app::App;
use uuid::Uuid;

use super::product::{ValueAnnotation, ValueType};

// ── Thresholds ────────────────────────────────────────────────────────────────

/// Below this strength a covered need is considered "weak".
pub const STRENGTH_WEAK_THRESHOLD: f32 = 0.5;

/// A Table Stake annotation must meet or exceed this strength to be considered
/// complete (i.e. the product is viable on this dimension).
pub const TABLE_STAKE_MIN_STRENGTH: f32 = 0.7;

/// A Differentiator annotation at or above this strength is "strong" — a
/// genuine source of competitive advantage worth highlighting.
pub const DIFFERENTIATOR_STRONG_THRESHOLD: f32 = 0.7;

// ── Core types ────────────────────────────────────────────────────────────────

/// Coverage descriptor for a single pain or gain relative to a product.
pub struct NeedCoverage {
    #[expect(dead_code)]
    pub id: Uuid,
    pub name: String,
    /// Customer-perceived importance weight (0.0–1.0).
    pub importance: f32,
    /// `true` for a pain, `false` for a gain.
    pub is_pain: bool,
    /// Best effective strength across all relievers/creators from this product.
    /// `None` means the product has no annotation targeting this need.
    pub effective_strength: Option<f32>,
    /// Value type of the strongest annotation, if any.
    pub value_type: Option<ValueType>,
}

/// Coverages bucketed into four actionable groups.
pub struct GapGroups {
    /// No annotation at all from this product.
    pub uncovered: Vec<NeedCoverage>,
    /// Covered but `effective_strength < STRENGTH_WEAK_THRESHOLD`.
    pub weak: Vec<NeedCoverage>,
    /// Table Stake annotation present but `effective_strength < TABLE_STAKE_MIN_STRENGTH`.
    pub incomplete_table_stakes: Vec<NeedCoverage>,
    /// Differentiator annotation with `effective_strength >= DIFFERENTIATOR_STRONG_THRESHOLD`.
    pub strong_differentiators: Vec<NeedCoverage>,
}

// ── Analytics functions ───────────────────────────────────────────────────────

/// Returns the features reachable from `product_id` via `product_feature_links`.
fn features_of_product(product_id: Uuid, app: &App) -> Vec<Uuid> {
    app.valueprop_page
        .product_feature_links
        .iter()
        .filter_map(|&(pid, fid)| (pid == product_id).then_some(fid))
        .collect()
}

/// Returns the relievers (pain relief items) reachable from a set of features.
fn relievers_of_features(feature_ids: &[Uuid], app: &App) -> Vec<Uuid> {
    app.valueprop_page
        .feature_pain_relief_links
        .iter()
        .filter_map(|&(fid, rid)| feature_ids.contains(&fid).then_some(rid))
        .collect()
}

/// Returns the creators (gain creator items) reachable from a set of features.
fn creators_of_features(feature_ids: &[Uuid], app: &App) -> Vec<Uuid> {
    app.valueprop_page
        .feature_gain_creator_links
        .iter()
        .filter_map(|&(fid, cid)| feature_ids.contains(&fid).then_some(cid))
        .collect()
}

/// Best effective strength for a pain (`is_pain = true`) or gain (`is_pain = false`)
/// from a specific product's solution set.
///
/// Walks `product → features → relievers/creators → annotations` and returns
/// the maximum `strength` (capped at 1.0) across all matching annotations,
/// together with the value type of that strongest annotation.
/// Returns `None` if the product has no annotation targeting this need.
pub fn best_strength_for_need(
    pain_or_gain_id: Uuid,
    is_pain: bool,
    product_id: Uuid,
    app: &App,
) -> Option<(f32, ValueType)> {
    let features = features_of_product(product_id, app);
    let by_strength = |a: &&ValueAnnotation, b: &&ValueAnnotation| {
        a.strength
            .partial_cmp(&b.strength)
            .unwrap_or(std::cmp::Ordering::Equal)
    };

    if is_pain {
        let relievers = relievers_of_features(&features, app);
        app.valueprop_page
            .pain_relief_annotations
            .iter()
            .filter(|ann| {
                ann.pain_or_gain_id == pain_or_gain_id
                    && relievers.contains(&ann.reliever_or_creator_id)
            })
            .max_by(by_strength)
            .map(|ann| (ann.strength.min(1.0), ann.value_type))
    } else {
        let creators = creators_of_features(&features, app);
        app.valueprop_page
            .gain_creator_annotations
            .iter()
            .filter(|ann| {
                ann.pain_or_gain_id == pain_or_gain_id
                    && creators.contains(&ann.reliever_or_creator_id)
            })
            .max_by(by_strength)
            .map(|ann| (ann.strength.min(1.0), ann.value_type))
    }
}

/// Weighted VP-Design fit score for a (product, segment) pair.
///
/// Formula: `Σ(importance_i × effective_strength_i) / Σ(importance_i)`
///
/// Returns `f32::NAN` when the segment has no needs defined (preserving the
/// existing binary-BFS fallback behaviour in `overview.rs`).
pub fn weighted_fit_score(product_id: Uuid, segment_id: Uuid, app: &App) -> f32 {
    let cs = &app.customer_segment_page;

    // Jobs belonging to this segment.
    let jobs: Vec<Uuid> = cs
        .segment_job_links
        .iter()
        .filter_map(|&(jid, sid)| (sid == segment_id).then_some(jid))
        .collect();

    // Pains and gains belonging to those jobs.
    let pain_ids: Vec<Uuid> = cs
        .job_pain_links
        .iter()
        .filter_map(|&(pid, jid)| jobs.contains(&jid).then_some(pid))
        .collect();
    let gain_ids: Vec<Uuid> = cs
        .job_gain_links
        .iter()
        .filter_map(|&(gid, jid)| jobs.contains(&jid).then_some(gid))
        .collect();

    // Deduplicate (a pain/gain may be shared by multiple jobs).
    let mut all_pain_ids = pain_ids;
    all_pain_ids.sort_unstable();
    all_pain_ids.dedup();
    let mut all_gain_ids = gain_ids;
    all_gain_ids.sort_unstable();
    all_gain_ids.dedup();

    let total_needs = all_pain_ids.len() + all_gain_ids.len();
    if total_needs == 0 {
        return f32::NAN;
    }

    let mut weight_sum = 0.0_f32;
    let mut weighted_strength_sum = 0.0_f32;

    for &pid in &all_pain_ids {
        let importance = cs
            .pains_state
            .pains
            .iter()
            .find(|p| p.id == pid)
            .map_or(0.5, |p| p.importance);
        let strength = best_strength_for_need(pid, true, product_id, app).map_or(0.0, |(s, _)| s);
        weight_sum += importance;
        weighted_strength_sum += importance * strength;
    }

    for &gid in &all_gain_ids {
        let importance = cs
            .gains_state
            .gains
            .iter()
            .find(|g| g.id == gid)
            .map_or(0.5, |g| g.importance);
        let strength = best_strength_for_need(gid, false, product_id, app).map_or(0.0, |(s, _)| s);
        weight_sum += importance;
        weighted_strength_sum += importance * strength;
    }

    if weight_sum == 0.0 {
        return f32::NAN;
    }

    (weighted_strength_sum / weight_sum).min(1.0)
}

/// Builds a `NeedCoverage` for every pain and gain in a segment's job graph,
/// relative to a single product.
pub fn segment_need_coverages(product_id: Uuid, segment_id: Uuid, app: &App) -> Vec<NeedCoverage> {
    let cs = &app.customer_segment_page;

    let jobs: Vec<Uuid> = cs
        .segment_job_links
        .iter()
        .filter_map(|&(jid, sid)| (sid == segment_id).then_some(jid))
        .collect();

    let mut pain_ids: Vec<Uuid> = cs
        .job_pain_links
        .iter()
        .filter_map(|&(pid, jid)| jobs.contains(&jid).then_some(pid))
        .collect();
    pain_ids.sort_unstable();
    pain_ids.dedup();

    let mut gain_ids: Vec<Uuid> = cs
        .job_gain_links
        .iter()
        .filter_map(|&(gid, jid)| jobs.contains(&jid).then_some(gid))
        .collect();
    gain_ids.sort_unstable();
    gain_ids.dedup();

    let mut coverages = Vec::with_capacity(pain_ids.len() + gain_ids.len());

    for pid in pain_ids {
        if let Some(pain) = cs.pains_state.pains.iter().find(|p| p.id == pid) {
            let best = best_strength_for_need(pid, true, product_id, app);
            coverages.push(NeedCoverage {
                id: pid,
                name: pain.name.clone(),
                importance: pain.importance,
                is_pain: true,
                effective_strength: best.map(|(s, _)| s),
                value_type: best.map(|(_, vt)| vt),
            });
        }
    }

    for gid in gain_ids {
        if let Some(gain) = cs.gains_state.gains.iter().find(|g| g.id == gid) {
            let best = best_strength_for_need(gid, false, product_id, app);
            coverages.push(NeedCoverage {
                id: gid,
                name: gain.name.clone(),
                importance: gain.importance,
                is_pain: false,
                effective_strength: best.map(|(s, _)| s),
                value_type: best.map(|(_, vt)| vt),
            });
        }
    }

    coverages
}

/// Groups a slice of `NeedCoverage` into the four actionable categories.
///
/// Note: categories are not mutually exclusive in the sense of priority:
/// - `incomplete_table_stakes` takes precedence over `weak` when both apply.
/// - `uncovered` items never appear in any other group.
pub fn compute_gap_groups(coverages: Vec<NeedCoverage>) -> GapGroups {
    let mut groups = GapGroups {
        uncovered: Vec::new(),
        weak: Vec::new(),
        incomplete_table_stakes: Vec::new(),
        strong_differentiators: Vec::new(),
    };

    for c in coverages {
        match (c.effective_strength, c.value_type) {
            (None, _) => groups.uncovered.push(c),
            (Some(s), Some(ValueType::TableStake)) if s < TABLE_STAKE_MIN_STRENGTH => {
                groups.incomplete_table_stakes.push(c);
            }
            (Some(s), Some(ValueType::Differentiator)) if s >= DIFFERENTIATOR_STRONG_THRESHOLD => {
                groups.strong_differentiators.push(c);
            }
            (Some(s), _) if s < STRENGTH_WEAK_THRESHOLD => groups.weak.push(c),
            _ => {} // adequate coverage — not a gap
        }
    }

    groups
}

/// Table-stake completeness: how many Table Stake annotations from `product_id`
/// meet `TABLE_STAKE_MIN_STRENGTH`, relative to the total number of Table Stake
/// annotations from that product.
///
/// When `segment_id` is `Some`, only needs belonging to that segment's jobs are
/// considered. When `None`, all needs that the product has any Table Stake
/// annotation for are counted.
///
/// Returns `(met, total)`.
pub fn table_stake_completeness(
    product_id: Uuid,
    segment_id: Option<Uuid>,
    app: &App,
) -> (usize, usize) {
    let features = features_of_product(product_id, app);
    let relievers = relievers_of_features(&features, app);
    let creators = creators_of_features(&features, app);

    // Collect all Table Stake annotations reachable from this product.
    let ts_pain: Vec<&ValueAnnotation> = app
        .valueprop_page
        .pain_relief_annotations
        .iter()
        .filter(|ann| {
            ann.value_type == ValueType::TableStake
                && relievers.contains(&ann.reliever_or_creator_id)
        })
        .collect();

    let ts_gain: Vec<&ValueAnnotation> = app
        .valueprop_page
        .gain_creator_annotations
        .iter()
        .filter(|ann| {
            ann.value_type == ValueType::TableStake
                && creators.contains(&ann.reliever_or_creator_id)
        })
        .collect();

    // When a segment filter is provided, restrict to needs belonging to that segment.
    let (relevant_pain_ids, relevant_gain_ids): (Option<Vec<Uuid>>, Option<Vec<Uuid>>) =
        if let Some(sid) = segment_id {
            let cs = &app.customer_segment_page;
            let jobs: Vec<Uuid> = cs
                .segment_job_links
                .iter()
                .filter_map(|&(jid, gsid)| (gsid == sid).then_some(jid))
                .collect();
            let mut pids: Vec<Uuid> = cs
                .job_pain_links
                .iter()
                .filter_map(|&(pid, jid)| jobs.contains(&jid).then_some(pid))
                .collect();
            pids.sort_unstable();
            pids.dedup();
            let mut gids: Vec<Uuid> = cs
                .job_gain_links
                .iter()
                .filter_map(|&(gid, jid)| jobs.contains(&jid).then_some(gid))
                .collect();
            gids.sort_unstable();
            gids.dedup();
            (Some(pids), Some(gids))
        } else {
            (None, None)
        };

    let count = |anns: &[&ValueAnnotation], relevant: &Option<Vec<Uuid>>| -> (usize, usize) {
        let filtered: Vec<&&ValueAnnotation> = if let Some(ids) = relevant {
            anns.iter()
                .filter(|ann| ids.contains(&ann.pain_or_gain_id))
                .collect()
        } else {
            anns.iter().collect()
        };
        let total = filtered.len();
        let met = filtered
            .iter()
            .filter(|ann| ann.strength >= TABLE_STAKE_MIN_STRENGTH)
            .count();
        (met, total)
    };

    let (pain_met, pain_total) = count(&ts_pain, &relevant_pain_ids);
    let (gain_met, gain_total) = count(&ts_gain, &relevant_gain_ids);

    (pain_met + gain_met, pain_total + gain_total)
}

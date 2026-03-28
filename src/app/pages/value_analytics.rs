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

/// Fallback importance used when a pain or gain entity cannot be found in
/// state (e.g. a link table references an entity that was deleted).
///
/// Using `0.0` would silently zero-weight the orphaned need; using this
/// mid-range value keeps it visible in the score at half weight so the
/// gap is not hidden from the user.
pub const DEFAULT_NEED_IMPORTANCE: f32 = 0.5;

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

impl GapGroups {
    /// Returns `true` when no coverages were placed in any group — i.e. the
    /// segment has no annotated needs, or all needs have adequate coverage.
    pub fn is_empty(&self) -> bool {
        self.uncovered.is_empty()
            && self.weak.is_empty()
            && self.incomplete_table_stakes.is_empty()
            && self.strong_differentiators.is_empty()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Sorts and deduplicates a `Vec<Uuid>` in-place and returns it.
/// Pains and gains can be shared by multiple jobs, so deduplication is needed
/// before iterating to avoid double-counting.
fn sort_dedup(mut v: Vec<Uuid>) -> Vec<Uuid> {
    v.sort_unstable();
    v.dedup();
    v
}

/// Collects the destination IDs from a `(source, destination)` link table
/// where the source is contained in `source_ids`.
///
/// Used to walk one hop along the entity graph: features → relievers,
/// features → creators, jobs → pains, jobs → gains, etc.
fn linked_ids_where_source_in(links: &[(Uuid, Uuid)], source_ids: &[Uuid]) -> Vec<Uuid> {
    links
        .iter()
        .filter_map(|&(src, dst)| source_ids.contains(&src).then_some(dst))
        .collect()
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
    linked_ids_where_source_in(&app.valueprop_page.feature_pain_relief_links, feature_ids)
}

/// Returns the creators (gain creator items) reachable from a set of features.
fn creators_of_features(feature_ids: &[Uuid], app: &App) -> Vec<Uuid> {
    linked_ids_where_source_in(&app.valueprop_page.feature_gain_creator_links, feature_ids)
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
    // `partial_cmp` returns `None` for NaN; treat NaN as equal so max_by
    // remains stable even if a strength value was never set (defaults to 0.0
    // in practice, but defensive against future data-loading edge cases).
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
    let all_pain_ids = sort_dedup(pain_ids);
    let all_gain_ids = sort_dedup(gain_ids);

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
            .map_or(DEFAULT_NEED_IMPORTANCE, |p| p.importance);
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
            .map_or(DEFAULT_NEED_IMPORTANCE, |g| g.importance);
        let strength = best_strength_for_need(gid, false, product_id, app).map_or(0.0, |(s, _)| s);
        weight_sum += importance;
        weighted_strength_sum += importance * strength;
    }

    // Theoretically reachable only if every pain/gain has importance == 0.0.
    // Return NaN so callers fall back to the binary-BFS score rather than
    // silently showing 0% fit for a product that may still be relevant.
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

    let pain_ids: Vec<Uuid> = sort_dedup(
        cs.job_pain_links
            .iter()
            .filter_map(|&(pid, jid)| jobs.contains(&jid).then_some(pid))
            .collect(),
    );
    let gain_ids: Vec<Uuid> = sort_dedup(
        cs.job_gain_links
            .iter()
            .filter_map(|&(gid, jid)| jobs.contains(&jid).then_some(gid))
            .collect(),
    );

    let mut coverages = Vec::with_capacity(pain_ids.len() + gain_ids.len());

    // Helper to build a NeedCoverage from an id + entity fields.
    // Avoids repeating the identical struct construction for pains and gains.
    let mut push = |id: Uuid, name: &str, importance: f32, is_pain: bool| {
        let best = best_strength_for_need(id, is_pain, product_id, app);
        coverages.push(NeedCoverage {
            id,
            name: name.to_owned(),
            importance,
            is_pain,
            effective_strength: best.map(|(s, _)| s),
            value_type: best.map(|(_, vt)| vt),
        });
    };

    for pid in pain_ids {
        if let Some(p) = cs.pains_state.pains.iter().find(|p| p.id == pid) {
            push(pid, &p.name, p.importance, true);
        }
    }
    for gid in gain_ids {
        if let Some(g) = cs.gains_state.gains.iter().find(|g| g.id == gid) {
            push(gid, &g.name, g.importance, false);
        }
    }

    coverages
}

/// Groups a slice of `NeedCoverage` into the four actionable categories.
///
/// The match arms are ordered by priority — each arm is only reached when all
/// earlier arms failed to match:
/// 1. `uncovered` — no annotation at all; highest priority.
/// 2. `incomplete_table_stakes` — Table Stake annotated but below min viable
///    strength; checked before the generic `weak` arm so a weak Table Stake
///    is flagged for its specific risk (product viability) rather than lumped
///    with ordinary weak Differentiators.
/// 3. `strong_differentiators` — a Differentiator at or above the strong
///    threshold; surfaces genuine competitive advantages.
/// 4. `weak` — any remaining coverage below the weak threshold; catches weak
///    Differentiators and any annotations without a `ValueType` set.
/// 5. catch-all `_` — adequate coverage; nothing to flag.
pub fn compute_gap_groups(coverages: Vec<NeedCoverage>) -> GapGroups {
    let mut groups = GapGroups {
        uncovered: Vec::new(),
        weak: Vec::new(),
        incomplete_table_stakes: Vec::new(),
        strong_differentiators: Vec::new(),
    };

    for c in coverages {
        match (c.effective_strength, c.value_type) {
            // 1. No annotation from this product — need is completely unaddressed.
            (None, _) => groups.uncovered.push(c),
            // 2. Table Stake below minimum viable strength — product is at risk.
            (Some(s), Some(ValueType::TableStake)) if s < TABLE_STAKE_MIN_STRENGTH => {
                groups.incomplete_table_stakes.push(c);
            }
            // 3. Strong Differentiator — highlight as a source of competitive advantage.
            (Some(s), Some(ValueType::Differentiator)) if s >= DIFFERENTIATOR_STRONG_THRESHOLD => {
                groups.strong_differentiators.push(c);
            }
            // 4. Weak coverage — below the weak threshold but not a Table Stake
            //    (Table Stakes are caught above regardless of weakness level).
            (Some(s), _) if s < STRENGTH_WEAK_THRESHOLD => groups.weak.push(c),
            // 5. Adequate coverage — not a gap, nothing to surface.
            _ => {}
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
            let pids = sort_dedup(
                cs.job_pain_links
                    .iter()
                    .filter_map(|&(pid, jid)| jobs.contains(&jid).then_some(pid))
                    .collect(),
            );
            let gids = sort_dedup(
                cs.job_gain_links
                    .iter()
                    .filter_map(|&(gid, jid)| jobs.contains(&jid).then_some(gid))
                    .collect(),
            );
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

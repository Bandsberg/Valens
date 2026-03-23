use std::str::FromStr as _;

use crate::App;
use crate::app::Tab;
use crate::app::ValuePropPage;
use crate::app::pages::CustomerSegment;
use crate::app::pages::Gain;
use crate::app::pages::Job;
use crate::app::pages::Pain;
use crate::app::pages::product::Feature;
use crate::app::pages::product::GainCreator;
use crate::app::pages::product::PainRelief;
use crate::app::pages::product::ValueAnnotation;
use crate::app::pages::product::ValueType;
use crate::app::pages::product::products_window::Product;
use uuid::Uuid;

/// Merges `items` into `vec` positionally: overwrites index `i` if it exists,
/// otherwise pushes. This lets demo data survive across restarts without
/// accumulating duplicates when the user already has saved state.
fn upsert_items<T>(vec: &mut Vec<T>, items: impl IntoIterator<Item = T>) {
    for (i, item) in items.into_iter().enumerate() {
        if let Some(slot) = vec.get_mut(i) {
            *slot = item;
        } else {
            vec.push(item);
        }
    }
}

#[expect(clippy::too_many_lines)]
pub fn load_demo_data(cc: &eframe::CreationContext<'_>) -> App {
    let mut demo_app: App = if let Some(storage) = cc.storage {
        eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
    } else {
        App {
            tab: Tab::ValueProp,
            valueprop_page: ValuePropPage::default(),
            customer_segment_page: Default::default(),
        }
    };

    // ── Demo products ─────────────────────────────────────────────────────────
    // UUIDs are fixed (not Uuid::new_v4()) so that cross-entity link tables
    // persisted from a previous session remain valid after a restart.
    // Random UUIDs would silently orphan every stored link (product_feature_links,
    // pain_relief_annotations, etc.) the moment the demo data was reloaded.
    let product_1_id =
        Uuid::from_str("e3142c46-5ac5-4425-8080-a8faff6e3ae4").expect("hardcoded UUID is valid");
    let product_2_id =
        Uuid::from_str("93a7b2b5-ce26-4078-bce6-ca7d2d941b70").expect("hardcoded UUID is valid");

    let product_1 = Product {
        id: product_1_id,
        name: "Payment Rails API".to_owned(),
        description:
            "Embedded payment processing covering card, ACH, and SWIFT via a single integration"
                .to_owned(),
        notes: "Primary revenue product; pricing is per-transaction with a monthly platform fee"
            .to_owned(),
        expanded: false,
    };
    let product_2 = Product {
        id: product_2_id,
        name: "KYC & Compliance Suite".to_owned(),
        description:
            "Automated identity verification and AML screening with regulator-ready audit trails"
                .to_owned(),
        notes: "Sold standalone or bundled with Payment Rails API; banks require on-prem option"
            .to_owned(),
        expanded: false,
    };

    upsert_items(
        &mut demo_app.valueprop_page.products_state.products,
        [product_1, product_2],
    );

    // ── Demo features ─────────────────────────────────────────────────────────
    let feature_1_id =
        Uuid::from_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").expect("hardcoded UUID is valid");
    let feature_2_id =
        Uuid::from_str("b2c3d4e5-f6a7-8901-bcde-f12345678901").expect("hardcoded UUID is valid");
    let feature_3_id =
        Uuid::from_str("c3d4e5f6-a7b8-9012-cdef-012345678902").expect("hardcoded UUID is valid");

    let feature_1 = Feature {
        id: feature_1_id,
        name: "Unified payment API".to_owned(),
        description: "Single REST endpoint routing transactions across card, ACH, and SWIFT".to_owned(),
        status: "In Progress".to_owned(),
        notes: "ISO 20022 message format required for SWIFT; versioned endpoints for backwards compatibility".to_owned(),
        user_story: "As a fintech developer, I want one API to initiate any payment type so that I don't have to integrate multiple schemes separately.".to_owned(),
        acceptance_criteria: "- Supports card authorisation, ACH credit/debit, and SWIFT MT103\n- Response time < 200 ms at p99\n- Returns a unified transaction ID regardless of scheme".to_owned(),
        expanded: false,
    };
    let feature_2 = Feature {
        id: feature_2_id,
        name: "Automated KYC decisioning".to_owned(),
        description: "ML-driven identity checks that approve clean applicants in seconds".to_owned(),
        status: "In Progress".to_owned(),
        notes: "Manual review queue for edge cases; all decisions stored with reasoning for audit".to_owned(),
        user_story: "As a compliance officer, I want automated KYC decisions so that most applicants are approved instantly while edge cases are flagged for human review.".to_owned(),
        acceptance_criteria: "- Auto-approve rate > 85 % for standard applicants\n- Full decision audit trail exportable as PDF\n- Manual review SLA < 4 hours".to_owned(),
        expanded: false,
    };
    let feature_3 = Feature {
        id: feature_3_id,
        name: "Real-time fraud scoring".to_owned(),
        description: "Risk engine scoring every transaction in under 50 ms with tunable thresholds".to_owned(),
        status: "Draft".to_owned(),
        notes: "Per-merchant risk profiles allow threshold tuning without touching global model".to_owned(),
        user_story: "As a risk manager, I want to tune fraud thresholds per merchant so that I can reduce false positives without accepting more fraud losses.".to_owned(),
        acceptance_criteria: "- Score latency < 50 ms at p99\n- Merchant-level threshold configuration via dashboard\n- False-positive rate < 0.5 % at default settings".to_owned(),
        expanded: false,
    };

    upsert_items(
        &mut demo_app.valueprop_page.features_state.features,
        [feature_1, feature_2, feature_3],
    );

    // ── Demo product-feature links ────────────────────────────────────────────
    // Payment Rails API   → Unified payment API, Real-time fraud scoring
    // KYC & Compliance    → Automated KYC decisioning, Real-time fraud scoring
    let demo_prod_feat_links = [
        (product_1_id, feature_1_id),
        (product_1_id, feature_3_id),
        (product_2_id, feature_2_id),
        (product_2_id, feature_3_id),
    ];
    let existing_links = &mut demo_app.valueprop_page.product_feature_links;
    for link in demo_prod_feat_links {
        if !existing_links.contains(&link) {
            existing_links.push(link);
        }
    }

    // ── Demo pain relief items ────────────────────────────────────────────────
    let pr_1_id =
        Uuid::from_str("1a2b3c4d-5e6f-7890-1a2b-3c4d5e6f7890").expect("hardcoded UUID is valid");
    let pr_2_id =
        Uuid::from_str("2b3c4d5e-6f7a-8901-2b3c-4d5e6f7a8901").expect("hardcoded UUID is valid");
    let pr_3_id =
        Uuid::from_str("3c4d5e6f-7a8b-9012-3c4d-5e6f7a8b9012").expect("hardcoded UUID is valid");

    let pr_1 = PainRelief {
        id: pr_1_id,
        name: "Pre-built SDK & sandbox".to_owned(),
        description:
            "Language SDKs and a realistic sandbox environment cut integration from weeks to hours"
                .to_owned(),
        notes:
            "SDKs available for Node, Python, Java, and Go; sandbox mirrors production data shapes"
                .to_owned(),
        expanded: false,
    };
    let pr_2 = PainRelief {
        id: pr_2_id,
        name: "Instant KYC auto-approval".to_owned(),
        description: "Clean applicants are approved in seconds; only genuine edge cases reach manual review".to_owned(),
        notes: "Reduces perceived friction; banks appreciate the audit trail that accompanies each decision".to_owned(),
        expanded: false,
    };
    let pr_3 = PainRelief {
        id: pr_3_id,
        name: "Tunable risk thresholds".to_owned(),
        description: "Per-merchant fraud profiles reduce false positives without relaxing overall fraud controls".to_owned(),
        notes: "Threshold changes take effect without redeployment; A/B testing supported".to_owned(),
        expanded: false,
    };

    upsert_items(
        &mut demo_app.valueprop_page.pain_relief_state.pain_reliefs,
        [pr_1, pr_2, pr_3],
    );

    // ── Demo feature-pain-relief links ────────────────────────────────────────
    // Pre-built SDK         → Unified payment API
    // Instant KYC approval  → Automated KYC decisioning
    // Tunable thresholds    → Real-time fraud scoring
    let demo_feat_pr_links = [
        (feature_1_id, pr_1_id),
        (feature_2_id, pr_2_id),
        (feature_3_id, pr_3_id),
    ];
    let existing_feat_pr = &mut demo_app.valueprop_page.feature_pain_relief_links;
    for link in demo_feat_pr_links {
        if !existing_feat_pr.contains(&link) {
            existing_feat_pr.push(link);
        }
    }

    // ── Demo pain-pain-relief links ───────────────────────────────────────────
    // (pain_id, pain_relief_id) — matched by theme
    // Pain 1 (complex API)       → PR 1 (SDK & sandbox)
    // Pain 2 (KYC drop-off)      → PR 2 (instant KYC approval)
    // Pain 3 (false positives)   → PR 3 (tunable thresholds)

    // ── Demo customer segments ────────────────────────────────────────────────
    let segment_1_id =
        Uuid::from_str("c1d2e3f4-a5b6-7890-cdef-012345678901").expect("hardcoded UUID is valid");
    let segment_2_id =
        Uuid::from_str("d2e3f4a5-b6c7-8901-defa-123456789012").expect("hardcoded UUID is valid");

    let segment_1 = CustomerSegment {
        id: segment_1_id,
        name: "Fintech Startups".to_owned(),
        description: "Early-stage fintechs embedding financial services into their product"
            .to_owned(),
        notes: String::new(),
        characteristics: "Small dev teams, need fast API integration, cost-sensitive, move quickly"
            .to_owned(),
        expanded: false,
    };
    let segment_2 = CustomerSegment {
        id: segment_2_id,
        name: "Regional Banks".to_owned(),
        description: "Mid-sized banks modernising their infrastructure via BaaS".to_owned(),
        notes: String::new(),
        characteristics:
            "Strong compliance requirements, legacy system integration, risk-averse procurement"
                .to_owned(),
        expanded: false,
    };

    upsert_items(
        &mut demo_app.customer_segment_page.segments_state.segments,
        [segment_1, segment_2],
    );

    // ── Demo jobs ─────────────────────────────────────────────────────────────
    let job_1_id =
        Uuid::from_str("e1f2a3b4-c5d6-7890-efab-012345678901").expect("hardcoded UUID is valid");
    let job_2_id =
        Uuid::from_str("f2a3b4c5-d6e7-8901-fabc-123456789012").expect("hardcoded UUID is valid");
    let job_3_id =
        Uuid::from_str("a3b4c5d6-e7f8-9012-abcd-234567890123").expect("hardcoded UUID is valid");

    let job_1 = Job {
        id: job_1_id,
        name: "Integrate payment rails".to_owned(),
        description: "Connect to card schemes, ACH, or SWIFT via a BaaS provider API".to_owned(),
        notes: "Critical path for fintechs launching; compliance sign-off required for banks"
            .to_owned(),
        expanded: false,
    };
    let job_2 = Job {
        id: job_2_id,
        name: "Manage KYC & onboarding".to_owned(),
        description: "Verify customer identities and satisfy AML obligations at account opening"
            .to_owned(),
        notes: "Fintechs want automated flows; banks need audit trails for regulators".to_owned(),
        expanded: false,
    };
    let job_3 = Job {
        id: job_3_id,
        name: "Monitor transactions for fraud".to_owned(),
        description: "Detect and act on suspicious activity in real time".to_owned(),
        notes: "False-positive rate is a key concern for both segments".to_owned(),
        expanded: false,
    };

    upsert_items(
        &mut demo_app.customer_segment_page.jobs_state.jobs,
        [job_1, job_2, job_3],
    );

    // ── Demo segment-job links ────────────────────────────────────────────────
    // Fintechs: payment rails + KYC + fraud
    // Regional banks: KYC + fraud (payment rails handled by legacy core)
    let demo_seg_job_links = [
        (job_1_id, segment_1_id),
        (job_2_id, segment_1_id),
        (job_3_id, segment_1_id),
        (job_2_id, segment_2_id),
        (job_3_id, segment_2_id),
    ];
    let existing_seg_job_links = &mut demo_app.customer_segment_page.segment_job_links;
    for link in demo_seg_job_links {
        if !existing_seg_job_links.contains(&link) {
            existing_seg_job_links.push(link);
        }
    }

    // ── Demo pains ────────────────────────────────────────────────────────────
    let pain_1_id =
        Uuid::from_str("b4c5d6e7-f8a9-0123-bcde-345678901234").expect("hardcoded UUID is valid");
    let pain_2_id =
        Uuid::from_str("c5d6e7f8-a9b0-1234-cdef-456789012345").expect("hardcoded UUID is valid");
    let pain_3_id =
        Uuid::from_str("d6e7f8a9-b0c1-2345-defa-567890123456").expect("hardcoded UUID is valid");

    let pain_1 = Pain {
        id: pain_1_id,
        name: "Complex API integration".to_owned(),
        description: "BaaS APIs are inconsistent across providers, slowing time-to-market"
            .to_owned(),
        notes: "Fintechs cite this as the top bottleneck before launch".to_owned(),
        importance: 0.9,
        expanded: false,
    };
    let pain_2 = Pain {
        id: pain_2_id,
        name: "KYC drop-off rates".to_owned(),
        description: "Lengthy identity checks cause customers to abandon onboarding".to_owned(),
        notes: "Manual review steps are the main culprit for regional banks".to_owned(),
        importance: 0.8,
        expanded: false,
    };
    let pain_3 = Pain {
        id: pain_3_id,
        name: "High false-positive fraud alerts".to_owned(),
        description: "Legitimate transactions blocked, damaging customer trust".to_owned(),
        notes: "Both segments lose revenue and incur support costs from false positives".to_owned(),
        importance: 0.7,
        expanded: false,
    };

    upsert_items(
        &mut demo_app.customer_segment_page.pains_state.pains,
        [pain_1, pain_2, pain_3],
    );

    // ── Demo pain-job links ───────────────────────────────────────────────────
    let demo_pain_job_links = [
        (pain_1_id, job_1_id),
        (pain_2_id, job_2_id),
        (pain_3_id, job_3_id),
    ];
    let existing_pain_job_links = &mut demo_app.customer_segment_page.job_pain_links;
    for link in demo_pain_job_links {
        if !existing_pain_job_links.contains(&link) {
            existing_pain_job_links.push(link);
        }
    }

    // ── Demo pain-relief annotations (deferred until pains are loaded) ──────────
    // Pain 1 (complex API integration) → PR 1: table stake, strong coverage
    // Pain 2 (KYC drop-off rates)      → PR 2: table stake, strong coverage
    // Pain 3 (false-positive alerts)   → PR 3: differentiator, partial coverage
    let demo_pain_pr_annotations = [
        ValueAnnotation {
            pain_or_gain_id: pain_1_id,
            reliever_or_creator_id: pr_1_id,
            value_type: ValueType::TableStake,
            strength: 0.9,
        },
        ValueAnnotation {
            pain_or_gain_id: pain_2_id,
            reliever_or_creator_id: pr_2_id,
            value_type: ValueType::TableStake,
            strength: 0.85,
        },
        ValueAnnotation {
            pain_or_gain_id: pain_3_id,
            reliever_or_creator_id: pr_3_id,
            value_type: ValueType::Differentiator,
            strength: 0.7,
        },
    ];
    let existing_pain_pr = &mut demo_app.valueprop_page.pain_relief_annotations;
    for ann in demo_pain_pr_annotations {
        let exists = existing_pain_pr.iter().any(|a| {
            a.pain_or_gain_id == ann.pain_or_gain_id
                && a.reliever_or_creator_id == ann.reliever_or_creator_id
        });
        if !exists {
            existing_pain_pr.push(ann);
        }
    }

    // ── Demo gains ────────────────────────────────────────────────────────────
    let gain_1_id =
        Uuid::from_str("e7f8a9b0-c1d2-3456-efab-678901234567").expect("hardcoded UUID is valid");
    let gain_2_id =
        Uuid::from_str("f8a9b0c1-d2e3-4567-fabc-789012345678").expect("hardcoded UUID is valid");
    let gain_3_id =
        Uuid::from_str("a9b0c1d2-e3f4-5678-abcd-890123456789").expect("hardcoded UUID is valid");

    let gain_1 = Gain {
        id: gain_1_id,
        name: "Fast API go-live".to_owned(),
        description: "Launch payment capabilities in days with well-documented, consistent APIs"
            .to_owned(),
        notes: "Top priority for fintechs racing to market".to_owned(),
        importance: 0.9,
        expanded: false,
    };
    let gain_2 = Gain {
        id: gain_2_id,
        name: "High KYC pass rates".to_owned(),
        description: "More customers complete onboarding with minimal friction".to_owned(),
        notes: "Automated decisioning with clear audit trail satisfies both segments".to_owned(),
        importance: 0.8,
        expanded: false,
    };
    let gain_3 = Gain {
        id: gain_3_id,
        name: "Accurate fraud detection".to_owned(),
        description: "Catch real fraud while keeping false-positive rates low".to_owned(),
        notes: "Tunable risk thresholds are valued by regional banks".to_owned(),
        importance: 0.75,
        expanded: false,
    };

    upsert_items(
        &mut demo_app.customer_segment_page.gains_state.gains,
        [gain_1, gain_2, gain_3],
    );

    // ── Demo gain-job links ───────────────────────────────────────────────────
    let demo_gain_job_links = [
        (gain_1_id, job_1_id),
        (gain_2_id, job_2_id),
        (gain_3_id, job_3_id),
    ];
    let existing_gain_job_links = &mut demo_app.customer_segment_page.job_gain_links;
    for link in demo_gain_job_links {
        if !existing_gain_job_links.contains(&link) {
            existing_gain_job_links.push(link);
        }
    }

    // ── Demo gain creators ────────────────────────────────────────────────────
    let gc_1_id =
        Uuid::from_str("4d5e6f7a-8b9c-0123-4d5e-6f7a8b9c0123").expect("hardcoded UUID is valid");
    let gc_2_id =
        Uuid::from_str("5e6f7a8b-9c0d-1234-5e6f-7a8b9c0d1234").expect("hardcoded UUID is valid");
    let gc_3_id =
        Uuid::from_str("6f7a8b9c-0d1e-2345-6f7a-8b9c0d1e2345").expect("hardcoded UUID is valid");

    let gc_1 = GainCreator {
        id: gc_1_id,
        name: "One-day API go-live".to_owned(),
        description: "Comprehensive docs, runbooks, and a sandbox let devs reach production in a single sprint".to_owned(),
        notes: "Interactive API explorer in the developer portal accelerates discovery".to_owned(),
        expanded: false,
    };
    let gc_2 = GainCreator {
        id: gc_2_id,
        name: "Frictionless onboarding flow".to_owned(),
        description:
            "Pre-filled forms, progressive KYC steps, and instant auto-approval maximise conversion"
                .to_owned(),
        notes: "Mobile-optimised flow reduces drop-off on smaller screens".to_owned(),
        expanded: false,
    };
    let gc_3 = GainCreator {
        id: gc_3_id,
        name: "Transparent fraud analytics".to_owned(),
        description: "Real-time dashboard with explainable scores builds confidence for customers and regulators".to_owned(),
        notes: "Exportable reports satisfy auditor requests without engineering involvement".to_owned(),
        expanded: false,
    };

    upsert_items(
        &mut demo_app.valueprop_page.gain_creator_state.gain_creators,
        [gc_1, gc_2, gc_3],
    );

    // ── Demo feature-gain-creator links ───────────────────────────────────────
    // One-day API go-live       → Unified payment API
    // Frictionless onboarding   → Automated KYC decisioning
    // Transparent analytics     → Real-time fraud scoring
    let demo_feat_gc_links = [
        (feature_1_id, gc_1_id),
        (feature_2_id, gc_2_id),
        (feature_3_id, gc_3_id),
    ];
    let existing_feat_gc = &mut demo_app.valueprop_page.feature_gain_creator_links;
    for link in demo_feat_gc_links {
        if !existing_feat_gc.contains(&link) {
            existing_feat_gc.push(link);
        }
    }

    // ── Demo gain-creator annotations ─────────────────────────────────────────
    // Gain 1 (fast API go-live)       → GC 1 (one-day API go-live):       differentiator
    // Gain 2 (high KYC pass rates)    → GC 2 (frictionless onboarding):   differentiator
    // Gain 3 (accurate fraud detect.) → GC 3 (transparent analytics):     differentiator
    let demo_gain_gc_annotations = [
        ValueAnnotation {
            pain_or_gain_id: gain_1_id,
            reliever_or_creator_id: gc_1_id,
            value_type: ValueType::Differentiator,
            strength: 0.85,
        },
        ValueAnnotation {
            pain_or_gain_id: gain_2_id,
            reliever_or_creator_id: gc_2_id,
            value_type: ValueType::Differentiator,
            strength: 0.8,
        },
        ValueAnnotation {
            pain_or_gain_id: gain_3_id,
            reliever_or_creator_id: gc_3_id,
            value_type: ValueType::Differentiator,
            strength: 0.75,
        },
    ];
    let existing_gain_gc = &mut demo_app.valueprop_page.gain_creator_annotations;
    for ann in demo_gain_gc_annotations {
        let exists = existing_gain_gc.iter().any(|a| {
            a.pain_or_gain_id == ann.pain_or_gain_id
                && a.reliever_or_creator_id == ann.reliever_or_creator_id
        });
        if !exists {
            existing_gain_gc.push(ann);
        }
    }

    demo_app
}

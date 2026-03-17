use std::str::FromStr as _;

use crate::App;
use crate::app::ProductPage;
use crate::app::Tab;
use crate::app::pages::CustomerSegment;
use crate::app::pages::Gain;
use crate::app::pages::Job;
use crate::app::pages::Pain;
use crate::app::pages::product::Feature;
use crate::app::pages::product::products_window::Product;
use uuid::Uuid;

pub fn load_demo_data(cc: &eframe::CreationContext<'_>) -> App {
    let mut demo_app: App;
    if let Some(storage) = cc.storage {
        demo_app = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
    } else {
        demo_app = App {
            label: String::new(),
            value: 0.0,
            tab: Tab::Product,
            product_page: ProductPage::default(),
            customer_page: Default::default(),
        };
    }

    let product_1_id =
        Uuid::from_str("e3142c46-5ac5-4425-8080-a8faff6e3ae4").expect("hardcoded UUID is valid");
    let product_2_id =
        Uuid::from_str("93a7b2b5-ce26-4078-bce6-ca7d2d941b70").expect("hardcoded UUID is valid");

    let product_1 = Product {
        name: "Product 1".to_owned(),
        description: "My product description".to_owned(),
        id: product_1_id,
        notes: String::new(),
        expanded: false,
    };
    let product_2 = Product {
        name: "Product 2".to_owned(),
        description: "My product description".to_owned(),
        id: product_2_id,
        notes: String::new(),
        expanded: false,
    };

    let prod_vec = &mut demo_app.product_page.products_state.products;
    // To be able to keep a bit of state cache from any products added during demo,
    // while ensuring the first two products are always defined here.
    let demo_products = [product_1, product_2];
    for (i, product) in demo_products.into_iter().enumerate() {
        if let Some(slot) = prod_vec.get_mut(i) {
            *slot = product;
        } else {
            prod_vec.push(product);
        }
    }

    let feature_1_id =
        Uuid::from_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").expect("hardcoded UUID is valid");
    let feature_2_id =
        Uuid::from_str("b2c3d4e5-f6a7-8901-bcde-f12345678901").expect("hardcoded UUID is valid");

    let feature_1 = Feature {
        name: "Feature 1".to_owned(),
        description: "My feature description".to_owned(),
        id: feature_1_id,
        status: "Draft".to_owned(),
        notes: String::new(),
        user_story: String::new(),
        acceptance_criteria: String::new(),
        expanded: false,
    };
    let feature_2 = Feature {
        name: "Feature 2".to_owned(),
        description: "My feature description".to_owned(),
        id: feature_2_id,
        status: "Draft".to_owned(),
        notes: String::new(),
        user_story: String::new(),
        acceptance_criteria: String::new(),
        expanded: false,
    };

    let feat_vec = &mut demo_app.product_page.features_state.features;
    // Same pattern as products: keep any user-added features beyond the demo slots,
    // while ensuring the first two features are always defined here.
    let demo_features = [feature_1, feature_2];
    for (i, feature) in demo_features.into_iter().enumerate() {
        if let Some(slot) = feat_vec.get_mut(i) {
            *slot = feature;
        } else {
            feat_vec.push(feature);
        }
    }

    // ── Demo links ────────────────────────────────────────────────────────────
    // Product 1 uses both Feature 1 and Feature 2.
    // Product 2 uses Feature 2 (shared feature).
    // These are only inserted when no links exist yet, so user edits are kept.
    let demo_links = [
        (product_1_id, feature_1_id),
        (product_1_id, feature_2_id),
        (product_2_id, feature_2_id),
    ];
    let existing_links = &mut demo_app.product_page.product_feature_links;
    for link in demo_links {
        if !existing_links.contains(&link) {
            existing_links.push(link);
        }
    }

    // ── Demo customer segments ────────────────────────────────────────────────
    let segment_1_id =
        Uuid::from_str("c1d2e3f4-a5b6-7890-cdef-012345678901").expect("hardcoded UUID is valid");
    let segment_2_id =
        Uuid::from_str("d2e3f4a5-b6c7-8901-defa-123456789012").expect("hardcoded UUID is valid");

    let segment_1 = CustomerSegment {
        id: segment_1_id,
        name: "Enterprise".to_owned(),
        description: "Large organisations with complex needs".to_owned(),
        notes: String::new(),
        characteristics: "500+ employees, multi-department procurement, long sales cycles"
            .to_owned(),
        expanded: false,
    };
    let segment_2 = CustomerSegment {
        id: segment_2_id,
        name: "SMB".to_owned(),
        description: "Small and medium-sized businesses".to_owned(),
        notes: String::new(),
        characteristics: "10–500 employees, faster decisions, price-sensitive".to_owned(),
        expanded: false,
    };

    let seg_vec = &mut demo_app.customer_page.segments_state.segments;
    let demo_segments = [segment_1, segment_2];
    for (i, segment) in demo_segments.into_iter().enumerate() {
        if let Some(slot) = seg_vec.get_mut(i) {
            *slot = segment;
        } else {
            seg_vec.push(segment);
        }
    }

    // ── Demo jobs ─────────────────────────────────────────────────────────────
    let job_1_id =
        Uuid::from_str("e1f2a3b4-c5d6-7890-efab-012345678901").expect("hardcoded UUID is valid");
    let job_2_id =
        Uuid::from_str("f2a3b4c5-d6e7-8901-fabc-123456789012").expect("hardcoded UUID is valid");
    let job_3_id =
        Uuid::from_str("a3b4c5d6-e7f8-9012-abcd-234567890123").expect("hardcoded UUID is valid");

    let job_1 = Job {
        id: job_1_id,
        name: "Evaluate vendors".to_owned(),
        description: "Compare and shortlist software vendors against requirements".to_owned(),
        notes: "Key decision makers are procurement and IT leads".to_owned(),
        expanded: false,
    };
    let job_2 = Job {
        id: job_2_id,
        name: "Onboard new team members".to_owned(),
        description: "Get new hires productive as quickly as possible".to_owned(),
        notes: "Pain point: scattered documentation across multiple tools".to_owned(),
        expanded: false,
    };
    let job_3 = Job {
        id: job_3_id,
        name: "Report progress to stakeholders".to_owned(),
        description: "Compile and present project status to leadership".to_owned(),
        notes: "Frequency varies: weekly for SMB, monthly for Enterprise".to_owned(),
        expanded: false,
    };

    let job_vec = &mut demo_app.customer_page.jobs_state.jobs;
    let demo_jobs = [job_1, job_2, job_3];
    for (i, job) in demo_jobs.into_iter().enumerate() {
        if let Some(slot) = job_vec.get_mut(i) {
            *slot = job;
        } else {
            job_vec.push(job);
        }
    }

    // ── Demo segment-job links ────────────────────────────────────────────────
    // Enterprise does all three jobs; SMB does job 2 and 3.
    let demo_seg_job_links = [
        (job_1_id, segment_1_id),
        (job_2_id, segment_1_id),
        (job_3_id, segment_1_id),
        (job_2_id, segment_2_id),
        (job_3_id, segment_2_id),
    ];
    let existing_seg_job_links = &mut demo_app.customer_page.segment_job_links;
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
        name: "Too many tools".to_owned(),
        description: "Context-switching between disconnected systems wastes time".to_owned(),
        notes: "Mentioned by 8 of 10 Enterprise interviewees".to_owned(),
        expanded: false,
    };
    let pain_2 = Pain {
        id: pain_2_id,
        name: "Stale documentation".to_owned(),
        description: "Docs go out of date quickly and cannot be trusted".to_owned(),
        notes: "Especially acute during onboarding".to_owned(),
        expanded: false,
    };
    let pain_3 = Pain {
        id: pain_3_id,
        name: "Manual reporting overhead".to_owned(),
        description: "Generating status reports takes hours of manual data gathering".to_owned(),
        notes: "SMB teams often skip reporting altogether due to effort".to_owned(),
        expanded: false,
    };

    let pain_vec = &mut demo_app.customer_page.pains_state.pains;
    let demo_pains = [pain_1, pain_2, pain_3];
    for (i, pain) in demo_pains.into_iter().enumerate() {
        if let Some(slot) = pain_vec.get_mut(i) {
            *slot = pain;
        } else {
            pain_vec.push(pain);
        }
    }

    // ── Demo pain-job links ───────────────────────────────────────────────────
    // Pain 1 (too many tools) → job 1 (evaluate vendors), job 2 (onboard)
    // Pain 2 (stale docs)     → job 2 (onboard)
    // Pain 3 (manual reports) → job 3 (report to stakeholders)
    let demo_pain_job_links = [
        (pain_1_id, job_1_id),
        (pain_1_id, job_2_id),
        (pain_2_id, job_2_id),
        (pain_3_id, job_3_id),
    ];
    let existing_pain_job_links = &mut demo_app.customer_page.job_pain_links;
    for link in demo_pain_job_links {
        if !existing_pain_job_links.contains(&link) {
            existing_pain_job_links.push(link);
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
        name: "Single source of truth".to_owned(),
        description: "All relevant information in one place, always up to date".to_owned(),
        notes: "Highest-ranked desired outcome across both segments".to_owned(),
        expanded: false,
    };
    let gain_2 = Gain {
        id: gain_2_id,
        name: "Faster onboarding".to_owned(),
        description: "New hires become productive within days, not weeks".to_owned(),
        notes: "SMB values speed; Enterprise values consistency".to_owned(),
        expanded: false,
    };
    let gain_3 = Gain {
        id: gain_3_id,
        name: "Automated reporting".to_owned(),
        description: "Status reports generated automatically from live data".to_owned(),
        notes: "Saves 2–4 hours per reporting cycle per team".to_owned(),
        expanded: false,
    };

    let gain_vec = &mut demo_app.customer_page.gains_state.gains;
    let demo_gains = [gain_1, gain_2, gain_3];
    for (i, gain) in demo_gains.into_iter().enumerate() {
        if let Some(slot) = gain_vec.get_mut(i) {
            *slot = gain;
        } else {
            gain_vec.push(gain);
        }
    }

    // ── Demo gain-job links ───────────────────────────────────────────────────
    // Gain 1 (single source of truth) → job 1 (evaluate vendors), job 2 (onboard)
    // Gain 2 (faster onboarding)      → job 2 (onboard)
    // Gain 3 (automated reporting)    → job 3 (report to stakeholders)
    let demo_gain_job_links = [
        (gain_1_id, job_1_id),
        (gain_1_id, job_2_id),
        (gain_2_id, job_2_id),
        (gain_3_id, job_3_id),
    ];
    let existing_gain_job_links = &mut demo_app.customer_page.job_gain_links;
    for link in demo_gain_job_links {
        if !existing_gain_job_links.contains(&link) {
            existing_gain_job_links.push(link);
        }
    }

    demo_app
}

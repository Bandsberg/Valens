use crate::app::App;
use crate::app::pages::accordion;
use eframe::egui;
use uuid::Uuid;

use super::super::products_window::navigate_to_feature;

// ── Detail panel window ───────────────────────────────────────────────────────

#[expect(clippy::too_many_lines)]
pub fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.valueprop_page.pain_relief_state.selected_id else {
        return;
    };

    // Snapshot linked features before the window closure.
    let linked_fids: Vec<Uuid> = app
        .valueprop_page
        .feature_pain_relief_links
        .iter()
        .filter(|(_, rid)| *rid == id)
        .map(|(fid, _)| *fid)
        .collect();

    let linked_features: Vec<(Uuid, String)> = app
        .valueprop_page
        .features_state
        .features
        .iter()
        .filter(|f| linked_fids.contains(&f.id))
        .map(|f| (f.id, f.name.clone()))
        .collect();

    let available_features: Vec<(Uuid, String)> = app
        .valueprop_page
        .features_state
        .features
        .iter()
        .filter(|f| !linked_fids.contains(&f.id))
        .map(|f| (f.id, f.name.clone()))
        .collect();

    // Snapshot linked pains before the window closure.
    let linked_pids: Vec<Uuid> = app
        .valueprop_page
        .pain_pain_relief_links
        .iter()
        .filter(|(_, rid)| *rid == id)
        .map(|(pid, _)| *pid)
        .collect();

    let linked_pains: Vec<(Uuid, String)> = app
        .customer_segment_page
        .pains_state
        .pains
        .iter()
        .filter(|p| linked_pids.contains(&p.id))
        .map(|p| (p.id, p.name.clone()))
        .collect();

    let available_pains: Vec<(Uuid, String)> = app
        .customer_segment_page
        .pains_state
        .pains
        .iter()
        .filter(|p| !linked_pids.contains(&p.id))
        .map(|p| (p.id, p.name.clone()))
        .collect();

    let mut feat_link_to_add: Option<(Uuid, Uuid)> = None;
    let mut feat_link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut pain_link_to_add: Option<(Uuid, Uuid)> = None;
    let mut pain_link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut navigate_to_feat: Option<Uuid> = None;

    let mut keep_open = true;
    egui::Window::new("Pain Relief Details")
        .collapsible(false)
        .resizable(true)
        .default_size([420.0, 600.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            let Some(item) = app
                .valueprop_page
                .pain_relief_state
                .pain_reliefs
                .iter_mut()
                .find(|r| r.id == id)
            else {
                ui.label("Pain relief item not found.");
                return;
            };

            egui::Grid::new("pain_relief_detail_grid")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .min_col_width(100.0)
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.add(egui::TextEdit::singleline(&mut item.name).desired_width(f32::INFINITY));
                    ui.end_row();

                    ui.label("Description:");
                    ui.add(
                        egui::TextEdit::singleline(&mut item.description)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Notes:");
                    ui.add(
                        egui::TextEdit::multiline(&mut item.notes)
                            .desired_rows(4)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    // ── Linked Features ───────────────────────────────────────
                    let (add, rem) = accordion::detail_link_row(
                        ui,
                        "Linked\nFeatures:",
                        egui::Id::new("pr_detail_link_feat").with(id),
                        "Add a feature…",
                        &available_features,
                        &linked_features,
                        &mut navigate_to_feat,
                        Some("Open in Features"),
                    );
                    // Link tuple: (feature_id, pain_relief_id).
                    if let Some(fid) = add {
                        feat_link_to_add = Some((fid, id));
                    }
                    if let Some(fid) = rem {
                        feat_link_to_remove = Some((fid, id));
                    }
                    ui.end_row();

                    // ── Relieves Pains ────────────────────────────────────────
                    let mut _nav_unused = None;
                    let (add, rem) = accordion::detail_link_row(
                        ui,
                        "Relieves\nPains:",
                        egui::Id::new("pr_detail_link_pain").with(id),
                        "Add a pain…",
                        &available_pains,
                        &linked_pains,
                        &mut _nav_unused,
                        None,
                    );
                    // Link tuple: (pain_id, pain_relief_id).
                    if let Some(pid) = add {
                        pain_link_to_add = Some((pid, id));
                    }
                    if let Some(pid) = rem {
                        pain_link_to_remove = Some((pid, id));
                    }
                    ui.end_row();
                });
        });

    if !keep_open {
        app.valueprop_page.pain_relief_state.selected_id = None;
    }

    if let Some(pair) = feat_link_to_add
        && !app.valueprop_page.feature_pain_relief_links.contains(&pair)
    {
        app.valueprop_page.feature_pain_relief_links.push(pair);
    }
    if let Some(pair) = feat_link_to_remove {
        app.valueprop_page
            .feature_pain_relief_links
            .retain(|l| l != &pair);
    }
    if let Some(pair) = pain_link_to_add
        && !app.valueprop_page.pain_pain_relief_links.contains(&pair)
    {
        app.valueprop_page.pain_pain_relief_links.push(pair);
    }
    if let Some(pair) = pain_link_to_remove {
        app.valueprop_page
            .pain_pain_relief_links
            .retain(|l| l != &pair);
    }
    if let Some(feat_id) = navigate_to_feat {
        navigate_to_feature(app, ctx, feat_id);
    }
}

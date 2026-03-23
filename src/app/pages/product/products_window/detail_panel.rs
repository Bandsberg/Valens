use crate::app::App;
use eframe::egui;
use uuid::Uuid;

use super::super::super::accordion;

// ── Detail panel window (Panel mode) ─────────────────────────────────────────

/// Renders the Product/Service detail panel window for the currently selected
/// product. Shows editable fields (name, description, notes) and the list of
/// linked features. Does nothing when no product is selected.
pub fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.valueprop_page.products_state.selected_id else {
        return;
    };

    // Snapshot linked / available features before entering the window closure
    // so we can borrow `products_state.products` mutably inside without conflict.
    // Link tuple: (product_id, feature_id) — product is in first position.
    let (linked_features, available_features) = accordion::partition_linked(
        &app.valueprop_page.product_feature_links,
        |(pid, fid)| (*pid == id).then_some(*fid),
        &app.valueprop_page.features_state.features,
        |f| f.id,
        |f| f.name.as_str(),
    );

    // Collect mutations during the window; apply them afterwards.
    let mut link_to_add: Option<(Uuid, Uuid)> = None;
    let mut link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut navigate_to_feat: Option<Uuid> = None;

    let mut keep_open = true;
    egui::Window::new("Product/Service Details")
        .collapsible(false)
        .resizable(true)
        .default_size([420.0, 380.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            let Some(product) = app
                .valueprop_page
                .products_state
                .products
                .iter_mut()
                .find(|p| p.id == id)
            else {
                ui.label("Product not found.");
                return;
            };

            egui::Grid::new("product_detail_grid")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .min_col_width(100.0)
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.add(
                        egui::TextEdit::singleline(&mut product.name).desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Description:");
                    ui.add(
                        egui::TextEdit::singleline(&mut product.description)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Notes:");
                    ui.add(
                        egui::TextEdit::multiline(&mut product.notes)
                            .desired_rows(5)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    // ── Linked Features ──────────────────────────────────────
                    let (add, rem) = accordion::detail_link_row(
                        ui,
                        "Linked\nFeatures:",
                        egui::Id::new("prod_detail_link_feat").with(id),
                        "Add a feature…",
                        &available_features,
                        &linked_features,
                        &mut navigate_to_feat,
                        Some("Open in Features"),
                    );
                    // Link tuple: (product_id, feature_id).
                    if let Some(fid) = add {
                        link_to_add = Some((id, fid));
                    }
                    if let Some(fid) = rem {
                        link_to_remove = Some((id, fid));
                    }
                    ui.end_row();
                });
        });

    // User dismissed with ✕ → deselect.
    if !keep_open {
        app.valueprop_page.products_state.selected_id = None;
    }

    // Apply mutations now that the closure has released all borrows.
    if let Some(pair) = link_to_add
        && !app.valueprop_page.product_feature_links.contains(&pair)
    {
        app.valueprop_page.product_feature_links.push(pair);
    }
    if let Some(pair) = link_to_remove {
        app.valueprop_page
            .product_feature_links
            .retain(|l| l != &pair);
    }
    if let Some(feat_id) = navigate_to_feat {
        navigate_to_feature(app, ctx, feat_id);
    }
}

// ── Navigation helper ─────────────────────────────────────────────────────────

/// Opens the Features window and ensures `feat_id` is visible regardless of
/// which expand mode is currently active:
/// - Accordion → sets `expanded = true` on the target feature row.
/// - Panel → sets `selected_id` so the detail window opens.
///
/// Both are applied so switching modes also works correctly.
pub fn navigate_to_feature(app: &mut App, ctx: &egui::Context, feat_id: Uuid) {
    app.valueprop_page.product_windows.features_open = true;
    if let Some(feat) = app
        .valueprop_page
        .features_state
        .features
        .iter_mut()
        .find(|f| f.id == feat_id)
    {
        feat.expanded = true;
    }
    app.valueprop_page.features_state.selected_id = Some(feat_id);
    app.valueprop_page.features_state.scroll_to_id = Some(feat_id);
    // Bring the Features window in front of all other windows.
    ctx.move_to_top(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("Features"),
    ));
}

use crate::app::App;
use crate::app::pages::accordion;
use eframe::egui;
use uuid::Uuid;

// ── Detail panel window (Panel mode) ─────────────────────────────────────────

#[expect(clippy::too_many_lines)]
pub fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.valueprop_page.features_state.selected_id else {
        return;
    };

    // Snapshot linked / available products before the window closure to avoid
    // borrow conflicts with the mutable borrow of features_state.features inside.
    // Link tuple: (product_id, feature_id) — feature is in second position.
    let (linked_products, available_products) = accordion::partition_linked(
        &app.valueprop_page.product_feature_links,
        |(pid, fid)| (*fid == id).then_some(*pid),
        &app.valueprop_page.products_state.products,
        |p| p.id,
        |p| p.name.as_str(),
    );

    // Collect mutations during the window; apply them afterwards.
    let mut link_to_add: Option<(Uuid, Uuid)> = None;
    let mut link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut navigate_to_prod: Option<Uuid> = None;

    let mut keep_open = true;
    egui::Window::new("Feature Details")
        .collapsible(false)
        .resizable(true)
        .default_size([420.0, 600.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            let Some(feature) = app
                .valueprop_page
                .features_state
                .features
                .iter_mut()
                .find(|f| f.id == id)
            else {
                ui.label("Feature not found.");
                return;
            };

            egui::Grid::new("feature_detail_grid")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .min_col_width(100.0)
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.add(
                        egui::TextEdit::singleline(&mut feature.name).desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Description:");
                    ui.add(
                        egui::TextEdit::singleline(&mut feature.description)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Status:");
                    ui.add(
                        egui::TextEdit::singleline(&mut feature.status)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Notes:");
                    ui.add(
                        egui::TextEdit::multiline(&mut feature.notes)
                            .desired_rows(4)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("User Story:");
                    ui.add(
                        egui::TextEdit::multiline(&mut feature.user_story)
                            .desired_rows(4)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Acceptance\nCriteria:");
                    ui.add(
                        egui::TextEdit::multiline(&mut feature.acceptance_criteria)
                            .desired_rows(4)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    // ── Used by Products ─────────────────────────────────────
                    let (add, rem) = accordion::detail_link_row(
                        ui,
                        "Used by\nProducts &\nServices:",
                        egui::Id::new("feat_detail_link_prod").with(id),
                        "Add a product…",
                        &available_products,
                        &linked_products,
                        &mut navigate_to_prod,
                        Some("Open in Products & Services"),
                    );
                    // Link tuple: (product_id, feature_id).
                    if let Some(pid) = add {
                        link_to_add = Some((pid, id));
                    }
                    if let Some(pid) = rem {
                        link_to_remove = Some((pid, id));
                    }
                    ui.end_row();
                });
        });

    // User dismissed with ✕ → deselect.
    if !keep_open {
        app.valueprop_page.features_state.selected_id = None;
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
    if let Some(prod_id) = navigate_to_prod {
        navigate_to_product(app, ctx, prod_id);
    }
}

// ── Navigation helper ─────────────────────────────────────────────────────────

/// Opens the Products window and ensures `prod_id` is visible regardless of
/// which expand mode is currently active:
/// - Accordion → sets `expanded = true` on the target product row.
/// - Panel → sets `selected_id` so the detail window opens.
///
/// Both are applied so switching modes also works correctly.
pub(super) fn navigate_to_product(app: &mut App, ctx: &egui::Context, prod_id: Uuid) {
    app.valueprop_page.product_windows.products_open = true;
    if let Some(prod) = app
        .valueprop_page
        .products_state
        .products
        .iter_mut()
        .find(|p| p.id == prod_id)
    {
        prod.expanded = true;
    }
    app.valueprop_page.products_state.selected_id = Some(prod_id);
    app.valueprop_page.products_state.scroll_to_id = Some(prod_id);
    // Bring the Products window in front of all other windows.
    ctx.move_to_top(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("Products & Services"),
    ));
}

use crate::app::App;
use eframe::egui;
use uuid::Uuid;

// ── Detail panel window (Panel mode) ─────────────────────────────────────────

pub fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.product_page.products_state.selected_product_id else {
        return;
    };

    // Snapshot linked / available features before entering the window closure
    // so we can borrow `products_state.products` mutably inside without conflict.
    let linked_fids: Vec<Uuid> = app
        .product_page
        .product_feature_links
        .iter()
        .filter(|(pid, _)| *pid == id)
        .map(|(_, fid)| *fid)
        .collect();

    let linked_features: Vec<(Uuid, String)> = app
        .product_page
        .features_state
        .features
        .iter()
        .filter(|f| linked_fids.contains(&f.id))
        .map(|f| (f.id, f.name.clone()))
        .collect();

    let available_features: Vec<(Uuid, String)> = app
        .product_page
        .features_state
        .features
        .iter()
        .filter(|f| !linked_fids.contains(&f.id))
        .map(|f| (f.id, f.name.clone()))
        .collect();

    // Collect mutations during the window; apply them afterwards.
    let mut link_to_add: Option<(Uuid, Uuid)> = None;
    let mut link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut navigate_to_feat: Option<Uuid> = None;

    let mut keep_open = true;
    egui::Window::new("Product Details")
        .collapsible(false)
        .resizable(true)
        .default_size([420.0, 380.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            let Some(product) = app
                .product_page
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
                    ui.label("Linked\nFeatures:");
                    ui.vertical(|ui| {
                        // List of linked features — name is a navigation link,
                        // ✕ button removes the link.
                        if linked_features.is_empty() {
                            ui.label(
                                egui::RichText::new("None")
                                    .italics()
                                    .color(ui.visuals().weak_text_color()),
                            );
                        } else {
                            for (fid, fname) in &linked_features {
                                ui.horizontal(|ui| {
                                    if ui.link(fname).on_hover_text("Open in Features").clicked() {
                                        navigate_to_feat = Some(*fid);
                                    }
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("✕")
                                                    .small()
                                                    .color(egui::Color32::from_rgb(200, 60, 60)),
                                            )
                                            .fill(egui::Color32::TRANSPARENT),
                                        )
                                        .on_hover_text("Remove link")
                                        .clicked()
                                    {
                                        link_to_remove = Some((id, *fid));
                                    }
                                });
                            }
                        }

                        // Dropdown to add a new link.
                        if !available_features.is_empty() {
                            ui.add_space(4.0);

                            // Use egui's per-id temp storage so the combo
                            // selection survives across frames until we act on it.
                            let combo_key = egui::Id::new("prod_detail_link_feat").with(id);
                            let mut sel: Uuid =
                                ui.data(|d| d.get_temp(combo_key).unwrap_or(Uuid::nil()));

                            let avail_w = ui.available_width();
                            egui::ComboBox::from_id_salt(combo_key)
                                .selected_text("Add a feature…")
                                .width(avail_w)
                                .show_ui(ui, |ui| {
                                    for (fid, fname) in &available_features {
                                        ui.selectable_value(&mut sel, *fid, fname);
                                    }
                                });

                            if sel != Uuid::nil() {
                                // A feature was chosen — queue the link and reset.
                                link_to_add = Some((id, sel));
                                ui.data_mut(|d| d.remove::<Uuid>(combo_key));
                            } else {
                                ui.data_mut(|d| d.insert_temp(combo_key, sel));
                            }
                        }
                    });
                    ui.end_row();
                });
        });

    // User dismissed with ✕ → deselect.
    if !keep_open {
        app.product_page.products_state.selected_product_id = None;
    }

    // Apply mutations now that the closure has released all borrows.
    if let Some(pair) = link_to_add {
        if !app.product_page.product_feature_links.contains(&pair) {
            app.product_page.product_feature_links.push(pair);
        }
    }
    if let Some(pair) = link_to_remove {
        app.product_page
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
///   - Accordion → sets `expanded = true` on the target feature row.
///   - Panel     → sets `selected_feature_id` so the detail window opens.
/// Both are applied so switching modes also works correctly.
pub fn navigate_to_feature(app: &mut App, ctx: &egui::Context, feat_id: Uuid) {
    app.product_page.product_windows.features_open = true;
    if let Some(feat) = app
        .product_page
        .features_state
        .features
        .iter_mut()
        .find(|f| f.id == feat_id)
    {
        feat.expanded = true;
    }
    app.product_page.features_state.selected_feature_id = Some(feat_id);
    app.product_page.features_state.scroll_to_id = Some(feat_id);
    // Bring the Features window in front of all other windows.
    ctx.move_to_top(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("Features"),
    ));
}

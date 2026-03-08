use crate::app::App;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use serde;
use uuid::Uuid;

use super::features_window::Feature;

const COLLAPSED_H: f32 = 30.0;
const EXPANDED_H: f32 = 130.0;
const MULTILINE_H: f32 = 60.0;
/// Height of one linked-item row (name + ✕ button).
const LINK_ROW_H: f32 = 22.0;

// ── Expand mode ───────────────────────────────────────────────────────────────

/// Controls how the per-row detail section is revealed.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq, Default)]
pub enum ExpandMode {
    /// Notes expand inline, making the row taller.
    #[default]
    Accordion,
    /// Notes open in a separate floating detail window.
    Panel,
}

// ── State structs ─────────────────────────────────────────────────────────────

/// Persistent and transient state for the Products table.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct ProductsState {
    /// All products.
    pub products: Vec<Product>,
    /// ID of the product awaiting delete confirmation.
    #[serde(skip)]
    pub pending_delete: Option<Uuid>,
    /// ID of the product whose detail window is open (Panel mode only, not persisted).
    #[serde(skip)]
    pub selected_product_id: Option<Uuid>,
    /// Which expand mode is currently active.
    pub expand_mode: ExpandMode,
}

/// A single product entry.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Product {
    /// Stable unique identifier.
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub notes: String,
    /// Whether this row is expanded in Accordion mode (UI state, not persisted).
    #[serde(skip)]
    pub expanded: bool,
}

// ── Delete confirmation dialog ────────────────────────────────────────────────

fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.product_page.products_state.pending_delete else {
        return;
    };

    let product_name = app
        .product_page
        .products_state
        .products
        .iter()
        .find(|p| p.id == id)
        .map(|p| {
            if p.name.is_empty() {
                "Unnamed product".to_owned()
            } else {
                p.name.clone()
            }
        })
        .unwrap_or_default();

    let mut keep_open = true;
    egui::Window::new("Delete product?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            ui.label(format!(
                "Are you sure you want to delete \"{product_name}\"?"
            ));
            ui.label("This action cannot be undone.");
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let delete_btn = ui.add(
                    egui::Button::new(egui::RichText::new("🗑  Delete").color(egui::Color32::WHITE))
                        .fill(egui::Color32::from_rgb(180, 40, 40)),
                );
                if delete_btn.clicked() {
                    // Remove all links associated with this product.
                    app.product_page
                        .product_feature_links
                        .retain(|(pid, _)| *pid != id);
                    app.product_page
                        .products_state
                        .products
                        .retain(|p| p.id != id);
                    app.product_page.products_state.pending_delete = None;
                }
                if ui.button("Cancel").clicked() {
                    app.product_page.products_state.pending_delete = None;
                }
            });
        });

    // User dismissed the dialog with ✕ → treat as cancel.
    if !keep_open {
        app.product_page.products_state.pending_delete = None;
    }
}

// ── Detail panel window (Panel mode) ─────────────────────────────────────────

fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
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
fn navigate_to_feature(app: &mut App, ctx: &egui::Context, feat_id: Uuid) {
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
    // Bring the Features window in front of all other windows.
    ctx.move_to_top(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("Features"),
    ));
}

// ── Accordion table ───────────────────────────────────────────────────────────

fn show_accordion(
    ui: &mut egui::Ui,
    state: &mut ProductsState,
    features: &[Feature],
    links: &mut Vec<(Uuid, Uuid)>,
    navigate_to: &mut Option<Uuid>,
) {
    let mut to_delete: Option<Uuid> = None;
    let mut link_to_add: Option<(Uuid, Uuid)> = None;
    let mut link_to_remove: Option<(Uuid, Uuid)> = None;

    // Snapshot links for reading inside row closures (avoids borrow conflict
    // with the mutable `links` we need to update afterwards).
    let links_snap = links.clone();

    TableBuilder::new(ui)
        .column(Column::exact(24.0)) // ▶ / ▼ toggle
        .column(Column::initial(170.0).resizable(true)) // Name
        .column(Column::remainder()) // Description + Notes + Linked Features
        .column(Column::exact(36.0)) // 🗑
        .header(20.0, |mut header| {
            header.col(|_ui| {});
            header.col(|ui| {
                ui.heading("Product name");
            });
            header.col(|ui| {
                ui.heading("Description");
            });
            header.col(|_ui| {});
        })
        .body(|mut body| {
            for product in &mut state.products {
                let id = product.id;
                let expanded = product.expanded;

                // Pre-compute linked feature IDs so we can size the row and
                // determine available features without borrowing `links` again.
                let linked_fids: Vec<Uuid> = links_snap
                    .iter()
                    .filter(|(pid, _)| *pid == id)
                    .map(|(_, fid)| *fid)
                    .collect();

                // Row height:
                //   base      = EXPANDED_H (description + separator + notes)
                //   separator = ~8 px
                //   label     = ~20 px   "Linked Features:"
                //   combo     = ~28 px   dropdown widget
                //   rows      = LINK_ROW_H × max(n_linked, 1)  (items or "None")
                //   padding   =  ~8 px
                let num_linked = linked_fids.len();
                let row_h = if expanded {
                    EXPANDED_H + 8.0 + 20.0 + 28.0 + (num_linked.max(1) as f32 * LINK_ROW_H) + 8.0
                } else {
                    COLLAPSED_H
                };

                body.row(row_h, |mut row| {
                    // ── Col 0 : toggle arrow ─────────────────────────────────
                    row.col(|ui| {
                        let arrow = if product.expanded { "▼" } else { "▶" };
                        let hover = if expanded { "Collapse" } else { "Expand" };
                        if ui
                            .add(egui::Button::new(arrow).fill(egui::Color32::TRANSPARENT))
                            .on_hover_text(hover)
                            .clicked()
                        {
                            product.expanded = !product.expanded;
                        }
                    });

                    // ── Col 1 : name ─────────────────────────────────────────
                    row.col(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut product.name)
                                .hint_text("Product name…"),
                        );
                    });

                    // ── Col 2 : description + (expanded) notes + linked features
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut product.description)
                                    .hint_text("Short description…"),
                            );
                            if expanded {
                                ui.separator();
                                ui.label("Notes:");
                                ui.add(
                                    egui::TextEdit::multiline(&mut product.notes)
                                        .desired_rows(3)
                                        .desired_width(ui.available_width())
                                        .min_size(egui::vec2(0.0, MULTILINE_H)),
                                );

                                // ── Linked Features ──────────────────────────
                                ui.separator();
                                ui.label("Linked Features:");

                                let available: Vec<&Feature> = features
                                    .iter()
                                    .filter(|f| !linked_fids.contains(&f.id))
                                    .collect();

                                if !available.is_empty() {
                                    // Use egui's per-id temp storage so the
                                    // selection survives across frames.
                                    let combo_key = egui::Id::new("prod_acc_link_feat").with(id);
                                    let mut sel: Uuid =
                                        ui.data(|d| d.get_temp(combo_key).unwrap_or(Uuid::nil()));

                                    let avail_w = ui.available_width();
                                    egui::ComboBox::from_id_salt(combo_key)
                                        .selected_text("Add a feature…")
                                        .width(avail_w)
                                        .show_ui(ui, |ui| {
                                            for feat in &available {
                                                ui.selectable_value(&mut sel, feat.id, &feat.name);
                                            }
                                        });

                                    if sel != Uuid::nil() {
                                        link_to_add = Some((id, sel));
                                        ui.data_mut(|d| d.remove::<Uuid>(combo_key));
                                    } else {
                                        ui.data_mut(|d| d.insert_temp(combo_key, sel));
                                    }
                                } else {
                                    // All features are already linked — show a
                                    // disabled placeholder so the layout is stable.
                                    ui.add_enabled(false, egui::Button::new("All features linked"));
                                }

                                // Linked features — name is a navigation link,
                                // ✕ button removes the link.
                                if !linked_fids.is_empty() {
                                    for fid in &linked_fids {
                                        if let Some(feat) = features.iter().find(|f| f.id == *fid) {
                                            ui.horizontal(|ui| {
                                                if ui
                                                    .link(&feat.name)
                                                    .on_hover_text("Open in Features")
                                                    .clicked()
                                                {
                                                    *navigate_to = Some(*fid);
                                                }
                                                if ui
                                                    .add(
                                                        egui::Button::new(
                                                            egui::RichText::new("✕").small().color(
                                                                egui::Color32::from_rgb(
                                                                    200, 60, 60,
                                                                ),
                                                            ),
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
                                } else {
                                    ui.label(
                                        egui::RichText::new("None")
                                            .italics()
                                            .color(ui.visuals().weak_text_color()),
                                    );
                                }
                            }
                        });
                    });

                    // ── Col 3 : delete ───────────────────────────────────────
                    row.col(|ui| {
                        if ui
                            .add(egui::Button::new("🗑").fill(egui::Color32::TRANSPARENT))
                            .on_hover_text("Delete product")
                            .clicked()
                        {
                            to_delete = Some(id);
                        }
                    });
                });
            }
        });

    // Apply deferred mutations.
    if let Some(id) = to_delete {
        state.pending_delete = Some(id);
    }
    if let Some(pair) = link_to_add {
        if !links.contains(&pair) {
            links.push(pair);
        }
    }
    if let Some(pair) = link_to_remove {
        links.retain(|l| l != &pair);
    }
}

// ── Panel-mode table ──────────────────────────────────────────────────────────

fn show_panel_table(ui: &mut egui::Ui, state: &mut ProductsState) {
    let mut to_delete: Option<Uuid> = None;
    // Two separate variables to avoid Option<Option<_>>.
    let mut do_select: Option<Uuid> = None;
    let mut do_deselect = false;

    // Snapshot before the closure so we can read it without re-borrowing state.
    let selected_id = state.selected_product_id;

    TableBuilder::new(ui)
        .column(Column::exact(24.0)) // ▶ / ▼ toggle
        .column(Column::initial(170.0).resizable(true)) // Name
        .column(Column::remainder()) // Description
        .column(Column::exact(36.0)) // 🗑
        .header(20.0, |mut header| {
            header.col(|_ui| {});
            header.col(|ui| {
                ui.heading("Product name");
            });
            header.col(|ui| {
                ui.heading("Description");
            });
            header.col(|_ui| {});
        })
        .body(|mut body| {
            for product in &mut state.products {
                let id = product.id;
                let is_selected = selected_id == Some(id);

                body.row(COLLAPSED_H, |mut row| {
                    // ── Col 0 : toggle arrow ─────────────────────────────────
                    row.col(|ui| {
                        let arrow = if is_selected { "▼" } else { "▶" };
                        let hover = if is_selected {
                            "Close details"
                        } else {
                            "Open details"
                        };
                        if ui
                            .add(egui::Button::new(arrow).fill(egui::Color32::TRANSPARENT))
                            .on_hover_text(hover)
                            .clicked()
                        {
                            if is_selected {
                                do_deselect = true;
                            } else {
                                do_select = Some(id);
                            }
                        }
                    });

                    // ── Col 1 : name ─────────────────────────────────────────
                    row.col(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut product.name)
                                .hint_text("Product name…"),
                        );
                    });

                    // ── Col 2 : description ──────────────────────────────────
                    row.col(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut product.description)
                                .hint_text("Short description…"),
                        );
                    });

                    // ── Col 3 : delete ───────────────────────────────────────
                    row.col(|ui| {
                        if ui
                            .add(egui::Button::new("🗑").fill(egui::Color32::TRANSPARENT))
                            .on_hover_text("Delete product")
                            .clicked()
                        {
                            to_delete = Some(id);
                        }
                    });
                });
            }
        });

    if let Some(id) = to_delete {
        state.pending_delete = Some(id);
    }
    if do_deselect {
        state.selected_product_id = None;
    } else if let Some(id) = do_select {
        state.selected_product_id = Some(id);
    }
}

// ── Main entry point ──────────────────────────────────────────────────────────

/// Renders the Products floating window (and any subordinate windows it spawns).
pub fn show_products_window(app: &mut App, ctx: &egui::Context) {
    // These must be rendered before the main window so they sit on top.
    show_delete_confirmation(app, ctx);
    show_detail_panel(app, ctx);

    // Collected inside the window closure; applied after it releases borrows.
    let mut nav_to_feat: Option<Uuid> = None;

    egui::Window::new("Products")
        .open(&mut app.product_page.product_windows.products_open)
        .default_size([720.0, 380.0])
        .show(ctx, |ui| {
            ui.heading("Products");

            // ── Mode toggle ───────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Expand style:");
                ui.selectable_value(
                    &mut app.product_page.products_state.expand_mode,
                    ExpandMode::Accordion,
                    "▶  Accordion",
                );
                ui.selectable_value(
                    &mut app.product_page.products_state.expand_mode,
                    ExpandMode::Panel,
                    "▶  Detail Panel",
                );
            });

            ui.add_space(4.0);

            if ui.button("➕ Add Product").clicked() {
                app.product_page.products_state.products.push(Product {
                    id: Uuid::new_v4(),
                    ..Default::default()
                });
            }

            ui.separator();

            match app.product_page.products_state.expand_mode {
                ExpandMode::Accordion => {
                    // Split borrows across different ProductPage fields.
                    let features = &app.product_page.features_state.features;
                    let links = &mut app.product_page.product_feature_links;
                    show_accordion(
                        ui,
                        &mut app.product_page.products_state,
                        features,
                        links,
                        &mut nav_to_feat,
                    );
                }
                ExpandMode::Panel => {
                    show_panel_table(ui, &mut app.product_page.products_state);
                }
            }
        });

    // Apply navigation now that the window closure has released all borrows.
    if let Some(feat_id) = nav_to_feat {
        navigate_to_feature(app, ctx, feat_id);
    }
}

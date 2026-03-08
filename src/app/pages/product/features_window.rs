use crate::app::App;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use uuid::Uuid;

use super::ExpandMode;
use super::products_window::Product;

const COLLAPSED_H: f32 = 30.0;
const EXPANDED_H: f32 = 240.0;
const MULTILINE_H: f32 = 58.0;
/// Height of one linked-item row (name + ✕ button).
const LINK_ROW_H: f32 = 22.0;

// ── State structs ─────────────────────────────────────────────────────────────

/// Persistent and transient state for the Features table.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct FeaturesState {
    /// All features.
    pub features: Vec<Feature>,
    /// ID of the feature awaiting delete confirmation.
    #[serde(skip)]
    pub pending_delete: Option<Uuid>,
    /// ID of the feature whose detail window is open (Panel mode only, not persisted).
    #[serde(skip)]
    pub selected_feature_id: Option<Uuid>,
    /// Which expand mode is currently active.
    pub expand_mode: ExpandMode,
}

/// A single feature entry.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Feature {
    /// Stable unique identifier.
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: String,
    pub notes: String,
    pub user_story: String,
    pub acceptance_criteria: String,
    /// Whether this row is expanded in Accordion mode (UI state, not persisted).
    #[serde(skip)]
    pub expanded: bool,
}

// ── Delete confirmation dialog ────────────────────────────────────────────────

fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.product_page.features_state.pending_delete else {
        return;
    };

    let feature_name = app
        .product_page
        .features_state
        .features
        .iter()
        .find(|f| f.id == id)
        .map(|f| {
            if f.name.is_empty() {
                "Unnamed feature".to_owned()
            } else {
                f.name.clone()
            }
        })
        .unwrap_or_default();

    let mut keep_open = true;
    egui::Window::new("Delete feature?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            ui.label(format!(
                "Are you sure you want to delete \"{feature_name}\"?"
            ));
            ui.label("This action cannot be undone.");
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let delete_btn = ui.add(
                    egui::Button::new(egui::RichText::new("🗑  Delete").color(egui::Color32::WHITE))
                        .fill(egui::Color32::from_rgb(180, 40, 40)),
                );
                if delete_btn.clicked() {
                    // Remove all links associated with this feature.
                    app.product_page
                        .product_feature_links
                        .retain(|(_, fid)| *fid != id);
                    app.product_page
                        .features_state
                        .features
                        .retain(|f| f.id != id);
                    app.product_page.features_state.pending_delete = None;
                }
                if ui.button("Cancel").clicked() {
                    app.product_page.features_state.pending_delete = None;
                }
            });
        });

    // User dismissed the dialog with ✕ → treat as cancel.
    if !keep_open {
        app.product_page.features_state.pending_delete = None;
    }
}

// ── Detail panel window (Panel mode) ─────────────────────────────────────────

fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.product_page.features_state.selected_feature_id else {
        return;
    };

    // Snapshot linked / available products before the window closure to avoid
    // borrow conflicts with the mutable borrow of features_state.features inside.
    let linked_pids: Vec<Uuid> = app
        .product_page
        .product_feature_links
        .iter()
        .filter(|(_, fid)| *fid == id)
        .map(|(pid, _)| *pid)
        .collect();

    let linked_products: Vec<(Uuid, String)> = app
        .product_page
        .products_state
        .products
        .iter()
        .filter(|p| linked_pids.contains(&p.id))
        .map(|p| (p.id, p.name.clone()))
        .collect();

    let available_products: Vec<(Uuid, String)> = app
        .product_page
        .products_state
        .products
        .iter()
        .filter(|p| !linked_pids.contains(&p.id))
        .map(|p| (p.id, p.name.clone()))
        .collect();

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
                .product_page
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
                    ui.label("Used by\nProducts:");
                    ui.vertical(|ui| {
                        // List of linked products — name is a navigation link,
                        // ✕ button removes the link.
                        if linked_products.is_empty() {
                            ui.label(
                                egui::RichText::new("None")
                                    .italics()
                                    .color(ui.visuals().weak_text_color()),
                            );
                        } else {
                            for (pid, pname) in &linked_products {
                                ui.horizontal(|ui| {
                                    if ui.link(pname).on_hover_text("Open in Products").clicked() {
                                        navigate_to_prod = Some(*pid);
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
                                        link_to_remove = Some((*pid, id));
                                    }
                                });
                            }
                        }

                        // Dropdown to add a new link.
                        if !available_products.is_empty() {
                            ui.add_space(4.0);

                            // Use egui's per-id temp storage so the combo
                            // selection survives across frames until we act on it.
                            let combo_key = egui::Id::new("feat_detail_link_prod").with(id);
                            let mut sel: Uuid =
                                ui.data(|d| d.get_temp(combo_key).unwrap_or(Uuid::nil()));

                            let avail_w = ui.available_width();
                            egui::ComboBox::from_id_salt(combo_key)
                                .selected_text("Add a product…")
                                .width(avail_w)
                                .show_ui(ui, |ui| {
                                    for (pid, pname) in &available_products {
                                        ui.selectable_value(&mut sel, *pid, pname);
                                    }
                                });

                            if sel != Uuid::nil() {
                                // A product was chosen — queue the link and reset.
                                link_to_add = Some((sel, id));
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
        app.product_page.features_state.selected_feature_id = None;
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
    if let Some(prod_id) = navigate_to_prod {
        navigate_to_product(app, ctx, prod_id);
    }
}

// ── Navigation helper ─────────────────────────────────────────────────────────

/// Opens the Products window and ensures `prod_id` is visible regardless of
/// which expand mode is currently active:
///   - Accordion → sets `expanded = true` on the target product row.
///   - Panel     → sets `selected_product_id` so the detail window opens.
/// Both are applied so switching modes also works correctly.
fn navigate_to_product(app: &mut App, ctx: &egui::Context, prod_id: Uuid) {
    app.product_page.product_windows.products_open = true;
    if let Some(prod) = app
        .product_page
        .products_state
        .products
        .iter_mut()
        .find(|p| p.id == prod_id)
    {
        prod.expanded = true;
    }
    app.product_page.products_state.selected_product_id = Some(prod_id);
    // Bring the Products window in front of all other windows.
    ctx.move_to_top(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("Products"),
    ));
}

// ── Accordion table ───────────────────────────────────────────────────────────

fn show_accordion(
    ui: &mut egui::Ui,
    state: &mut FeaturesState,
    products: &[Product],
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
        .column(Column::initial(170.0).resizable(true)) // Name + left details
        .column(Column::remainder()) // Description + right details + linked products
        .column(Column::exact(36.0)) // 🗑
        .header(20.0, |mut header| {
            header.col(|_ui| {});
            header.col(|ui| {
                ui.heading("Feature name");
            });
            header.col(|ui| {
                ui.heading("Description");
            });
            header.col(|_ui| {});
        })
        .body(|mut body| {
            for feature in &mut state.features {
                let id = feature.id;
                let expanded = feature.expanded;

                // Pre-compute linked product IDs so we can size the row and
                // determine available products without borrowing `links` again.
                let linked_pids: Vec<Uuid> = links_snap
                    .iter()
                    .filter(|(_, fid)| *fid == id)
                    .map(|(pid, _)| *pid)
                    .collect();

                // Row height:
                //   base      = EXPANDED_H (description + separator + user story + AC)
                //   separator = ~8 px
                //   label     = ~20 px   "Used by Products:"
                //   combo     = ~28 px   dropdown widget
                //   rows      = LINK_ROW_H × max(n_linked, 1)  (items or "None")
                //   padding   =  ~8 px
                let num_linked = linked_pids.len();
                let row_h = if expanded {
                    EXPANDED_H + 8.0 + 20.0 + 28.0 + (num_linked.max(1) as f32 * LINK_ROW_H) + 8.0
                } else {
                    COLLAPSED_H
                };

                body.row(row_h, |mut row| {
                    // ── Col 0 : toggle arrow ─────────────────────────────────
                    row.col(|ui| {
                        let arrow = if feature.expanded { "▼" } else { "▶" };
                        let hover = if expanded { "Collapse" } else { "Expand" };
                        if ui
                            .add(egui::Button::new(arrow).fill(egui::Color32::TRANSPARENT))
                            .on_hover_text(hover)
                            .clicked()
                        {
                            feature.expanded = !feature.expanded;
                        }
                    });

                    // ── Col 1 : name  +  (expanded) status & notes ───────────
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut feature.name)
                                    .hint_text("Feature name…"),
                            );
                            if expanded {
                                ui.separator();
                                ui.label("Status:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut feature.status)
                                        .hint_text("e.g. Draft, In Progress, Done"),
                                );
                                ui.add_space(4.0);
                                ui.label("Notes:");
                                ui.add(
                                    egui::TextEdit::multiline(&mut feature.notes)
                                        .desired_rows(3)
                                        .desired_width(ui.available_width())
                                        .min_size(egui::vec2(0.0, MULTILINE_H)),
                                );
                            }
                        });
                    });

                    // ── Col 2 : description + (expanded) user story & AC + linked products
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut feature.description)
                                    .hint_text("Short description…"),
                            );
                            if expanded {
                                ui.separator();
                                ui.label("User Story:");
                                ui.add(
                                    egui::TextEdit::multiline(&mut feature.user_story)
                                        .desired_rows(3)
                                        .desired_width(ui.available_width())
                                        .min_size(egui::vec2(0.0, MULTILINE_H)),
                                );
                                ui.add_space(4.0);
                                ui.label("Acceptance Criteria:");
                                ui.add(
                                    egui::TextEdit::multiline(&mut feature.acceptance_criteria)
                                        .desired_rows(3)
                                        .desired_width(ui.available_width())
                                        .min_size(egui::vec2(0.0, MULTILINE_H)),
                                );

                                // ── Used by Products ─────────────────────────
                                ui.separator();
                                ui.label("Used by Products:");

                                let available: Vec<&Product> = products
                                    .iter()
                                    .filter(|p| !linked_pids.contains(&p.id))
                                    .collect();

                                if !available.is_empty() {
                                    // Use egui's per-id temp storage so the
                                    // selection survives across frames.
                                    let combo_key = egui::Id::new("feat_acc_link_prod").with(id);
                                    let mut sel: Uuid =
                                        ui.data(|d| d.get_temp(combo_key).unwrap_or(Uuid::nil()));

                                    let avail_w = ui.available_width();
                                    egui::ComboBox::from_id_salt(combo_key)
                                        .selected_text("Add a product…")
                                        .width(avail_w)
                                        .show_ui(ui, |ui| {
                                            for prod in &available {
                                                ui.selectable_value(&mut sel, prod.id, &prod.name);
                                            }
                                        });

                                    if sel != Uuid::nil() {
                                        link_to_add = Some((sel, id));
                                        ui.data_mut(|d| d.remove::<Uuid>(combo_key));
                                    } else {
                                        ui.data_mut(|d| d.insert_temp(combo_key, sel));
                                    }
                                } else {
                                    // All products are already linked — show a
                                    // disabled placeholder so the layout is stable.
                                    ui.add_enabled(false, egui::Button::new("All products linked"));
                                }

                                // Linked products — name is a navigation link,
                                // ✕ button removes the link.
                                if !linked_pids.is_empty() {
                                    for pid in &linked_pids {
                                        if let Some(prod) = products.iter().find(|p| p.id == *pid) {
                                            ui.horizontal(|ui| {
                                                if ui
                                                    .link(&prod.name)
                                                    .on_hover_text("Open in Products")
                                                    .clicked()
                                                {
                                                    *navigate_to = Some(*pid);
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
                                                    link_to_remove = Some((*pid, id));
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
                            .on_hover_text("Delete feature")
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

fn show_panel_table(ui: &mut egui::Ui, state: &mut FeaturesState) {
    let mut to_delete: Option<Uuid> = None;
    // Use two separate variables to avoid Option<Option<_>>.
    let mut do_select: Option<Uuid> = None;
    let mut do_deselect = false;

    // Snapshot before the closure so we can read it without re-borrowing state.
    let selected_id = state.selected_feature_id;

    TableBuilder::new(ui)
        .column(Column::exact(24.0)) // ▶ / ▼ toggle
        .column(Column::initial(170.0).resizable(true)) // Name
        .column(Column::remainder()) // Description
        .column(Column::exact(36.0)) // 🗑
        .header(20.0, |mut header| {
            header.col(|_ui| {});
            header.col(|ui| {
                ui.heading("Feature name");
            });
            header.col(|ui| {
                ui.heading("Description");
            });
            header.col(|_ui| {});
        })
        .body(|mut body| {
            for feature in &mut state.features {
                let id = feature.id;
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
                            egui::TextEdit::singleline(&mut feature.name)
                                .hint_text("Feature name…"),
                        );
                    });

                    // ── Col 2 : description ──────────────────────────────────
                    row.col(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut feature.description)
                                .hint_text("Short description…"),
                        );
                    });

                    // ── Col 3 : delete ───────────────────────────────────────
                    row.col(|ui| {
                        if ui
                            .add(egui::Button::new("🗑").fill(egui::Color32::TRANSPARENT))
                            .on_hover_text("Delete feature")
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
        state.selected_feature_id = None;
    } else if let Some(id) = do_select {
        state.selected_feature_id = Some(id);
    }
}

// ── Main entry point ──────────────────────────────────────────────────────────

/// Renders the Features floating window (and any subordinate windows it spawns).
pub fn show_features_window(app: &mut App, ctx: &egui::Context) {
    // These must be rendered before the main window so they sit on top.
    show_delete_confirmation(app, ctx);
    show_detail_panel(app, ctx);

    // Collected inside the window closure; applied after it releases borrows.
    let mut nav_to_prod: Option<Uuid> = None;

    egui::Window::new("Features")
        .open(&mut app.product_page.product_windows.features_open)
        .default_size([720.0, 380.0])
        .show(ctx, |ui| {
            ui.heading("Features");

            // ── Mode toggle ───────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Expand style:");
                ui.selectable_value(
                    &mut app.product_page.features_state.expand_mode,
                    ExpandMode::Accordion,
                    "▶  Accordion",
                );
                ui.selectable_value(
                    &mut app.product_page.features_state.expand_mode,
                    ExpandMode::Panel,
                    "▶  Detail Panel",
                );
            });

            ui.add_space(4.0);

            if ui.button("➕ Add Feature").clicked() {
                app.product_page.features_state.features.push(Feature {
                    id: Uuid::new_v4(),
                    ..Default::default()
                });
            }

            ui.separator();

            match app.product_page.features_state.expand_mode {
                ExpandMode::Accordion => {
                    // Split borrows across different ProductPage fields.
                    let products = &app.product_page.products_state.products;
                    let links = &mut app.product_page.product_feature_links;
                    show_accordion(
                        ui,
                        &mut app.product_page.features_state,
                        products,
                        links,
                        &mut nav_to_prod,
                    );
                }
                ExpandMode::Panel => {
                    show_panel_table(ui, &mut app.product_page.features_state);
                }
            }
        });

    // Apply navigation now that the window closure has released all borrows.
    if let Some(prod_id) = nav_to_prod {
        navigate_to_product(app, ctx, prod_id);
    }
}

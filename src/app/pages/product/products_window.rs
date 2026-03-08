use crate::app::App;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use serde;
use uuid::Uuid;

const COLLAPSED_H: f32 = 30.0;
const EXPANDED_H: f32 = 130.0;
const MULTILINE_H: f32 = 60.0;

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

    let mut keep_open = true;
    egui::Window::new("Product Details")
        .collapsible(false)
        .resizable(true)
        .default_size([380.0, 260.0])
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
                });
        });

    // User dismissed the detail window with ✕ → deselect.
    if !keep_open {
        app.product_page.products_state.selected_product_id = None;
    }
}

// ── Accordion table ───────────────────────────────────────────────────────────

fn show_accordion(ui: &mut egui::Ui, state: &mut ProductsState) {
    let mut to_delete: Option<Uuid> = None;

    TableBuilder::new(ui)
        .column(Column::exact(24.0)) // ▶ / ▼ toggle
        .column(Column::initial(170.0).resizable(true)) // Name
        .column(Column::remainder()) // Description + Notes
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
                // Read `expanded` once as a Copy bool so later column
                // closures don't need to re-borrow the same field.
                let expanded = product.expanded;
                let row_h = if expanded { EXPANDED_H } else { COLLAPSED_H };

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

                    // ── Col 2 : description  +  (expanded) notes ─────────────
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

    if let Some(id) = to_delete {
        state.pending_delete = Some(id);
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
                    show_accordion(ui, &mut app.product_page.products_state);
                }
                ExpandMode::Panel => {
                    show_panel_table(ui, &mut app.product_page.products_state);
                }
            }
        });
}

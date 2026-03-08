use crate::app::App;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use serde;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct ProductsState {
    pub products: Vec<Product>,
    /// ID of the product awaiting delete confirmation.
    #[serde(skip)]
    pub pending_delete: Option<Uuid>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Product {
    pub name: String,
    pub description: String,
    pub id: Uuid,
}

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

    // User dismissed the dialog with ✕ → treat as cancel
    if !keep_open {
        app.product_page.products_state.pending_delete = None;
    }
}

pub fn show_products_window(app: &mut App, ctx: &egui::Context) {
    // ── Confirmation dialog (rendered before the table window) ────────────────
    show_delete_confirmation(app, ctx);

    // ── Products list window ──────────────────────────────────────────────────
    egui::Window::new("Products")
        .scroll(true)
        .open(&mut app.product_page.product_windows.products_open)
        .default_size([600.0, 320.0])
        .show(ctx, |ui| {
            ui.heading("Products");
            ui.label("Your products content here…");
            if ui.button("➕ Add Product").clicked() {
                app.product_page.products_state.products.push(Product {
                    id: Uuid::new_v4(),
                    ..Default::default()
                });
            }
            ui.separator();

            // Capture which row's delete button was clicked outside the closure
            // so we can mutate state after the TableBuilder borrow ends.
            let mut to_delete: Option<Uuid> = None;

            TableBuilder::new(ui)
                .column(Column::auto().resizable(true)) // Product name
                .column(Column::remainder()) // Description
                .column(Column::exact(36.0)) // 🗑 button
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Product name");
                    });
                    header.col(|ui| {
                        ui.heading("Description");
                    });
                    header.col(|_ui| {});
                })
                .body(|mut body| {
                    for product in &mut app.product_page.products_state.products {
                        let id = product.id;
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut product.name);
                            });
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut product.description);
                            });
                            row.col(|ui| {
                                let btn = ui
                                    .add(egui::Button::new("🗑").fill(egui::Color32::TRANSPARENT))
                                    .on_hover_text("Delete product");
                                if btn.clicked() {
                                    to_delete = Some(id);
                                }
                            });
                        });
                    }
                });

            if let Some(id) = to_delete {
                app.product_page.products_state.pending_delete = Some(id);
            }
        });
}

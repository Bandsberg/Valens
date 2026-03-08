use crate::app::App;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use serde;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct ProductsState {
    // Define fields for the products window state
    pub products: Vec<Product>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Product {
    // Define fields for the product state
    pub name: String,
    pub description: String,
    pub id: Uuid,
}

pub fn show_products_window(app: &mut App, ctx: &egui::Context) {
    egui::Window::new("Products")
        .scroll(true)
        .open(&mut app.product_page.product_windows.products_open)
        .default_size([520.0, 320.0])
        .show(ctx, |ui| {
            ui.heading("Products");
            ui.label("Your products content here…");
            ui.separator();
            // Create a simple table
            TableBuilder::new(ui)
                .column(Column::auto().resizable(true))
                .column(Column::remainder())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Product name");
                    });
                    header.col(|ui| {
                        ui.heading("Second column");
                    });
                })
                .body(|mut body| {
                    for i in 0..app.product_page.products_state.products.len() {
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.text_edit_singleline(
                                    &mut app.product_page.products_state.products[i].name,
                                );
                            });
                            row.col(|ui| {
                                let _ = ui.button("world!");
                            });
                        });
                        /*
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Hello2");
                            });
                            row.col(|ui| {
                                ui.button("world!2");
                            });
                        });
                        */
                    }
                });
        });
}

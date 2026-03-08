use crate::app::App;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use serde;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct FeaturesState {
    pub features: Vec<Feature>,
    /// ID of the feature awaiting delete confirmation.
    #[serde(skip)]
    pub pending_delete: Option<Uuid>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Feature {
    pub name: String,
    pub description: String,
    pub id: Uuid,
}

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

    // User dismissed the dialog with ✕ → treat as cancel
    if !keep_open {
        app.product_page.features_state.pending_delete = None;
    }
}

pub fn show_features_window(app: &mut App, ctx: &egui::Context) {
    // ── Confirmation dialog (rendered before the table window) ────────────────
    show_delete_confirmation(app, ctx);

    // ── Features list window ──────────────────────────────────────────────────
    egui::Window::new("Features")
        .scroll(true)
        .open(&mut app.product_page.product_windows.features_open)
        .default_size([600.0, 320.0])
        .show(ctx, |ui| {
            ui.heading("Features");
            ui.label("Your features content here…");
            if ui.button("➕ Add Feature").clicked() {
                app.product_page.features_state.features.push(Feature {
                    id: Uuid::new_v4(),
                    ..Default::default()
                });
            }
            ui.separator();

            // Capture which row's delete button was clicked outside the closure
            // so we can mutate state after the TableBuilder borrow ends.
            let mut to_delete: Option<Uuid> = None;

            TableBuilder::new(ui)
                .column(Column::auto().resizable(true)) // Feature name
                .column(Column::remainder()) // Description
                .column(Column::exact(36.0)) // 🗑 button
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Feature name");
                    });
                    header.col(|ui| {
                        ui.heading("Description");
                    });
                    header.col(|_ui| {});
                })
                .body(|mut body| {
                    for feature in &mut app.product_page.features_state.features {
                        let id = feature.id;
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut feature.name);
                            });
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut feature.description);
                            });
                            row.col(|ui| {
                                let btn = ui
                                    .add(egui::Button::new("🗑").fill(egui::Color32::TRANSPARENT))
                                    .on_hover_text("Delete feature");
                                if btn.clicked() {
                                    to_delete = Some(id);
                                }
                            });
                        });
                    }
                });

            if let Some(id) = to_delete {
                app.product_page.features_state.pending_delete = Some(id);
            }
        });
}

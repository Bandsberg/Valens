use crate::app::App;
use crate::app::pages::accordion;
use eframe::egui;

// ── Delete confirmation dialog ────────────────────────────────────────────────

pub fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.product_page.products_state.pending_delete else {
        return;
    };

    let product_name = app
        .product_page
        .products_state
        .products
        .iter()
        .find(|p| p.id == id)
        .map(|p| accordion::display_name(&p.name, "Unnamed product").to_owned())
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

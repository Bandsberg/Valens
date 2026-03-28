use crate::app::App;
use crate::app::pages::accordion;
use eframe::egui;

// ── Delete confirmation dialog ────────────────────────────────────────────────

/// Shows the delete confirmation dialog for the pending product deletion.
///
/// On confirmation, removes the product and its `product_feature_links`, then
/// clears `selected_id` if the deleted product was open in the detail panel.
/// On dismissal, clears `pending_delete` without deleting anything.
pub fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.valueprop_page.products_state.pending_delete else {
        return;
    };

    let item_name = app
        .valueprop_page
        .products_state
        .products
        .iter()
        .find(|p| p.id == id)
        .map(|p| accordion::display_name(&p.name, "Unnamed product").to_owned())
        .unwrap_or_default();

    let (confirmed, dismissed) = accordion::delete_dialog(ctx, "Delete product?", &item_name);
    if confirmed {
        app.valueprop_page
            .product_feature_links
            .retain(|(pid, _)| *pid != id);
        app.valueprop_page
            .products_state
            .products
            .retain(|p| p.id != id);
        if app.valueprop_page.products_state.selected_id == Some(id) {
            app.valueprop_page.products_state.selected_id = None;
        }
    }
    if confirmed || dismissed {
        app.valueprop_page.products_state.pending_delete = None;
    }
}

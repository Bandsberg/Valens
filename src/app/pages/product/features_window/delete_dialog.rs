use crate::app::App;
use crate::app::pages::accordion;
use eframe::egui;

// ── Delete confirmation dialog ────────────────────────────────────────────────

pub fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.valueprop_page.features_state.pending_delete else {
        return;
    };

    let item_name = app
        .valueprop_page
        .features_state
        .features
        .iter()
        .find(|f| f.id == id)
        .map(|f| accordion::display_name(&f.name, "Unnamed feature").to_owned())
        .unwrap_or_default();

    let (confirmed, dismissed) = accordion::delete_dialog(ctx, "Delete feature?", &item_name);
    if confirmed {
        app.valueprop_page
            .product_feature_links
            .retain(|(_, fid)| *fid != id);
        app.valueprop_page
            .feature_pain_relief_links
            .retain(|(fid, _)| *fid != id);
        app.valueprop_page
            .feature_gain_creator_links
            .retain(|(fid, _)| *fid != id);
        app.valueprop_page
            .features_state
            .features
            .retain(|f| f.id != id);
    }
    if confirmed || dismissed {
        app.valueprop_page.features_state.pending_delete = None;
    }
}

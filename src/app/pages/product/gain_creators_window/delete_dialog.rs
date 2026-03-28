use crate::app::App;
use crate::app::pages::accordion;
use eframe::egui;

// ── Delete confirmation dialog ────────────────────────────────────────────────

/// Shows the delete confirmation dialog for the pending gain creator deletion.
///
/// On confirmation, removes the gain creator from `feature_gain_creator_links`
/// and `gain_creator_annotations`, then deletes the item itself and clears
/// `selected_id` if it was open in the detail panel. On dismissal, clears
/// `pending_delete` without deleting anything.
pub fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.valueprop_page.gain_creator_state.pending_delete else {
        return;
    };

    let item_name = app
        .valueprop_page
        .gain_creator_state
        .gain_creators
        .iter()
        .find(|r| r.id == id)
        .map(|r| accordion::display_name(&r.name, "Unnamed gain creator").to_owned())
        .unwrap_or_default();

    let (confirmed, dismissed) = accordion::delete_dialog(ctx, "Delete gain creator?", &item_name);
    if confirmed {
        app.valueprop_page
            .feature_gain_creator_links
            .retain(|(_, rid)| *rid != id);
        app.valueprop_page
            .gain_creator_annotations
            .retain(|ann| ann.reliever_or_creator_id != id);
        app.valueprop_page
            .gain_creator_state
            .gain_creators
            .retain(|r| r.id != id);
        if app.valueprop_page.gain_creator_state.selected_id == Some(id) {
            app.valueprop_page.gain_creator_state.selected_id = None;
        }
    }
    if confirmed || dismissed {
        app.valueprop_page.gain_creator_state.pending_delete = None;
    }
}

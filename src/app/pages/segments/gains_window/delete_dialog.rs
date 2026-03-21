use crate::app::App;
use eframe::egui;

use super::super::super::accordion;

pub fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.customer_segment_page.gains_state.pending_delete else {
        return;
    };

    let item_name = app
        .customer_segment_page
        .gains_state
        .gains
        .iter()
        .find(|g| g.id == id)
        .map(|g| accordion::display_name(&g.name, "Unnamed gain").to_owned())
        .unwrap_or_default();

    let (confirmed, dismissed) = accordion::delete_dialog(ctx, "Delete Gain?", &item_name);

    if confirmed {
        app.customer_segment_page
            .job_gain_links
            .retain(|(gid, _)| *gid != id);
        app.customer_segment_page
            .gains_state
            .gains
            .retain(|g| g.id != id);
        if app.customer_segment_page.gains_state.selected_id == Some(id) {
            app.customer_segment_page.gains_state.selected_id = None;
        }
    }
    if confirmed || dismissed {
        app.customer_segment_page.gains_state.pending_delete = None;
    }
}

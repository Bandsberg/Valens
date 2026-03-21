use crate::app::App;
use eframe::egui;

use super::super::super::accordion;

pub fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.customer_segment_page.gains_state.pending_delete else {
        return;
    };

    let gain_name = app
        .customer_segment_page
        .gains_state
        .gains
        .iter()
        .find(|g| g.id == id)
        .map(|g| g.name.as_str())
        .unwrap_or("this gain")
        .to_owned();

    let (confirmed, dismissed) = accordion::delete_dialog(ctx, "Delete Gain?", &gain_name);

    if confirmed {
        app.customer_segment_page
            .job_gain_links
            .retain(|(gid, _)| *gid != id);
        app.customer_segment_page
            .gains_state
            .gains
            .retain(|g| g.id != id);
        if app.customer_segment_page.gains_state.selected_gain_id == Some(id) {
            app.customer_segment_page.gains_state.selected_gain_id = None;
        }
    }
    if confirmed || dismissed {
        app.customer_segment_page.gains_state.pending_delete = None;
    }
}

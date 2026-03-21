use crate::app::App;
use eframe::egui;

use super::super::super::accordion;

pub fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.customer_segment_page.pains_state.pending_delete else {
        return;
    };

    let item_name = app
        .customer_segment_page
        .pains_state
        .pains
        .iter()
        .find(|p| p.id == id)
        .map(|p| accordion::display_name(&p.name, "Unnamed pain").to_owned())
        .unwrap_or_default();

    let (confirmed, dismissed) = accordion::delete_dialog(ctx, "Delete Pain?", &item_name);

    if confirmed {
        app.customer_segment_page
            .job_pain_links
            .retain(|(pid, _)| *pid != id);
        app.customer_segment_page
            .pains_state
            .pains
            .retain(|p| p.id != id);
        if app.customer_segment_page.pains_state.selected_id == Some(id) {
            app.customer_segment_page.pains_state.selected_id = None;
        }
    }
    if confirmed || dismissed {
        app.customer_segment_page.pains_state.pending_delete = None;
    }
}

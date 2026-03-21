use crate::app::App;
use eframe::egui;

use super::super::super::accordion;

pub fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.customer_segment_page.jobs_state.pending_delete else {
        return;
    };

    let item_name = app
        .customer_segment_page
        .jobs_state
        .jobs
        .iter()
        .find(|j| j.id == id)
        .map(|j| accordion::display_name(&j.name, "Unnamed job").to_owned())
        .unwrap_or_default();

    let (confirmed, dismissed) = accordion::delete_dialog(ctx, "Delete Job?", &item_name);

    if confirmed {
        app.customer_segment_page
            .segment_job_links
            .retain(|(jid, _)| *jid != id);
        app.customer_segment_page
            .jobs_state
            .jobs
            .retain(|j| j.id != id);
        if app.customer_segment_page.jobs_state.selected_id == Some(id) {
            app.customer_segment_page.jobs_state.selected_id = None;
        }
    }
    if confirmed || dismissed {
        app.customer_segment_page.jobs_state.pending_delete = None;
    }
}

use crate::app::App;
use eframe::egui;

use super::super::accordion;

/// Shows the delete confirmation dialog for the pending segment deletion.
///
/// On confirmation, removes the segment itself and clears `selected_id` if it
/// was open in the detail panel. Does **not** cascade to `segment_job_links` —
/// orphaned job links are harmless because jobs are independent entities.
/// On dismissal, clears `pending_delete` without deleting anything.
pub fn show_delete_confirmation(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.customer_segment_page.segments_state.pending_delete else {
        return;
    };

    let item_name = app
        .customer_segment_page
        .segments_state
        .segments
        .iter()
        .find(|s| s.id == id)
        .map(|s| accordion::display_name(&s.name, "Unnamed segment").to_owned())
        .unwrap_or_default();

    let (confirmed, dismissed) =
        accordion::delete_dialog(ctx, "Delete Customer Segment?", &item_name);

    if confirmed {
        // Cascade: remove all sub-segments that belong to this parent first.
        let children: Vec<_> = app
            .customer_segment_page
            .segments_state
            .segments
            .iter()
            .filter(|s| s.parent_id == Some(id))
            .map(|s| s.id)
            .collect();
        for child_id in children {
            app.customer_segment_page
                .segments_state
                .segments
                .retain(|s| s.id != child_id);
            if app.customer_segment_page.segments_state.selected_id == Some(child_id) {
                app.customer_segment_page.segments_state.selected_id = None;
            }
        }
        // Remove the parent segment itself.
        app.customer_segment_page
            .segments_state
            .segments
            .retain(|s| s.id != id);
        if app.customer_segment_page.segments_state.selected_id == Some(id) {
            app.customer_segment_page.segments_state.selected_id = None;
        }
    }
    if confirmed || dismissed {
        app.customer_segment_page.segments_state.pending_delete = None;
    }
}

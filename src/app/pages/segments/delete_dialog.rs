use crate::app::App;
use eframe::egui;

use super::super::accordion;

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

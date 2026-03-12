use crate::app::App;
use eframe::egui;

// ── Detail panel window ───────────────────────────────────────────────────────

pub fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.customer_page.segments_state.selected_segment_id else {
        return;
    };

    let mut keep_open = true;

    egui::Window::new("Customer Segment Details")
        .collapsible(false)
        .resizable(true)
        .default_size([420.0, 380.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            let Some(segment) = app
                .customer_page
                .segments_state
                .segments
                .iter_mut()
                .find(|s| s.id == id)
            else {
                ui.label("Segment not found.");
                return;
            };

            egui::Grid::new("segment_detail_grid")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .min_col_width(120.0)
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.add(
                        egui::TextEdit::singleline(&mut segment.name).desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Description:");
                    ui.add(
                        egui::TextEdit::singleline(&mut segment.description)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Notes:");
                    ui.add(
                        egui::TextEdit::multiline(&mut segment.notes)
                            .desired_rows(5)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Characteristics:");
                    ui.add(
                        egui::TextEdit::multiline(&mut segment.characteristics)
                            .desired_rows(5)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();
                });
        });

    // User dismissed with ✕ → deselect.
    if !keep_open {
        app.customer_page.segments_state.selected_segment_id = None;
    }
}

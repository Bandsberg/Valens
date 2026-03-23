use crate::app::App;
use eframe::egui;

/// Renders the Thoughtful Execution floating window.
///
/// **Thoughtful Execution** is the operational layer of the value proposition:
/// *how* you actually deliver the value you promise — team, process, culture,
/// and capabilities that make the offering reliable and scalable.
///
/// This window is currently a placeholder; content will be added in a future
/// iteration once the data model for execution factors is defined.
pub fn show_thoughtful_execution_window(app: &mut App, ctx: &egui::Context) {
    egui::Window::new("Thoughtful Execution")
        .scroll(true)
        .open(&mut app.valueprop_page.product_windows.thoughtful_execution_open)
        .default_size([520.0, 320.0])
        .show(ctx, |ui| {
            ui.heading("Thoughtful Execution");
            ui.label("Your Thoughtful Execution content here…");
        });
}

use crate::app::App;
use eframe::egui;

pub fn show_thoughtful_execution_window(app: &mut App, ctx: &egui::Context) {
    egui::Window::new("Thoughtful Execution")
        .scroll(true)
        .open(&mut app.product_page.product_windows.thoughtful_execution_open)
        .default_size([520.0, 320.0])
        .show(ctx, |ui| {
            ui.heading("Thoughtful Execution");
            ui.label("Your Thoughtful Execution content here…");
        });
}

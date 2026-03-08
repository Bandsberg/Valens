use crate::app::App;
use eframe::egui;

pub fn show_thoughtfull_execution_window(app: &mut App, ctx: &egui::Context) {
    egui::Window::new("Thoughtfull Execution")
        .scroll(true)
        .open(&mut app.product_page.product_windows.thoughtfull_execution_open)
        .default_size([520.0, 320.0])
        .show(ctx, |ui| {
            ui.heading("Thoughtfull Exectuion");
            ui.label("Your Thoughtfull Execution content here…");
        });
}

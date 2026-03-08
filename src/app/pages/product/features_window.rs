use crate::app::App;
use eframe::egui;

pub fn show_features_window(app: &mut App, ctx: &egui::Context) {
    egui::Window::new("Features")
        .scroll(true)
        .open(&mut app.product_page.product_windows.features_open)
        .default_size([520.0, 320.0])
        .show(ctx, |ui| {
            ui.heading("Features");
            ui.label("Your features content here…");
        });
}

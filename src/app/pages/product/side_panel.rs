use crate::app::App;
use eframe::egui;
pub fn product_sidepanel(app: &mut App, ctx: &egui::Context) {
    egui::SidePanel::right("demo_right_panel")
        .resizable(false)
        .default_width(160.0)
        .min_width(160.0)
        .show(ctx, |ui| {
            ui.heading("Tools");
            ui.separator();
            // toggles / list / etc
            ui.checkbox(
                &mut app.product_page.product_windows.products_open,
                "Products & Services",
            );
            ui.checkbox(
                &mut app.product_page.product_windows.features_open,
                "Features",
            );
            ui.checkbox(
                &mut app.product_page.product_windows.pain_relief_open,
                "Pain Relief",
            );
            ui.checkbox(
                &mut app.product_page.product_windows.thoughtfull_execution_open,
                "Thoughtfull Execution",
            );
        });
}

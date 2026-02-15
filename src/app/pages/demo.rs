use crate::app::TemplateApp;
use eframe::egui;

pub fn show_demo(template_app: &mut TemplateApp, ui: &mut egui::Ui) {
    ui.heading("eframe template");

    ui.horizontal(|ui| {
        ui.label("Write something ");
        ui.text_edit_singleline(&mut template_app.label);
    });

    ui.add(egui::Slider::new(&mut template_app.value, 0.0..=10.0).text("value"));
    if ui.button("Increment").clicked() {
        template_app.value += 1.0;
    }

    ui.separator();

    ui.horizontal(|ui| {
        ui.label("A sector ");
    });

    ui.separator();

    ui.add(egui::github_link_file!(
        "https://github.com/emilk/eframe_template/blob/main/",
        "Source code."
    ));

    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        powered_by_egui_and_eframe(ui);
        egui::warn_if_debug_build(ui);
    });
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

use eframe::egui;

fn heading_text_width(ui: &egui::Ui, text: &str) -> f32 {
    let wt = egui::WidgetText::from(egui::RichText::new(text).heading());
    let galley = wt.into_galley(ui, Some(egui::TextWrapMode::Extend), f32::INFINITY, egui::TextStyle::Heading);
    galley.size().x
}

/// Renders the two-column heading row (name label + "Description") and a
/// separator, matching the layout of collapsed accordion rows.
pub fn header(ui: &mut egui::Ui, name_label: &str) {
    ui.horizontal(|ui| {
        ui.add_space(28.0); // arrow button column
        let w = heading_text_width(ui, name_label);
        ui.add_sized(
            [w, 20.0],
            egui::Label::new(egui::RichText::new(name_label).heading()),
        );
        ui.label(egui::RichText::new("Description").heading());
    });
    ui.separator();
}

/// Returns `(name_width, description_width)` for a collapsed accordion row,
/// reserving space for two 36 px action buttons on the right.
/// `name_label` must match the label passed to [`header`] so the input column
/// aligns with the heading text.
pub fn row_field_widths(ui: &egui::Ui, name_label: &str) -> (f32, f32) {
    let spacing = ui.spacing().item_spacing.x;
    let btn_space = 36.0 * 2.0 + spacing * 2.0;
    let avail = ui.available_width() - btn_space;
    let name_w = heading_text_width(ui, name_label).min(avail * 0.5);
    let desc_w = (avail - name_w - spacing).max(0.0);
    (name_w, desc_w)
}

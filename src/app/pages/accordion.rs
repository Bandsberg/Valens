use eframe::egui;
use uuid::Uuid;

const DRAG_HANDLE_W: f32 = 6.0;

fn col_id(name_label: &str) -> egui::Id {
    egui::Id::new("accordion_name_col_w").with(name_label)
}

fn heading_text_width(ui: &egui::Ui, text: &str) -> f32 {
    let wt = egui::WidgetText::from(egui::RichText::new(text).heading());
    let galley = wt.into_galley(ui, Some(egui::TextWrapMode::Extend), f32::INFINITY, egui::TextStyle::Heading);
    galley.size().x
}

/// Returns `(current_width, min_width)` for the name column.
/// `current_width` is the stored drag value clamped to at least the heading text width.
fn current_name_w(ui: &egui::Ui, name_label: &str) -> (f32, f32) {
    let min_w = heading_text_width(ui, name_label);
    let stored = ui.data(|d| d.get_temp::<f32>(col_id(name_label)));
    (stored.unwrap_or(min_w).max(min_w), min_w)
}

/// Renders the two-column heading row (name label + drag handle + "Description")
/// and a separator. Drag the handle to widen the name column.
pub fn header(ui: &mut egui::Ui, name_label: &str) {
    ui.horizontal(|ui| {
        ui.add_space(28.0); // arrow button column

        let id = col_id(name_label);
        let (name_w, min_w) = current_name_w(ui, name_label);

        ui.add_sized(
            [name_w, 20.0],
            egui::Label::new(egui::RichText::new(name_label).heading()),
        );

        // Drag handle between name and description columns.
        let (handle_rect, response) =
            ui.allocate_exact_size(egui::vec2(DRAG_HANDLE_W, 20.0), egui::Sense::drag());

        if response.dragged() {
            let new_w = (name_w + response.drag_delta().x).max(min_w);
            ui.data_mut(|d| d.insert_temp(id, new_w));
        }
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeColumn);
        }

        let line_color = if response.hovered() || response.dragged() {
            ui.visuals().widgets.active.fg_stroke.color
        } else {
            ui.visuals().widgets.noninteractive.bg_stroke.color
        };
        ui.painter().line_segment(
            [handle_rect.center_top(), handle_rect.center_bottom()],
            egui::Stroke::new(1.5, line_color),
        );

        ui.label(egui::RichText::new("Description").heading());
    });
    ui.separator();
}

/// Renders a combo box whose selection is persisted in egui temp storage
/// between frames. The `show` closure receives `&mut egui::Ui` and a
/// `&mut Uuid` selection value to pass to `selectable_value` calls.
/// Returns `Some(uuid)` the frame the user makes a pick, `None` otherwise.
pub fn link_combo_pick(
    ui: &mut egui::Ui,
    key: egui::Id,
    show: impl FnOnce(&mut egui::Ui, &mut Uuid),
) -> Option<Uuid> {
    let mut sel: Uuid = ui.data(|d| d.get_temp(key).unwrap_or(Uuid::nil()));
    show(ui, &mut sel);
    if sel != Uuid::nil() {
        ui.data_mut(|d| d.remove::<Uuid>(key));
        Some(sel)
    } else {
        ui.data_mut(|d| d.insert_temp(key, sel));
        None
    }
}

/// Displays an italic "None" label in the weak text colour — used when a
/// linked-items list is empty.
pub fn none_label(ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("None")
            .italics()
            .color(ui.visuals().weak_text_color()),
    );
}

/// Returns `name` if non-empty, otherwise `fallback`.
/// Used by delete dialogs to display a human-readable item name.
pub fn display_name<'a>(name: &'a str, fallback: &'a str) -> &'a str {
    if name.is_empty() { fallback } else { name }
}

/// Expand/collapse arrow button for accordion rows. Returns `true` if clicked.
pub fn expand_button(ui: &mut egui::Ui, expanded: bool) -> bool {
    let arrow = if expanded { "▼" } else { "▶" };
    let hover = if expanded { "Collapse" } else { "Expand" };
    ui.add(egui::Button::new(arrow).fill(egui::Color32::TRANSPARENT))
        .on_hover_text(hover)
        .clicked()
}

/// Detail-panel toggle button (⊞/⊟). Returns `true` if clicked.
pub fn panel_toggle_button(ui: &mut egui::Ui, is_open: bool) -> bool {
    let icon = if is_open { "⊟" } else { "⊞" };
    let hover = if is_open { "Close detail panel" } else { "Open detail panel" };
    ui.add(egui::Button::new(icon).fill(egui::Color32::TRANSPARENT))
        .on_hover_text(hover)
        .clicked()
}

/// Small red ✕ button used to remove a link between two entities.
pub fn unlink_button(ui: &mut egui::Ui) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new("✕")
                .small()
                .color(egui::Color32::from_rgb(200, 60, 60)),
        )
        .fill(egui::Color32::TRANSPARENT),
    )
    .on_hover_text("Remove link")
}

/// Returns `(name_width, description_width)` for a collapsed accordion row,
/// reserving space for two 36 px action buttons on the right.
/// `name_label` must match the label passed to [`header`].
pub fn row_field_widths(ui: &egui::Ui, name_label: &str) -> (f32, f32) {
    let spacing = ui.spacing().item_spacing.x;
    // Account for the drag handle allocated in the header so description columns align.
    let btn_space = 36.0 * 2.0 + spacing * 2.0;
    let avail = ui.available_width() - btn_space;
    let (name_w, _) = current_name_w(ui, name_label);
    let name_w = name_w.min(avail - DRAG_HANDLE_W - spacing * 2.0);
    let desc_w = (avail - name_w - DRAG_HANDLE_W - spacing * 2.0).max(0.0);
    (name_w, desc_w)
}

use crate::app::App;
use crate::app::pages::accordion::{color_gain, color_pain};
use crate::app::pages::product::ValueType;
use crate::app::pages::value_analytics;
use eframe::egui;
use uuid::Uuid;

use super::value_gap_window::{SELECTED_PRODUCT_KEY, SELECTED_SEGMENT_KEY};

// ── Window ────────────────────────────────────────────────────────────────────

pub fn show_value_quadrant_window(app: &App, ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Importance × Strength Quadrant")
        .open(open)
        .default_size([480.0, 520.0])
        .resizable(true)
        .show(ctx, |ui| {
            show_contents(app, ctx, ui);
        });
}

#[expect(clippy::too_many_lines)]
fn show_contents(app: &App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let prod_key = egui::Id::new(SELECTED_PRODUCT_KEY);
    let seg_key = egui::Id::new(SELECTED_SEGMENT_KEY);

    let mut selected_product: Option<Uuid> =
        ctx.data(|d| d.get_temp::<Option<Uuid>>(prod_key)).flatten();
    let mut selected_segment: Option<Uuid> =
        ctx.data(|d| d.get_temp::<Option<Uuid>>(seg_key)).flatten();

    // ── Selectors (shared state with gap window) ──────────────────────────────
    let products = &app.valueprop_page.products_state.products;
    let segments = &app.customer_segment_page.segments_state.segments;

    ui.horizontal(|ui| {
        let prod_label = selected_product
            .and_then(|id| products.iter().find(|p| p.id == id))
            .map_or("Select product…", |p| p.name.as_str());

        egui::ComboBox::new(egui::Id::new("vq_prod_combo"), "Product")
            .selected_text(prod_label)
            .width(180.0)
            .show_ui(ui, |ui| {
                for p in products {
                    ui.selectable_value(&mut selected_product, Some(p.id), &p.name);
                }
            });

        let seg_label = selected_segment
            .and_then(|id| segments.iter().find(|s| s.id == id))
            .map_or("Select segment…", |s| s.name.as_str());

        egui::ComboBox::new(egui::Id::new("vq_seg_combo"), "Segment")
            .selected_text(seg_label)
            .width(180.0)
            .show_ui(ui, |ui| {
                for s in segments {
                    ui.selectable_value(&mut selected_segment, Some(s.id), &s.name);
                }
            });
    });

    ctx.data_mut(|d| d.insert_temp(prod_key, selected_product));
    ctx.data_mut(|d| d.insert_temp(seg_key, selected_segment));

    let (Some(prod_id), Some(seg_id)) = (selected_product, selected_segment) else {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("Select a product and segment to see the quadrant.")
                .italics()
                .color(ui.visuals().weak_text_color()),
        );
        return;
    };

    ui.add_space(6.0);

    let coverages = value_analytics::segment_need_coverages(prod_id, seg_id, app);
    if coverages.is_empty() {
        ui.label(
            egui::RichText::new("No needs found for this segment.")
                .italics()
                .color(ui.visuals().weak_text_color()),
        );
        return;
    }

    // ── Axis legend ───────────────────────────────────────────────────────────
    ui.label(
        egui::RichText::new(
            "X axis: Importance (how critical is this need?)    \
             Y axis: Strength (how well do you address it?)",
        )
        .small()
        .color(ui.visuals().weak_text_color()),
    );

    // ── Canvas ────────────────────────────────────────────────────────────────
    let avail = ui.available_width().min(440.0);
    let canvas_size = egui::vec2(avail, avail * 0.85);
    let (rect, response) = ui.allocate_exact_size(canvas_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    // Background
    painter.rect_filled(rect, 4.0, ui.visuals().faint_bg_color);

    let weak_color = ui.visuals().weak_text_color();
    let mid_stroke = egui::Stroke::new(1.0, weak_color.gamma_multiply(0.4));

    // Midpoint dividers at importance=0.5 and strength=0.5
    let mid_x = rect.min.x + 0.5 * rect.width();
    let mid_y = rect.min.y + 0.5 * rect.height();
    painter.line_segment(
        [egui::pos2(mid_x, rect.min.y), egui::pos2(mid_x, rect.max.y)],
        mid_stroke,
    );
    painter.line_segment(
        [egui::pos2(rect.min.x, mid_y), egui::pos2(rect.max.x, mid_y)],
        mid_stroke,
    );

    // Quadrant labels
    let label_color = weak_color.gamma_multiply(0.6);
    let font = egui::FontId::proportional(11.0);
    let quad_labels = [
        (0.25_f32, 0.25_f32, "Low Priority"),
        (0.75_f32, 0.25_f32, "Critical Gap"),
        (0.25_f32, 0.75_f32, "Solid Foundation"),
        (0.75_f32, 0.75_f32, "Key Differentiator"),
    ];
    for (ix, iy, label) in quad_labels {
        let pos = to_screen(rect, ix, iy);
        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            label,
            font.clone(),
            label_color,
        );
    }

    // Dots
    const DOT_RADIUS: f32 = 6.0;
    const HOVER_RADIUS: f32 = 10.0;
    let hover_pos = response.hover_pos();
    let mut hovered_name: Option<String> = None;

    for c in &coverages {
        let importance = c.importance;
        let strength = c.effective_strength.unwrap_or(0.0);

        let center = to_screen(rect, importance, strength);

        // Track hover
        if let Some(hp) = hover_pos
            && center.distance(hp) < HOVER_RADIUS
            && hovered_name.is_none()
        {
            hovered_name = Some(format!(
                "{}\nimp: {:.0}%  str: {}",
                c.name,
                importance * 100.0,
                c.effective_strength
                    .map_or("—".to_owned(), |s| format!("{:.0}%", s * 100.0))
            ));
        }

        let base_color = if c.is_pain {
            color_pain()
        } else {
            color_gain()
        };

        if c.effective_strength.is_none() {
            // Uncovered: outline only
            painter.circle_stroke(center, DOT_RADIUS, egui::Stroke::new(1.5, base_color));
        } else {
            // Covered: filled; TableStake uses a square marker to distinguish
            match c.value_type {
                Some(ValueType::TableStake) => {
                    let half = DOT_RADIUS;
                    let sq =
                        egui::Rect::from_center_size(center, egui::vec2(half * 2.0, half * 2.0));
                    painter.rect_filled(sq, 2.0, base_color);
                }
                _ => {
                    painter.circle_filled(center, DOT_RADIUS, base_color);
                }
            }
        }
    }

    // Tooltip — attach to canvas response so egui shows it at pointer position.
    if let Some(name) = hovered_name {
        response.on_hover_ui(|ui| {
            ui.label(name);
        });
    }

    // ── Legend ────────────────────────────────────────────────────────────────
    // NOTE: use ui.painter() here — NOT the canvas painter (painter_at(rect)),
    // which is clipped to the canvas area and would hide these shapes.
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(2.0);
    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new("Legend — marker shape:")
                .small()
                .strong(),
        );
        ui.add_space(6.0);

        let sz = egui::vec2(14.0, 14.0);
        let ink = ui.visuals().text_color();
        let outline = egui::Stroke::new(1.5, ink);

        let (r, _) = ui.allocate_exact_size(sz, egui::Sense::empty());
        ui.painter().circle_filled(r.center(), 5.0, ink);
        ui.label(egui::RichText::new("Differentiator").small());
        ui.add_space(8.0);

        let (r2, _) = ui.allocate_exact_size(sz, egui::Sense::empty());
        ui.painter().rect_filled(
            egui::Rect::from_center_size(r2.center(), egui::vec2(10.0, 10.0)),
            1.0,
            ink,
        );
        ui.label(egui::RichText::new("Table Stake").small());
        ui.add_space(8.0);

        let (r3, _) = ui.allocate_exact_size(sz, egui::Sense::empty());
        ui.painter().circle_stroke(r3.center(), 5.0, outline);
        ui.label(egui::RichText::new("Uncovered need").small());
    });

    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new("Legend — colour:").small().strong());
        ui.add_space(6.0);

        let sz = egui::vec2(14.0, 14.0);

        let (r4, _) = ui.allocate_exact_size(sz, egui::Sense::empty());
        ui.painter().circle_filled(r4.center(), 5.0, color_pain());
        ui.label(egui::RichText::new("Pain").small());
        ui.add_space(8.0);

        let (r5, _) = ui.allocate_exact_size(sz, egui::Sense::empty());
        ui.painter().circle_filled(r5.center(), 5.0, color_gain());
        ui.label(egui::RichText::new("Gain").small());
    });
}

/// Maps (importance, strength) → canvas pixel position.
/// Importance runs left→right; strength runs bottom→top (egui Y is inverted).
fn to_screen(rect: egui::Rect, importance: f32, strength: f32) -> egui::Pos2 {
    let margin = 16.0;
    let inner = rect.shrink(margin);
    egui::pos2(
        inner.min.x + importance.clamp(0.0, 1.0) * inner.width(),
        inner.max.y - strength.clamp(0.0, 1.0) * inner.height(),
    )
}

use crate::app::App;
use crate::app::pages::accordion::{color_gain, color_pain};
use crate::app::pages::value_analytics::{self, NeedCoverage, TABLE_STAKE_MIN_STRENGTH};
use eframe::egui;
use uuid::Uuid;

// Shared egui::Id keys — also used by value_quadrant_window so both windows
// reflect the same product/segment selection.
pub const SELECTED_PRODUCT_KEY: &str = "vg_selected_product";
pub const SELECTED_SEGMENT_KEY: &str = "vg_selected_segment";

// ── Shared product/segment selector ──────────────────────────────────────────

/// Renders the product + segment combo dropdowns and persists the selection in
/// egui temp storage under [`SELECTED_PRODUCT_KEY`] / [`SELECTED_SEGMENT_KEY`].
///
/// Both the gap window and the quadrant window call this so they always reflect
/// the same selection — changing one automatically updates the other.
///
/// `combo_id_prefix` differentiates the egui widget IDs between windows
/// (e.g. `"vg"` for gap, `"vq"` for quadrant) to avoid ID collisions when
/// both windows are open simultaneously.
///
/// Returns `(selected_product_id, selected_segment_id)`.
pub fn show_product_segment_selectors(
    app: &App,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    combo_id_prefix: &str,
) -> (Option<Uuid>, Option<Uuid>) {
    let prod_key = egui::Id::new(SELECTED_PRODUCT_KEY);
    let seg_key = egui::Id::new(SELECTED_SEGMENT_KEY);

    let mut selected_product: Option<Uuid> =
        ctx.data(|d| d.get_temp::<Option<Uuid>>(prod_key)).flatten();
    let mut selected_segment: Option<Uuid> =
        ctx.data(|d| d.get_temp::<Option<Uuid>>(seg_key)).flatten();

    let products = &app.valueprop_page.products_state.products;
    let segments = &app.customer_segment_page.segments_state.segments;

    ui.horizontal(|ui| {
        let prod_label = selected_product
            .and_then(|id| products.iter().find(|p| p.id == id))
            .map_or("Select product…", |p| p.name.as_str());

        egui::ComboBox::new(
            egui::Id::new(format!("{combo_id_prefix}_prod_combo")),
            "Product",
        )
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

        egui::ComboBox::new(
            egui::Id::new(format!("{combo_id_prefix}_seg_combo")),
            "Segment",
        )
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

    (selected_product, selected_segment)
}

// ── Shared UI helpers ─────────────────────────────────────────────────────────

/// Renders an italicised, dimmed placeholder message. Used when there is no
/// data to display yet (e.g. no product/segment selected, or no needs found).
pub fn show_placeholder_msg(ui: &mut egui::Ui, msg: &str) {
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(msg)
            .italics()
            .color(ui.visuals().weak_text_color()),
    );
}

// ── Window ────────────────────────────────────────────────────────────────────

pub fn show_value_gap_window(app: &App, ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Value Gap Analysis")
        .open(open)
        .default_size([540.0, 500.0])
        .resizable(true)
        .scroll(true)
        .show(ctx, |ui| {
            show_contents(app, ctx, ui);
        });
}

fn show_contents(app: &App, ctx: &egui::Context, ui: &mut egui::Ui) {
    // ── Selectors ─────────────────────────────────────────────────────────────
    let (selected_product, selected_segment) =
        show_product_segment_selectors(app, ctx, ui, "vg");

    let (Some(prod_id), Some(seg_id)) = (selected_product, selected_segment) else {
        show_placeholder_msg(ui, "Select a product and segment to see the gap analysis.");
        return;
    };

    ui.add_space(6.0);
    ui.separator();

    // ── Table Stake completeness bar ──────────────────────────────────────────
    let (ts_met, ts_total) = value_analytics::table_stake_completeness(prod_id, Some(seg_id), app);
    if ts_total > 0 {
        let fraction = ts_met as f32 / ts_total as f32;
        ui.horizontal(|ui| {
            ui.label("Table Stake completeness:");
            let bar_color = if ts_met == ts_total {
                egui::Color32::from_rgb(80, 160, 80)
            } else {
                egui::Color32::from_rgb(200, 60, 60)
            };
            ui.add(
                egui::ProgressBar::new(fraction)
                    .desired_width(160.0)
                    .text(format!(
                        "{ts_met}/{ts_total} ({:.0}%) ≥{:.0}%",
                        fraction * 100.0,
                        TABLE_STAKE_MIN_STRENGTH * 100.0
                    ))
                    .fill(bar_color),
            );
        });
        ui.add_space(4.0);
    }

    // ── Gap groups ────────────────────────────────────────────────────────────
    let coverages = value_analytics::segment_need_coverages(prod_id, seg_id, app);
    let groups = value_analytics::compute_gap_groups(coverages);

    if groups.is_empty() {
        show_placeholder_msg(
            ui,
            "No needs found for this segment, or no annotations on this product's relievers/creators.",
        );
        return;
    }

    show_group(
        ui,
        &format!("⚠ Uncovered  ({})", groups.uncovered.len()),
        &groups.uncovered,
        true,
        false,
    );
    show_group(
        ui,
        &format!("↓ Weak Coverage  ({})", groups.weak.len()),
        &groups.weak,
        true,
        true,
    );
    show_group(
        ui,
        &format!(
            "✗ Incomplete Table Stakes  ({})",
            groups.incomplete_table_stakes.len()
        ),
        &groups.incomplete_table_stakes,
        true,
        true,
    );
    show_group(
        ui,
        &format!(
            "★ Strong Differentiators  ({})",
            groups.strong_differentiators.len()
        ),
        &groups.strong_differentiators,
        false,
        true,
    );
}

fn show_group(
    ui: &mut egui::Ui,
    header: &str,
    items: &[NeedCoverage],
    default_open: bool,
    show_strength: bool,
) {
    if items.is_empty() {
        return;
    }
    egui::CollapsingHeader::new(header)
        .default_open(default_open)
        .show(ui, |ui| {
            for c in items {
                need_row(ui, c, show_strength);
            }
        });
    ui.add_space(2.0);
}

fn need_row(ui: &mut egui::Ui, c: &NeedCoverage, show_strength: bool) {
    ui.horizontal(|ui| {
        // Colour indicator dot
        let dot_color = if c.is_pain {
            color_pain()
        } else {
            color_gain()
        };
        let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::empty());
        ui.painter()
            .circle_filled(dot_rect.center(), 4.0, dot_color);

        ui.label(&c.name);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if show_strength && let Some(s) = c.effective_strength {
                ui.add(
                    egui::ProgressBar::new(s)
                        .desired_width(56.0)
                        .text(format!("{:.0}%", s * 100.0)),
                );
                ui.label(
                    egui::RichText::new("str")
                        .small()
                        .color(ui.visuals().weak_text_color()),
                );
            }
            ui.add(
                egui::ProgressBar::new(c.importance)
                    .desired_width(56.0)
                    .text(format!("{:.0}%", c.importance * 100.0)),
            );
            ui.label(
                egui::RichText::new("imp")
                    .small()
                    .color(ui.visuals().weak_text_color()),
            );
        });
    });
}

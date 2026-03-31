//! Four relationship visualisation modes for the Entity Relationships window.
//!
//! Toggle from the Value Proposition side panel → "Entity Relationships".
//!
//! Each tab lives in its own sub-module:
//! - [`flow_tab`]     — bezier-curve flow diagram (Jobs → Needs → Solutions)
//! - [`coverage_tab`] — heatmap grids (Pains × Pain Reliefs, Gains × Gain Creators)
//! - [`web_tab`]      — radial spider diagram focused on a single Job
//! - [`stories_tab`]  — scrollable coverage cards sorted by gap

use crate::app::App;
use eframe::egui;

mod coverage_tab;
mod flow_tab;
mod stories_tab;
mod web_tab;

// ── Entity colour palette ─────────────────────────────────────────────────────
// Shared by all four tabs so colours stay consistent as you switch between them.

const PAIN_RGB: [u8; 3] = [200, 80, 80];
const GAIN_RGB: [u8; 3] = [60, 175, 100];
const JOB_RGB: [u8; 3] = [140, 90, 210];
/// Pain relief — slightly muted relative to pain so solutions are visually
/// distinct from the need they address.
const PAIN_RELIEF_RGB: [u8; 3] = [185, 100, 100];
/// Gain creator — slightly muted relative to gain for the same reason.
const GAIN_CREATOR_RGB: [u8; 3] = [60, 155, 100];

// ── Flow canvas layout ────────────────────────────────────────────────────────
// These constants define the visual geometry of the Flow canvas view.

/// Fractional x-positions (as a proportion of canvas width) for the three flow
/// columns: Jobs, Pains & Gains, and Solutions.
const FLOW_COL_X_FRACS: [f32; 3] = [0.17, 0.50, 0.83];
/// Fractional width of each pill as a proportion of canvas width.
const FLOW_PILL_W_FRAC: f32 = 0.27;
/// Height of a pill rectangle in the Flow canvas view (pixels).
const FLOW_PILL_H: f32 = 22.0;
/// Vertical row height in the Flow canvas view (pixels).
const FLOW_ROW_H: f32 = 30.0;
/// Vertical padding above and below the pill rows in the Flow canvas (pixels).
const FLOW_PAD_V: f32 = 26.0;

// ── Tab ───────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Default)]
enum RelTab {
    #[default]
    Flow,
    Coverage,
    Web,
    Stories,
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn show_relationships_window(app: &App, ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Entity Relationships")
        .open(open)
        .default_size([720.0, 560.0])
        .resizable(true)
        .show(ctx, |ui| {
            let tab_key = egui::Id::new("rel_active_tab");
            let mut tab: RelTab = ctx.data(|d| d.get_temp(tab_key).unwrap_or_default());
            ui.horizontal(|ui| {
                ui.selectable_value(&mut tab, RelTab::Flow, "Flow");
                ui.selectable_value(&mut tab, RelTab::Coverage, "Coverage");
                ui.selectable_value(&mut tab, RelTab::Web, "Web");
                ui.selectable_value(&mut tab, RelTab::Stories, "Stories");
            });
            ctx.data_mut(|d| d.insert_temp(tab_key, tab));
            ui.separator();
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| match tab {
                    RelTab::Flow => flow_tab::show_flow(app, ui),
                    RelTab::Coverage => coverage_tab::show_coverage(app, ui),
                    RelTab::Web => web_tab::show_web(app, ctx, ui),
                    RelTab::Stories => stories_tab::show_stories(app, ui),
                });
        });
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn show_placeholder(ui: &mut egui::Ui, msg: &str) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new(msg)
            .italics()
            .color(ui.visuals().weak_text_color()),
    );
}

fn truncate(name: &str, max: usize) -> String {
    if name.chars().count() > max {
        let s: String = name.chars().take(max.saturating_sub(1)).collect();
        format!("{s}…")
    } else {
        name.to_owned()
    }
}

/// Map annotation strength (0–1) to a fill colour: amber at 0, green at 1.
fn strength_fill(strength: f32) -> egui::Color32 {
    let s = strength.clamp(0.0, 1.0);
    egui::Color32::from_rgba_unmultiplied(
        (220.0 - s * 160.0) as u8,
        (100.0 + s * 80.0) as u8,
        30_u8,
        (50.0 + s * 170.0) as u8,
    )
}

/// Draw a smooth horizontal S-curve bezier from `start` to `end`.
fn draw_bezier(painter: &egui::Painter, start: egui::Pos2, end: egui::Pos2, stroke: egui::Stroke) {
    let dx = (end.x - start.x) * 0.4;
    let p1 = egui::pos2(start.x + dx, start.y);
    let p2 = egui::pos2(end.x - dx, end.y);
    const SEGS: usize = 16;
    let mut prev = start;
    for i in 1..=SEGS {
        let t = i as f32 / SEGS as f32;
        let mt = 1.0 - t;
        let next = egui::pos2(
            mt * mt * mt * start.x
                + 3.0 * mt * mt * t * p1.x
                + 3.0 * mt * t * t * p2.x
                + t * t * t * end.x,
            mt * mt * mt * start.y
                + 3.0 * mt * mt * t * p1.y
                + 3.0 * mt * t * t * p2.y
                + t * t * t * end.y,
        );
        painter.line_segment([prev, next], stroke);
        prev = next;
    }
}

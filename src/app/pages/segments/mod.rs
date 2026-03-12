use crate::app::App;
use eframe::egui;
use uuid::Uuid;

mod accordion;
mod delete_dialog;
mod detail_panel;
mod model;

use accordion::show_accordion;
use delete_dialog::show_delete_confirmation;
use detail_panel::show_detail_panel;

pub use model::{CustomerSegment, SegmentsState};

// ── Page structs ──────────────────────────────────────────────────────────────

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct CustomerPage {
    customer_windows: CustomerWindows,
    pub segments_state: SegmentsState,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct CustomerWindows {
    segments_open: bool,
}

// ── Side panel ────────────────────────────────────────────────────────────────

pub fn customer_sidepanel(app: &mut App, ctx: &egui::Context) {
    egui::SidePanel::right("customer_right_panel")
        .resizable(false)
        .default_width(160.0)
        .min_width(160.0)
        .show(ctx, |ui| {
            ui.heading("Tools");
            ui.separator();
            ui.checkbox(
                &mut app.customer_page.customer_windows.segments_open,
                "Customer Segments",
            );
        });
}

// ── Central panel entry point ─────────────────────────────────────────────────

pub fn show_customer(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Customer Segment");
    ui.label("Use the Tools panel to open the Customer Segments window.");

    if app.customer_page.customer_windows.segments_open {
        show_segments_window(app, ctx);
    }
}

// ── Floating window ───────────────────────────────────────────────────────────

fn show_segments_window(app: &mut App, ctx: &egui::Context) {
    // Rendered before the main window so they sit on top.
    show_delete_confirmation(app, ctx);
    show_detail_panel(app, ctx);

    egui::Window::new("Customer Segments")
        .open(&mut app.customer_page.customer_windows.segments_open)
        .default_size([720.0, 380.0])
        .show(ctx, |ui| {
            ui.heading("Customer Segments");

            ui.add_space(4.0);

            if ui.button("➕ Add Segment").clicked() {
                app.customer_page
                    .segments_state
                    .segments
                    .push(CustomerSegment {
                        id: Uuid::new_v4(),
                        ..Default::default()
                    });
            }

            ui.separator();

            show_accordion(ui, &mut app.customer_page.segments_state);
        });
}

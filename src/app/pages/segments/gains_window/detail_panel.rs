use crate::app::App;
use eframe::egui;
use uuid::Uuid;

// ── Detail panel window ───────────────────────────────────────────────────────

pub fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.customer_page.gains_state.selected_gain_id else {
        return;
    };

    // Snapshot linked / available segments before entering the window closure
    // so we can borrow `gains_state.gains` mutably inside without conflict.
    // Link tuple: (gain_id, segment_id)
    let linked_sids: Vec<Uuid> = app
        .customer_page
        .segment_gain_links
        .iter()
        .filter(|(gid, _)| *gid == id)
        .map(|(_, sid)| *sid)
        .collect();

    let linked_segments: Vec<(Uuid, String)> = app
        .customer_page
        .segments_state
        .segments
        .iter()
        .filter(|s| linked_sids.contains(&s.id))
        .map(|s| (s.id, s.name.clone()))
        .collect();

    let available_segments: Vec<(Uuid, String)> = app
        .customer_page
        .segments_state
        .segments
        .iter()
        .filter(|s| !linked_sids.contains(&s.id))
        .map(|s| (s.id, s.name.clone()))
        .collect();

    // Collect mutations during the window; apply them afterwards.
    let mut link_to_add: Option<(Uuid, Uuid)> = None;
    let mut link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut navigate_to_seg: Option<Uuid> = None;

    let mut keep_open = true;
    egui::Window::new("Gain Details")
        .collapsible(false)
        .resizable(true)
        .default_size([420.0, 380.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            let Some(gain) = app
                .customer_page
                .gains_state
                .gains
                .iter_mut()
                .find(|g| g.id == id)
            else {
                ui.label("Gain not found.");
                return;
            };

            egui::Grid::new("gain_detail_grid")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .min_col_width(100.0)
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.add(
                        egui::TextEdit::singleline(&mut gain.name).desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Description:");
                    ui.add(
                        egui::TextEdit::singleline(&mut gain.description)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Notes:");
                    ui.add(
                        egui::TextEdit::multiline(&mut gain.notes)
                            .desired_rows(5)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    // ── Used by Segments ─────────────────────────────────────
                    ui.label("Used by\nSegments:");
                    ui.vertical(|ui| {
                        if linked_segments.is_empty() {
                            ui.label(
                                egui::RichText::new("None")
                                    .italics()
                                    .color(ui.visuals().weak_text_color()),
                            );
                        } else {
                            for (sid, sname) in &linked_segments {
                                ui.horizontal(|ui| {
                                    if ui.link(sname).on_hover_text("Open in Segments").clicked() {
                                        navigate_to_seg = Some(*sid);
                                    }
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("✕")
                                                    .small()
                                                    .color(egui::Color32::from_rgb(200, 60, 60)),
                                            )
                                            .fill(egui::Color32::TRANSPARENT),
                                        )
                                        .on_hover_text("Remove link")
                                        .clicked()
                                    {
                                        link_to_remove = Some((id, *sid));
                                    }
                                });
                            }
                        }

                        if !available_segments.is_empty() {
                            ui.add_space(4.0);

                            let combo_key = egui::Id::new("gain_detail_link_seg").with(id);
                            let mut sel: Uuid =
                                ui.data(|d| d.get_temp(combo_key).unwrap_or(Uuid::nil()));

                            let avail_w = ui.available_width();
                            egui::ComboBox::from_id_salt(combo_key)
                                .selected_text("Add a segment…")
                                .width(avail_w)
                                .show_ui(ui, |ui| {
                                    for (sid, sname) in &available_segments {
                                        ui.selectable_value(&mut sel, *sid, sname);
                                    }
                                });

                            if sel != Uuid::nil() {
                                link_to_add = Some((id, sel));
                                ui.data_mut(|d| d.remove::<Uuid>(combo_key));
                            } else {
                                ui.data_mut(|d| d.insert_temp(combo_key, sel));
                            }
                        }
                    });
                    ui.end_row();
                });
        });

    if !keep_open {
        app.customer_page.gains_state.selected_gain_id = None;
    }

    if let Some(pair) = link_to_add {
        if !app.customer_page.segment_gain_links.contains(&pair) {
            app.customer_page.segment_gain_links.push(pair);
        }
    }
    if let Some(pair) = link_to_remove {
        app.customer_page.segment_gain_links.retain(|l| l != &pair);
    }
    if let Some(seg_id) = navigate_to_seg {
        navigate_to_segment(app, ctx, seg_id);
    }
}

// ── Navigation helpers ────────────────────────────────────────────────────────

/// Opens the Customer Segments window and ensures `seg_id` is visible.
pub fn navigate_to_segment(app: &mut App, ctx: &egui::Context, seg_id: Uuid) {
    app.customer_page.customer_windows.segments_open = true;
    if let Some(seg) = app
        .customer_page
        .segments_state
        .segments
        .iter_mut()
        .find(|s| s.id == seg_id)
    {
        seg.expanded = true;
    }
    app.customer_page.segments_state.selected_segment_id = Some(seg_id);
    app.customer_page.segments_state.scroll_to_id = Some(seg_id);
    ctx.move_to_top(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("Customer Segments"),
    ));
}

/// Opens the Gains window and ensures `gain_id` is visible.
#[allow(dead_code)]
pub fn navigate_to_gain(app: &mut App, ctx: &egui::Context, gain_id: Uuid) {
    app.customer_page.customer_windows.gains_open = true;
    if let Some(gain) = app
        .customer_page
        .gains_state
        .gains
        .iter_mut()
        .find(|g| g.id == gain_id)
    {
        gain.expanded = true;
    }
    app.customer_page.gains_state.selected_gain_id = Some(gain_id);
    app.customer_page.gains_state.scroll_to_id = Some(gain_id);
    ctx.move_to_top(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("Gains"),
    ));
}

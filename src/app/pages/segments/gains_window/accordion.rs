use eframe::egui;
use uuid::Uuid;

use super::super::jobs_window::Job;
use super::model::GainsState;

use super::super::super::accordion;

const MULTILINE_H: f32 = 60.0;

pub fn show_accordion(
    ui: &mut egui::Ui,
    state: &mut GainsState,
    jobs: &[Job],
    links: &mut Vec<(Uuid, Uuid)>,
    navigate_to: &mut Option<Uuid>,
) {
    let mut to_delete: Option<Uuid> = None;
    let mut link_to_add: Option<(Uuid, Uuid)> = None;
    let mut link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut did_scroll = false;
    let mut do_panel_select: Option<Uuid> = None;
    let mut do_panel_deselect = false;

    let links_snap = links.clone();
    let scroll_to = state.scroll_to_id;
    let selected_id = state.selected_gain_id;

    accordion::header(ui, "Gain name");

    egui::ScrollArea::vertical().show(ui, |ui| {
        for gain in &mut state.gains {
            let id = gain.id;
            let expanded = gain.expanded;
            let is_panel_open = selected_id == Some(id);

            // Link tuple: (gain_id, job_id)
            let linked_jids: Vec<Uuid> = links_snap
                .iter()
                .filter(|(gid, _)| *gid == id)
                .map(|(_, jid)| *jid)
                .collect();

            if scroll_to == Some(id) {
                ui.scroll_to_cursor(Some(egui::Align::Center));
                did_scroll = true;
            }

            ui.horizontal(|ui| {
                let arrow = if expanded { "▼" } else { "▶" };
                let hover = if expanded { "Collapse" } else { "Expand" };
                if ui
                    .add(egui::Button::new(arrow).fill(egui::Color32::TRANSPARENT))
                    .on_hover_text(hover)
                    .clicked()
                {
                    gain.expanded = !gain.expanded;
                }

                let (name_w, desc_w) = accordion::row_field_widths(ui, "Gain name");

                ui.add_sized(
                    [name_w, 20.0],
                    egui::TextEdit::singleline(&mut gain.name).hint_text("Gain name…"),
                );
                ui.add_sized(
                    [desc_w, 20.0],
                    egui::TextEdit::singleline(&mut gain.description)
                        .hint_text("Short description…"),
                );

                let icon = if is_panel_open { "⊟" } else { "⊞" };
                let panel_hover = if is_panel_open {
                    "Close detail panel"
                } else {
                    "Open detail panel"
                };
                if ui
                    .add(egui::Button::new(icon).fill(egui::Color32::TRANSPARENT))
                    .on_hover_text(panel_hover)
                    .clicked()
                {
                    if is_panel_open {
                        do_panel_deselect = true;
                    } else {
                        do_panel_select = Some(id);
                    }
                }
                if ui
                    .add(egui::Button::new("🗑").fill(egui::Color32::TRANSPARENT))
                    .on_hover_text("Delete gain")
                    .clicked()
                {
                    to_delete = Some(id);
                }
            });

            if expanded {
                ui.indent(id, |ui| {
                    ui.add_space(4.0);
                    ui.label("Notes:");
                    ui.add(
                        egui::TextEdit::multiline(&mut gain.notes)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .min_size(egui::vec2(0.0, MULTILINE_H)),
                    );

                    // ── Used by Jobs ──────────────────────────────────────────
                    ui.separator();
                    ui.label("Used by Jobs:");

                    let available: Vec<&Job> = jobs
                        .iter()
                        .filter(|j| !linked_jids.contains(&j.id))
                        .collect();

                    if !available.is_empty() {
                        let combo_key = egui::Id::new("gain_acc_link_job").with(id);
                        let mut sel: Uuid =
                            ui.data(|d| d.get_temp(combo_key).unwrap_or(Uuid::nil()));

                        let avail_w = ui.available_width();
                        egui::ComboBox::from_id_salt(combo_key)
                            .selected_text("Add a job…")
                            .width(avail_w)
                            .show_ui(ui, |ui| {
                                for job in &available {
                                    ui.selectable_value(&mut sel, job.id, &job.name);
                                }
                            });

                        if sel != Uuid::nil() {
                            link_to_add = Some((id, sel));
                            ui.data_mut(|d| d.remove::<Uuid>(combo_key));
                        } else {
                            ui.data_mut(|d| d.insert_temp(combo_key, sel));
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("All jobs linked"));
                    }

                    if !linked_jids.is_empty() {
                        for jid in &linked_jids {
                            if let Some(job) = jobs.iter().find(|j| j.id == *jid) {
                                ui.horizontal(|ui| {
                                    if ui
                                        .link(&job.name)
                                        .on_hover_text("Open in Jobs")
                                        .clicked()
                                    {
                                        *navigate_to = Some(*jid);
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
                                        link_to_remove = Some((id, *jid));
                                    }
                                });
                            }
                        }
                    } else {
                        ui.label(
                            egui::RichText::new("None")
                                .italics()
                                .color(ui.visuals().weak_text_color()),
                        );
                    }
                    ui.add_space(4.0);
                });
            }

            ui.separator();
        }
    });

    if did_scroll {
        state.scroll_to_id = None;
    }
    if let Some(id) = to_delete {
        state.pending_delete = Some(id);
    }
    if let Some(pair) = link_to_add {
        if !links.contains(&pair) {
            links.push(pair);
        }
    }
    if let Some(pair) = link_to_remove {
        links.retain(|l| l != &pair);
    }
    if do_panel_deselect {
        state.selected_gain_id = None;
    } else if let Some(id) = do_panel_select {
        state.selected_gain_id = Some(id);
    }
}

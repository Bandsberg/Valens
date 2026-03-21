use crate::app::App;
use eframe::egui;
use uuid::Uuid;

use super::super::super::accordion;

#[expect(clippy::too_many_lines)]
pub fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.customer_segment_page.gains_state.selected_gain_id else {
        return;
    };

    // Link tuple: (gain_id, job_id)
    let linked_jids: Vec<Uuid> = app
        .customer_segment_page
        .job_gain_links
        .iter()
        .filter(|(gid, _)| *gid == id)
        .map(|(_, jid)| *jid)
        .collect();

    let linked_jobs: Vec<(Uuid, String)> = app
        .customer_segment_page
        .jobs_state
        .jobs
        .iter()
        .filter(|j| linked_jids.contains(&j.id))
        .map(|j| (j.id, j.name.clone()))
        .collect();

    let available_jobs: Vec<(Uuid, String)> = app
        .customer_segment_page
        .jobs_state
        .jobs
        .iter()
        .filter(|j| !linked_jids.contains(&j.id))
        .map(|j| (j.id, j.name.clone()))
        .collect();

    let mut link_to_add: Option<(Uuid, Uuid)> = None;
    let mut link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut navigate_to_job_id: Option<Uuid> = None;

    let mut keep_open = true;
    egui::Window::new("Gain Details")
        .collapsible(false)
        .resizable(true)
        .default_size([420.0, 380.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            let Some(gain) = app
                .customer_segment_page
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
                    ui.add(egui::TextEdit::singleline(&mut gain.name).desired_width(f32::INFINITY));
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

                    // ── Used by Jobs ──────────────────────────────────────────
                    ui.label("Used by\nJobs:");
                    ui.vertical(|ui| {
                        if linked_jobs.is_empty() {
                            accordion::none_label(ui);
                        } else {
                            for (jid, jname) in &linked_jobs {
                                ui.horizontal(|ui| {
                                    if ui.link(jname).on_hover_text("Open in Jobs").clicked() {
                                        navigate_to_job_id = Some(*jid);
                                    }
                                    if accordion::unlink_button(ui).clicked() {
                                        link_to_remove = Some((id, *jid));
                                    }
                                });
                            }
                        }

                        if !available_jobs.is_empty() {
                            ui.add_space(4.0);
                            let combo_key = egui::Id::new("gain_detail_link_job").with(id);
                            let avail_w = ui.available_width();
                            if let Some(sel) =
                                accordion::link_combo_pick(ui, combo_key, |ui, sel| {
                                    egui::ComboBox::from_id_salt(combo_key)
                                        .selected_text("Add a job…")
                                        .width(avail_w)
                                        .show_ui(ui, |ui| {
                                            for (jid, jname) in &available_jobs {
                                                ui.selectable_value(sel, *jid, jname);
                                            }
                                        });
                                })
                            {
                                link_to_add = Some((id, sel));
                            }
                        }
                    });
                    ui.end_row();
                });
        });

    if !keep_open {
        app.customer_segment_page.gains_state.selected_gain_id = None;
    }
    if let Some(pair) = link_to_add
        && !app.customer_segment_page.job_gain_links.contains(&pair)
    {
        app.customer_segment_page.job_gain_links.push(pair);
    }
    if let Some(pair) = link_to_remove {
        app.customer_segment_page
            .job_gain_links
            .retain(|l| l != &pair);
    }
    if let Some(job_id) = navigate_to_job_id {
        navigate_to_job(app, ctx, job_id);
    }
}

// ── Navigation helpers ────────────────────────────────────────────────────────

/// Opens the Jobs window and ensures `job_id` is visible.
pub fn navigate_to_job(app: &mut App, ctx: &egui::Context, job_id: Uuid) {
    app.customer_segment_page.customer_windows.jobs_open = true;
    if let Some(job) = app
        .customer_segment_page
        .jobs_state
        .jobs
        .iter_mut()
        .find(|j| j.id == job_id)
    {
        job.expanded = true;
    }
    app.customer_segment_page.jobs_state.selected_job_id = Some(job_id);
    app.customer_segment_page.jobs_state.scroll_to_id = Some(job_id);
    ctx.move_to_top(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("Jobs"),
    ));
}

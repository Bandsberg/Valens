use crate::app::App;
use eframe::egui;
use uuid::Uuid;

use super::super::super::accordion;
use super::super::detail_panel::navigate_to_job;

pub fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.customer_segment_page.gains_state.selected_id else {
        return;
    };

    // Link tuple: (gain_id, job_id) — gain is in first position.
    let (linked_jobs, available_jobs) = accordion::partition_linked(
        &app.customer_segment_page.job_gain_links,
        |(gid, jid)| (*gid == id).then_some(*jid),
        &app.customer_segment_page.jobs_state.jobs,
        |j| j.id,
        |j| j.name.as_str(),
    );

    let mut link_to_add: Option<(Uuid, Uuid)> = None;
    let mut link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut nav_to_job: Option<Uuid> = None;

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

                    ui.label("Importance:");
                    ui.add(
                        egui::DragValue::new(&mut gain.importance)
                            .range(0.0..=1.0)
                            .speed(0.01)
                            .fixed_decimals(2),
                    );
                    ui.end_row();

                    // ── Used by Jobs ──────────────────────────────────────────
                    let (add, rem) = accordion::detail_link_row(
                        ui,
                        "Used by\nJobs:",
                        egui::Id::new("gain_detail_link_job").with(id),
                        "Add a job…",
                        &available_jobs,
                        &linked_jobs,
                        &mut nav_to_job,
                        Some("Open in Jobs"),
                    );
                    // Link tuple: (gain_id, job_id).
                    if let Some(jid) = add {
                        link_to_add = Some((id, jid));
                    }
                    if let Some(jid) = rem {
                        link_to_remove = Some((id, jid));
                    }
                    ui.end_row();
                });
        });

    if !keep_open {
        app.customer_segment_page.gains_state.selected_id = None;
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
    if let Some(job_id) = nav_to_job {
        navigate_to_job(app, ctx, job_id);
    }
}

use crate::app::App;
use eframe::egui;
use uuid::Uuid;

use super::super::super::accordion;
use super::super::detail_panel::navigate_to_job;

pub fn show_detail_panel(app: &mut App, ctx: &egui::Context) {
    let Some(id) = app.customer_segment_page.pains_state.selected_id else {
        return;
    };

    // Link tuple: (pain_id, job_id) — pain is in first position.
    let (linked_jobs, available_jobs) = accordion::partition_linked(
        &app.customer_segment_page.job_pain_links,
        |(pid, jid)| (*pid == id).then_some(*jid),
        &app.customer_segment_page.jobs_state.jobs,
        |j| j.id,
        |j| j.name.as_str(),
    );

    let mut link_to_add: Option<(Uuid, Uuid)> = None;
    let mut link_to_remove: Option<(Uuid, Uuid)> = None;
    let mut navigate_to_job_id: Option<Uuid> = None;

    let mut keep_open = true;
    egui::Window::new("Pain Details")
        .collapsible(false)
        .resizable(true)
        .default_size([420.0, 380.0])
        .open(&mut keep_open)
        .show(ctx, |ui| {
            let Some(pain) = app
                .customer_segment_page
                .pains_state
                .pains
                .iter_mut()
                .find(|p| p.id == id)
            else {
                ui.label("Pain not found.");
                return;
            };

            egui::Grid::new("pain_detail_grid")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .min_col_width(100.0)
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.add(egui::TextEdit::singleline(&mut pain.name).desired_width(f32::INFINITY));
                    ui.end_row();

                    ui.label("Description:");
                    ui.add(
                        egui::TextEdit::singleline(&mut pain.description)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    ui.label("Notes:");
                    ui.add(
                        egui::TextEdit::multiline(&mut pain.notes)
                            .desired_rows(5)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();

                    // ── Used by Jobs ──────────────────────────────────────────
                    let (add, rem) = accordion::detail_link_row(
                        ui,
                        "Used by\nJobs:",
                        egui::Id::new("pain_detail_link_job").with(id),
                        "Add a job…",
                        &available_jobs,
                        &linked_jobs,
                        &mut navigate_to_job_id,
                        Some("Open in Jobs"),
                    );
                    // Link tuple: (pain_id, job_id).
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
        app.customer_segment_page.pains_state.selected_id = None;
    }
    if let Some(pair) = link_to_add
        && !app.customer_segment_page.job_pain_links.contains(&pair)
    {
        app.customer_segment_page.job_pain_links.push(pair);
    }
    if let Some(pair) = link_to_remove {
        app.customer_segment_page
            .job_pain_links
            .retain(|l| l != &pair);
    }
    if let Some(job_id) = navigate_to_job_id {
        navigate_to_job(app, ctx, job_id);
    }
}

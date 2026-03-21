use eframe::egui;
use uuid::Uuid;

use super::super::jobs_window::Job;
use super::model::PainsState;

use super::super::super::accordion;

const MULTILINE_H: f32 = 60.0;

// ── Accordion table ───────────────────────────────────────────────────────────

#[expect(clippy::too_many_lines)]
pub fn show_accordion(
    ui: &mut egui::Ui,
    state: &mut PainsState,
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
    let selected_id = state.selected_id;

    accordion::header(ui, "Pain name");

    egui::ScrollArea::vertical().show(ui, |ui| {
        for pain in &mut state.pains {
            let id = pain.id;
            let expanded = pain.expanded;
            let is_panel_open = selected_id == Some(id);

            if scroll_to == Some(id) {
                ui.scroll_to_cursor(Some(egui::Align::Center));
                did_scroll = true;
            }

            ui.horizontal(|ui| {
                if accordion::expand_button(ui, expanded) {
                    pain.expanded = !pain.expanded;
                }

                let (name_w, desc_w) = accordion::row_field_widths(ui, "Pain name");

                ui.add_sized(
                    [name_w, 20.0],
                    egui::TextEdit::singleline(&mut pain.name).hint_text("Pain name…"),
                );
                ui.add_sized(
                    [desc_w, 20.0],
                    egui::TextEdit::singleline(&mut pain.description)
                        .hint_text("Short description…"),
                );

                if accordion::panel_toggle_button(ui, is_panel_open) {
                    if is_panel_open {
                        do_panel_deselect = true;
                    } else {
                        do_panel_select = Some(id);
                    }
                }
                if ui
                    .add(egui::Button::new("🗑").fill(egui::Color32::TRANSPARENT))
                    .on_hover_text("Delete pain")
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
                        egui::TextEdit::multiline(&mut pain.notes)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .min_size(egui::vec2(0.0, MULTILINE_H)),
                    );

                    // ── Used by Jobs ──────────────────────────────────────────
                    // Link tuple: (pain_id, job_id) — pain is first.
                    ui.separator();
                    let (linked_jobs, avail_jobs) = accordion::partition_linked(
                        &links_snap,
                        |(pid, jid)| (*pid == id).then_some(*jid),
                        jobs,
                        |j| j.id,
                        |j| j.name.as_str(),
                    );
                    let (add, rem) = accordion::acc_link_section(
                        ui,
                        "Used by Jobs:",
                        egui::Id::new("pain_acc_link_job").with(id),
                        "Add a job…",
                        "All jobs linked",
                        &avail_jobs,
                        &linked_jobs,
                        navigate_to,
                        Some("Open in Jobs"),
                    );
                    // Link tuple is (pain_id, job_id).
                    if let Some(jid) = add {
                        link_to_add = Some((id, jid));
                    }
                    if let Some(jid) = rem {
                        link_to_remove = Some((id, jid));
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
    if let Some(pair) = link_to_add
        && !links.contains(&pair)
    {
        links.push(pair);
    }
    if let Some(pair) = link_to_remove {
        links.retain(|l| l != &pair);
    }
    if do_panel_deselect {
        state.selected_id = None;
    } else if let Some(id) = do_panel_select {
        state.selected_id = Some(id);
    }
}

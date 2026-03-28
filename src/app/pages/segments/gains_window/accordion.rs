use eframe::egui;
use uuid::Uuid;

use super::super::jobs_window::Job;
use super::model::GainsState;

use super::super::super::accordion::{self, ROW_H};

/// Minimum pixel height for multiline text-edit fields in the expanded row.
/// Chosen to display roughly three lines at the default font size.
const MULTILINE_H: f32 = 60.0;

#[expect(clippy::too_many_lines)]
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

    // egui closures (ScrollArea, indent, horizontal) borrow `ui` exclusively,
    // so we cannot also hold a mutable borrow on `state` or `links` inside
    // them. The pattern here is:
    //   1. Snapshot the data we need to *read* during rendering.
    //   2. Accumulate any mutations in local variables during the render loop.
    //   3. Apply all mutations after the scroll area exits (see bottom of fn).
    let links_snap = links.clone();
    let scroll_to = state.scroll_to_id;
    let selected_id = state.selected_id;

    accordion::header(ui, "Gain name");

    egui::ScrollArea::vertical().show(ui, |ui| {
        for gain in &mut state.gains {
            let id = gain.id;
            let expanded = gain.expanded;
            let is_panel_open = selected_id == Some(id);

            if scroll_to == Some(id) {
                ui.scroll_to_cursor(Some(egui::Align::Center));
                did_scroll = true;
            }

            ui.horizontal(|ui| {
                if accordion::expand_button(ui, expanded) {
                    gain.expanded = !gain.expanded;
                }

                let (name_w, desc_w) = accordion::row_field_widths(ui, "Gain name");

                ui.add_sized(
                    [name_w, ROW_H],
                    egui::TextEdit::singleline(&mut gain.name).hint_text("Gain name…"),
                );
                ui.add_sized(
                    [desc_w, ROW_H],
                    egui::TextEdit::singleline(&mut gain.description)
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
                    // Link tuple: (gain_id, job_id) — gain is first.
                    ui.separator();
                    let (linked_jobs, avail_jobs) = accordion::partition_linked(
                        &links_snap,
                        |(gid, jid)| (*gid == id).then_some(*jid),
                        jobs,
                        |j| j.id,
                        |j| j.name.as_str(),
                    );
                    let (add, rem) = accordion::acc_link_section(
                        ui,
                        "Used by Jobs:",
                        egui::Id::new("gain_acc_link_job").with(id),
                        "Add a job…",
                        "All jobs linked",
                        &avail_jobs,
                        &linked_jobs,
                        navigate_to,
                        Some("Open in Jobs"),
                    );
                    // Link tuple is (gain_id, job_id).
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
    // Deselect (close the panel) takes the `if` branch so it always wins.
    // Opening a new row while another is open just replaces `selected_id` —
    // `do_panel_deselect` is only set when the user clicks the toggle on the
    // row that is *already* open, so both flags are never set together.
    if do_panel_deselect {
        state.selected_id = None;
    } else if let Some(id) = do_panel_select {
        state.selected_id = Some(id);
    }
}

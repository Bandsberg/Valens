use eframe::egui;
use uuid::Uuid;

/// Width of the resize drag handle between the name and description columns.
const DRAG_HANDLE_W: f32 = 6.0;
/// Width reserved for the expand/collapse arrow button column.
const ARROW_COLUMN_W: f32 = 28.0;
/// Height of a single accordion header/row widget.
/// Referenced by sub-accordion modules for `add_sized` text-edit rows.
pub(crate) const ROW_H: f32 = 20.0;
/// Width of a single action button (e.g. detail-panel toggle, delete).
const ACTION_BTN_W: f32 = 36.0;

/// Cell / progress-bar colour for Table Stake indicators when all stakes are met.
pub const TABLE_STAKE_MET: egui::Color32 = egui::Color32::from_rgb(80, 160, 80);
/// Cell / progress-bar colour for Table Stake indicators when one or more stakes
/// are below the minimum strength threshold (product viability at risk).
pub const TABLE_STAKE_UNMET: egui::Color32 = egui::Color32::from_rgb(200, 60, 60);

/// Fill colour for destructive-action buttons such as the delete confirmation button.
const DANGER_RED: egui::Color32 = egui::Color32::from_rgb(180, 40, 40);
/// Icon colour for unlink (✕) buttons — softer than `DANGER_RED` since removing
/// a link is less severe than deleting an entity.
const UNLINK_RED: egui::Color32 = egui::Color32::from_rgb(200, 60, 60);

/// Hover-highlight colour for pain-related entities (warm red, low opacity).
pub fn color_pain() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(220, 80, 80, 40)
}
/// Hover-highlight colour for gain-related entities (soft green, low opacity).
pub fn color_gain() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(80, 200, 120, 40)
}
/// Hover-highlight colour for job entities (muted purple, low opacity).
pub fn color_job() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(160, 100, 220, 40)
}
/// Hover-highlight colour for customer segment entities (muted blue, low opacity).
pub fn color_segment() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(60, 140, 220, 40)
}

fn col_id(name_label: &str) -> egui::Id {
    egui::Id::new("accordion_name_col_w").with(name_label)
}

fn heading_text_width(ui: &egui::Ui, text: &str) -> f32 {
    let wt = egui::WidgetText::from(egui::RichText::new(text).heading());
    let galley = wt.into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        f32::INFINITY,
        egui::TextStyle::Heading,
    );
    galley.size().x
}

/// Returns `(current_width, min_width)` for the name column.
/// `current_width` is the stored drag value clamped to at least the heading text width.
fn current_name_w(ui: &egui::Ui, name_label: &str) -> (f32, f32) {
    let min_w = heading_text_width(ui, name_label);
    let stored = ui.data(|d| d.get_temp::<f32>(col_id(name_label)));
    (stored.unwrap_or(min_w).max(min_w), min_w)
}

/// Renders the two-column heading row (name label + drag handle + "Description")
/// and a separator. Drag the handle to widen the name column.
pub fn header(ui: &mut egui::Ui, name_label: &str) {
    ui.horizontal(|ui| {
        ui.add_space(ARROW_COLUMN_W); // arrow button column

        let id = col_id(name_label);
        let (name_w, min_w) = current_name_w(ui, name_label);

        ui.add_sized(
            [name_w, ROW_H],
            egui::Label::new(egui::RichText::new(name_label).heading()),
        );

        // Drag handle between name and description columns.
        let (handle_rect, response) =
            ui.allocate_exact_size(egui::vec2(DRAG_HANDLE_W, ROW_H), egui::Sense::drag());

        if response.dragged() {
            let new_w = (name_w + response.drag_delta().x).max(min_w);
            ui.data_mut(|d| d.insert_temp(id, new_w));
        }
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeColumn);
        }

        let line_color = if response.hovered() || response.dragged() {
            ui.visuals().widgets.active.fg_stroke.color
        } else {
            ui.visuals().widgets.noninteractive.bg_stroke.color
        };
        ui.painter().line_segment(
            [handle_rect.center_top(), handle_rect.center_bottom()],
            egui::Stroke::new(1.5, line_color),
        );

        ui.label(egui::RichText::new("Description").heading());
    });
    ui.separator();
}

/// Renders a combo box whose selection is persisted in egui temp storage
/// between frames. The `show` closure receives `&mut egui::Ui` and a
/// `&mut Uuid` selection value to pass to `selectable_value` calls.
/// Returns `Some(uuid)` the frame the user makes a pick, `None` otherwise.
///
/// `Uuid::nil()` (all-zeroes) is used as the "nothing selected yet" sentinel
/// because egui temp storage requires a concrete type, not `Option<T>`.
/// Real entity UUIDs are always `Uuid::new_v4()` (random), so a nil UUID
/// will never collide with a legitimate selection.
pub fn link_combo_pick(
    ui: &mut egui::Ui,
    key: egui::Id,
    show: impl FnOnce(&mut egui::Ui, &mut Uuid),
) -> Option<Uuid> {
    let mut sel: Uuid = ui.data(|d| d.get_temp(key).unwrap_or(Uuid::nil()));
    show(ui, &mut sel);
    if sel != Uuid::nil() {
        ui.data_mut(|d| d.remove::<Uuid>(key));
        Some(sel)
    } else {
        ui.data_mut(|d| d.insert_temp(key, sel));
        None
    }
}

/// Displays an italic "None" label in the weak text colour — used when a
/// linked-items list is empty.
pub fn none_label(ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("None")
            .italics()
            .color(ui.visuals().weak_text_color()),
    );
}

/// Returns `name` if non-empty, otherwise `fallback`.
/// Used by delete dialogs to display a human-readable item name.
pub fn display_name<'a>(name: &'a str, fallback: &'a str) -> &'a str {
    if name.is_empty() { fallback } else { name }
}

/// Scales the alpha of a premultiplied `Color32` by `factor` (0.0–1.0).
///
/// In premultiplied alpha the stored RGB values are already multiplied by
/// alpha, so reducing alpha requires scaling the RGB channels by the same
/// ratio (`new_a / old_a`). Without this correction, halving only the alpha
/// would make the colour look lighter rather than more transparent.
pub fn scale_color(color: egui::Color32, factor: f32) -> egui::Color32 {
    let [r, g, b, a] = color.to_array();
    let new_a = (a as f32 * factor).round() as u8;
    // Keep RGB proportional to the new alpha so premultiplied invariant holds.
    let scale = if a > 0 { new_a as f32 / a as f32 } else { 0.0 };
    egui::Color32::from_rgba_premultiplied(
        (r as f32 * scale).round() as u8,
        (g as f32 * scale).round() as u8,
        (b as f32 * scale).round() as u8,
        new_a,
    )
}

/// Renders a label and paints a coloured highlight behind it on hover or when
/// `highlight > 0.0` (i.e. this entity is linked to whatever is hovered).
/// `highlight` is 0.0–1.0; the color's alpha is scaled proportionally.
/// Stores the hovered entity UUID in egui temp storage under `hover_key` so
/// the caller can read it next frame to compute cross-highlighted siblings.
pub fn label_with_hover_id(
    ui: &mut egui::Ui,
    text: &str,
    id: Uuid,
    color: egui::Color32,
    highlight: f32,
    hover_key: egui::Id,
) {
    let response = ui.label(text);
    if response.hovered() {
        ui.ctx().data_mut(|d| d.insert_temp(hover_key, id));
    }
    let effective = if response.hovered() {
        1.0_f32
    } else {
        highlight
    };
    if effective > 0.0 {
        let scaled = scale_color(color, effective);
        ui.painter().rect_filled(response.rect, 3.0, scaled);
    }
}

/// Renders a centered delete-confirmation dialog.
/// Returns `(confirmed, dismissed)` — the caller handles cleanup on `confirmed`
/// and should clear `pending_delete` on either flag.
pub fn delete_dialog(ctx: &egui::Context, title: &str, item_name: &str) -> (bool, bool) {
    let mut confirmed = false;
    let mut dismissed = false;

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!("Delete \"{item_name}\"?"));
            ui.label(
                egui::RichText::new("This cannot be undone.").color(ui.visuals().warn_fg_color),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Delete").color(egui::Color32::WHITE),
                        )
                        .fill(DANGER_RED),
                    )
                    .clicked()
                {
                    confirmed = true;
                }
                if ui.button("Cancel").clicked() {
                    dismissed = true;
                }
            });
        });

    (confirmed, dismissed)
}

/// Expand/collapse arrow button for accordion rows. Returns `true` if clicked.
pub fn expand_button(ui: &mut egui::Ui, expanded: bool) -> bool {
    let arrow = if expanded { "▼" } else { "▶" };
    let hover = if expanded { "Collapse" } else { "Expand" };
    ui.add(egui::Button::new(arrow).fill(egui::Color32::TRANSPARENT))
        .on_hover_text(hover)
        .clicked()
}

/// Detail-panel toggle button (⊞/⊟). Returns `true` if clicked.
pub fn panel_toggle_button(ui: &mut egui::Ui, is_open: bool) -> bool {
    let icon = if is_open { "⊟" } else { "⊞" };
    let hover = if is_open {
        "Close detail panel"
    } else {
        "Open detail panel"
    };
    ui.add(egui::Button::new(icon).fill(egui::Color32::TRANSPARENT))
        .on_hover_text(hover)
        .clicked()
}

/// Small red ✕ button used to remove a link between two entities.
pub fn unlink_button(ui: &mut egui::Ui) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new("✕").small().color(UNLINK_RED))
            .fill(egui::Color32::TRANSPARENT),
    )
    .on_hover_text("Remove link")
}

/// Renders a linked-items section inside an expanded accordion row.
///
/// `available` and `linked` are pre-computed `(id, display_name)` pairs where
/// `available` contains items not yet linked and `linked` contains items already linked.
/// When `navigate_hover` is `Some("…")`, linked items render as clickable links and
/// `navigate_to` is set to the clicked ID; when `None`, items render as plain labels.
///
/// Returns `(link_to_add_id, link_to_remove_id)`. The caller is responsible for
/// forming the full link tuple from the returned ID and the owning entity's ID.
#[expect(clippy::too_many_arguments)]
pub fn acc_link_section(
    ui: &mut egui::Ui,
    label: &str,
    combo_key: egui::Id,
    add_prompt: &str,
    all_linked_text: &str,
    available: &[(Uuid, String)],
    linked: &[(Uuid, String)],
    navigate_to: &mut Option<Uuid>,
    navigate_hover: Option<&str>,
) -> (Option<Uuid>, Option<Uuid>) {
    ui.label(label);
    let mut link_to_add = None;
    let mut link_to_remove = None;

    if !available.is_empty() {
        let avail_w = ui.available_width();
        if let Some(sel) = link_combo_pick(ui, combo_key, |ui, sel| {
            egui::ComboBox::from_id_salt(combo_key)
                .selected_text(add_prompt)
                .width(avail_w)
                .show_ui(ui, |ui| {
                    for (id, name) in available {
                        ui.selectable_value(sel, *id, name);
                    }
                });
        }) {
            link_to_add = Some(sel);
        }
    } else {
        ui.add_enabled(false, egui::Button::new(all_linked_text));
    }

    if linked.is_empty() {
        none_label(ui);
    } else {
        for (linked_id, name) in linked {
            ui.horizontal(|ui| {
                if let Some(hover) = navigate_hover {
                    if ui.link(name).on_hover_text(hover).clicked() {
                        *navigate_to = Some(*linked_id);
                    }
                } else {
                    ui.label(name);
                }
                if unlink_button(ui).clicked() {
                    link_to_remove = Some(*linked_id);
                }
            });
        }
    }

    (link_to_add, link_to_remove)
}

/// Renders one linked-items grid row inside a detail-panel window: the label
/// in the left cell, then the linked list + add-combo in the right cell.
///
/// `linked` and `available` are pre-computed `(id, display_name)` pairs.
/// When `navigate_hover` is `Some("…")`, linked items render as clickable links
/// that set `navigate_to`; when `None`, items render as plain labels.
///
/// Returns `(link_to_add_id, link_to_remove_id)`. The caller builds the full
/// link tuple and calls `ui.end_row()` afterwards.
#[expect(clippy::too_many_arguments)]
pub fn detail_link_row(
    ui: &mut egui::Ui,
    label: &str,
    combo_key: egui::Id,
    add_prompt: &str,
    available: &[(Uuid, String)],
    linked: &[(Uuid, String)],
    navigate_to: &mut Option<Uuid>,
    navigate_hover: Option<&str>,
) -> (Option<Uuid>, Option<Uuid>) {
    let mut link_to_add: Option<Uuid> = None;
    let mut link_to_remove: Option<Uuid> = None;

    ui.label(label);
    ui.vertical(|ui| {
        if linked.is_empty() {
            none_label(ui);
        } else {
            for (item_id, item_name) in linked {
                ui.horizontal(|ui| {
                    if let Some(hover) = navigate_hover {
                        if ui.link(item_name).on_hover_text(hover).clicked() {
                            *navigate_to = Some(*item_id);
                        }
                    } else {
                        ui.label(item_name);
                    }
                    if unlink_button(ui).clicked() {
                        link_to_remove = Some(*item_id);
                    }
                });
            }
        }
        if !available.is_empty() {
            ui.add_space(4.0);
            let avail_w = ui.available_width();
            if let Some(sel) = link_combo_pick(ui, combo_key, |ui, sel| {
                egui::ComboBox::from_id_salt(combo_key)
                    .selected_text(add_prompt)
                    .width(avail_w)
                    .show_ui(ui, |ui| {
                        for (item_id, item_name) in available {
                            ui.selectable_value(sel, *item_id, item_name);
                        }
                    });
            }) {
                link_to_add = Some(sel);
            }
        }
    });

    (link_to_add, link_to_remove)
}

/// Splits `items` into `(linked, available)` lists for a detail panel snapshot.
///
/// `links` is a `[(a_id, b_id)]` link table. `extract_other_id` receives each
/// link tuple and returns `Some(foreign_id)` when the tuple belongs to this
/// entity, or `None` to skip it — the closure encodes which position holds
/// `self_id` and which holds the foreign key.
///
/// Both returned vecs are `(id, name)` pairs ready to pass to
/// [`detail_link_row`] or [`acc_link_section`].
///
/// # Example
/// ```rust,ignore
/// // Link table stores (job_id, segment_id). Self is a segment, so filter on
/// // the second position and return the first (the job id).
/// let (linked_jobs, available_jobs) = accordion::partition_linked(
///     &app.segment_job_links,
///     |(jid, sid)| (*sid == self_id).then_some(*jid),
///     &app.jobs,
///     |j| j.id,
///     |j| j.name.as_str(),
/// );
/// ```
#[expect(clippy::type_complexity)]
pub fn partition_linked<L, T>(
    links: &[L],
    extract_other_id: impl Fn(&L) -> Option<Uuid>,
    items: &[T],
    get_id: impl Fn(&T) -> Uuid,
    get_name: impl Fn(&T) -> &str,
) -> (Vec<(Uuid, String)>, Vec<(Uuid, String)>) {
    let linked_ids: Vec<Uuid> = links.iter().filter_map(extract_other_id).collect();
    let linked = items
        .iter()
        .filter_map(|item| {
            let id = get_id(item);
            linked_ids
                .contains(&id)
                .then(|| (id, get_name(item).to_owned()))
        })
        .collect();
    let available = items
        .iter()
        .filter_map(|item| {
            let id = get_id(item);
            (!linked_ids.contains(&id)).then(|| (id, get_name(item).to_owned()))
        })
        .collect();
    (linked, available)
}

/// Returns `(name_width, description_width)` for a collapsed accordion row,
/// reserving space for two 36 px action buttons on the right.
/// `name_label` must match the label passed to [`header`].
pub fn row_field_widths(ui: &egui::Ui, name_label: &str) -> (f32, f32) {
    let spacing = ui.spacing().item_spacing.x;
    // Account for the drag handle allocated in the header so description columns align.
    let btn_space = ACTION_BTN_W * 2.0 + spacing * 2.0;
    let avail = ui.available_width() - btn_space;
    let (name_w, _) = current_name_w(ui, name_label);
    let name_w = name_w.min(avail - DRAG_HANDLE_W - spacing * 2.0);
    let desc_w = (avail - name_w - DRAG_HANDLE_W - spacing * 2.0).max(0.0);
    (name_w, desc_w)
}

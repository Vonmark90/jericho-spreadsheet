use super::theme::SpreadsheetTheme;
use crate::engine::dependency::recalculate_sheet;
use crate::model::workbook::Workbook;
use egui::{Key, Response, RichText, Ui, Vec2};

pub struct TabsState {
    pub renaming_idx: Option<usize>,
    pub rename_buffer: String,
}

impl Default for TabsState {
    fn default() -> Self {
        Self {
            renaming_idx: None,
            rename_buffer: String::new(),
        }
    }
}

pub fn render_tabs(
    ui: &mut Ui,
    theme: &SpreadsheetTheme,
    state: &mut TabsState,
    wb: &mut Workbook,
) -> bool {
    let mut modified = false;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);

        // Add Sheet '+' button
        let add_btn = ui.add(
            egui::Button::new(
                RichText::new("＋")
                    .font(egui::FontId::new(13.5, egui::FontFamily::Name("Bold".into())))
                    .color(theme.tab_active_fg),
            )
            .min_size(Vec2::new(28.0, 26.0)),
        );
        if add_btn.clicked() {
            wb.add_sheet(None);
            recalculate_sheet(wb.active_sheet_mut());
            modified = true;
        }

        ui.separator();

        let num_sheets = wb.sheets.len();
        let mut switch_to = None;
        let mut delete_sheet = None;

        for i in 0..num_sheets {
            let is_active = i == wb.active_sheet_index;
            let sheet_name = wb.sheets[i].name.clone();

            if state.renaming_idx == Some(i) {
                let text_edit: Response = ui.add_sized(
                    [110.0, 26.0],
                    egui::TextEdit::singleline(&mut state.rename_buffer)
                        .font(egui::FontId::new(13.0, egui::FontFamily::Proportional)),
                );

                if text_edit.lost_focus() || ui.input(|inp| inp.key_pressed(Key::Enter)) {
                    wb.rename_sheet(i, state.rename_buffer.clone());
                    state.renaming_idx = None;
                    modified = true;
                }
                if ui.input(|inp| inp.key_pressed(Key::Escape)) {
                    state.renaming_idx = None;
                }
            } else {
                let text = if is_active {
                    RichText::new(&sheet_name)
                        .font(egui::FontId::new(13.5, egui::FontFamily::Name("Bold".into())))
                        .color(theme.tab_active_fg)
                } else {
                    RichText::new(&sheet_name)
                        .font(egui::FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(theme.tab_inactive_fg)
                };

                let tab_btn = ui.add(
                    egui::Button::new(text)
                        .min_size(Vec2::new(76.0, 26.0))
                        .fill(if is_active {
                            theme.tab_active_bg
                        } else {
                            theme.tab_inactive_bg
                        }),
                );

                if tab_btn.clicked() {
                    switch_to = Some(i);
                }

                if tab_btn.double_clicked() {
                    state.renaming_idx = Some(i);
                    state.rename_buffer = sheet_name.clone();
                }

                tab_btn.context_menu(|ui| {
                    if ui.button("Rename Sheet").clicked() {
                        state.renaming_idx = Some(i);
                        state.rename_buffer = sheet_name.clone();
                        ui.close_menu();
                    }
                    if num_sheets > 1 && ui.button("Delete Sheet").clicked() {
                        delete_sheet = Some(i);
                        ui.close_menu();
                    }
                });
            }
        }

        if let Some(idx) = switch_to {
            wb.active_sheet_index = idx;
            recalculate_sheet(wb.active_sheet_mut());
            modified = true;
        }

        if let Some(idx) = delete_sheet {
            wb.remove_sheet(idx);
            modified = true;
        }
    });

    modified
}

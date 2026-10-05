use super::theme::SpreadsheetTheme;
use crate::engine::dependency::recalculate_incremental;
use crate::model::cell::CellCoord;
use crate::model::workbook::Workbook;
use egui::{Key, Response, RichText, Ui, Vec2};

pub struct FormulaBarState {
    pub text_buffer: String,
    pub active_coord: CellCoord,
    pub is_editing: bool,
    pub name_box_buffer: String,
}

impl Default for FormulaBarState {
    fn default() -> Self {
        Self {
            text_buffer: String::new(),
            active_coord: CellCoord::new(0, 0),
            is_editing: false,
            name_box_buffer: "A1".to_string(),
        }
    }
}

impl FormulaBarState {
    pub fn sync_with_cell(&mut self, wb: &Workbook) {
        let sheet = wb.active_sheet();
        let cursor = sheet.selection.cursor;
        if self.active_coord != cursor || !self.is_editing {
            self.active_coord = cursor;
            self.name_box_buffer = if sheet.selection.range.is_single_cell() {
                cursor.to_a1()
            } else {
                sheet.selection.range.to_range_string()
            };
            let raw = sheet
                .get_cell(cursor)
                .map(|c| c.raw_input.as_str())
                .unwrap_or("");
            self.text_buffer = raw.to_string();
            self.is_editing = false;
        }
    }
}

pub fn render_formula_bar(
    ui: &mut Ui,
    theme: &SpreadsheetTheme,
    state: &mut FormulaBarState,
    wb: &mut Workbook,
) -> bool {
    let mut modified = false;
    let sheet = wb.active_sheet();
    let current_cursor = sheet.selection.cursor;

    if state.active_coord != current_cursor && !state.is_editing {
        state.sync_with_cell(wb);
    }

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);

        // --- Name Box (A1 Coordinate) ---
        ui.style_mut().visuals.extreme_bg_color = theme.formula_bar_bg;
        let name_box = ui.add_sized(
            [76.0, 26.0],
            egui::TextEdit::singleline(&mut state.name_box_buffer)
                .font(egui::FontId::new(13.5, egui::FontFamily::Name("Bold".into())))
                .hint_text("A1")
                .horizontal_align(egui::Align::Center),
        );

        if name_box.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
            if let Ok(target) = CellCoord::from_a1(&state.name_box_buffer) {
                wb.active_sheet_mut().selection.select_cell(target);
                state.sync_with_cell(wb);
            }
        }

        // --- fx button ---
        let fx_btn = ui.add(
            egui::Button::new(
                RichText::new("fx")
                    .font(egui::FontId::new(14.0, egui::FontFamily::Name("Bold".into())))
                    .italics()
                    .color(theme.ribbon_accent),
            )
            .min_size(Vec2::new(30.0, 26.0)),
        );

        if fx_btn.clicked() {
            if !state.text_buffer.starts_with('=') {
                state.text_buffer = format!("={}", state.text_buffer);
            }
            state.is_editing = true;
        }

        // --- Commit / Cancel buttons when editing ---
        if state.is_editing {
            let commit_btn = ui.add(
                egui::Button::new(
                    RichText::new("✓")
                        .font(egui::FontId::new(14.0, egui::FontFamily::Name("Bold".into())))
                        .color(egui::Color32::from_rgb(16, 124, 65)),
                )
                .min_size(Vec2::new(26.0, 26.0)),
            );
            if commit_btn.clicked() {
                commit_formula(wb, state);
                modified = true;
            }

            let cancel_btn = ui.add(
                egui::Button::new(
                    RichText::new("✕")
                        .font(egui::FontId::new(14.0, egui::FontFamily::Name("Bold".into())))
                        .color(egui::Color32::from_rgb(200, 50, 50)),
                )
                .min_size(Vec2::new(26.0, 26.0)),
            );
            if cancel_btn.clicked() {
                state.sync_with_cell(wb);
            }
        }

        // --- Formula / Value Text Input ---
        let formula_edit: Response = ui.add_sized(
            [ui.available_width() - 8.0, 26.0],
            egui::TextEdit::singleline(&mut state.text_buffer)
                .font(egui::FontId::new(14.0, egui::FontFamily::Monospace))
                .hint_text("Enter formula or value (e.g. =SUM(A1:A10))"),
        );

        if formula_edit.changed() {
            state.is_editing = true;
        }

        if formula_edit.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
            commit_formula(wb, state);
            // Move cursor down on Enter like Excel
            wb.active_sheet_mut().selection.move_cursor(1, 0, false);
            state.sync_with_cell(wb);
            modified = true;
        }

        if formula_edit.lost_focus() && ui.input(|i| i.key_pressed(Key::Escape)) {
            state.sync_with_cell(wb);
        }
    });

    modified
}

fn commit_formula(wb: &mut Workbook, state: &mut FormulaBarState) {
    let coord = state.active_coord;
    let sheet = wb.active_sheet_mut();
    sheet.push_undo();
    sheet.set_cell_input(coord, state.text_buffer.clone());
    recalculate_incremental(sheet, coord);
    wb.is_dirty = true;
    state.is_editing = false;
}

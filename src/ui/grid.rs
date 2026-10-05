use super::theme::SpreadsheetTheme;
use crate::engine::dependency::{recalculate_incremental, recalculate_sheet};
use crate::model::cell::{
    CellCoord, CellData, HorizAlign,
};
use crate::model::selection::SelectionRange;
use crate::model::sheet::{
    COL_HEADER_HEIGHT, DEFAULT_COL_WIDTH, DEFAULT_ROW_HEIGHT, ROW_HEADER_WIDTH, Sheet,
};
use crate::model::workbook::Workbook;
use egui::{
    pos2, vec2, Align2, Color32, CursorIcon, FontId, Key, Pos2, Rect,
    Sense, Stroke, StrokeKind, Ui, Vec2,
};

pub const TOTAL_ROWS: usize = 100_000;
pub const TOTAL_COLS: usize = 200;

#[derive(Debug, Default)]
pub struct GridState {
    pub is_cell_editing: bool,
    pub cell_edit_buffer: String,
    pub edit_coord: CellCoord,
    pub resizing_col: Option<(usize, f32, f32)>, // (col_idx, start_mouse_x, start_width)
    pub resizing_row: Option<(usize, f32, f32)>, // (row_idx, start_mouse_y, start_height)
    pub is_dragging_autofill: bool,
    pub autofill_end_coord: Option<CellCoord>,
    pub focus_editor_next_frame: bool,
}

pub struct GridLayout {
    pub col_offsets: Vec<f32>,
    pub total_cols_width: f32,
    pub row_prefix: Vec<f32>,
    pub max_custom_row: usize,
    pub total_rows_height: f32,
    pub origin: Pos2,
}

impl GridLayout {
    pub fn new(sheet: &Sheet, origin: Pos2) -> Self {
        // Precompute prefix sums for all columns
        let mut col_offsets = Vec::with_capacity(TOTAL_COLS + 1);
        col_offsets.push(0.0f32);
        for c in 0..TOTAL_COLS {
            let w = sheet.col_width(c);
            let prev = *col_offsets.last().unwrap();
            col_offsets.push(prev + w);
        }
        let total_cols_width = *col_offsets.last().unwrap();

        // Precompute prefix sums for customized rows (up to max customized row + 1)
        let max_custom = sheet.row_heights.keys().copied().max().unwrap_or(0);
        let prefix_len = (max_custom + 1).min(TOTAL_ROWS);
        let mut row_prefix = Vec::with_capacity(prefix_len + 1);
        row_prefix.push(0.0f32);
        for r in 0..prefix_len {
            let h = sheet.row_height(r);
            let prev = *row_prefix.last().unwrap();
            row_prefix.push(prev + h);
        }

        let total_rows_height = if prefix_len < TOTAL_ROWS {
            row_prefix[prefix_len] + ((TOTAL_ROWS - prefix_len) as f32) * DEFAULT_ROW_HEIGHT
        } else {
            row_prefix[prefix_len]
        };

        Self {
            col_offsets,
            total_cols_width,
            row_prefix,
            max_custom_row: prefix_len,
            total_rows_height,
            origin,
        }
    }

    pub fn col_left(&self, c: usize) -> f32 {
        let offset = if c < self.col_offsets.len() {
            self.col_offsets[c]
        } else {
            self.total_cols_width + ((c - TOTAL_COLS) as f32) * DEFAULT_COL_WIDTH
        };
        self.origin.x + ROW_HEADER_WIDTH + offset
    }

    pub fn col_right(&self, c: usize) -> f32 {
        let offset = if c + 1 < self.col_offsets.len() {
            self.col_offsets[c + 1]
        } else {
            self.total_cols_width + ((c + 1 - TOTAL_COLS) as f32) * DEFAULT_COL_WIDTH
        };
        self.origin.x + ROW_HEADER_WIDTH + offset
    }

    pub fn col_width(&self, c: usize) -> f32 {
        self.col_right(c) - self.col_left(c)
    }

    pub fn row_top(&self, r: usize) -> f32 {
        let offset = if r < self.row_prefix.len() {
            self.row_prefix[r]
        } else {
            self.row_prefix[self.max_custom_row]
                + ((r - self.max_custom_row) as f32) * DEFAULT_ROW_HEIGHT
        };
        self.origin.y + COL_HEADER_HEIGHT + offset
    }

    pub fn row_bottom(&self, r: usize) -> f32 {
        let offset = if r + 1 < self.row_prefix.len() {
            self.row_prefix[r + 1]
        } else {
            self.row_prefix[self.max_custom_row]
                + ((r + 1 - self.max_custom_row) as f32) * DEFAULT_ROW_HEIGHT
        };
        self.origin.y + COL_HEADER_HEIGHT + offset
    }

    pub fn row_height(&self, r: usize) -> f32 {
        self.row_bottom(r) - self.row_top(r)
    }

    pub fn cell_rect(&self, r: usize, c: usize) -> Rect {
        Rect::from_min_max(
            pos2(self.col_left(c), self.row_top(r)),
            pos2(self.col_right(c), self.row_bottom(r)),
        )
    }

    pub fn col_header_rect(&self, c: usize, visible_y: f32) -> Rect {
        Rect::from_min_max(
            pos2(self.col_left(c), visible_y),
            pos2(self.col_right(c), visible_y + COL_HEADER_HEIGHT),
        )
    }

    pub fn row_header_rect(&self, r: usize, visible_x: f32) -> Rect {
        Rect::from_min_max(
            pos2(visible_x, self.row_top(r)),
            pos2(visible_x + ROW_HEADER_WIDTH, self.row_bottom(r)),
        )
    }

    pub fn col_at(&self, screen_x: f32) -> usize {
        let rel_x = screen_x - self.origin.x - ROW_HEADER_WIDTH;
        if rel_x <= 0.0 {
            return 0;
        }
        let idx = self.col_offsets.partition_point(|&off| off <= rel_x);
        idx.saturating_sub(1).min(TOTAL_COLS.saturating_sub(1))
    }

    pub fn row_at(&self, screen_y: f32) -> usize {
        let rel_y = screen_y - self.origin.y - COL_HEADER_HEIGHT;
        if rel_y <= 0.0 {
            return 0;
        }
        if !self.row_prefix.is_empty() && rel_y <= *self.row_prefix.last().unwrap() {
            let idx = self.row_prefix.partition_point(|&off| off <= rel_y);
            idx.saturating_sub(1).min(TOTAL_ROWS.saturating_sub(1))
        } else {
            let last_prefix = *self.row_prefix.last().unwrap_or(&0.0);
            let rem_y = rel_y - last_prefix;
            let add_rows = (rem_y / DEFAULT_ROW_HEIGHT).floor() as usize;
            (self.max_custom_row + add_rows).min(TOTAL_ROWS.saturating_sub(1))
        }
    }
}

pub fn render_grid(
    ui: &mut Ui,
    theme: &SpreadsheetTheme,
    state: &mut GridState,
    wb: &mut Workbook,
) -> bool {
    let mut modified = false;

    // Outer scroll area for the spreadsheet
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let sheet = wb.active_sheet_mut();
            modified |= render_grid_canvas(ui, theme, state, sheet);
        });

    modified
}

fn render_grid_canvas(
    ui: &mut Ui,
    theme: &SpreadsheetTheme,
    state: &mut GridState,
    sheet: &mut Sheet,
) -> bool {
    let mut modified = false;

    // Build the dynamic grid layout from current column widths and row heights
    let dummy_origin = Pos2::ZERO;
    let temp_layout = GridLayout::new(sheet, dummy_origin);
    let total_width = ROW_HEADER_WIDTH + temp_layout.total_cols_width;
    let total_height = COL_HEADER_HEIGHT + temp_layout.total_rows_height;

    let (response, painter) =
        ui.allocate_painter(Vec2::new(total_width, total_height), Sense::click_and_drag());
    let origin = response.rect.min;
    let visible_rect = ui.clip_rect();

    // Reconstruct with actual origin
    let layout = GridLayout::new(sheet, origin);

    // Compute visible row range based on viewport
    let header_y = visible_rect.min.y.max(origin.y);
    let header_x = visible_rect.min.x.max(origin.x);

    let min_row = layout.row_at(visible_rect.min.y);
    let max_row = (layout.row_at(visible_rect.max.y) + 1).min(TOTAL_ROWS.saturating_sub(1));

    let min_col = layout.col_at(visible_rect.min.x);
    let max_col = (layout.col_at(visible_rect.max.x) + 1).min(TOTAL_COLS.saturating_sub(1));

    // --- Active Dragging of Column or Row Resizers ---
    let pointer_pos = ui.input(|i| i.pointer.latest_pos());

    if let Some((col_idx, start_x, start_w)) = state.resizing_col {
        ui.output_mut(|o| o.cursor_icon = CursorIcon::ResizeColumn);
        if let Some(pos) = pointer_pos {
            let delta = pos.x - start_x;
            let new_w = (start_w + delta).clamp(35.0, 700.0);
            sheet.set_col_width(col_idx, new_w);
            modified = true;
        }
        if ui.input(|i| i.pointer.primary_released()) {
            state.resizing_col = None;
            sheet.push_undo();
            modified = true;
        }
    } else if let Some((row_idx, start_y, start_h)) = state.resizing_row {
        ui.output_mut(|o| o.cursor_icon = CursorIcon::ResizeRow);
        if let Some(pos) = pointer_pos {
            let delta = pos.y - start_y;
            let new_h = (start_h + delta).clamp(18.0, 350.0);
            sheet.set_row_height(row_idx, new_h);
            modified = true;
        }
        if ui.input(|i| i.pointer.primary_released()) {
            state.resizing_row = None;
            sheet.push_undo();
            modified = true;
        }
    }

    // --- Keyboard Navigation & Shortcuts ---
    if ui.input(|i| i.modifiers.command || i.modifiers.ctrl) {
        // Ctrl+Z: Undo
        if ui.input(|i| i.key_pressed(Key::Z)) && !ui.input(|i| i.modifiers.shift) {
            if sheet.undo() {
                recalculate_sheet(sheet);
                modified = true;
            }
        }
        // Ctrl+Y or Ctrl+Shift+Z: Redo
        if ui.input(|i| i.key_pressed(Key::Y))
            || (ui.input(|i| i.modifiers.shift && i.key_pressed(Key::Z)))
        {
            if sheet.redo() {
                recalculate_sheet(sheet);
                modified = true;
            }
        }
        // Ctrl+C: Copy TSV to clipboard
        if ui.input(|i| i.key_pressed(Key::C)) {
            copy_selection_to_clipboard(ui, sheet);
        }
        // Ctrl+V: Paste TSV from clipboard
        if ui.input(|i| i.key_pressed(Key::V)) {
            if paste_from_clipboard(ui, sheet) {
                modified = true;
            }
        }
        // Ctrl+B: Bold
        if ui.input(|i| i.key_pressed(Key::B)) {
            let range = sheet.selection.range;
            let current_bold = sheet
                .get_cell(range.start)
                .map(|c| c.format.bold)
                .unwrap_or(false);
            sheet.format_range(range, |fmt| fmt.bold = !current_bold);
            modified = true;
        }
        // Ctrl+I: Italic
        if ui.input(|i| i.key_pressed(Key::I)) {
            let range = sheet.selection.range;
            let current_it = sheet
                .get_cell(range.start)
                .map(|c| c.format.italic)
                .unwrap_or(false);
            sheet.format_range(range, |fmt| fmt.italic = !current_it);
            modified = true;
        }
        // Ctrl+U: Underline
        if ui.input(|i| i.key_pressed(Key::U)) {
            let range = sheet.selection.range;
            let current_u = sheet
                .get_cell(range.start)
                .map(|c| c.format.underline)
                .unwrap_or(false);
            sheet.format_range(range, |fmt| fmt.underline = !current_u);
            modified = true;
        }
    } else if !state.is_cell_editing && state.resizing_col.is_none() && state.resizing_row.is_none() {
        // Arrow navigation
        let shift = ui.input(|i| i.modifiers.shift);
        if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
            sheet.selection.move_cursor(-1, 0, shift);
        }
        if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
            sheet.selection.move_cursor(1, 0, shift);
        }
        if ui.input(|i| i.key_pressed(Key::ArrowLeft)) {
            sheet.selection.move_cursor(0, -1, shift);
        }
        if ui.input(|i| i.key_pressed(Key::ArrowRight)) {
            sheet.selection.move_cursor(0, 1, shift);
        }
        if ui.input(|i| i.key_pressed(Key::Enter)) {
            let d_row = if shift { -1 } else { 1 };
            sheet.selection.move_cursor(d_row, 0, false);
        }
        if ui.input(|i| i.key_pressed(Key::Tab)) {
            let d_col = if shift { -1 } else { 1 };
            sheet.selection.move_cursor(0, d_col, false);
        }
        // Delete / Backspace: Clear range
        if ui.input(|i| i.key_pressed(Key::Backspace) || i.key_pressed(Key::Delete)) {
            sheet.push_undo();
            sheet.clear_range(sheet.selection.range);
            recalculate_sheet(sheet);
            modified = true;
        }
        // F2: Start editing active cell
        if ui.input(|i| i.key_pressed(Key::F2)) {
            start_editing(sheet.selection.cursor, sheet, state);
        }
    }

    // --- Mouse Interaction: Headers, Grid, Autofill & Dividers ---
    if let Some(mouse_pos) = pointer_pos {
        let is_over_col_headers = mouse_pos.y >= header_y
            && mouse_pos.y <= header_y + COL_HEADER_HEIGHT
            && mouse_pos.x >= origin.x + ROW_HEADER_WIDTH;

        let is_over_row_headers = mouse_pos.x >= header_x
            && mouse_pos.x <= header_x + ROW_HEADER_WIDTH
            && mouse_pos.y >= origin.y + COL_HEADER_HEIGHT;

        let is_over_grid = mouse_pos.x > origin.x + ROW_HEADER_WIDTH
            && mouse_pos.y > origin.y + COL_HEADER_HEIGHT;

        // 1. Column header boundary hover / drag detection
        if is_over_col_headers && state.resizing_col.is_none() && state.resizing_row.is_none() {
            let mut hovered_col_divider = None;
            for c in min_col..=max_col {
                let right_x = layout.col_right(c);
                if (mouse_pos.x - right_x).abs() <= 5.0 {
                    hovered_col_divider = Some(c);
                    break;
                }
            }

            if let Some(c) = hovered_col_divider {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::ResizeColumn);
                if ui.input(|i| i.pointer.primary_down()) {
                    state.resizing_col = Some((c, mouse_pos.x, sheet.col_width(c)));
                }
                if response.double_clicked() {
                    sheet.push_undo();
                    sheet.auto_fit_col(c);
                    modified = true;
                }
            } else if response.clicked() {
                // Click column header: select entire column
                let c = layout.col_at(mouse_pos.x);
                let max_used = (sheet.max_used_row() + 25).min(TOTAL_ROWS.saturating_sub(1));
                sheet.selection.range = SelectionRange::new(
                    CellCoord::new(0, c),
                    CellCoord::new(max_used, c),
                );
                sheet.selection.cursor = CellCoord::new(0, c);
            }
        }

        // 2. Row header boundary hover / drag detection
        else if is_over_row_headers && state.resizing_row.is_none() && state.resizing_col.is_none() {
            let mut hovered_row_divider = None;
            for r in min_row..=max_row {
                let bottom_y = layout.row_bottom(r);
                if (mouse_pos.y - bottom_y).abs() <= 4.0 {
                    hovered_row_divider = Some(r);
                    break;
                }
            }

            if let Some(r) = hovered_row_divider {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::ResizeRow);
                if ui.input(|i| i.pointer.primary_down()) {
                    state.resizing_row = Some((r, mouse_pos.y, sheet.row_height(r)));
                }
                if response.double_clicked() {
                    sheet.push_undo();
                    sheet.set_row_height(r, DEFAULT_ROW_HEIGHT);
                    modified = true;
                }
            } else if response.clicked() {
                // Click row header: select entire row
                let r = layout.row_at(mouse_pos.y);
                let max_used = (sheet.max_used_col() + 20).min(TOTAL_COLS.saturating_sub(1));
                sheet.selection.range = SelectionRange::new(
                    CellCoord::new(r, 0),
                    CellCoord::new(r, max_used),
                );
                sheet.selection.cursor = CellCoord::new(r, 0);
            }
        }

        // 3. Grid area interaction
        else if is_over_grid && state.resizing_col.is_none() && state.resizing_row.is_none() {
            let hovered_col = layout.col_at(mouse_pos.x);
            let hovered_row = layout.row_at(mouse_pos.y);
            let hovered_coord = CellCoord::new(hovered_row, hovered_col);

            // Check if mouse is hovering over the autofill handle of current selection
            let sel = &sheet.selection.range;
            let br_rect = layout.cell_rect(sel.max_row(), sel.max_col());
            let handle_rect = Rect::from_center_size(br_rect.max, vec2(9.0, 9.0));

            if handle_rect.contains(mouse_pos) && !state.is_cell_editing {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::Crosshair);
                if ui.input(|i| i.pointer.primary_down()) {
                    state.is_dragging_autofill = true;
                }
            }

            if state.is_dragging_autofill {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::Crosshair);
                state.autofill_end_coord = Some(hovered_coord);

                if ui.input(|i| i.pointer.primary_released()) {
                    if let Some(target) = state.autofill_end_coord {
                        perform_autofill(sheet, target);
                        modified = true;
                    }
                    state.is_dragging_autofill = false;
                    state.autofill_end_coord = None;
                }
            } else if response.clicked() {
                if state.is_cell_editing {
                    commit_in_cell_editor(sheet, state);
                    modified = true;
                }
                if ui.input(|i| i.modifiers.shift) {
                    sheet.selection.extend_to(hovered_coord);
                } else {
                    sheet.selection.select_cell(hovered_coord);
                }
            } else if response.double_clicked() {
                start_editing(hovered_coord, sheet, state);
            } else if response.dragged() && !state.is_cell_editing {
                sheet.selection.extend_to(hovered_coord);
            }
        }
    }

    // --- Paint Grid Background & Cells ---
    painter.rect_filled(response.rect, 0.0, theme.cell_bg);

    // Draw visible cell backgrounds and text
    for r in min_row..=max_row {
        for c in min_col..=max_col {
            let coord = CellCoord::new(r, c);
            let rect = layout.cell_rect(r, c);

            // Cell custom background
            if let Some(cell) = sheet.get_cell(coord) {
                if let Some([cr, cg, cb, ca]) = cell.format.bg_color {
                    painter.rect_filled(rect, 0.0, Color32::from_rgba_premultiplied(cr, cg, cb, ca));
                }
            }

            // Cell border grid lines
            painter.line_segment([rect.right_top(), rect.right_bottom()], theme.grid_stroke());
            painter.line_segment([rect.left_bottom(), rect.right_bottom()], theme.grid_stroke());

            // Cell text (skip active cell if currently editing in overlay)
            if state.is_cell_editing && state.edit_coord == coord {
                continue;
            }

            if let Some(cell) = sheet.get_cell(coord) {
                let display_text = cell.value.format_display(&cell.format);
                if !display_text.is_empty() {
                    let text_color = if cell.value.is_error() {
                        Color32::from_rgb(220, 50, 50)
                    } else if let Some([tr, tg, tb, _]) = cell.format.text_color {
                        Color32::from_rgb(tr, tg, tb)
                    } else {
                        theme.cell_text
                    };

                    let font_family = if cell.format.bold {
                        egui::FontFamily::Name("Bold".into())
                    } else {
                        egui::FontFamily::Proportional
                    };
                    let font_id = FontId::new(cell.format.font_size, font_family);
                    let text_rect = rect.shrink2(vec2(6.0, 2.0));

                    let align = match cell.format.align_h {
                        HorizAlign::Left => Align2::LEFT_CENTER,
                        HorizAlign::Center => Align2::CENTER_CENTER,
                        HorizAlign::Right => Align2::RIGHT_CENTER,
                    };

                    let text_pos = match align {
                        Align2::LEFT_CENTER => text_rect.left_center(),
                        Align2::CENTER_CENTER => text_rect.center(),
                        Align2::RIGHT_CENTER => text_rect.right_center(),
                        _ => text_rect.left_center(),
                    };

                    painter.text(text_pos, align, display_text, font_id, text_color);
                }
            }
        }
    }

    // --- Paint Selection Box & Autofill Handle ---
    let sel = &sheet.selection.range;
    let sel_min_rect = layout.cell_rect(sel.min_row(), sel.min_col());
    let sel_max_rect = layout.cell_rect(sel.max_row(), sel.max_col());
    let selection_rect = Rect::from_min_max(sel_min_rect.min, sel_max_rect.max);

    // Range highlight fill
    painter.rect_filled(selection_rect, 0.0, theme.selection_fill);
    // Crisp 2px boundary border
    painter.rect_stroke(selection_rect, 0.0, theme.selection_stroke(), StrokeKind::Outside);

    // Autofill square handle at bottom-right of selection
    if !state.is_cell_editing {
        let handle_center = selection_rect.max;
        let handle_rect = Rect::from_center_size(handle_center, vec2(6.0, 6.0));
        painter.rect_filled(handle_rect, 0.0, theme.autofill_handle);
        painter.rect_stroke(
            handle_rect,
            0.0,
            Stroke::new(1.0f32, Color32::from_rgb(255, 255, 255)),
            StrokeKind::Outside,
        );
    }

    // If dragging autofill, paint dashed preview box
    if state.is_dragging_autofill {
        if let Some(target) = state.autofill_end_coord {
            let target_rect = layout.cell_rect(target.row, target.col);
            let preview_rect = Rect::from_min_max(
                pos2(
                    selection_rect.min.x.min(target_rect.min.x),
                    selection_rect.min.y.min(target_rect.min.y),
                ),
                pos2(
                    selection_rect.max.x.max(target_rect.max.x),
                    selection_rect.max.y.max(target_rect.max.y),
                ),
            );
            painter.rect_stroke(
                preview_rect,
                0.0,
                Stroke::new(1.5f32, theme.autofill_handle),
                StrokeKind::Outside,
            );
        }
    }

    // --- Paint Active Resize Guidelines if dragging divider ---
    if let Some((col_idx, _, _)) = state.resizing_col {
        let line_x = layout.col_right(col_idx);
        painter.line_segment(
            [pos2(line_x, header_y), pos2(line_x, visible_rect.max.y)],
            Stroke::new(2.0f32, theme.ribbon_accent),
        );
    }

    if let Some((row_idx, _, _)) = state.resizing_row {
        let line_y = layout.row_bottom(row_idx);
        painter.line_segment(
            [pos2(header_x, line_y), pos2(visible_rect.max.x, line_y)],
            Stroke::new(2.0f32, theme.ribbon_accent),
        );
    }

    // --- Paint Pinned Column Headers (A, B, C...) ---
    let header_bg_rect = Rect::from_min_max(
        pos2(origin.x, header_y),
        pos2(origin.x + ROW_HEADER_WIDTH + layout.total_cols_width, header_y + COL_HEADER_HEIGHT),
    );
    painter.rect_filled(header_bg_rect, 0.0, theme.header_bg);
    painter.line_segment(
        [header_bg_rect.left_bottom(), header_bg_rect.right_bottom()],
        theme.header_stroke(),
    );

    let sel_min_c = sheet.selection.range.min_col();
    let sel_max_c = sheet.selection.range.max_col();

    for c in min_col..=max_col {
        let h_rect = layout.col_header_rect(c, header_y);
        let is_selected = c >= sel_min_c && c <= sel_max_c;

        if is_selected {
            painter.rect_filled(h_rect, 0.0, theme.header_active_bg);
        }

        painter.line_segment([h_rect.right_top(), h_rect.right_bottom()], theme.header_stroke());

        let col_name = CellCoord::col_to_name(c);
        let fg = if is_selected {
            theme.header_active_fg
        } else {
            theme.header_fg
        };
        painter.text(
            h_rect.center(),
            Align2::CENTER_CENTER,
            col_name,
            FontId::new(12.5, egui::FontFamily::Name("Bold".into())),
            fg,
        );
    }

    // --- Paint Pinned Row Headers (1, 2, 3...) ---
    let row_header_bg = Rect::from_min_max(
        pos2(header_x, origin.y),
        pos2(header_x + ROW_HEADER_WIDTH, origin.y + COL_HEADER_HEIGHT + layout.total_rows_height),
    );
    painter.rect_filled(row_header_bg, 0.0, theme.header_bg);
    painter.line_segment(
        [row_header_bg.right_top(), row_header_bg.right_bottom()],
        theme.header_stroke(),
    );

    let sel_min_r = sheet.selection.range.min_row();
    let sel_max_r = sheet.selection.range.max_row();

    for r in min_row..=max_row {
        let r_rect = layout.row_header_rect(r, header_x);
        let is_selected = r >= sel_min_r && r <= sel_max_r;

        if is_selected {
            painter.rect_filled(r_rect, 0.0, theme.header_active_bg);
        }

        painter.line_segment([r_rect.left_bottom(), r_rect.right_bottom()], theme.header_stroke());

        let row_num = format!("{}", r + 1);
        let fg = if is_selected {
            theme.header_active_fg
        } else {
            theme.header_fg
        };
        painter.text(
            r_rect.center(),
            Align2::CENTER_CENTER,
            row_num,
            FontId::new(12.0, egui::FontFamily::Name("Bold".into())),
            fg,
        );
    }

    // --- Top-Left Select All Corner Cell ---
    let corner_rect = Rect::from_min_size(
        pos2(header_x, header_y),
        vec2(ROW_HEADER_WIDTH, COL_HEADER_HEIGHT),
    );
    painter.rect_filled(corner_rect, 0.0, theme.header_bg);
    painter.rect_stroke(corner_rect, 0.0, theme.header_stroke(), StrokeKind::Outside);

    // --- In-Cell Editor Overlay ---
    if state.is_cell_editing {
        let edit_rect = layout.cell_rect(state.edit_coord.row, state.edit_coord.col);
        let id = ui.make_persistent_id("in_cell_editor");

        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(edit_rect));
        let text_edit = child_ui.add(
            egui::TextEdit::singleline(&mut state.cell_edit_buffer)
                .id(id)
                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                .margin(vec2(5.0, 2.0)),
        );

        if state.focus_editor_next_frame {
            text_edit.request_focus();
            state.focus_editor_next_frame = false;
        }

        if text_edit.lost_focus() || ui.input(|i| i.key_pressed(Key::Enter)) {
            commit_in_cell_editor(sheet, state);
            sheet.selection.move_cursor(1, 0, false);
            modified = true;
        }

        if ui.input(|i| i.key_pressed(Key::Escape)) {
            state.is_cell_editing = false;
        }
    }

    modified
}

fn start_editing(coord: CellCoord, sheet: &Sheet, state: &mut GridState) {
    state.is_cell_editing = true;
    state.edit_coord = coord;
    state.focus_editor_next_frame = true;
    state.cell_edit_buffer = sheet
        .get_cell(coord)
        .map(|c| c.raw_input.clone())
        .unwrap_or_default();
}

fn commit_in_cell_editor(sheet: &mut Sheet, state: &mut GridState) {
    let coord = state.edit_coord;
    sheet.push_undo();
    sheet.set_cell_input(coord, state.cell_edit_buffer.clone());
    recalculate_incremental(sheet, coord);
    state.is_cell_editing = false;
}

fn perform_autofill(sheet: &mut Sheet, target: CellCoord) {
    sheet.push_undo();
    let sel = sheet.selection.range;
    let min_r = sel.min_row();
    let max_r = sel.max_row();
    let min_c = sel.min_col();
    let max_c = sel.max_col();

    // Downward fill
    if target.row > max_r {
        let pattern_len = max_r - min_r + 1;
        for r in (max_r + 1)..=target.row {
            let offset = r - max_r;
            let src_r = min_r + ((r - max_r - 1) % pattern_len);
            for c in min_c..=max_c {
                let src_coord = CellCoord::new(src_r, c);
                let dst_coord = CellCoord::new(r, c);
                if let Some(src_cell) = sheet.get_cell(src_coord).cloned() {
                    let new_input = if src_cell.is_formula() {
                        offset_formula_refs(&src_cell.raw_input, offset as isize, 0)
                    } else if let Some(n) = src_cell.value.as_number() {
                        // Linear increment
                        format!("{}", n + offset as f64)
                    } else {
                        src_cell.raw_input.clone()
                    };
                    let mut new_cell = CellData::new(new_input);
                    new_cell.format = src_cell.format;
                    sheet.set_cell_data(dst_coord, new_cell);
                }
            }
        }
        sheet.selection.range = SelectionRange::new(
            CellCoord::new(min_r, min_c),
            CellCoord::new(target.row, max_c),
        );
    }
    // Rightward fill
    else if target.col > max_c {
        let pattern_len = max_c - min_c + 1;
        for c in (max_c + 1)..=target.col {
            let offset = c - max_c;
            let src_c = min_c + ((c - max_c - 1) % pattern_len);
            for r in min_r..=max_r {
                let src_coord = CellCoord::new(r, src_c);
                let dst_coord = CellCoord::new(r, c);
                if let Some(src_cell) = sheet.get_cell(src_coord).cloned() {
                    let new_input = if src_cell.is_formula() {
                        offset_formula_refs(&src_cell.raw_input, 0, offset as isize)
                    } else if let Some(n) = src_cell.value.as_number() {
                        format!("{}", n + offset as f64)
                    } else {
                        src_cell.raw_input.clone()
                    };
                    let mut new_cell = CellData::new(new_input);
                    new_cell.format = src_cell.format;
                    sheet.set_cell_data(dst_coord, new_cell);
                }
            }
        }
        sheet.selection.range = SelectionRange::new(
            CellCoord::new(min_r, min_c),
            CellCoord::new(max_r, target.col),
        );
    }

    recalculate_sheet(sheet);
}

/// Offsets cell references in a formula string when dragging / autofilling
fn offset_formula_refs(formula: &str, d_row: isize, d_col: isize) -> String {
    let mut result = String::new();
    let mut chars = formula.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_ascii_alphabetic() {
            let mut letters = String::new();
            letters.push(c);
            while let Some(&next_c) = chars.peek() {
                if next_c.is_ascii_alphabetic() {
                    letters.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            let mut digits = String::new();
            while let Some(&next_c) = chars.peek() {
                if next_c.is_ascii_digit() {
                    digits.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            if !digits.is_empty() {
                if let (Some(col_idx), Ok(row_num)) =
                    (CellCoord::name_to_col(&letters), digits.parse::<usize>())
                {
                    let new_row = ((row_num as isize - 1) + d_row).max(0) as usize;
                    let new_col = (col_idx as isize + d_col).max(0) as usize;
                    result.push_str(&CellCoord::new(new_row, new_col).to_a1());
                    continue;
                }
            }
            result.push_str(&letters);
            result.push_str(&digits);
        } else {
            result.push(c);
        }
    }

    result
}

fn copy_selection_to_clipboard(ui: &mut Ui, sheet: &Sheet) {
    let sel = sheet.selection.range;
    let mut tsv = String::new();

    for r in sel.min_row()..=sel.max_row() {
        for c in sel.min_col()..=sel.max_col() {
            if c > sel.min_col() {
                tsv.push('\t');
            }
            if let Some(cell) = sheet.get_cell(CellCoord::new(r, c)) {
                tsv.push_str(&cell.value.as_string());
            }
        }
        tsv.push('\n');
    }

    ui.ctx().copy_text(tsv);
}

fn paste_from_clipboard(_ui: &mut Ui, sheet: &mut Sheet) -> bool {
    let tsv = match arboard::Clipboard::new().and_then(|mut cb| cb.get_text()) {
        Ok(text) => text,
        Err(_) => return false,
    };
    if tsv.trim().is_empty() {
        return false;
    }

    sheet.push_undo();
    let start = sheet.selection.cursor;

    for (r_offset, line) in tsv.lines().enumerate() {
        for (c_offset, val) in line.split('\t').enumerate() {
            let coord = CellCoord::new(start.row + r_offset, start.col + c_offset);
            sheet.set_cell_input(coord, val.to_string());
        }
    }

    recalculate_sheet(sheet);
    true
}

use super::theme::{SpreadsheetTheme, ThemeMode};
use crate::engine::dependency::{recalculate_incremental, recalculate_sheet};
use crate::io::{export_csv, import_csv, load_json, load_xlsx, save_xlsx};
use crate::model::cell::{CellCoord, CellData, CellFormat, HorizAlign, NumberFormat};
use crate::model::selection::SelectionRange;
use crate::model::workbook::Workbook;
use egui::{RichText, Ui, Vec2};
use rfd::FileDialog;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RibbonTab {
    Home,
    Formulas,
    Data,
    File,
}

pub struct RibbonState {
    pub active_tab: RibbonTab,
    pub status_message: Option<(String, std::time::Instant)>,
}

impl Default for RibbonState {
    fn default() -> Self {
        Self {
            active_tab: RibbonTab::Home,
            status_message: None,
        }
    }
}

pub fn render_ribbon(
    ui: &mut Ui,
    theme: &mut SpreadsheetTheme,
    state: &mut RibbonState,
    wb: &mut Workbook,
) -> bool {
    let mut modified = false;

    // --- Ribbon Tabs Header ---
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

        let tabs = [
            ("File", RibbonTab::File),
            ("Home", RibbonTab::Home),
            ("Formulas", RibbonTab::Formulas),
            ("Data", RibbonTab::Data),
        ];

        for (name, tab) in tabs {
            let is_selected = state.active_tab == tab;
            let text = if is_selected {
                RichText::new(name)
                    .font(egui::FontId::new(13.5, egui::FontFamily::Name("Bold".into())))
                    .color(if tab == RibbonTab::File {
                        theme.ribbon_accent
                    } else {
                        theme.ribbon_text
                    })
            } else {
                RichText::new(name)
                    .font(egui::FontId::new(13.5, egui::FontFamily::Proportional))
                    .color(theme.tab_inactive_fg)
            };

            let btn = ui.add(
                egui::Button::new(text)
                    .min_size(Vec2::new(52.0, 26.0))
                    .frame(is_selected),
            );

            if btn.clicked() {
                state.active_tab = tab;
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Theme toggle (Dark / Light)
            let theme_label = if theme.mode == ThemeMode::Dark {
                "☀ Light"
            } else {
                "🌙 Dark"
            };
            if ui
                .button(
                    RichText::new(theme_label)
                        .font(egui::FontId::new(12.0, egui::FontFamily::Proportional)),
                )
                .clicked()
            {
                *theme = if theme.mode == ThemeMode::Dark {
                    SpreadsheetTheme::light()
                } else {
                    SpreadsheetTheme::dark()
                };
            }

            // Filename / title indicator
            let file_title = wb
                .file_path
                .as_ref()
                .map(|p| std::path::Path::new(p).file_name().and_then(|n| n.to_str()).unwrap_or("Workbook"))
                .unwrap_or("Untitled Workbook");
            let dirty_mark = if wb.is_dirty { " •" } else { "" };
            ui.label(
                RichText::new(format!("{}{}", file_title, dirty_mark))
                    .font(egui::FontId::new(13.0, egui::FontFamily::Name("Bold".into())))
                    .color(theme.ribbon_text),
            );
        });
    });

    ui.separator();

    // --- Tab Contents Toolbar ---
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);

        match state.active_tab {
            RibbonTab::File => {
                if ui.button("📄 New").clicked() {
                    *wb = Workbook::new();
                    state.status_message = Some(("Created new workbook".to_string(), std::time::Instant::now()));
                    modified = true;
                }

                if ui.button("📂 Open XLSX / CSV").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Spreadsheets (*.xlsx, *.csv, *.json)", &["xlsx", "csv", "json"])
                        .pick_file()
                    {
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                        let result = if ext == "xlsx" {
                            load_xlsx(&path)
                        } else if ext == "csv" {
                            import_csv(&path).map(|sheet| {
                                let mut new_wb = Workbook::new();
                                new_wb.sheets = vec![sheet];
                                new_wb.file_path = Some(path.to_string_lossy().to_string());
                                new_wb
                            })
                        } else {
                            load_json(&path)
                        };

                        match result {
                            Ok(loaded) => {
                                *wb = loaded;
                                state.status_message = Some((
                                    format!("Opened {}", path.file_name().unwrap().to_string_lossy()),
                                    std::time::Instant::now(),
                                ));
                                modified = true;
                            }
                            Err(e) => {
                                state.status_message = Some((format!("Error: {}", e), std::time::Instant::now()));
                            }
                        }
                    }
                }

                if ui.button("💾 Save XLSX").clicked() {
                    let path = wb.file_path.clone().unwrap_or_else(|| "workbook.xlsx".to_string());
                    if let Err(e) = save_xlsx(wb, &path) {
                        state.status_message = Some((format!("Save Error: {}", e), std::time::Instant::now()));
                    } else {
                        wb.is_dirty = false;
                        state.status_message = Some(("Saved workbook to XLSX".to_string(), std::time::Instant::now()));
                    }
                }

                if ui.button("💾 Save As...").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Excel Workbook (*.xlsx)", &["xlsx"])
                        .add_filter("CSV (*.csv)", &["csv"])
                        .save_file()
                    {
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("xlsx").to_lowercase();
                        let res = if ext == "csv" {
                            export_csv(wb.active_sheet(), &path)
                        } else {
                            save_xlsx(wb, &path)
                        };
                        if let Err(e) = res {
                            state.status_message = Some((format!("Error: {}", e), std::time::Instant::now()));
                        } else {
                            wb.file_path = Some(path.to_string_lossy().to_string());
                            wb.is_dirty = false;
                            state.status_message = Some(("File saved successfully".to_string(), std::time::Instant::now()));
                        }
                    }
                }

                ui.separator();

                if ui
                    .button(RichText::new("📊 Load 5-Yr Financial Model Template").strong().color(theme.ribbon_accent))
                    .clicked()
                {
                    *wb = Workbook::create_sample_financial_model();
                    recalculate_sheet(wb.active_sheet_mut());
                    state.status_message = Some(("Loaded 5-Year Financial Model Template".to_string(), std::time::Instant::now()));
                    modified = true;
                }
            }
            RibbonTab::Home => {
                let range = wb.active_sheet().selection.range;

                // Undo / Redo
                if ui.button("⮪ Undo").clicked() {
                    let sheet = wb.active_sheet_mut();
                    if sheet.undo() {
                        recalculate_sheet(sheet);
                        wb.is_dirty = true;
                        modified = true;
                    }
                }
                if ui.button("⮫ Redo").clicked() {
                    let sheet = wb.active_sheet_mut();
                    if sheet.redo() {
                        recalculate_sheet(sheet);
                        wb.is_dirty = true;
                        modified = true;
                    }
                }

                ui.separator();

                // Font Styling: Bold, Italic, Underline
                let is_bold = wb
                    .active_sheet()
                    .get_cell(range.start)
                    .map(|c| c.format.bold)
                    .unwrap_or(false);
                if ui.selectable_label(is_bold, RichText::new("B").strong()).clicked() {
                    wb.active_sheet_mut().format_range(range, |fmt| fmt.bold = !is_bold);
                    wb.is_dirty = true;
                    modified = true;
                }

                let is_italic = wb
                    .active_sheet()
                    .get_cell(range.start)
                    .map(|c| c.format.italic)
                    .unwrap_or(false);
                if ui.selectable_label(is_italic, RichText::new("I").italics()).clicked() {
                    wb.active_sheet_mut().format_range(range, |fmt| fmt.italic = !is_italic);
                    wb.is_dirty = true;
                    modified = true;
                }

                let is_underline = wb
                    .active_sheet()
                    .get_cell(range.start)
                    .map(|c| c.format.underline)
                    .unwrap_or(false);
                if ui.selectable_label(is_underline, RichText::new("U").underline()).clicked() {
                    wb.active_sheet_mut().format_range(range, |fmt| fmt.underline = !is_underline);
                    wb.is_dirty = true;
                    modified = true;
                }

                ui.separator();

                // Text Alignment
                if ui.button("⫷ Left").clicked() {
                    wb.active_sheet_mut().format_range(range, |fmt| fmt.align_h = HorizAlign::Left);
                    wb.is_dirty = true;
                    modified = true;
                }
                if ui.button("≡ Center").clicked() {
                    wb.active_sheet_mut().format_range(range, |fmt| fmt.align_h = HorizAlign::Center);
                    wb.is_dirty = true;
                    modified = true;
                }
                if ui.button("⫸ Right").clicked() {
                    wb.active_sheet_mut().format_range(range, |fmt| fmt.align_h = HorizAlign::Right);
                    wb.is_dirty = true;
                    modified = true;
                }

                ui.separator();

                // Number Formatting (Currency, Percent, Number, Decimals)
                if ui.button(RichText::new("$").strong()).clicked() {
                    wb.active_sheet_mut().format_range(range, |fmt| {
                        fmt.number_format = NumberFormat::Currency {
                            symbol: "$".to_string(),
                            decimals: 2,
                        };
                        fmt.align_h = HorizAlign::Right;
                    });
                    wb.is_dirty = true;
                    modified = true;
                }

                if ui.button(RichText::new("%").strong()).clicked() {
                    wb.active_sheet_mut().format_range(range, |fmt| {
                        fmt.number_format = NumberFormat::Percentage { decimals: 1 };
                        fmt.align_h = HorizAlign::Right;
                    });
                    wb.is_dirty = true;
                    modified = true;
                }

                if ui.button("1,234").clicked() {
                    wb.active_sheet_mut().format_range(range, |fmt| {
                        fmt.number_format = NumberFormat::Number {
                            decimals: 2,
                            use_commas: true,
                        };
                        fmt.align_h = HorizAlign::Right;
                    });
                    wb.is_dirty = true;
                    modified = true;
                }

                if ui.button(".00→.0").clicked() {
                    // Decrease decimals
                    wb.active_sheet_mut().format_range(range, |fmt| match &mut fmt.number_format {
                        NumberFormat::Number { decimals, .. }
                        | NumberFormat::Currency { decimals, .. }
                        | NumberFormat::Percentage { decimals } => {
                            *decimals = decimals.saturating_sub(1);
                        }
                        _ => {}
                    });
                    wb.is_dirty = true;
                    modified = true;
                }

                if ui.button(".0→.00").clicked() {
                    // Increase decimals
                    wb.active_sheet_mut().format_range(range, |fmt| match &mut fmt.number_format {
                        NumberFormat::Number { decimals, .. }
                        | NumberFormat::Currency { decimals, .. }
                        | NumberFormat::Percentage { decimals } => {
                            *decimals = (*decimals + 1).min(10);
                        }
                        _ => {}
                    });
                    wb.is_dirty = true;
                    modified = true;
                }

                ui.separator();

                // Colors: Highlight / Fill
                ui.menu_button("🎨 Fill", |ui| {
                    let colors = [
                        ("No Fill", None),
                        ("Navy Header", Some([22, 60, 110, 255])),
                        ("Emerald Accent", Some([16, 124, 65, 255])),
                        ("Soft Blue", Some([240, 244, 250, 255])),
                        ("Light Mint", Some([232, 245, 233, 255])),
                        ("Pale Yellow", Some([255, 249, 196, 255])),
                        ("Soft Red", Some([255, 235, 238, 255])),
                    ];
                    for (label, col) in colors {
                        if ui.button(label).clicked() {
                            wb.active_sheet_mut().format_range(range, |fmt| fmt.bg_color = col);
                            wb.is_dirty = true;
                            ui.close_menu();
                        }
                    }
                });

                ui.menu_button("Text Color", |ui| {
                    let colors = [
                        ("Default", None),
                        ("White", Some([255, 255, 255, 255])),
                        ("Black", Some([20, 20, 20, 255])),
                        ("Navy", Some([22, 60, 110, 255])),
                        ("Emerald", Some([16, 124, 65, 255])),
                        ("Dark Red", Some([180, 20, 20, 255])),
                    ];
                    for (label, col) in colors {
                        if ui.button(label).clicked() {
                            wb.active_sheet_mut().format_range(range, |fmt| fmt.text_color = col);
                            wb.is_dirty = true;
                            ui.close_menu();
                        }
                    }
                });

                ui.separator();

                // AutoSum Quick Button
                if ui.button(RichText::new("Σ AutoSum").strong()).clicked() {
                    let sheet = wb.active_sheet_mut();
                    let cursor = sheet.selection.cursor;
                    // Look upward from cursor for numbers
                    let r = cursor.row;
                    let c = cursor.col;
                    let mut start_r = r;
                    while start_r > 0 {
                        let check = CellCoord::new(start_r - 1, c);
                        if let Some(cell) = sheet.get_cell(check) {
                            if matches!(cell.value, crate::model::cell::CellValue::Number(_)) {
                                start_r -= 1;
                                continue;
                            }
                        }
                        break;
                    }
                    if start_r < r {
                        let top = CellCoord::new(start_r, c);
                        let bottom = CellCoord::new(r - 1, c);
                        let sum_formula = format!("=SUM({}:{})", top.to_a1(), bottom.to_a1());
                        sheet.push_undo();
                        sheet.set_cell_input(cursor, sum_formula);
                        recalculate_incremental(sheet, cursor);
                        wb.is_dirty = true;
                        modified = true;
                    }
                }

                // Clear button
                ui.menu_button("Clear", |ui| {
                    if ui.button("Clear All").clicked() {
                        let sheet = wb.active_sheet_mut();
                        sheet.push_undo();
                        sheet.clear_range(range);
                        recalculate_sheet(sheet);
                        wb.is_dirty = true;
                        ui.close_menu();
                    }
                    if ui.button("Clear Formats").clicked() {
                        let sheet = wb.active_sheet_mut();
                        sheet.push_undo();
                        sheet.format_range(range, |fmt| *fmt = CellFormat::default());
                        wb.is_dirty = true;
                        ui.close_menu();
                    }
                });
            }
            RibbonTab::Formulas => {
                let funcs = [
                    ("SUM", "=SUM("),
                    ("AVERAGE", "=AVERAGE("),
                    ("COUNT", "=COUNT("),
                    ("MAX", "=MAX("),
                    ("MIN", "=MIN("),
                    ("IF", "=IF("),
                    ("VLOOKUP", "=VLOOKUP("),
                    ("PMT", "=PMT("),
                    ("NPV", "=NPV("),
                    ("TODAY", "=TODAY()"),
                ];

                for (name, formula_stub) in funcs {
                    if ui.button(name).clicked() {
                        let sheet = wb.active_sheet_mut();
                        let cursor = sheet.selection.cursor;
                        sheet.push_undo();
                        sheet.set_cell_input(cursor, formula_stub);
                        wb.is_dirty = true;
                        modified = true;
                    }
                }
            }
            RibbonTab::Data => {
                let range = wb.active_sheet().selection.range;

                if ui.button("Sort A→Z").clicked() {
                    sort_range(wb, range, true);
                    modified = true;
                }
                if ui.button("Sort Z→A").clicked() {
                    sort_range(wb, range, false);
                    modified = true;
                }

                ui.separator();

                if ui.button("Insert Row Above").clicked() {
                    insert_row(wb, range.min_row());
                    modified = true;
                }
                if ui.button("Delete Row").clicked() {
                    delete_row(wb, range.min_row());
                    modified = true;
                }
                if ui.button("Insert Col Left").clicked() {
                    insert_col(wb, range.min_col());
                    modified = true;
                }
                if ui.button("Delete Col").clicked() {
                    delete_col(wb, range.min_col());
                    modified = true;
                }

                ui.separator();

                if ui.button("↔ AutoFit Column").clicked() {
                    let sheet = wb.active_sheet_mut();
                    sheet.push_undo();
                    for c in range.min_col()..=range.max_col() {
                        sheet.auto_fit_col(c);
                    }
                    wb.is_dirty = true;
                    modified = true;
                }
                if ui.button("↕ Reset Row Height").clicked() {
                    let sheet = wb.active_sheet_mut();
                    sheet.push_undo();
                    for r in range.min_row()..=range.max_row() {
                        sheet.set_row_height(r, crate::model::sheet::DEFAULT_ROW_HEIGHT);
                    }
                    wb.is_dirty = true;
                    modified = true;
                }
            }
        }
    });

    modified
}

fn sort_range(wb: &mut Workbook, range: SelectionRange, ascending: bool) {
    let sheet = wb.active_sheet_mut();
    sheet.push_undo();
    let min_r = range.min_row();
    let max_r = range.max_row();
    let min_c = range.min_col();
    let max_c = range.max_col();

    let mut rows: Vec<Vec<CellData>> = Vec::new();
    for r in min_r..=max_r {
        let mut row_cells = Vec::new();
        for c in min_c..=max_c {
            let data = sheet
                .get_cell(CellCoord::new(r, c))
                .cloned()
                .unwrap_or_else(|| CellData::new(""));
            row_cells.push(data);
        }
        rows.push(row_cells);
    }

    rows.sort_by(|a, b| {
        let val_a = a.first().map(|c| &c.value);
        let val_b = b.first().map(|c| &c.value);
        match (val_a, val_b) {
            (Some(va), Some(vb)) => {
                let cmp = match (va.as_number(), vb.as_number()) {
                    (Some(na), Some(nb)) => na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal),
                    _ => va.as_string().cmp(&vb.as_string()),
                };
                if ascending { cmp } else { cmp.reverse() }
            }
            _ => std::cmp::Ordering::Equal,
        }
    });

    for (i, row_cells) in rows.into_iter().enumerate() {
        let r = min_r + i;
        for (j, data) in row_cells.into_iter().enumerate() {
            let c = min_c + j;
            sheet.set_cell_data(CellCoord::new(r, c), data);
        }
    }
    recalculate_sheet(sheet);
    wb.is_dirty = true;
}

fn insert_row(wb: &mut Workbook, at_row: usize) {
    let sheet = wb.active_sheet_mut();
    sheet.push_undo();
    let max_r = sheet.max_used_row() + 1;
    let max_c = sheet.max_used_col() + 1;

    for r in (at_row..=max_r).rev() {
        for c in 0..=max_c {
            let src = CellCoord::new(r, c);
            let dst = CellCoord::new(r + 1, c);
            if let Some(cell) = sheet.cells.remove(&src) {
                sheet.cells.insert(dst, cell);
            }
        }
    }
    recalculate_sheet(sheet);
    wb.is_dirty = true;
}

fn delete_row(wb: &mut Workbook, at_row: usize) {
    let sheet = wb.active_sheet_mut();
    sheet.push_undo();
    let max_r = sheet.max_used_row() + 1;
    let max_c = sheet.max_used_col() + 1;

    for c in 0..=max_c {
        sheet.cells.remove(&CellCoord::new(at_row, c));
    }

    for r in at_row..=max_r {
        for c in 0..=max_c {
            let src = CellCoord::new(r + 1, c);
            let dst = CellCoord::new(r, c);
            if let Some(cell) = sheet.cells.remove(&src) {
                sheet.cells.insert(dst, cell);
            }
        }
    }
    recalculate_sheet(sheet);
    wb.is_dirty = true;
}

fn insert_col(wb: &mut Workbook, at_col: usize) {
    let sheet = wb.active_sheet_mut();
    sheet.push_undo();
    let max_r = sheet.max_used_row() + 1;
    let max_c = sheet.max_used_col() + 1;

    for c in (at_col..=max_c).rev() {
        for r in 0..=max_r {
            let src = CellCoord::new(r, c);
            let dst = CellCoord::new(r, c + 1);
            if let Some(cell) = sheet.cells.remove(&src) {
                sheet.cells.insert(dst, cell);
            }
        }
    }
    recalculate_sheet(sheet);
    wb.is_dirty = true;
}

fn delete_col(wb: &mut Workbook, at_col: usize) {
    let sheet = wb.active_sheet_mut();
    sheet.push_undo();
    let max_r = sheet.max_used_row() + 1;
    let max_c = sheet.max_used_col() + 1;

    for r in 0..=max_r {
        sheet.cells.remove(&CellCoord::new(r, at_col));
    }

    for c in at_col..=max_c {
        for r in 0..=max_r {
            let src = CellCoord::new(r, c + 1);
            let dst = CellCoord::new(r, c);
            if let Some(cell) = sheet.cells.remove(&src) {
                sheet.cells.insert(dst, cell);
            }
        }
    }
    recalculate_sheet(sheet);
    wb.is_dirty = true;
}

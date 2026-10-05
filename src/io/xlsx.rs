use crate::engine::dependency::recalculate_sheet;
use crate::model::cell::{CellCoord, CellData, CellValue, HorizAlign, NumberFormat};
use crate::model::sheet::Sheet;
use crate::model::workbook::Workbook;
use calamine::{open_workbook_auto, Data, Reader};
use rust_xlsxwriter::{Color, Format, FormatAlign, Workbook as XlsxWorkbook};
use std::path::Path;

pub fn load_xlsx(path: impl AsRef<Path>) -> Result<Workbook, String> {
    let mut excel = open_workbook_auto(&path).map_err(|e| format!("Failed to open Excel file: {}", e))?;
    let sheet_names = excel.sheet_names();
    if sheet_names.is_empty() {
        return Err("Workbook contains no sheets".to_string());
    }

    let mut wb = Workbook {
        sheets: Vec::new(),
        active_sheet_index: 0,
        file_path: Some(path.as_ref().to_string_lossy().to_string()),
        is_dirty: false,
    };

    for name in sheet_names {
        if let Ok(range) = excel.worksheet_range(&name) {
            let mut sheet = Sheet::new(&name);
            for (row_idx, row) in range.rows().enumerate() {
                for (col_idx, val) in row.iter().enumerate() {
                    let coord = CellCoord::new(row_idx, col_idx);
                    let (raw, cell_val) = match val {
                        Data::Empty => continue,
                        Data::String(s) => (s.clone(), CellValue::Text(s.clone())),
                        &Data::Float(f) => {
                            let text = if f.fract() == 0.0 && f.abs() < 1e15 {
                                format!("{}", f as i64)
                            } else {
                                format!("{}", f)
                            };
                            (text, CellValue::Number(f))
                        }
                        &Data::Int(i) => (format!("{}", i), CellValue::Number(i as f64)),
                        &Data::Bool(b) => {
                            (if b { "TRUE" } else { "FALSE" }.to_string(), CellValue::Bool(b))
                        }
                        Data::DateTime(dt) => {
                            let f = dt.as_f64();
                            (format!("{}", f), CellValue::Number(f))
                        }
                        Data::Error(err) => (format!("{:?}", err), CellValue::Text(format!("{:?}", err))),
                        _ => continue,
                    };

                    let mut data = CellData::new(raw);
                    data.value = cell_val;
                    sheet.set_cell_data(coord, data);
                }
            }
            recalculate_sheet(&mut sheet);
            wb.sheets.push(sheet);
        }
    }

    if wb.sheets.is_empty() {
        wb.sheets.push(Sheet::new("Sheet1"));
    }

    Ok(wb)
}

pub fn save_xlsx(wb: &Workbook, path: impl AsRef<Path>) -> Result<(), String> {
    let mut xlsx = XlsxWorkbook::new();

    for sheet in &wb.sheets {
        let worksheet = xlsx
            .add_worksheet()
            .set_name(&sheet.name)
            .map_err(|e| format!("Failed to set worksheet name: {}", e))?;

        // Write custom column widths
        for (&col, &width) in &sheet.col_widths {
            // Excel column width units: approx 1 char ~= 8 pixels
            let char_width = (width / 8.0).max(4.0) as f64;
            worksheet
                .set_column_width(col as u16, char_width)
                .map_err(|e| format!("Failed to set column width: {}", e))?;
        }

        for (&coord, cell) in &sheet.cells {
            let row = coord.row as u32;
            let col = coord.col as u16;

            let mut fmt = Format::new();
            if cell.format.bold {
                fmt = fmt.set_bold();
            }
            if cell.format.italic {
                fmt = fmt.set_italic();
            }
            if cell.format.underline {
                fmt = fmt.set_underline(rust_xlsxwriter::FormatUnderline::Single);
            }
            match cell.format.align_h {
                HorizAlign::Left => {
                    fmt = fmt.set_align(FormatAlign::Left);
                }
                HorizAlign::Center => {
                    fmt = fmt.set_align(FormatAlign::Center);
                }
                HorizAlign::Right => {
                    fmt = fmt.set_align(FormatAlign::Right);
                }
            }

            if let Some([r, g, b, _]) = cell.format.bg_color {
                fmt = fmt.set_background_color(Color::RGB(((r as u32) << 16) | ((g as u32) << 8) | (b as u32)));
            }
            if let Some([r, g, b, _]) = cell.format.text_color {
                fmt = fmt.set_font_color(Color::RGB(((r as u32) << 16) | ((g as u32) << 8) | (b as u32)));
            }

            match &cell.format.number_format {
                NumberFormat::Currency { symbol, decimals } => {
                    let num_fmt = if *decimals == 0 {
                        format!("{}$#,##0", symbol)
                    } else {
                        let zeroes = "0".repeat(*decimals as usize);
                        format!("{}$#,##0.{}", symbol, zeroes)
                    };
                    fmt = fmt.set_num_format(&num_fmt);
                }
                NumberFormat::Percentage { decimals } => {
                    let num_fmt = if *decimals == 0 {
                        "0%".to_string()
                    } else {
                        let zeroes = "0".repeat(*decimals as usize);
                        format!("0.{}%", zeroes)
                    };
                    fmt = fmt.set_num_format(&num_fmt);
                }
                NumberFormat::Number { decimals, use_commas } => {
                    let base = if *use_commas { "#,##0" } else { "0" };
                    let num_fmt = if *decimals == 0 {
                        base.to_string()
                    } else {
                        let zeroes = "0".repeat(*decimals as usize);
                        format!("{}.{}", base, zeroes)
                    };
                    fmt = fmt.set_num_format(&num_fmt);
                }
                _ => {}
            }

            if let Some(formula) = cell.formula_text() {
                worksheet
                    .write_formula_with_format(row, col, formula, &fmt)
                    .map_err(|e| format!("Failed to write formula: {}", e))?;
            } else {
                match &cell.value {
                    CellValue::Number(n) => {
                        worksheet
                            .write_number_with_format(row, col, *n, &fmt)
                            .map_err(|e| format!("Failed to write number: {}", e))?;
                    }
                    CellValue::Bool(b) => {
                        worksheet
                            .write_boolean_with_format(row, col, *b, &fmt)
                            .map_err(|e| format!("Failed to write boolean: {}", e))?;
                    }
                    CellValue::Text(s) => {
                        worksheet
                            .write_string_with_format(row, col, s, &fmt)
                            .map_err(|e| format!("Failed to write string: {}", e))?;
                    }
                    _ => {
                        worksheet
                            .write_blank(row, col, &fmt)
                            .map_err(|e| format!("Failed to write blank: {}", e))?;
                    }
                }
            }
        }
    }

    xlsx.save(path)
        .map_err(|e| format!("Failed to save Excel file: {}", e))?;
    Ok(())
}

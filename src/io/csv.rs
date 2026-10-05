use crate::model::cell::CellCoord;
use crate::model::sheet::Sheet;
use std::fs::File;
use std::path::Path;

pub fn import_csv(path: impl AsRef<Path>) -> Result<Sheet, String> {
    let file = File::open(&path).map_err(|e| format!("Failed to open CSV file: {}", e))?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(file);

    let sheet_name = path
        .as_ref()
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("CSV Data");
    let mut sheet = Sheet::new(sheet_name);

    for (row_idx, record_res) in rdr.records().enumerate() {
        let record = record_res.map_err(|e| format!("CSV read error at row {}: {}", row_idx + 1, e))?;
        for (col_idx, field) in record.iter().enumerate() {
            if !field.is_empty() {
                let coord = CellCoord::new(row_idx, col_idx);
                sheet.set_cell_input(coord, field);
            }
        }
    }

    Ok(sheet)
}

pub fn export_csv(sheet: &Sheet, path: impl AsRef<Path>) -> Result<(), String> {
    let file = File::create(&path).map_err(|e| format!("Failed to create CSV file: {}", e))?;
    let mut wtr = csv::WriterBuilder::new().from_writer(file);

    let max_row = sheet.max_used_row();
    let max_col = sheet.max_used_col();

    for r in 0..=max_row {
        let mut row_record = Vec::new();
        for c in 0..=max_col {
            let val = sheet
                .get_cell(CellCoord::new(r, c))
                .map(|cell| cell.value.as_string())
                .unwrap_or_default();
            row_record.push(val);
        }
        wtr.write_record(&row_record)
            .map_err(|e| format!("CSV write error at row {}: {}", r + 1, e))?;
    }

    wtr.flush()
        .map_err(|e| format!("CSV flush error: {}", e))?;
    Ok(())
}

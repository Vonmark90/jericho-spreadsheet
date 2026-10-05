use crate::model::workbook::Workbook;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub fn save_json(wb: &Workbook, path: impl AsRef<Path>) -> Result<(), String> {
    let file = File::create(&path).map_err(|e| format!("Failed to create JSON file: {}", e))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, wb).map_err(|e| format!("Failed to serialize workbook: {}", e))?;
    Ok(())
}

pub fn load_json(path: impl AsRef<Path>) -> Result<Workbook, String> {
    let file = File::open(&path).map_err(|e| format!("Failed to open JSON file: {}", e))?;
    let reader = BufReader::new(file);
    let mut wb: Workbook = serde_json::from_reader(reader).map_err(|e| format!("Failed to deserialize workbook: {}", e))?;
    wb.file_path = Some(path.as_ref().to_string_lossy().to_string());
    wb.is_dirty = false;
    Ok(wb)
}

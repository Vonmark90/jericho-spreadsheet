use super::cell::{CellCoord, CellData, CellFormat, CellValue};
use super::selection::{SelectionRange, SheetSelection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DEFAULT_COL_WIDTH: f32 = 94.0;
pub const DEFAULT_ROW_HEIGHT: f32 = 26.0;
pub const ROW_HEADER_WIDTH: f32 = 52.0;
pub const COL_HEADER_HEIGHT: f32 = 28.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetSnapshot {
    pub cells: HashMap<CellCoord, CellData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheet {
    pub name: String,
    pub cells: HashMap<CellCoord, CellData>,
    pub col_widths: HashMap<usize, f32>,
    pub row_heights: HashMap<usize, f32>,
    pub selection: SheetSelection,
    #[serde(skip)]
    pub undo_stack: Vec<SheetSnapshot>,
    #[serde(skip)]
    pub redo_stack: Vec<SheetSnapshot>,
}

impl Sheet {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            cells: HashMap::new(),
            col_widths: HashMap::new(),
            row_heights: HashMap::new(),
            selection: SheetSelection::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn push_undo(&mut self) {
        if self.undo_stack.len() >= 50 {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(SheetSnapshot {
            cells: self.cells.clone(),
        });
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(SheetSnapshot {
                cells: self.cells.clone(),
            });
            self.cells = prev.cells;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(SheetSnapshot {
                cells: self.cells.clone(),
            });
            self.cells = next.cells;
            true
        } else {
            false
        }
    }

    pub fn get_cell(&self, coord: CellCoord) -> Option<&CellData> {
        self.cells.get(&coord)
    }

    pub fn get_cell_mut(&mut self, coord: CellCoord) -> Option<&mut CellData> {
        self.cells.get_mut(&coord)
    }

    pub fn get_cell_value(&self, coord: CellCoord) -> CellValue {
        self.cells
            .get(&coord)
            .map(|c| c.value.clone())
            .unwrap_or(CellValue::Empty)
    }

    pub fn set_cell_input(&mut self, coord: CellCoord, input: impl Into<String>) {
        let input_str = input.into();
        if input_str.trim().is_empty() {
            self.cells.remove(&coord);
        } else {
            let cell = self
                .cells
                .entry(coord)
                .or_insert_with(|| CellData::new(""));
            cell.raw_input = input_str.clone();
            cell.value = CellData::infer_value(&input_str);
        }
    }

    pub fn set_cell_data(&mut self, coord: CellCoord, data: CellData) {
        if data.raw_input.is_empty() && data.value.is_empty() {
            self.cells.remove(&coord);
        } else {
            self.cells.insert(coord, data);
        }
    }

    pub fn clear_cell(&mut self, coord: CellCoord) {
        self.cells.remove(&coord);
    }

    pub fn clear_range(&mut self, range: SelectionRange) {
        for r in range.min_row()..=range.max_row() {
            for c in range.min_col()..=range.max_col() {
                self.cells.remove(&CellCoord::new(r, c));
            }
        }
    }

    pub fn format_range<F>(&mut self, range: SelectionRange, mut f: F)
    where
        F: FnMut(&mut CellFormat),
    {
        for r in range.min_row()..=range.max_row() {
            for c in range.min_col()..=range.max_col() {
                let coord = CellCoord::new(r, c);
                let cell = self
                    .cells
                    .entry(coord)
                    .or_insert_with(|| CellData::new(""));
                f(&mut cell.format);
            }
        }
    }

    pub fn col_width(&self, col: usize) -> f32 {
        self.col_widths
            .get(&col)
            .copied()
            .unwrap_or(DEFAULT_COL_WIDTH)
    }

    pub fn set_col_width(&mut self, col: usize, width: f32) {
        let w = width.clamp(30.0, 600.0);
        self.col_widths.insert(col, w);
    }

    pub fn auto_fit_col(&mut self, col: usize) {
        let mut max_chars = CellCoord::col_to_name(col).len();
        for (coord, cell) in &self.cells {
            if coord.col == col {
                let s = cell.value.format_display(&cell.format);
                max_chars = max_chars.max(s.chars().count());
            }
        }
        let width = (max_chars as f32 * 9.2 + 28.0).clamp(55.0, 600.0);
        self.set_col_width(col, width);
    }

    pub fn row_height(&self, row: usize) -> f32 {
        self.row_heights
            .get(&row)
            .copied()
            .unwrap_or(DEFAULT_ROW_HEIGHT)
    }

    pub fn set_row_height(&mut self, row: usize, height: f32) {
        let h = height.clamp(16.0, 300.0);
        self.row_heights.insert(row, h);
    }

    pub fn max_used_row(&self) -> usize {
        self.cells.keys().map(|c| c.row).max().unwrap_or(0)
    }

    pub fn max_used_col(&self) -> usize {
        self.cells.keys().map(|c| c.col).max().unwrap_or(0)
    }

    /// Calculate aggregate values for a given range (useful for Excel status bar)
    pub fn calculate_range_aggregates(&self, range: SelectionRange) -> RangeAggregates {
        let mut count: usize = 0;
        let mut num_count: usize = 0;
        let mut sum: f64 = 0.0;
        let mut min: Option<f64> = None;
        let mut max: Option<f64> = None;

        for r in range.min_row()..=range.max_row() {
            for c in range.min_col()..=range.max_col() {
                if let Some(cell) = self.cells.get(&CellCoord::new(r, c)) {
                    if !cell.value.is_empty() {
                        count += 1;
                        if let Some(num) = cell.value.as_number() {
                            num_count += 1;
                            sum += num;
                            min = Some(min.map_or(num, |m| m.min(num)));
                            max = Some(max.map_or(num, |m| m.max(num)));
                        }
                    }
                }
            }
        }

        let average = if num_count > 0 {
            Some(sum / num_count as f64)
        } else {
            None
        };

        RangeAggregates {
            cell_count: range.cell_count(),
            non_empty_count: count,
            num_count,
            sum: if num_count > 0 { Some(sum) } else { None },
            average,
            min,
            max,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RangeAggregates {
    pub cell_count: usize,
    pub non_empty_count: usize,
    pub num_count: usize,
    pub sum: Option<f64>,
    pub average: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

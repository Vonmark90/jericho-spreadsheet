use super::cell::CellCoord;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionRange {
    pub start: CellCoord,
    pub end: CellCoord,
}

impl SelectionRange {
    pub fn new(start: CellCoord, end: CellCoord) -> Self {
        Self { start, end }
    }

    pub fn single(coord: CellCoord) -> Self {
        Self {
            start: coord,
            end: coord,
        }
    }

    pub fn min_row(&self) -> usize {
        self.start.row.min(self.end.row)
    }

    pub fn max_row(&self) -> usize {
        self.start.row.max(self.end.row)
    }

    pub fn min_col(&self) -> usize {
        self.start.col.min(self.end.col)
    }

    pub fn max_col(&self) -> usize {
        self.start.col.max(self.end.col)
    }

    pub fn contains(&self, coord: CellCoord) -> bool {
        coord.row >= self.min_row()
            && coord.row <= self.max_row()
            && coord.col >= self.min_col()
            && coord.col <= self.max_col()
    }

    pub fn is_single_cell(&self) -> bool {
        self.start.row == self.end.row && self.start.col == self.end.col
    }

    pub fn cell_count(&self) -> usize {
        (self.max_row() - self.min_row() + 1) * (self.max_col() - self.min_col() + 1)
    }

    pub fn to_range_string(&self) -> String {
        if self.is_single_cell() {
            self.start.to_a1()
        } else {
            let top_left = CellCoord::new(self.min_row(), self.min_col());
            let bottom_right = CellCoord::new(self.max_row(), self.max_col());
            format!("{}:{}", top_left.to_a1(), bottom_right.to_a1())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheetSelection {
    pub cursor: CellCoord,
    pub range: SelectionRange,
    pub is_dragging: bool,
    pub is_autofilling: bool,
    pub autofill_range: Option<SelectionRange>,
}

impl Default for SheetSelection {
    fn default() -> Self {
        let origin = CellCoord::new(0, 0);
        Self {
            cursor: origin,
            range: SelectionRange::single(origin),
            is_dragging: false,
            is_autofilling: false,
            autofill_range: None,
        }
    }
}

impl SheetSelection {
    pub fn select_cell(&mut self, coord: CellCoord) {
        self.cursor = coord;
        self.range = SelectionRange::single(coord);
        self.is_dragging = false;
        self.is_autofilling = false;
        self.autofill_range = None;
    }

    pub fn extend_to(&mut self, coord: CellCoord) {
        self.range.end = coord;
    }

    pub fn move_cursor(&mut self, d_row: isize, d_col: isize, extend_selection: bool) {
        let new_row = (self.cursor.row as isize + d_row).max(0) as usize;
        let new_col = (self.cursor.col as isize + d_col).max(0) as usize;
        let new_coord = CellCoord::new(new_row, new_col);

        if extend_selection {
            self.cursor = new_coord;
            self.range.end = new_coord;
        } else {
            self.select_cell(new_coord);
        }
    }
}

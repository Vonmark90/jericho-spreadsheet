use egui::{Color32, Stroke};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

#[derive(Debug, Clone)]
pub struct SpreadsheetTheme {
    pub mode: ThemeMode,
    pub ribbon_bg: Color32,
    pub ribbon_accent: Color32,
    pub ribbon_text: Color32,
    pub formula_bar_bg: Color32,
    pub formula_bar_border: Color32,
    pub header_bg: Color32,
    pub header_fg: Color32,
    pub header_active_bg: Color32,
    pub header_active_fg: Color32,
    pub header_border: Color32,
    pub grid_line: Color32,
    pub cell_bg: Color32,
    pub cell_text: Color32,
    pub selection_border: Color32,
    pub selection_fill: Color32,
    pub autofill_handle: Color32,
    pub status_bar_bg: Color32,
    pub status_bar_fg: Color32,
    pub tab_bar_bg: Color32,
    pub tab_active_bg: Color32,
    pub tab_inactive_bg: Color32,
    pub tab_active_fg: Color32,
    pub tab_inactive_fg: Color32,
}

impl SpreadsheetTheme {
    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            ribbon_bg: Color32::from_rgb(243, 245, 247),
            ribbon_accent: Color32::from_rgb(16, 124, 65), // Microsoft Excel Emerald Green
            ribbon_text: Color32::from_rgb(32, 33, 36),
            formula_bar_bg: Color32::from_rgb(255, 255, 255),
            formula_bar_border: Color32::from_rgb(218, 220, 224),
            header_bg: Color32::from_rgb(243, 244, 246),
            header_fg: Color32::from_rgb(90, 95, 102),
            header_active_bg: Color32::from_rgb(209, 237, 219),
            header_active_fg: Color32::from_rgb(16, 124, 65),
            header_border: Color32::from_rgb(218, 222, 228),
            grid_line: Color32::from_rgb(226, 229, 234),
            cell_bg: Color32::from_rgb(255, 255, 255),
            cell_text: Color32::from_rgb(30, 31, 34),
            selection_border: Color32::from_rgb(16, 124, 65),
            selection_fill: Color32::from_rgba_premultiplied(16, 124, 65, 24),
            autofill_handle: Color32::from_rgb(16, 124, 65),
            status_bar_bg: Color32::from_rgb(238, 241, 244),
            status_bar_fg: Color32::from_rgb(90, 95, 102),
            tab_bar_bg: Color32::from_rgb(238, 241, 244),
            tab_active_bg: Color32::from_rgb(255, 255, 255),
            tab_inactive_bg: Color32::from_rgb(228, 231, 236),
            tab_active_fg: Color32::from_rgb(16, 124, 65),
            tab_inactive_fg: Color32::from_rgb(90, 95, 102),
        }
    }

    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            ribbon_bg: Color32::from_rgb(28, 30, 33),
            ribbon_accent: Color32::from_rgb(33, 160, 90),
            ribbon_text: Color32::from_rgb(230, 233, 238),
            formula_bar_bg: Color32::from_rgb(36, 38, 43),
            formula_bar_border: Color32::from_rgb(52, 56, 62),
            header_bg: Color32::from_rgb(32, 34, 38),
            header_fg: Color32::from_rgb(160, 166, 178),
            header_active_bg: Color32::from_rgb(24, 55, 38),
            header_active_fg: Color32::from_rgb(60, 195, 120),
            header_border: Color32::from_rgb(50, 54, 60),
            grid_line: Color32::from_rgb(46, 50, 56),
            cell_bg: Color32::from_rgb(22, 24, 27),
            cell_text: Color32::from_rgb(230, 233, 238),
            selection_border: Color32::from_rgb(33, 160, 90),
            selection_fill: Color32::from_rgba_premultiplied(33, 160, 90, 36),
            autofill_handle: Color32::from_rgb(33, 160, 90),
            status_bar_bg: Color32::from_rgb(25, 27, 30),
            status_bar_fg: Color32::from_rgb(160, 166, 178),
            tab_bar_bg: Color32::from_rgb(25, 27, 30),
            tab_active_bg: Color32::from_rgb(36, 38, 43),
            tab_inactive_bg: Color32::from_rgb(22, 24, 27),
            tab_active_fg: Color32::from_rgb(60, 195, 120),
            tab_inactive_fg: Color32::from_rgb(140, 145, 155),
        }
    }

    pub fn grid_stroke(&self) -> Stroke {
        Stroke::new(1.0f32, self.grid_line)
    }

    pub fn header_stroke(&self) -> Stroke {
        Stroke::new(1.0f32, self.header_border)
    }

    pub fn selection_stroke(&self) -> Stroke {
        Stroke::new(2.0f32, self.selection_border)
    }
}

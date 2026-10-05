use super::cell::{CellCoord, CellData, CellFormat, HorizAlign, NumberFormat};
use super::sheet::Sheet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workbook {
    pub sheets: Vec<Sheet>,
    pub active_sheet_index: usize,
    pub file_path: Option<String>,
    #[serde(skip)]
    pub is_dirty: bool,
}

impl Default for Workbook {
    fn default() -> Self {
        Self::new()
    }
}

impl Workbook {
    pub fn new() -> Self {
        let sheet1 = Sheet::new("Sheet1");
        Self {
            sheets: vec![sheet1],
            active_sheet_index: 0,
            file_path: None,
            is_dirty: false,
        }
    }

    pub fn active_sheet(&self) -> &Sheet {
        &self.sheets[self.active_sheet_index]
    }

    pub fn active_sheet_mut(&mut self) -> &mut Sheet {
        &mut self.sheets[self.active_sheet_index]
    }

    pub fn add_sheet(&mut self, name: Option<String>) -> usize {
        let name = name.unwrap_or_else(|| {
            let mut i = self.sheets.len() + 1;
            loop {
                let candidate = format!("Sheet{}", i);
                if !self.sheets.iter().any(|s| s.name == candidate) {
                    return candidate;
                }
                i += 1;
            }
        });
        self.sheets.push(Sheet::new(name));
        self.active_sheet_index = self.sheets.len() - 1;
        self.is_dirty = true;
        self.active_sheet_index
    }

    pub fn remove_sheet(&mut self, index: usize) -> bool {
        if self.sheets.len() <= 1 || index >= self.sheets.len() {
            return false;
        }
        self.sheets.remove(index);
        if self.active_sheet_index >= self.sheets.len() {
            self.active_sheet_index = self.sheets.len() - 1;
        }
        self.is_dirty = true;
        true
    }

    pub fn rename_sheet(&mut self, index: usize, new_name: String) -> bool {
        let trimmed = new_name.trim();
        if trimmed.is_empty() || self.sheets.iter().enumerate().any(|(i, s)| i != index && s.name == trimmed) {
            return false;
        }
        if let Some(sheet) = self.sheets.get_mut(index) {
            sheet.name = trimmed.to_string();
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    /// Generates an industry-standard 5-Year Financial & DCF Model template
    /// demonstrating formulas, cell references, formatting, currency, percentages, and multi-row logic.
    pub fn create_sample_financial_model() -> Self {
        let mut wb = Self::new();
        let sheet = wb.active_sheet_mut();
        sheet.name = "Financial Model".to_string();

        // Custom column widths
        sheet.set_col_width(0, 220.0); // Line item description
        sheet.set_col_width(1, 100.0); // 2024A
        sheet.set_col_width(2, 100.0); // 2025E
        sheet.set_col_width(3, 100.0); // 2026E
        sheet.set_col_width(4, 100.0); // 2027E
        sheet.set_col_width(5, 100.0); // 2028E

        let mut set_cell = |r: usize, c: usize, input: &str, fmt: CellFormat| {
            let coord = CellCoord::new(r, c);
            let mut data = CellData::new(input);
            data.format = fmt;
            sheet.set_cell_data(coord, data);
        };

        let header_fmt = CellFormat {
            bold: true,
            font_size: 14.0,
            text_color: Some([255, 255, 255, 255]),
            bg_color: Some([22, 60, 110, 255]), // Deep Navy
            align_h: HorizAlign::Center,
            ..Default::default()
        };

        let title_fmt = CellFormat {
            bold: true,
            font_size: 16.5,
            text_color: Some([22, 60, 110, 255]),
            ..Default::default()
        };

        let section_fmt = CellFormat {
            bold: true,
            font_size: 14.5,
            bg_color: Some([240, 244, 250, 255]),
            ..Default::default()
        };

        let row_label_fmt = CellFormat {
            bold: false,
            font_size: 14.0,
            align_h: HorizAlign::Left,
            ..Default::default()
        };

        let bold_label_fmt = CellFormat {
            bold: true,
            font_size: 14.0,
            align_h: HorizAlign::Left,
            ..Default::default()
        };

        let cur_fmt = CellFormat {
            number_format: NumberFormat::Currency {
                symbol: "$".to_string(),
                decimals: 0,
            },
            align_h: HorizAlign::Right,
            ..Default::default()
        };

        let cur_bold_fmt = CellFormat {
            bold: true,
            number_format: NumberFormat::Currency {
                symbol: "$".to_string(),
                decimals: 0,
            },
            align_h: HorizAlign::Right,
            bg_color: Some([245, 248, 255, 255]),
            ..Default::default()
        };

        let pct_fmt = CellFormat {
            number_format: NumberFormat::Percentage { decimals: 1 },
            align_h: HorizAlign::Right,
            ..Default::default()
        };

        // Title
        set_cell(0, 0, "ACME CORP - 5-YEAR PRO-FORMA INCOME STATEMENT", title_fmt);
        set_cell(1, 0, "(USD in thousands)", CellFormat { italic: true, ..Default::default() });

        // Column headers
        set_cell(3, 0, "Line Item", CellFormat { align_h: HorizAlign::Left, ..header_fmt.clone() });
        set_cell(3, 1, "2024A", header_fmt.clone());
        set_cell(3, 2, "2025E", header_fmt.clone());
        set_cell(3, 3, "2026E", header_fmt.clone());
        set_cell(3, 4, "2027E", header_fmt.clone());
        set_cell(3, 5, "2028E", header_fmt.clone());

        // Revenue Section
        set_cell(4, 0, "REVENUE & GROWTH", section_fmt.clone());
        set_cell(5, 0, "Product Revenue", row_label_fmt.clone());
        set_cell(5, 1, "45000", cur_fmt.clone());
        set_cell(5, 2, "=B6*1.15", cur_fmt.clone());
        set_cell(5, 3, "=C6*1.12", cur_fmt.clone());
        set_cell(5, 4, "=D6*1.10", cur_fmt.clone());
        set_cell(5, 5, "=E6*1.08", cur_fmt.clone());

        set_cell(6, 0, "Services & Subscription", row_label_fmt.clone());
        set_cell(6, 1, "18000", cur_fmt.clone());
        set_cell(6, 2, "=B7*1.25", cur_fmt.clone());
        set_cell(6, 3, "=C7*1.22", cur_fmt.clone());
        set_cell(6, 4, "=D7*1.18", cur_fmt.clone());
        set_cell(6, 5, "=E7*1.15", cur_fmt.clone());

        set_cell(7, 0, "Total Gross Revenue", bold_label_fmt.clone());
        set_cell(7, 1, "=SUM(B6:B7)", cur_bold_fmt.clone());
        set_cell(7, 2, "=SUM(C6:C7)", cur_bold_fmt.clone());
        set_cell(7, 3, "=SUM(D6:D7)", cur_bold_fmt.clone());
        set_cell(7, 4, "=SUM(E6:E7)", cur_bold_fmt.clone());
        set_cell(7, 5, "=SUM(F6:F7)", cur_bold_fmt.clone());

        set_cell(8, 0, "YoY Revenue Growth Rate", row_label_fmt.clone());
        set_cell(8, 1, "0.182", pct_fmt.clone());
        set_cell(8, 2, "=(C8-B8)/B8", pct_fmt.clone());
        set_cell(8, 3, "=(D8-C8)/C8", pct_fmt.clone());
        set_cell(8, 4, "=(E8-D8)/D8", pct_fmt.clone());
        set_cell(8, 5, "=(F8-E8)/E8", pct_fmt.clone());

        // Cost of Goods Sold
        set_cell(10, 0, "COST OF REVENUE", section_fmt.clone());
        set_cell(11, 0, "Cost of Goods Sold (COGS)", row_label_fmt.clone());
        set_cell(11, 1, "=B8*0.38", cur_fmt.clone());
        set_cell(11, 2, "=C8*0.36", cur_fmt.clone());
        set_cell(11, 3, "=D8*0.35", cur_fmt.clone());
        set_cell(11, 4, "=E8*0.34", cur_fmt.clone());
        set_cell(11, 5, "=F8*0.33", cur_fmt.clone());

        set_cell(12, 0, "Gross Profit", bold_label_fmt.clone());
        set_cell(12, 1, "=B8-B12", cur_bold_fmt.clone());
        set_cell(12, 2, "=C8-C12", cur_bold_fmt.clone());
        set_cell(12, 3, "=D8-D12", cur_bold_fmt.clone());
        set_cell(12, 4, "=E8-E12", cur_bold_fmt.clone());
        set_cell(12, 5, "=F8-F12", cur_bold_fmt.clone());

        set_cell(13, 0, "Gross Margin %", row_label_fmt.clone());
        set_cell(13, 1, "=B13/B8", pct_fmt.clone());
        set_cell(13, 2, "=C13/C8", pct_fmt.clone());
        set_cell(13, 3, "=D13/D8", pct_fmt.clone());
        set_cell(13, 4, "=E13/E8", pct_fmt.clone());
        set_cell(13, 5, "=F13/F8", pct_fmt.clone());

        // Operating Expenses
        set_cell(15, 0, "OPERATING EXPENSES (OPEX)", section_fmt.clone());
        set_cell(16, 0, "Research & Development (R&D)", row_label_fmt.clone());
        set_cell(16, 1, "9500", cur_fmt.clone());
        set_cell(16, 2, "=B17*1.10", cur_fmt.clone());
        set_cell(16, 3, "=C17*1.08", cur_fmt.clone());
        set_cell(16, 4, "=D17*1.06", cur_fmt.clone());
        set_cell(16, 5, "=E17*1.05", cur_fmt.clone());

        set_cell(17, 0, "Sales & Marketing (S&M)", row_label_fmt.clone());
        set_cell(17, 1, "12500", cur_fmt.clone());
        set_cell(17, 2, "=B18*1.12", cur_fmt.clone());
        set_cell(17, 3, "=C18*1.09", cur_fmt.clone());
        set_cell(17, 4, "=D18*1.07", cur_fmt.clone());
        set_cell(17, 5, "=E18*1.05", cur_fmt.clone());

        set_cell(18, 0, "General & Administrative (G&A)", row_label_fmt.clone());
        set_cell(18, 1, "5200", cur_fmt.clone());
        set_cell(18, 2, "=B19*1.05", cur_fmt.clone());
        set_cell(18, 3, "=C19*1.04", cur_fmt.clone());
        set_cell(18, 4, "=D19*1.03", cur_fmt.clone());
        set_cell(18, 5, "=E19*1.03", cur_fmt.clone());

        set_cell(19, 0, "Total OPEX", bold_label_fmt.clone());
        set_cell(19, 1, "=SUM(B17:B19)", cur_bold_fmt.clone());
        set_cell(19, 2, "=SUM(C17:C19)", cur_bold_fmt.clone());
        set_cell(19, 3, "=SUM(D17:D19)", cur_bold_fmt.clone());
        set_cell(19, 4, "=SUM(E17:E19)", cur_bold_fmt.clone());
        set_cell(19, 5, "=SUM(F17:F19)", cur_bold_fmt.clone());

        // EBITDA & Net Income
        set_cell(21, 0, "OPERATING PROFITABILITY", section_fmt.clone());
        set_cell(22, 0, "EBITDA", bold_label_fmt.clone());
        set_cell(22, 1, "=B13-B20", cur_bold_fmt.clone());
        set_cell(22, 2, "=C13-C20", cur_bold_fmt.clone());
        set_cell(22, 3, "=D13-D20", cur_bold_fmt.clone());
        set_cell(22, 4, "=E13-E20", cur_bold_fmt.clone());
        set_cell(22, 5, "=F13-F20", cur_bold_fmt.clone());

        set_cell(23, 0, "EBITDA Margin %", row_label_fmt.clone());
        set_cell(23, 1, "=B23/B8", pct_fmt.clone());
        set_cell(23, 2, "=C23/C8", pct_fmt.clone());
        set_cell(23, 3, "=D23/D8", pct_fmt.clone());
        set_cell(23, 4, "=E23/E8", pct_fmt.clone());
        set_cell(23, 5, "=F23/F8", pct_fmt.clone());

        // Financial KPIs / Averages
        set_cell(25, 0, "5-YEAR SUMMARY & VALUATION", section_fmt);
        set_cell(26, 0, "Average 5-Yr EBITDA", bold_label_fmt.clone());
        set_cell(26, 1, "=AVERAGE(B23:F23)", cur_bold_fmt.clone());

        set_cell(27, 0, "Discount Rate (WACC)", row_label_fmt);
        set_cell(27, 1, "0.095", pct_fmt);

        set_cell(28, 0, "DCF Enterprise Value (NPV)", bold_label_fmt);
        set_cell(28, 1, "=NPV(B28, B23, C23, D23, E23, F23)", cur_bold_fmt);

        wb
    }
}

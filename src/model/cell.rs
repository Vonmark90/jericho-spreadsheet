use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct CellCoord {
    pub row: usize, // 0-indexed (row 0 = Row 1 in Excel)
    pub col: usize, // 0-indexed (col 0 = Column A in Excel)
}

impl CellCoord {
    pub const fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    /// Converts column index (0-indexed) to Excel column letters (0 -> "A", 25 -> "Z", 26 -> "AA")
    pub fn col_to_name(mut col: usize) -> String {
        let mut result = Vec::new();
        loop {
            let rem = (col % 26) as u8;
            result.push(b'A' + rem);
            if col < 26 {
                break;
            }
            col = (col / 26) - 1;
        }
        result.reverse();
        String::from_utf8(result).unwrap_or_else(|_| "A".to_string())
    }

    /// Converts Excel column letters to 0-indexed column ("A" -> 0, "Z" -> 25, "AA" -> 26)
    pub fn name_to_col(name: &str) -> Option<usize> {
        let name = name.trim().to_ascii_uppercase();
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphabetic()) {
            return None;
        }
        let mut col: usize = 0;
        for c in name.chars() {
            let val = (c as usize) - ('A' as usize) + 1;
            col = col.checked_mul(26)?.checked_add(val)?;
        }
        col.checked_sub(1)
    }

    /// Format as Excel A1 reference (e.g., (0, 0) -> "A1")
    pub fn to_a1(&self) -> String {
        format!("{}{}", Self::col_to_name(self.col), self.row + 1)
    }

    /// Parse an A1 reference string into CellCoord (handles "$A$1", "A1", "c12", etc.)
    pub fn from_a1(s: &str) -> Result<Self, String> {
        let clean = s.trim().replace('$', "");
        let letter_end = clean
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .count();
        if letter_end == 0 || letter_end == clean.len() {
            return Err(format!("Invalid cell reference: {}", s));
        }
        let (letters, digits) = clean.split_at(letter_end);
        let col = Self::name_to_col(letters)
            .ok_or_else(|| format!("Invalid column identifier in: {}", s))?;
        let row_num: usize = digits
            .parse()
            .map_err(|_| format!("Invalid row number in: {}", s))?;
        if row_num == 0 {
            return Err("Excel row numbers are 1-based".to_string());
        }
        Ok(Self::new(row_num - 1, col))
    }
}

impl fmt::Display for CellCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_a1())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CellError {
    Null,
    Div0,
    Value,
    Ref,
    Name,
    Num,
    NA,
    Circular,
}

impl fmt::Display for CellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            CellError::Null => "#NULL!",
            CellError::Div0 => "#DIV/0!",
            CellError::Value => "#VALUE!",
            CellError::Ref => "#REF!",
            CellError::Name => "#NAME?",
            CellError::Num => "#NUM!",
            CellError::NA => "#N/A",
            CellError::Circular => "#CIRCULAR!",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellValue {
    Empty,
    Number(f64),
    Text(String),
    Bool(bool),
    Error(CellError),
}

impl CellValue {
    pub fn is_empty(&self) -> bool {
        matches!(self, CellValue::Empty)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, CellValue::Error(_))
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            CellValue::Number(n) => Some(*n),
            CellValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            CellValue::Text(s) => s.trim().parse::<f64>().ok(),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            CellValue::Bool(b) => Some(*b),
            CellValue::Number(n) => Some(*n != 0.0),
            CellValue::Text(s) => match s.trim().to_ascii_uppercase().as_str() {
                "TRUE" => Some(true),
                "FALSE" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            CellValue::Empty => String::new(),
            CellValue::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            CellValue::Text(s) => s.clone(),
            CellValue::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            CellValue::Error(e) => e.to_string(),
        }
    }

    /// Formats the value according to the specified CellFormat
    pub fn format_display(&self, format: &CellFormat) -> String {
        match self {
            CellValue::Empty => String::new(),
            CellValue::Error(e) => e.to_string(),
            CellValue::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            CellValue::Text(s) => s.clone(),
            CellValue::Number(n) => format.number_format.format_number(*n),
        }
    }
}

impl Default for CellValue {
    fn default() -> Self {
        CellValue::Empty
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HorizAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NumberFormat {
    General,
    Number { decimals: u8, use_commas: bool },
    Currency { symbol: String, decimals: u8 },
    Percentage { decimals: u8 },
    Accounting { symbol: String, decimals: u8 },
    Scientific { decimals: u8 },
    Text,
}

impl NumberFormat {
    pub fn format_number(&self, n: f64) -> String {
        if n.is_nan() {
            return "#NUM!".to_string();
        }
        if n.is_infinite() {
            return if n > 0.0 { "#NUM!" } else { "-#NUM!" }.to_string();
        }

        match self {
            NumberFormat::General => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{}", n as i64)
                } else {
                    format!("{:.6}", n)
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string()
                }
            }
            NumberFormat::Number {
                decimals,
                use_commas,
            } => {
                let dec = *decimals as usize;
                let formatted = format!("{:.prec$}", n.abs(), prec = dec);
                let (int_part, frac_part) = if dec > 0 {
                    let parts: Vec<&str> = formatted.split('.').collect();
                    (parts[0], Some(parts[1]))
                } else {
                    (formatted.as_str(), None)
                };

                let int_str = if *use_commas {
                    insert_commas(int_part)
                } else {
                    int_part.to_string()
                };

                let sign = if n < 0.0 { "-" } else { "" };
                if let Some(frac) = frac_part {
                    format!("{}{}.{}", sign, int_str, frac)
                } else {
                    format!("{}{}", sign, int_str)
                }
            }
            NumberFormat::Currency { symbol, decimals } => {
                let dec = *decimals as usize;
                let formatted = format!("{:.prec$}", n.abs(), prec = dec);
                let parts: Vec<&str> = formatted.split('.').collect();
                let int_str = insert_commas(parts[0]);
                let sign = if n < 0.0 { "-" } else { "" };
                if dec > 0 && parts.len() > 1 {
                    format!("{}{}{}.{}", sign, symbol, int_str, parts[1])
                } else {
                    format!("{}{}{}", sign, symbol, int_str)
                }
            }
            NumberFormat::Accounting { symbol, decimals } => {
                let dec = *decimals as usize;
                let formatted = format!("{:.prec$}", n.abs(), prec = dec);
                let parts: Vec<&str> = formatted.split('.').collect();
                let int_str = insert_commas(parts[0]);
                if n < 0.0 {
                    if dec > 0 && parts.len() > 1 {
                        format!("({}{}.{})", symbol, int_str, parts[1])
                    } else {
                        format!("({}{})", symbol, int_str)
                    }
                } else if dec > 0 && parts.len() > 1 {
                    format!(" {}{}.{} ", symbol, int_str, parts[1])
                } else {
                    format!(" {}{} ", symbol, int_str)
                }
            }
            NumberFormat::Percentage { decimals } => {
                let pct = n * 100.0;
                format!("{:.prec$}%", pct, prec = *decimals as usize)
            }
            NumberFormat::Scientific { decimals } => {
                format!("{:.prec$E}", n, prec = *decimals as usize)
            }
            NumberFormat::Text => format!("{}", n),
        }
    }
}

fn insert_commas(s: &str) -> String {
    let mut result = String::new();
    let len = s.len();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellFormat {
    pub number_format: NumberFormat,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub font_size: f32,
    pub text_color: Option<[u8; 4]>,
    pub bg_color: Option<[u8; 4]>,
    pub align_h: HorizAlign,
    pub align_v: VertAlign,
}

impl Default for CellFormat {
    fn default() -> Self {
        Self {
            number_format: NumberFormat::General,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            font_size: 14.0,
            text_color: None,
            bg_color: None,
            align_h: HorizAlign::Left,
            align_v: VertAlign::Center,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellData {
    pub raw_input: String,
    pub value: CellValue,
    pub format: CellFormat,
}

impl CellData {
    pub fn new(input: impl Into<String>) -> Self {
        let raw = input.into();
        let value = Self::infer_value(&raw);
        let mut format = CellFormat::default();
        if matches!(value, CellValue::Number(_)) {
            format.align_h = HorizAlign::Right;
        }
        Self {
            raw_input: raw,
            value,
            format,
        }
    }

    pub fn with_value(value: CellValue) -> Self {
        let raw_input = value.as_string();
        let mut format = CellFormat::default();
        if matches!(value, CellValue::Number(_)) {
            format.align_h = HorizAlign::Right;
        }
        Self {
            raw_input,
            value,
            format,
        }
    }

    pub fn is_formula(&self) -> bool {
        self.raw_input.trim_start().starts_with('=')
    }

    pub fn formula_text(&self) -> Option<&str> {
        let trimmed = self.raw_input.trim();
        if trimmed.starts_with('=') {
            Some(&trimmed[1..])
        } else {
            None
        }
    }

    pub fn infer_value(raw: &str) -> CellValue {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return CellValue::Empty;
        }
        if trimmed.starts_with('=') {
            // Formula will be computed by formula engine
            return CellValue::Empty;
        }
        if let Ok(b) = trimmed.parse::<bool>() {
            return CellValue::Bool(b);
        }
        if trimmed.eq_ignore_ascii_case("TRUE") {
            return CellValue::Bool(true);
        }
        if trimmed.eq_ignore_ascii_case("FALSE") {
            return CellValue::Bool(false);
        }
        // Try parsing number (clean commas if any)
        let cleaned = trimmed.replace(',', "");
        if let Ok(num) = cleaned.parse::<f64>() {
            return CellValue::Number(num);
        }
        // Check for percentage
        if trimmed.ends_with('%') {
            let pct_str = trimmed[..trimmed.len() - 1].trim().replace(',', "");
            if let Ok(num) = pct_str.parse::<f64>() {
                return CellValue::Number(num / 100.0);
            }
        }
        // Check for currency e.g. $1,234.56
        if trimmed.starts_with('$') {
            let cur_str = trimmed[1..].trim().replace(',', "");
            if let Ok(num) = cur_str.parse::<f64>() {
                return CellValue::Number(num);
            }
        }
        CellValue::Text(raw.to_string())
    }
}

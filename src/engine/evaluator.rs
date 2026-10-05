use super::functions::eval_function;
use super::parser::{BinaryOp, Expr, FormulaParser, UnaryOp};
use crate::model::cell::{CellCoord, CellError, CellValue};
use crate::model::sheet::Sheet;

pub struct Evaluator<'a> {
    sheet: &'a Sheet,
    current_evaluating: CellCoord,
}

impl<'a> Evaluator<'a> {
    pub fn new(sheet: &'a Sheet, current_evaluating: CellCoord) -> Self {
        Self {
            sheet,
            current_evaluating,
        }
    }

    pub fn evaluate_formula(sheet: &'a Sheet, coord: CellCoord, formula: &str) -> CellValue {
        match FormulaParser::parse(formula) {
            Ok(expr) => {
                let eval = Evaluator::new(sheet, coord);
                match eval.eval_expr(&expr) {
                    Ok(val) => val,
                    Err(err) => CellValue::Error(err),
                }
            }
            Err(_) => CellValue::Error(CellError::Value),
        }
    }

    pub fn eval_expr(&self, expr: &Expr) -> Result<CellValue, CellError> {
        match expr {
            Expr::Number(n) => Ok(CellValue::Number(*n)),
            Expr::Text(s) => Ok(CellValue::Text(s.clone())),
            Expr::Bool(b) => Ok(CellValue::Bool(*b)),
            Expr::CellRef(coord) => {
                if *coord == self.current_evaluating {
                    return Err(CellError::Circular);
                }
                Ok(self.sheet.get_cell_value(*coord))
            }
            Expr::RangeRef { start, end } => {
                // If a range is evaluated as a scalar expression, return the top-left value
                let top_left = CellCoord::new(start.row.min(end.row), start.col.min(end.col));
                if top_left == self.current_evaluating {
                    return Err(CellError::Circular);
                }
                Ok(self.sheet.get_cell_value(top_left))
            }
            Expr::Unary { op, expr } => {
                let val = self.eval_expr(expr)?;
                match op {
                    UnaryOp::Neg => {
                        let n = val.as_number().ok_or(CellError::Value)?;
                        Ok(CellValue::Number(-n))
                    }
                    UnaryOp::Pos => {
                        let n = val.as_number().ok_or(CellError::Value)?;
                        Ok(CellValue::Number(n))
                    }
                    UnaryOp::Percent => {
                        let n = val.as_number().ok_or(CellError::Value)?;
                        Ok(CellValue::Number(n / 100.0))
                    }
                }
            }
            Expr::Binary { op, left, right } => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                self.eval_binary_op(*op, left_val, right_val)
            }
            Expr::FunctionCall { name, args } => self.eval_function_call(name, args),
        }
    }

    fn eval_binary_op(
        &self,
        op: BinaryOp,
        left: CellValue,
        right: CellValue,
    ) -> Result<CellValue, CellError> {
        if left.is_error() {
            return Ok(left);
        }
        if right.is_error() {
            return Ok(right);
        }

        match op {
            BinaryOp::Add => {
                let l = left.as_number().ok_or(CellError::Value)?;
                let r = right.as_number().ok_or(CellError::Value)?;
                Ok(CellValue::Number(l + r))
            }
            BinaryOp::Sub => {
                let l = left.as_number().ok_or(CellError::Value)?;
                let r = right.as_number().ok_or(CellError::Value)?;
                Ok(CellValue::Number(l - r))
            }
            BinaryOp::Mul => {
                let l = left.as_number().ok_or(CellError::Value)?;
                let r = right.as_number().ok_or(CellError::Value)?;
                Ok(CellValue::Number(l * r))
            }
            BinaryOp::Div => {
                let l = left.as_number().ok_or(CellError::Value)?;
                let r = right.as_number().ok_or(CellError::Value)?;
                if r == 0.0 {
                    Err(CellError::Div0)
                } else {
                    Ok(CellValue::Number(l / r))
                }
            }
            BinaryOp::Power => {
                let l = left.as_number().ok_or(CellError::Value)?;
                let r = right.as_number().ok_or(CellError::Value)?;
                let res = l.powf(r);
                if res.is_nan() || res.is_infinite() {
                    Err(CellError::Num)
                } else {
                    Ok(CellValue::Number(res))
                }
            }
            BinaryOp::Concat => {
                let l_str = left.as_string();
                let r_str = right.as_string();
                Ok(CellValue::Text(format!("{}{}", l_str, r_str)))
            }
            BinaryOp::Eq => Ok(CellValue::Bool(values_equal(&left, &right))),
            BinaryOp::NotEq => Ok(CellValue::Bool(!values_equal(&left, &right))),
            BinaryOp::Lt => compare_values(&left, &right).map(|cmp| CellValue::Bool(cmp < 0)),
            BinaryOp::LtEq => compare_values(&left, &right).map(|cmp| CellValue::Bool(cmp <= 0)),
            BinaryOp::Gt => compare_values(&left, &right).map(|cmp| CellValue::Bool(cmp > 0)),
            BinaryOp::GtEq => compare_values(&left, &right).map(|cmp| CellValue::Bool(cmp >= 0)),
        }
    }

    fn eval_function_call(&self, name: &str, args: &[Expr]) -> Result<CellValue, CellError> {
        let upper = name.to_ascii_uppercase();

        // Short-circuiting IF
        if upper == "IF" {
            if args.is_empty() {
                return Err(CellError::Value);
            }
            let cond_val = self.eval_expr(&args[0])?;
            let cond = cond_val.as_bool().ok_or(CellError::Value)?;
            if cond {
                return if args.len() > 1 {
                    self.eval_expr(&args[1])
                } else {
                    Ok(CellValue::Bool(true))
                };
            } else {
                return if args.len() > 2 {
                    self.eval_expr(&args[2])
                } else {
                    Ok(CellValue::Bool(false))
                };
            }
        }

        // Table / Array Lookup functions
        if upper == "VLOOKUP" {
            return self.eval_vlookup(args);
        }
        if upper == "INDEX" {
            return self.eval_index(args);
        }
        if upper == "MATCH" {
            return self.eval_match(args);
        }
        if upper == "XLOOKUP" {
            return self.eval_xlookup(args);
        }
        if upper == "SUMIF" {
            return self.eval_sumif(args);
        }
        if upper == "COUNTIF" {
            return self.eval_countif(args);
        }

        // Flatten range arguments for standard functions like SUM, AVERAGE, MIN, etc.
        let mut evaluated_args = Vec::new();
        for arg in args {
            if let Expr::RangeRef { start, end } = arg {
                let min_r = start.row.min(end.row);
                let max_r = start.row.max(end.row);
                let min_c = start.col.min(end.col);
                let max_c = start.col.max(end.col);
                for r in min_r..=max_r {
                    for c in min_c..=max_c {
                        let coord = CellCoord::new(r, c);
                        if coord == self.current_evaluating {
                            return Err(CellError::Circular);
                        }
                        evaluated_args.push(self.sheet.get_cell_value(coord));
                    }
                }
            } else {
                let val = self.eval_expr(arg)?;
                evaluated_args.push(val);
            }
        }

        eval_function(&upper, &evaluated_args)
    }

    /// VLOOKUP(lookup_value, table_array, col_index, [range_lookup])
    fn eval_vlookup(&self, args: &[Expr]) -> Result<CellValue, CellError> {
        if args.len() < 3 {
            return Err(CellError::Value);
        }
        let lookup_val = self.eval_expr(&args[0])?;
        let col_index = self
            .eval_expr(&args[2])?
            .as_number()
            .ok_or(CellError::Value)? as usize;
        if col_index == 0 {
            return Err(CellError::Value);
        }

        let exact_match = if args.len() > 3 {
            !self.eval_expr(&args[3])?.as_bool().unwrap_or(true)
        } else {
            false
        };

        if let Expr::RangeRef { start, end } = &args[1] {
            let min_r = start.row.min(end.row);
            let max_r = start.row.max(end.row);
            let min_c = start.col.min(end.col);
            let max_c = start.col.max(end.col);

            let target_col = min_c + col_index - 1;
            if target_col > max_c {
                return Err(CellError::Ref);
            }

            for r in min_r..=max_r {
                let cell_val = self.sheet.get_cell_value(CellCoord::new(r, min_c));
                let is_match = if exact_match {
                    values_equal(&lookup_val, &cell_val)
                } else {
                    // Approximate or exact
                    values_equal(&lookup_val, &cell_val)
                };
                if is_match {
                    return Ok(self.sheet.get_cell_value(CellCoord::new(r, target_col)));
                }
            }
            Err(CellError::NA)
        } else {
            Err(CellError::Value)
        }
    }

    /// INDEX(array, row_num, [col_num])
    fn eval_index(&self, args: &[Expr]) -> Result<CellValue, CellError> {
        if args.len() < 2 {
            return Err(CellError::Value);
        }
        let row_num = self
            .eval_expr(&args[1])?
            .as_number()
            .ok_or(CellError::Value)? as usize;
        let col_num = if args.len() > 2 {
            self.eval_expr(&args[2])?
                .as_number()
                .ok_or(CellError::Value)? as usize
        } else {
            1
        };

        if row_num == 0 || col_num == 0 {
            return Err(CellError::Value);
        }

        if let Expr::RangeRef { start, end } = &args[0] {
            let min_r = start.row.min(end.row);
            let max_r = start.row.max(end.row);
            let min_c = start.col.min(end.col);
            let max_c = start.col.max(end.col);

            let r = min_r + row_num - 1;
            let c = min_c + col_num - 1;
            if r > max_r || c > max_c {
                return Err(CellError::Ref);
            }
            Ok(self.sheet.get_cell_value(CellCoord::new(r, c)))
        } else {
            Err(CellError::Value)
        }
    }

    /// MATCH(lookup_value, lookup_array, [match_type])
    fn eval_match(&self, args: &[Expr]) -> Result<CellValue, CellError> {
        if args.len() < 2 {
            return Err(CellError::Value);
        }
        let lookup_val = self.eval_expr(&args[0])?;
        if let Expr::RangeRef { start, end } = &args[1] {
            let min_r = start.row.min(end.row);
            let max_r = start.row.max(end.row);
            let min_c = start.col.min(end.col);
            let max_c = start.col.max(end.col);

            let mut idx = 1;
            for r in min_r..=max_r {
                for c in min_c..=max_c {
                    let cell_val = self.sheet.get_cell_value(CellCoord::new(r, c));
                    if values_equal(&lookup_val, &cell_val) {
                        return Ok(CellValue::Number(idx as f64));
                    }
                    idx += 1;
                }
            }
            Err(CellError::NA)
        } else {
            Err(CellError::Value)
        }
    }

    /// XLOOKUP(lookup_value, lookup_array, return_array, [if_not_found])
    fn eval_xlookup(&self, args: &[Expr]) -> Result<CellValue, CellError> {
        if args.len() < 3 {
            return Err(CellError::Value);
        }
        let lookup_val = self.eval_expr(&args[0])?;
        let if_not_found = if args.len() > 3 {
            Some(self.eval_expr(&args[3])?)
        } else {
            None
        };

        if let (
            Expr::RangeRef {
                start: l_start,
                end: l_end,
            },
            Expr::RangeRef {
                start: r_start,
                end: r_end,
            },
        ) = (&args[1], &args[2])
        {
            let l_min_r = l_start.row.min(l_end.row);
            let l_max_r = l_start.row.max(l_end.row);
            let r_min_r = r_start.row.min(r_end.row);

            let l_min_c = l_start.col.min(l_end.col);
            let r_min_c = r_start.col.min(r_end.col);

            for (i, r) in (l_min_r..=l_max_r).enumerate() {
                let cell_val = self.sheet.get_cell_value(CellCoord::new(r, l_min_c));
                if values_equal(&lookup_val, &cell_val) {
                    let ret_coord = CellCoord::new(r_min_r + i, r_min_c);
                    return Ok(self.sheet.get_cell_value(ret_coord));
                }
            }

            if let Some(fallback) = if_not_found {
                Ok(fallback)
            } else {
                Err(CellError::NA)
            }
        } else {
            Err(CellError::Value)
        }
    }

    /// SUMIF(range, criteria, [sum_range])
    fn eval_sumif(&self, args: &[Expr]) -> Result<CellValue, CellError> {
        if args.len() < 2 {
            return Err(CellError::Value);
        }
        let criteria = self.eval_expr(&args[1])?;

        if let Expr::RangeRef { start, end } = &args[0] {
            let min_r = start.row.min(end.row);
            let max_r = start.row.max(end.row);
            let min_c = start.col.min(end.col);
            let max_c = start.col.max(end.col);

            let sum_offset = if args.len() > 2 {
                if let Expr::RangeRef {
                    start: s_start,
                    end: _,
                } = &args[2]
                {
                    Some((s_start.row as isize - min_r as isize, s_start.col as isize - min_c as isize))
                } else {
                    None
                }
            } else {
                None
            };

            let mut total = 0.0;
            for r in min_r..=max_r {
                for c in min_c..=max_c {
                    let check_val = self.sheet.get_cell_value(CellCoord::new(r, c));
                    if matches_criteria(&check_val, &criteria) {
                        let sum_coord = if let Some((dr, dc)) = sum_offset {
                            let sr = (r as isize + dr).max(0) as usize;
                            let sc = (c as isize + dc).max(0) as usize;
                            CellCoord::new(sr, sc)
                        } else {
                            CellCoord::new(r, c)
                        };
                        if let Some(n) = self.sheet.get_cell_value(sum_coord).as_number() {
                            total += n;
                        }
                    }
                }
            }
            Ok(CellValue::Number(total))
        } else {
            Err(CellError::Value)
        }
    }

    /// COUNTIF(range, criteria)
    fn eval_countif(&self, args: &[Expr]) -> Result<CellValue, CellError> {
        if args.len() < 2 {
            return Err(CellError::Value);
        }
        let criteria = self.eval_expr(&args[1])?;

        if let Expr::RangeRef { start, end } = &args[0] {
            let min_r = start.row.min(end.row);
            let max_r = start.row.max(end.row);
            let min_c = start.col.min(end.col);
            let max_c = start.col.max(end.col);

            let mut count = 0;
            for r in min_r..=max_r {
                for c in min_c..=max_c {
                    let check_val = self.sheet.get_cell_value(CellCoord::new(r, c));
                    if matches_criteria(&check_val, &criteria) {
                        count += 1;
                    }
                }
            }
            Ok(CellValue::Number(count as f64))
        } else {
            Err(CellError::Value)
        }
    }
}

fn values_equal(a: &CellValue, b: &CellValue) -> bool {
    match (a, b) {
        (CellValue::Empty, CellValue::Empty) => true,
        (CellValue::Number(n1), CellValue::Number(n2)) => (n1 - n2).abs() < 1e-12,
        (CellValue::Text(s1), CellValue::Text(s2)) => s1.eq_ignore_ascii_case(s2),
        (CellValue::Bool(b1), CellValue::Bool(b2)) => b1 == b2,
        (CellValue::Number(n), CellValue::Text(s)) | (CellValue::Text(s), CellValue::Number(n)) => {
            s.parse::<f64>().map(|num| (num - n).abs() < 1e-12).unwrap_or(false)
        }
        _ => false,
    }
}

fn compare_values(a: &CellValue, b: &CellValue) -> Result<i8, CellError> {
    if let (Some(n1), Some(n2)) = (a.as_number(), b.as_number()) {
        if (n1 - n2).abs() < 1e-12 {
            Ok(0)
        } else if n1 < n2 {
            Ok(-1)
        } else {
            Ok(1)
        }
    } else {
        let s1 = a.as_string();
        let s2 = b.as_string();
        Ok(match s1.cmp(&s2) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        })
    }
}

fn matches_criteria(val: &CellValue, crit: &CellValue) -> bool {
    let crit_str = crit.as_string();
    if crit_str.starts_with(">=") {
        if let Ok(target) = crit_str[2..].trim().parse::<f64>() {
            return val.as_number().map(|n| n >= target).unwrap_or(false);
        }
    } else if crit_str.starts_with("<=") {
        if let Ok(target) = crit_str[2..].trim().parse::<f64>() {
            return val.as_number().map(|n| n <= target).unwrap_or(false);
        }
    } else if crit_str.starts_with('>') {
        if let Ok(target) = crit_str[1..].trim().parse::<f64>() {
            return val.as_number().map(|n| n > target).unwrap_or(false);
        }
    } else if crit_str.starts_with('<') {
        if let Ok(target) = crit_str[1..].trim().parse::<f64>() {
            return val.as_number().map(|n| n < target).unwrap_or(false);
        }
    } else if crit_str.starts_with("<>") {
        let target = &crit_str[2..].trim();
        return !val.as_string().eq_ignore_ascii_case(target);
    }
    values_equal(val, crit)
}

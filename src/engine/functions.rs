use crate::model::cell::{CellError, CellValue};
use chrono::{Datelike, Local};

pub fn eval_function(name: &str, args: &[CellValue]) -> Result<CellValue, CellError> {
    match name.to_ascii_uppercase().as_str() {
        // --- Math & Statistics ---
        "SUM" => fn_sum(args),
        "AVERAGE" | "AVG" => fn_average(args),
        "MIN" => fn_min(args),
        "MAX" => fn_max(args),
        "COUNT" => fn_count(args),
        "COUNTA" => fn_counta(args),
        "PRODUCT" => fn_product(args),
        "ABS" => fn_abs(args),
        "SQRT" => fn_sqrt(args),
        "ROUND" => fn_round(args),
        "ROUNDUP" => fn_roundup(args),
        "ROUNDDOWN" => fn_rounddown(args),
        "FLOOR" => fn_floor(args),
        "CEILING" => fn_ceiling(args),
        "MOD" => fn_mod(args),
        "POWER" => fn_power(args),
        "INT" => fn_int(args),
        "MEDIAN" => fn_median(args),
        "STDEV" | "STDEV.S" => fn_stdev(args, false),
        "STDEVP" | "STDEV.P" => fn_stdev(args, true),

        // --- Logic ---
        "IF" => fn_if(args),
        "IFS" => fn_ifs(args),
        "AND" => fn_and(args),
        "OR" => fn_or(args),
        "NOT" => fn_not(args),
        "XOR" => fn_xor(args),
        "IFERROR" => fn_iferror(args),

        // --- Financial ---
        "PMT" => fn_pmt(args),
        "PV" => fn_pv(args),
        "FV" => fn_fv(args),
        "NPV" => fn_npv(args),
        "NPER" => fn_nper(args),

        // --- Text ---
        "CONCAT" | "CONCATENATE" => fn_concat(args),
        "LEFT" => fn_left(args),
        "RIGHT" => fn_right(args),
        "MID" => fn_mid(args),
        "LEN" => fn_len(args),
        "TRIM" => fn_trim(args),
        "UPPER" => fn_upper(args),
        "LOWER" => fn_lower(args),
        "PROPER" => fn_proper(args),

        // --- Date & Time ---
        "TODAY" => fn_today(args),
        "NOW" => fn_now(args),
        "DATE" => fn_date(args),
        "YEAR" => fn_year(args),
        "MONTH" => fn_month(args),
        "DAY" => fn_day(args),

        _ => Err(CellError::Name),
    }
}

// ---------------- Math & Stats ----------------

fn fn_sum(args: &[CellValue]) -> Result<CellValue, CellError> {
    let mut sum = 0.0;
    for arg in args {
        if let Some(n) = arg.as_number() {
            sum += n;
        }
    }
    Ok(CellValue::Number(sum))
}

fn fn_average(args: &[CellValue]) -> Result<CellValue, CellError> {
    let mut sum = 0.0;
    let mut count = 0;
    for arg in args {
        if let Some(n) = arg.as_number() {
            sum += n;
            count += 1;
        }
    }
    if count == 0 {
        Err(CellError::Div0)
    } else {
        Ok(CellValue::Number(sum / count as f64))
    }
}

fn fn_min(args: &[CellValue]) -> Result<CellValue, CellError> {
    let mut min: Option<f64> = None;
    for arg in args {
        if let Some(n) = arg.as_number() {
            min = Some(min.map_or(n, |m| m.min(n)));
        }
    }
    min.map(CellValue::Number).ok_or(CellError::Value)
}

fn fn_max(args: &[CellValue]) -> Result<CellValue, CellError> {
    let mut max: Option<f64> = None;
    for arg in args {
        if let Some(n) = arg.as_number() {
            max = Some(max.map_or(n, |m| m.max(n)));
        }
    }
    max.map(CellValue::Number).ok_or(CellError::Value)
}

fn fn_count(args: &[CellValue]) -> Result<CellValue, CellError> {
    let mut count = 0;
    for arg in args {
        if let CellValue::Number(_) = arg {
            count += 1;
        }
    }
    Ok(CellValue::Number(count as f64))
}

fn fn_counta(args: &[CellValue]) -> Result<CellValue, CellError> {
    let mut count = 0;
    for arg in args {
        if !arg.is_empty() {
            count += 1;
        }
    }
    Ok(CellValue::Number(count as f64))
}

fn fn_product(args: &[CellValue]) -> Result<CellValue, CellError> {
    let mut prod = 1.0;
    let mut has_num = false;
    for arg in args {
        if let Some(n) = arg.as_number() {
            prod *= n;
            has_num = true;
        }
    }
    if has_num {
        Ok(CellValue::Number(prod))
    } else {
        Ok(CellValue::Number(0.0))
    }
}

fn fn_abs(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    Ok(CellValue::Number(n.abs()))
}

fn fn_sqrt(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if n < 0.0 {
        Err(CellError::Num)
    } else {
        Ok(CellValue::Number(n.sqrt()))
    }
}

fn fn_round(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let decimals = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as i32;
    let factor = 10f64.powi(decimals);
    Ok(CellValue::Number((n * factor).round() / factor))
}

fn fn_roundup(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let decimals = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as i32;
    let factor = 10f64.powi(decimals);
    let sign = if n < 0.0 { -1.0 } else { 1.0 };
    Ok(CellValue::Number(sign * (n.abs() * factor).ceil() / factor))
}

fn fn_rounddown(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let decimals = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as i32;
    let factor = 10f64.powi(decimals);
    let sign = if n < 0.0 { -1.0 } else { 1.0 };
    Ok(CellValue::Number(sign * (n.abs() * factor).floor() / factor))
}

fn fn_floor(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let sig = args.get(1).and_then(|v| v.as_number()).unwrap_or(1.0);
    if sig == 0.0 {
        return Err(CellError::Div0);
    }
    Ok(CellValue::Number((n / sig).floor() * sig))
}

fn fn_ceiling(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let sig = args.get(1).and_then(|v| v.as_number()).unwrap_or(1.0);
    if sig == 0.0 {
        return Err(CellError::Div0);
    }
    Ok(CellValue::Number((n / sig).ceil() * sig))
}

fn fn_mod(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let d = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if d == 0.0 {
        Err(CellError::Div0)
    } else {
        Ok(CellValue::Number(((n % d) + d) % d))
    }
}

fn fn_power(args: &[CellValue]) -> Result<CellValue, CellError> {
    let base = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let exp = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let result = base.powf(exp);
    if result.is_nan() || result.is_infinite() {
        Err(CellError::Num)
    } else {
        Ok(CellValue::Number(result))
    }
}

fn fn_int(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    Ok(CellValue::Number(n.floor()))
}

fn fn_median(args: &[CellValue]) -> Result<CellValue, CellError> {
    let mut nums: Vec<f64> = args.iter().filter_map(|v| v.as_number()).collect();
    if nums.is_empty() {
        return Err(CellError::Num);
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = nums.len() / 2;
    if nums.len() % 2 == 1 {
        Ok(CellValue::Number(nums[mid]))
    } else {
        Ok(CellValue::Number((nums[mid - 1] + nums[mid]) / 2.0))
    }
}

fn fn_stdev(args: &[CellValue], is_pop: bool) -> Result<CellValue, CellError> {
    let nums: Vec<f64> = args.iter().filter_map(|v| v.as_number()).collect();
    let n = nums.len();
    if (!is_pop && n < 2) || (is_pop && n < 1) {
        return Err(CellError::Div0);
    }
    let mean = nums.iter().sum::<f64>() / n as f64;
    let variance_sum = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>();
    let denom = if is_pop { n as f64 } else { (n - 1) as f64 };
    Ok(CellValue::Number((variance_sum / denom).sqrt()))
}

// ---------------- Logic ----------------

fn fn_if(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.is_empty() {
        return Err(CellError::Value);
    }
    let cond = args[0].as_bool().ok_or(CellError::Value)?;
    if cond {
        Ok(args.get(1).cloned().unwrap_or(CellValue::Bool(true)))
    } else {
        Ok(args.get(2).cloned().unwrap_or(CellValue::Bool(false)))
    }
}

fn fn_ifs(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 2 || args.len() % 2 != 0 {
        return Err(CellError::Value);
    }
    for chunk in args.chunks(2) {
        let cond = chunk[0].as_bool().ok_or(CellError::Value)?;
        if cond {
            return Ok(chunk[1].clone());
        }
    }
    Err(CellError::NA)
}

fn fn_and(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.is_empty() {
        return Err(CellError::Value);
    }
    for arg in args {
        let b = arg.as_bool().ok_or(CellError::Value)?;
        if !b {
            return Ok(CellValue::Bool(false));
        }
    }
    Ok(CellValue::Bool(true))
}

fn fn_or(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.is_empty() {
        return Err(CellError::Value);
    }
    for arg in args {
        let b = arg.as_bool().ok_or(CellError::Value)?;
        if b {
            return Ok(CellValue::Bool(true));
        }
    }
    Ok(CellValue::Bool(false))
}

fn fn_not(args: &[CellValue]) -> Result<CellValue, CellError> {
    let b = args.first().and_then(|v| v.as_bool()).ok_or(CellError::Value)?;
    Ok(CellValue::Bool(!b))
}

fn fn_xor(args: &[CellValue]) -> Result<CellValue, CellError> {
    let mut true_count = 0;
    for arg in args {
        let b = arg.as_bool().ok_or(CellError::Value)?;
        if b {
            true_count += 1;
        }
    }
    Ok(CellValue::Bool(true_count % 2 == 1))
}

fn fn_iferror(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 2 {
        return Err(CellError::Value);
    }
    if args[0].is_error() {
        Ok(args[1].clone())
    } else {
        Ok(args[0].clone())
    }
}

// ---------------- Financial ----------------

/// PMT(rate, nper, pv, [fv], [type])
/// Calculates the payment for a loan based on constant payments and a constant interest rate.
fn fn_pmt(args: &[CellValue]) -> Result<CellValue, CellError> {
    let rate = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let nper = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let pv = args.get(2).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let fv = args.get(3).and_then(|v| v.as_number()).unwrap_or(0.0);
    let pay_type = args.get(4).and_then(|v| v.as_number()).unwrap_or(0.0);

    if nper == 0.0 {
        return Err(CellError::Div0);
    }

    if rate == 0.0 {
        return Ok(CellValue::Number(-(pv + fv) / nper));
    }

    let pvif = (1.0 + rate).powf(nper);
    let mut pmt = (rate / (pvif - 1.0)) * -(pv * pvif + fv);
    if pay_type != 0.0 {
        pmt /= 1.0 + rate;
    }

    Ok(CellValue::Number(pmt))
}

/// PV(rate, nper, pmt, [fv], [type])
fn fn_pv(args: &[CellValue]) -> Result<CellValue, CellError> {
    let rate = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let nper = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let pmt = args.get(2).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let fv = args.get(3).and_then(|v| v.as_number()).unwrap_or(0.0);
    let pay_type = args.get(4).and_then(|v| v.as_number()).unwrap_or(0.0);

    if rate == 0.0 {
        return Ok(CellValue::Number(-pmt * nper - fv));
    }

    let pvif = (1.0 + rate).powf(nper);
    let factor = if pay_type != 0.0 { 1.0 + rate } else { 1.0 };
    let pv = (-fv - pmt * factor * ((pvif - 1.0) / rate)) / pvif;
    Ok(CellValue::Number(pv))
}

/// FV(rate, nper, pmt, [pv], [type])
fn fn_fv(args: &[CellValue]) -> Result<CellValue, CellError> {
    let rate = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let nper = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let pmt = args.get(2).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let pv = args.get(3).and_then(|v| v.as_number()).unwrap_or(0.0);
    let pay_type = args.get(4).and_then(|v| v.as_number()).unwrap_or(0.0);

    if rate == 0.0 {
        return Ok(CellValue::Number(-pv - pmt * nper));
    }

    let pvif = (1.0 + rate).powf(nper);
    let factor = if pay_type != 0.0 { 1.0 + rate } else { 1.0 };
    let fv = -pv * pvif - pmt * factor * ((pvif - 1.0) / rate);
    Ok(CellValue::Number(fv))
}

/// NPV(rate, value1, [value2], ...)
fn fn_npv(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 2 {
        return Err(CellError::Value);
    }
    let rate = args[0].as_number().ok_or(CellError::Value)?;
    let mut total_npv = 0.0;
    for (i, val) in args.iter().skip(1).enumerate() {
        if let Some(cf) = val.as_number() {
            total_npv += cf / (1.0 + rate).powi((i + 1) as i32);
        }
    }
    Ok(CellValue::Number(total_npv))
}

/// NPER(rate, pmt, pv, [fv], [type])
fn fn_nper(args: &[CellValue]) -> Result<CellValue, CellError> {
    let rate = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let pmt = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let pv = args.get(2).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let fv = args.get(3).and_then(|v| v.as_number()).unwrap_or(0.0);
    let pay_type = args.get(4).and_then(|v| v.as_number()).unwrap_or(0.0);

    if rate == 0.0 {
        if pmt == 0.0 {
            return Err(CellError::Div0);
        }
        return Ok(CellValue::Number(-(pv + fv) / pmt));
    }

    let factor = if pay_type != 0.0 { 1.0 + rate } else { 1.0 };
    let num = (pmt * factor - fv * rate) / (pmt * factor + pv * rate);
    if num <= 0.0 {
        return Err(CellError::Num);
    }
    Ok(CellValue::Number(num.ln() / (1.0 + rate).ln()))
}

// ---------------- Text ----------------

fn fn_concat(args: &[CellValue]) -> Result<CellValue, CellError> {
    let mut result = String::new();
    for arg in args {
        result.push_str(&arg.as_string());
    }
    Ok(CellValue::Text(result))
}

fn fn_left(args: &[CellValue]) -> Result<CellValue, CellError> {
    let text = args.first().map(|v| v.as_string()).unwrap_or_default();
    let num = args.get(1).and_then(|v| v.as_number()).unwrap_or(1.0) as usize;
    let s: String = text.chars().take(num).collect();
    Ok(CellValue::Text(s))
}

fn fn_right(args: &[CellValue]) -> Result<CellValue, CellError> {
    let text = args.first().map(|v| v.as_string()).unwrap_or_default();
    let num = args.get(1).and_then(|v| v.as_number()).unwrap_or(1.0) as usize;
    let chars: Vec<char> = text.chars().collect();
    let start = chars.len().saturating_sub(num);
    let s: String = chars[start..].iter().collect();
    Ok(CellValue::Text(s))
}

fn fn_mid(args: &[CellValue]) -> Result<CellValue, CellError> {
    let text = args.first().map(|v| v.as_string()).unwrap_or_default();
    let start = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)? as usize;
    let len = args.get(2).and_then(|v| v.as_number()).ok_or(CellError::Value)? as usize;
    let start_idx = start.saturating_sub(1);
    let s: String = text.chars().skip(start_idx).take(len).collect();
    Ok(CellValue::Text(s))
}

fn fn_len(args: &[CellValue]) -> Result<CellValue, CellError> {
    let text = args.first().map(|v| v.as_string()).unwrap_or_default();
    Ok(CellValue::Number(text.chars().count() as f64))
}

fn fn_trim(args: &[CellValue]) -> Result<CellValue, CellError> {
    let text = args.first().map(|v| v.as_string()).unwrap_or_default();
    Ok(CellValue::Text(text.trim().to_string()))
}

fn fn_upper(args: &[CellValue]) -> Result<CellValue, CellError> {
    let text = args.first().map(|v| v.as_string()).unwrap_or_default();
    Ok(CellValue::Text(text.to_uppercase()))
}

fn fn_lower(args: &[CellValue]) -> Result<CellValue, CellError> {
    let text = args.first().map(|v| v.as_string()).unwrap_or_default();
    Ok(CellValue::Text(text.to_lowercase()))
}

fn fn_proper(args: &[CellValue]) -> Result<CellValue, CellError> {
    let text = args.first().map(|v| v.as_string()).unwrap_or_default();
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in text.chars() {
        if c.is_alphabetic() {
            if capitalize_next {
                result.extend(c.to_uppercase());
                capitalize_next = false;
            } else {
                result.extend(c.to_lowercase());
            }
        } else {
            result.push(c);
            capitalize_next = true;
        }
    }
    Ok(CellValue::Text(result))
}

// ---------------- Date & Time ----------------

fn fn_today(_args: &[CellValue]) -> Result<CellValue, CellError> {
    let now = Local::now().date_naive();
    // Excel serial date: days since Dec 30, 1899
    let base = chrono::NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
    let days = (now - base).num_days() as f64;
    Ok(CellValue::Number(days))
}

fn fn_now(_args: &[CellValue]) -> Result<CellValue, CellError> {
    let now = Local::now().naive_local();
    let base = chrono::NaiveDateTime::new(
        chrono::NaiveDate::from_ymd_opt(1899, 12, 30).unwrap(),
        chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
    );
    let seconds = (now - base).num_seconds() as f64;
    let days = seconds / 86400.0;
    Ok(CellValue::Number(days))
}

fn fn_date(args: &[CellValue]) -> Result<CellValue, CellError> {
    let year = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)? as i32;
    let month = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)? as u32;
    let day = args.get(2).and_then(|v| v.as_number()).ok_or(CellError::Value)? as u32;

    let d = chrono::NaiveDate::from_ymd_opt(year, month, day).ok_or(CellError::Num)?;
    let base = chrono::NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
    let days = (d - base).num_days() as f64;
    Ok(CellValue::Number(days))
}

fn fn_year(args: &[CellValue]) -> Result<CellValue, CellError> {
    let serial = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)? as i64;
    let base = chrono::NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
    let d = base
        .checked_add_signed(chrono::Duration::days(serial))
        .ok_or(CellError::Num)?;
    Ok(CellValue::Number(d.year() as f64))
}

fn fn_month(args: &[CellValue]) -> Result<CellValue, CellError> {
    let serial = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)? as i64;
    let base = chrono::NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
    let d = base
        .checked_add_signed(chrono::Duration::days(serial))
        .ok_or(CellError::Num)?;
    Ok(CellValue::Number(d.month() as f64))
}

fn fn_day(args: &[CellValue]) -> Result<CellValue, CellError> {
    let serial = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)? as i64;
    let base = chrono::NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
    let d = base
        .checked_add_signed(chrono::Duration::days(serial))
        .ok_or(CellError::Num)?;
    Ok(CellValue::Number(d.day() as f64))
}

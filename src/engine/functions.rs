use crate::model::cell::{CellCoord, CellError, CellValue};
use chrono::{Datelike, Local, NaiveDate};

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
        "TRUNC" => fn_trunc(args),
        "EVEN" => fn_even(args),
        "ODD" => fn_odd(args),
        "MEDIAN" => fn_median(args),
        "STDEV" | "STDEV.S" => fn_stdev(args, false),
        "STDEVP" | "STDEV.P" => fn_stdev(args, true),
        "VAR" | "VAR.S" => fn_var(args, false),
        "VARP" | "VAR.P" => fn_var(args, true),
        "AVEDEV" => fn_avedev(args),
        "LARGE" => fn_large(args),
        "SMALL" => fn_small(args),
        "PERCENTILE" | "PERCENTILE.INC" => fn_percentile(args),
        "QUARTILE" | "QUARTILE.INC" => fn_quartile(args),
        "RANK" | "RANK.EQ" => fn_rank(args),
        "MODE" | "MODE.SNGL" => fn_mode(args),
        "LN" => fn_ln(args),
        "LOG" => fn_log(args),
        "LOG10" => fn_log10(args),
        "EXP" => fn_exp(args),
        "PI" => fn_pi(args),
        "E" => fn_e(args),
        "SIGN" => fn_sign(args),
        "SIN" => fn_sin(args),
        "COS" => fn_cos(args),
        "TAN" => fn_tan(args),
        "ASIN" => fn_asin(args),
        "ACOS" => fn_acos(args),
        "ATAN" => fn_atan(args),
        "ATAN2" => fn_atan2(args),
        "RADIANS" => fn_radians(args),
        "DEGREES" => fn_degrees(args),
        "FACT" => fn_fact(args),
        "FACTDOUBLE" => fn_factdouble(args),
        "COMBIN" => fn_combin(args),
        "PERMUT" => fn_permut(args),
        "QUOTIENT" => fn_quotient(args),
        "GCD" => fn_gcd(args),
        "LCM" => fn_lcm(args),
        "RAND" => fn_rand(args),
        "RANDBETWEEN" => fn_randbetween(args),

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
        "IRR" => fn_irr(args),
        "RATE" => fn_rate(args),
        "EFFECT" => fn_effect(args),
        "NOMINAL" => fn_nominal(args),
        "SLN" => fn_sln(args),
        "SYD" => fn_syd(args),
        "DDB" => fn_ddb(args),
        "IPMT" => fn_ipmt(args),
        "PPMT" => fn_ppmt(args),
        "CUMIPMT" => fn_cumipmt(args),
        "CUMPRINC" => fn_cumprinc(args),

        // --- Information ---
        "ISBLANK" => fn_isblank(args),
        "ISNUMBER" => fn_isnumber(args),
        "ISTEXT" => fn_istext(args),
        "ISNONTEXT" => fn_isnontext(args),
        "ISLOGICAL" => fn_islogical(args),
        "ISERROR" => fn_iserror(args),
        "ISNA" => fn_isna(args),
        "N" => fn_n(args),
        "T" => fn_t(args),
        "TYPE" => fn_type(args),

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
        "REPLACE" => fn_replace(args),
        "SUBSTITUTE" => fn_substitute(args),
        "REPT" => fn_rept(args),
        "FIND" => fn_find(args),
        "SEARCH" => fn_search(args),
        "EXACT" => fn_exact(args),
        "TEXTJOIN" => fn_textjoin(args),
        "VALUE" => fn_value(args),
        "TEXT" => fn_text(args),
        "CHAR" => fn_char(args),
        "CODE" => fn_code(args),

        // --- Date & Time ---
        "TODAY" => fn_today(args),
        "NOW" => fn_now(args),
        "DATE" => fn_date(args),
        "YEAR" => fn_year(args),
        "MONTH" => fn_month(args),
        "DAY" => fn_day(args),
        "HOUR" => fn_hour(args),
        "MINUTE" => fn_minute(args),
        "SECOND" => fn_second(args),
        "TIME" => fn_time(args),
        "DAYS" => fn_days(args),
        "WEEKDAY" => fn_weekday(args),
        "EDATE" => fn_edate(args),
        "EOMONTH" => fn_eomonth(args),
        "DATEDIF" => fn_datedif(args),

        // --- Reference ---
        "ADDRESS" => fn_address(args),

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

fn serial_to_date(serial: f64) -> Result<NaiveDate, CellError> {
    let days = serial.floor() as i64;
    let base = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
    base.checked_add_signed(chrono::Duration::days(days))
        .ok_or(CellError::Num)
}

fn fn_hour(args: &[CellValue]) -> Result<CellValue, CellError> {
    let serial = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let fract = (serial.fract().abs() * 24.0).floor() as u32;
    Ok(CellValue::Number((fract % 24) as f64))
}

fn fn_minute(args: &[CellValue]) -> Result<CellValue, CellError> {
    let serial = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let fract = (serial.fract().abs() * 1440.0).floor() as u32;
    Ok(CellValue::Number((fract % 60) as f64))
}

fn fn_second(args: &[CellValue]) -> Result<CellValue, CellError> {
    let serial = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let fract = (serial.fract().abs() * 86400.0).round() as u32;
    Ok(CellValue::Number((fract % 60) as f64))
}

fn fn_time(args: &[CellValue]) -> Result<CellValue, CellError> {
    let h = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let m = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let s = args.get(2).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let total_secs = h * 3600.0 + m * 60.0 + s;
    Ok(CellValue::Number((total_secs / 86400.0).fract()))
}

fn fn_days(args: &[CellValue]) -> Result<CellValue, CellError> {
    let end_d = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let start_d = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    Ok(CellValue::Number(end_d.floor() - start_d.floor()))
}

fn fn_weekday(args: &[CellValue]) -> Result<CellValue, CellError> {
    let serial = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let return_type = args.get(1).and_then(|v| v.as_number()).unwrap_or(1.0) as i32;
    let d = serial_to_date(serial)?;
    // chrono weekday: Mon=0 .. Sun=6
    let w = d.weekday().num_days_from_sunday(); // Sun=0, Mon=1 .. Sat=6
    let res = match return_type {
        1 => w + 1, // Sun=1 .. Sat=7 (Excel default)
        2 => d.weekday().num_days_from_monday() + 1, // Mon=1 .. Sun=7
        _ => w + 1,
    };
    Ok(CellValue::Number(res as f64))
}

fn fn_edate(args: &[CellValue]) -> Result<CellValue, CellError> {
    let serial = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let months = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)? as i32;
    let d = serial_to_date(serial)?;
    let total_months = d.year() * 12 + (d.month0() as i32) + months;
    let new_year = total_months.div_euclid(12);
    let new_month = (total_months.rem_euclid(12) + 1) as u32;
    let max_day = match new_month {
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (new_year % 4 == 0 && new_year % 100 != 0) || (new_year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 31,
    };
    let new_day = d.day().min(max_day);
    let new_date = NaiveDate::from_ymd_opt(new_year, new_month, new_day).ok_or(CellError::Num)?;
    let base = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
    Ok(CellValue::Number((new_date - base).num_days() as f64))
}

fn fn_eomonth(args: &[CellValue]) -> Result<CellValue, CellError> {
    let serial = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let months = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)? as i32;
    let d = serial_to_date(serial)?;
    let total_months = d.year() * 12 + (d.month0() as i32) + months;
    let new_year = total_months.div_euclid(12);
    let new_month = (total_months.rem_euclid(12) + 1) as u32;
    let max_day = match new_month {
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (new_year % 4 == 0 && new_year % 100 != 0) || (new_year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 31,
    };
    let new_date = NaiveDate::from_ymd_opt(new_year, new_month, max_day).ok_or(CellError::Num)?;
    let base = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
    Ok(CellValue::Number((new_date - base).num_days() as f64))
}

fn fn_datedif(args: &[CellValue]) -> Result<CellValue, CellError> {
    let s1 = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let s2 = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let unit = args.get(2).map(|v| v.as_string().to_ascii_uppercase()).ok_or(CellError::Value)?;
    let d1 = serial_to_date(s1)?;
    let d2 = serial_to_date(s2)?;
    if d1 > d2 {
        return Err(CellError::Num);
    }
    match unit.as_str() {
        "Y" => {
            let mut y = d2.year() - d1.year();
            if (d2.month(), d2.day()) < (d1.month(), d1.day()) {
                y -= 1;
            }
            Ok(CellValue::Number(y.max(0) as f64))
        }
        "M" => {
            let total_m2 = d2.year() * 12 + d2.month0() as i32;
            let total_m1 = d1.year() * 12 + d1.month0() as i32;
            let mut m = total_m2 - total_m1;
            if d2.day() < d1.day() {
                m -= 1;
            }
            Ok(CellValue::Number(m.max(0) as f64))
        }
        "D" => Ok(CellValue::Number((d2 - d1).num_days() as f64)),
        _ => Err(CellError::Value),
    }
}

// ---------------- Advanced Math, Stats & Financial ----------------

fn fn_trunc(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let dec = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as i32;
    let factor = 10f64.powi(dec);
    Ok(CellValue::Number((n * factor).trunc() / factor))
}

fn fn_even(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let sign = if n < 0.0 { -1.0 } else { 1.0 };
    let mut rounded = n.abs().ceil();
    if (rounded as i64) % 2 != 0 {
        rounded += 1.0;
    }
    Ok(CellValue::Number(rounded * sign))
}

fn fn_odd(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let sign = if n < 0.0 { -1.0 } else { 1.0 };
    let mut rounded = n.abs().ceil();
    if (rounded as i64) % 2 == 0 {
        rounded += 1.0;
    }
    Ok(CellValue::Number(rounded * sign))
}

fn fn_var(args: &[CellValue], is_pop: bool) -> Result<CellValue, CellError> {
    let nums: Vec<f64> = args.iter().filter_map(|v| v.as_number()).collect();
    let n = nums.len();
    if (!is_pop && n < 2) || (is_pop && n < 1) {
        return Err(CellError::Div0);
    }
    let mean = nums.iter().sum::<f64>() / n as f64;
    let sum_sq: f64 = nums.iter().map(|&x| (x - mean).powi(2)).sum();
    let divisor = if is_pop { n as f64 } else { (n - 1) as f64 };
    Ok(CellValue::Number(sum_sq / divisor))
}

fn fn_avedev(args: &[CellValue]) -> Result<CellValue, CellError> {
    let nums: Vec<f64> = args.iter().filter_map(|v| v.as_number()).collect();
    if nums.is_empty() {
        return Err(CellError::Div0);
    }
    let mean = nums.iter().sum::<f64>() / nums.len() as f64;
    let sum_dev: f64 = nums.iter().map(|&x| (x - mean).abs()).sum();
    Ok(CellValue::Number(sum_dev / nums.len() as f64))
}

fn fn_large(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 2 {
        return Err(CellError::Value);
    }
    let k = args.last().and_then(|v| v.as_number()).ok_or(CellError::Value)? as usize;
    let mut nums: Vec<f64> = args[..args.len() - 1].iter().filter_map(|v| v.as_number()).collect();
    if k == 0 || k > nums.len() {
        return Err(CellError::Num);
    }
    nums.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    Ok(CellValue::Number(nums[k - 1]))
}

fn fn_small(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 2 {
        return Err(CellError::Value);
    }
    let k = args.last().and_then(|v| v.as_number()).ok_or(CellError::Value)? as usize;
    let mut nums: Vec<f64> = args[..args.len() - 1].iter().filter_map(|v| v.as_number()).collect();
    if k == 0 || k > nums.len() {
        return Err(CellError::Num);
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Ok(CellValue::Number(nums[k - 1]))
}

fn fn_percentile(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 2 {
        return Err(CellError::Value);
    }
    let k = args.last().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if !(0.0..=1.0).contains(&k) {
        return Err(CellError::Num);
    }
    let mut nums: Vec<f64> = args[..args.len() - 1].iter().filter_map(|v| v.as_number()).collect();
    if nums.is_empty() {
        return Err(CellError::Num);
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if nums.len() == 1 {
        return Ok(CellValue::Number(nums[0]));
    }
    let rank = k * (nums.len() - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    let weight = rank - lower as f64;
    let val = nums[lower] + weight * (nums[upper] - nums[lower]);
    Ok(CellValue::Number(val))
}

fn fn_quartile(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 2 {
        return Err(CellError::Value);
    }
    let q = args.last().and_then(|v| v.as_number()).ok_or(CellError::Value)? as i32;
    let k = match q {
        0 => 0.0,
        1 => 0.25,
        2 => 0.50,
        3 => 0.75,
        4 => 1.0,
        _ => return Err(CellError::Num),
    };
    let mut new_args = args[..args.len() - 1].to_vec();
    new_args.push(CellValue::Number(k));
    fn_percentile(&new_args)
}

fn fn_rank(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 2 {
        return Err(CellError::Value);
    }
    let target = args[0].as_number().ok_or(CellError::Value)?;
    let order = args.get(2).and_then(|v| v.as_number()).unwrap_or(0.0) as i32;
    let mut nums: Vec<f64> = args[1..].iter().filter_map(|v| v.as_number()).collect();
    if !nums.iter().any(|&n| (n - target).abs() < 1e-12) {
        return Err(CellError::NA);
    }
    if order == 0 {
        nums.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    } else {
        nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    }
    for (i, &n) in nums.iter().enumerate() {
        if (n - target).abs() < 1e-12 {
            return Ok(CellValue::Number((i + 1) as f64));
        }
    }
    Err(CellError::NA)
}

fn fn_mode(args: &[CellValue]) -> Result<CellValue, CellError> {
    let nums: Vec<f64> = args.iter().filter_map(|v| v.as_number()).collect();
    if nums.is_empty() {
        return Err(CellError::NA);
    }
    let mut counts: std::collections::HashMap<i64, (usize, f64)> = std::collections::HashMap::new();
    for &n in &nums {
        let key = (n * 1_000_000.0).round() as i64;
        let entry = counts.entry(key).or_insert((0, n));
        entry.0 += 1;
    }
    let max_count = counts.values().map(|(c, _)| *c).max().unwrap_or(0);
    if max_count <= 1 {
        return Err(CellError::NA);
    }
    for &n in &nums {
        let key = (n * 1_000_000.0).round() as i64;
        if let Some((c, val)) = counts.get(&key) {
            if *c == max_count {
                return Ok(CellValue::Number(*val));
            }
        }
    }
    Err(CellError::NA)
}

fn fn_ln(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if n <= 0.0 {
        return Err(CellError::Num);
    }
    Ok(CellValue::Number(n.ln()))
}

fn fn_log10(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if n <= 0.0 {
        return Err(CellError::Num);
    }
    Ok(CellValue::Number(n.log10()))
}

fn fn_log(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let base = args.get(1).and_then(|v| v.as_number()).unwrap_or(10.0);
    if n <= 0.0 || base <= 0.0 || (base - 1.0).abs() < 1e-12 {
        return Err(CellError::Num);
    }
    Ok(CellValue::Number(n.log(base)))
}

fn fn_exp(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    Ok(CellValue::Number(n.exp()))
}

fn fn_pi(_args: &[CellValue]) -> Result<CellValue, CellError> {
    Ok(CellValue::Number(std::f64::consts::PI))
}

fn fn_e(_args: &[CellValue]) -> Result<CellValue, CellError> {
    Ok(CellValue::Number(std::f64::consts::E))
}

fn fn_sign(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if n > 0.0 {
        Ok(CellValue::Number(1.0))
    } else if n < 0.0 {
        Ok(CellValue::Number(-1.0))
    } else {
        Ok(CellValue::Number(0.0))
    }
}

fn fn_sin(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    Ok(CellValue::Number(n.sin()))
}

fn fn_cos(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    Ok(CellValue::Number(n.cos()))
}

fn fn_tan(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    Ok(CellValue::Number(n.tan()))
}

fn fn_asin(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if !(-1.0..=1.0).contains(&n) {
        return Err(CellError::Num);
    }
    Ok(CellValue::Number(n.asin()))
}

fn fn_acos(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if !(-1.0..=1.0).contains(&n) {
        return Err(CellError::Num);
    }
    Ok(CellValue::Number(n.acos()))
}

fn fn_atan(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    Ok(CellValue::Number(n.atan()))
}

fn fn_atan2(args: &[CellValue]) -> Result<CellValue, CellError> {
    let x = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let y = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if x == 0.0 && y == 0.0 {
        return Err(CellError::Div0);
    }
    Ok(CellValue::Number(y.atan2(x)))
}

fn fn_radians(args: &[CellValue]) -> Result<CellValue, CellError> {
    let deg = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    Ok(CellValue::Number(deg.to_radians()))
}

fn fn_degrees(args: &[CellValue]) -> Result<CellValue, CellError> {
    let rad = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    Ok(CellValue::Number(rad.to_degrees()))
}

fn fn_fact(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if n < 0.0 || n > 170.0 {
        return Err(CellError::Num);
    }
    let mut res = 1.0;
    for i in 1..=(n.floor() as u64) {
        res *= i as f64;
    }
    Ok(CellValue::Number(res))
}

fn fn_factdouble(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if n < 0.0 || n > 300.0 {
        return Err(CellError::Num);
    }
    let mut res = 1.0;
    let mut i = n.floor() as u64;
    while i > 0 {
        res *= i as f64;
        if i >= 2 {
            i -= 2;
        } else {
            break;
        }
    }
    Ok(CellValue::Number(res))
}

fn fn_combin(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?.floor() as u64;
    let k = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?.floor() as u64;
    if k > n {
        return Err(CellError::Num);
    }
    let k = k.min(n - k);
    let mut c = 1.0;
    for i in 1..=k {
        c = c * (n - k + i) as f64 / i as f64;
    }
    Ok(CellValue::Number(c.round()))
}

fn fn_permut(args: &[CellValue]) -> Result<CellValue, CellError> {
    let n = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?.floor() as u64;
    let k = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?.floor() as u64;
    if k > n {
        return Err(CellError::Num);
    }
    let mut p = 1.0;
    for i in (n - k + 1)..=n {
        p *= i as f64;
    }
    Ok(CellValue::Number(p.round()))
}

fn fn_quotient(args: &[CellValue]) -> Result<CellValue, CellError> {
    let num = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let den = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if den == 0.0 {
        return Err(CellError::Div0);
    }
    Ok(CellValue::Number((num / den).trunc()))
}

fn gcd_2(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn fn_gcd(args: &[CellValue]) -> Result<CellValue, CellError> {
    let nums: Vec<u64> = args.iter().filter_map(|v| v.as_number().map(|n| n.abs().floor() as u64)).collect();
    if nums.is_empty() {
        return Err(CellError::Value);
    }
    let mut g = nums[0];
    for &n in &nums[1..] {
        g = gcd_2(g, n);
    }
    Ok(CellValue::Number(g as f64))
}

fn fn_lcm(args: &[CellValue]) -> Result<CellValue, CellError> {
    let nums: Vec<u64> = args.iter().filter_map(|v| v.as_number().map(|n| n.abs().floor() as u64)).collect();
    if nums.is_empty() {
        return Err(CellError::Value);
    }
    if nums.contains(&0) {
        return Ok(CellValue::Number(0.0));
    }
    let mut l = nums[0];
    for &n in &nums[1..] {
        l = (l / gcd_2(l, n)) * n;
    }
    Ok(CellValue::Number(l as f64))
}

fn random_f64() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(123456789);
    let mut x = nanos as u64;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    ((x % 1_000_000_000) as f64) / 1_000_000_000.0
}

fn fn_rand(_args: &[CellValue]) -> Result<CellValue, CellError> {
    Ok(CellValue::Number(random_f64()))
}

fn fn_randbetween(args: &[CellValue]) -> Result<CellValue, CellError> {
    let min = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?.floor() as i64;
    let max = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?.floor() as i64;
    if min > max {
        return Err(CellError::Num);
    }
    let span = (max - min + 1) as f64;
    let val = min + (random_f64() * span).floor() as i64;
    Ok(CellValue::Number(val as f64))
}

// ---------------- Expanded Financial ----------------

fn fn_irr(args: &[CellValue]) -> Result<CellValue, CellError> {
    let mut cfs = Vec::new();
    for a in args {
        if let Some(n) = a.as_number() {
            cfs.push(n);
        }
    }
    if cfs.len() < 2 {
        return Err(CellError::Num);
    }
    let has_pos = cfs.iter().any(|&c| c > 0.0);
    let has_neg = cfs.iter().any(|&c| c < 0.0);
    if !has_pos || !has_neg {
        return Err(CellError::Num);
    }

    let mut rate = 0.1f64;
    for _ in 0..100 {
        let mut f = 0.0;
        let mut df = 0.0;
        for (t, &cf) in cfs.iter().enumerate() {
            let denom = (1.0 + rate).powi(t as i32);
            if denom.abs() < 1e-15 {
                return Err(CellError::Div0);
            }
            f += cf / denom;
            if t > 0 {
                df -= (t as f64) * cf / (1.0 + rate).powi((t + 1) as i32);
            }
        }
        if df.abs() < 1e-12 {
            break;
        }
        let next_rate = rate - f / df;
        if (next_rate - rate).abs() < 1e-7 {
            return Ok(CellValue::Number(next_rate));
        }
        rate = next_rate;
    }
    Ok(CellValue::Number(rate))
}

fn fn_rate(args: &[CellValue]) -> Result<CellValue, CellError> {
    let nper = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let pmt = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let pv = args.get(2).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let fv = args.get(3).and_then(|v| v.as_number()).unwrap_or(0.0);
    let pay_type = args.get(4).and_then(|v| v.as_number()).unwrap_or(0.0);
    let guess = args.get(5).and_then(|v| v.as_number()).unwrap_or(0.1);

    let mut rate = guess;
    for _ in 0..100 {
        let y = if rate.abs() < 1e-9 {
            pv + pmt * nper + fv
        } else {
            let pvif = (1.0 + rate).powf(nper);
            let factor = if pay_type != 0.0 { 1.0 + rate } else { 1.0 };
            pv * pvif + pmt * factor * ((pvif - 1.0) / rate) + fv
        };
        if y.abs() < 1e-7 {
            return Ok(CellValue::Number(rate));
        }
        let r2 = rate * 1.0001 + 1e-6;
        let y2 = {
            let pvif = (1.0 + r2).powf(nper);
            let factor = if pay_type != 0.0 { 1.0 + r2 } else { 1.0 };
            pv * pvif + pmt * factor * ((pvif - 1.0) / r2) + fv
        };
        let dy = (y2 - y) / (r2 - rate);
        if dy.abs() < 1e-12 {
            break;
        }
        let next_rate = rate - y / dy;
        if (next_rate - rate).abs() < 1e-7 {
            return Ok(CellValue::Number(next_rate));
        }
        rate = next_rate;
    }
    Ok(CellValue::Number(rate))
}

fn fn_effect(args: &[CellValue]) -> Result<CellValue, CellError> {
    let nom = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let npery = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if nom <= 0.0 || npery < 1.0 {
        return Err(CellError::Num);
    }
    let npery = npery.floor();
    Ok(CellValue::Number((1.0 + nom / npery).powf(npery) - 1.0))
}

fn fn_nominal(args: &[CellValue]) -> Result<CellValue, CellError> {
    let eff = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let npery = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if eff <= 0.0 || npery < 1.0 {
        return Err(CellError::Num);
    }
    let npery = npery.floor();
    Ok(CellValue::Number(npery * ((1.0 + eff).powf(1.0 / npery) - 1.0)))
}

fn fn_sln(args: &[CellValue]) -> Result<CellValue, CellError> {
    let cost = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let salvage = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let life = args.get(2).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if life <= 0.0 {
        return Err(CellError::Div0);
    }
    Ok(CellValue::Number((cost - salvage) / life))
}

fn fn_syd(args: &[CellValue]) -> Result<CellValue, CellError> {
    let cost = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let salvage = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let life = args.get(2).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let per = args.get(3).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    if life <= 0.0 || per < 1.0 || per > life {
        return Err(CellError::Num);
    }
    let dep = ((cost - salvage) * (life - per + 1.0) * 2.0) / (life * (life + 1.0));
    Ok(CellValue::Number(dep))
}

fn fn_ddb(args: &[CellValue]) -> Result<CellValue, CellError> {
    let cost = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let salvage = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let life = args.get(2).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let period = args.get(3).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let factor = args.get(4).and_then(|v| v.as_number()).unwrap_or(2.0);
    if cost < 0.0 || salvage < 0.0 || life <= 0.0 || period <= 0.0 || period > life || factor <= 0.0 {
        return Err(CellError::Num);
    }
    let mut book_val = cost;
    let mut dep = 0.0;
    for _ in 1..=(period as usize) {
        dep = (book_val * factor / life).min(book_val - salvage).max(0.0);
        book_val -= dep;
    }
    Ok(CellValue::Number(dep))
}

fn fn_ipmt(args: &[CellValue]) -> Result<CellValue, CellError> {
    let rate = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let per = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let nper = args.get(2).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let pv = args.get(3).and_then(|v| v.as_number()).ok_or(CellError::Value)?;
    let _fv = args.get(4).and_then(|v| v.as_number()).unwrap_or(0.0);
    let pay_type = args.get(5).and_then(|v| v.as_number()).unwrap_or(0.0);
    if per < 1.0 || per > nper {
        return Err(CellError::Num);
    }
    let pmt_val = fn_pmt(args)?.as_number().unwrap_or(0.0);
    let fv_prev = fn_fv(&[
        CellValue::Number(rate),
        CellValue::Number(per - 1.0),
        CellValue::Number(pmt_val),
        CellValue::Number(pv),
        CellValue::Number(pay_type),
    ])?
    .as_number()
    .unwrap_or(0.0);
    let mut ipmt = -fv_prev * rate;
    if pay_type != 0.0 && (per - 1.0).abs() < 1e-12 {
        ipmt = 0.0;
    }
    Ok(CellValue::Number(ipmt))
}

fn fn_ppmt(args: &[CellValue]) -> Result<CellValue, CellError> {
    let pmt_val = fn_pmt(args)?.as_number().ok_or(CellError::Value)?;
    let ipmt_val = fn_ipmt(args)?.as_number().ok_or(CellError::Value)?;
    Ok(CellValue::Number(pmt_val - ipmt_val))
}

fn fn_cumipmt(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 6 {
        return Err(CellError::Value);
    }
    let rate = args[0].as_number().ok_or(CellError::Value)?;
    let nper = args[1].as_number().ok_or(CellError::Value)?;
    let pv = args[2].as_number().ok_or(CellError::Value)?;
    let start_p = args[3].as_number().ok_or(CellError::Value)? as usize;
    let end_p = args[4].as_number().ok_or(CellError::Value)? as usize;
    let pay_type = args[5].as_number().ok_or(CellError::Value)?;
    if start_p < 1 || end_p < start_p || end_p > nper as usize {
        return Err(CellError::Num);
    }
    let mut total = 0.0;
    for p in start_p..=end_p {
        let p_args = [
            CellValue::Number(rate),
            CellValue::Number(p as f64),
            CellValue::Number(nper),
            CellValue::Number(pv),
            CellValue::Number(0.0),
            CellValue::Number(pay_type),
        ];
        total += fn_ipmt(&p_args)?.as_number().unwrap_or(0.0);
    }
    Ok(CellValue::Number(total))
}

fn fn_cumprinc(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 6 {
        return Err(CellError::Value);
    }
    let rate = args[0].as_number().ok_or(CellError::Value)?;
    let nper = args[1].as_number().ok_or(CellError::Value)?;
    let pv = args[2].as_number().ok_or(CellError::Value)?;
    let start_p = args[3].as_number().ok_or(CellError::Value)? as usize;
    let end_p = args[4].as_number().ok_or(CellError::Value)? as usize;
    let pay_type = args[5].as_number().ok_or(CellError::Value)?;
    if start_p < 1 || end_p < start_p || end_p > nper as usize {
        return Err(CellError::Num);
    }
    let mut total = 0.0;
    for p in start_p..=end_p {
        let p_args = [
            CellValue::Number(rate),
            CellValue::Number(p as f64),
            CellValue::Number(nper),
            CellValue::Number(pv),
            CellValue::Number(0.0),
            CellValue::Number(pay_type),
        ];
        total += fn_ppmt(&p_args)?.as_number().unwrap_or(0.0);
    }
    Ok(CellValue::Number(total))
}

// ---------------- Information & Type Inspection ----------------

fn fn_isblank(args: &[CellValue]) -> Result<CellValue, CellError> {
    Ok(CellValue::Bool(matches!(args.first(), Some(CellValue::Empty))))
}

fn fn_isnumber(args: &[CellValue]) -> Result<CellValue, CellError> {
    Ok(CellValue::Bool(matches!(args.first(), Some(CellValue::Number(_)))))
}

fn fn_istext(args: &[CellValue]) -> Result<CellValue, CellError> {
    Ok(CellValue::Bool(matches!(args.first(), Some(CellValue::Text(_)))))
}

fn fn_isnontext(args: &[CellValue]) -> Result<CellValue, CellError> {
    Ok(CellValue::Bool(!matches!(args.first(), Some(CellValue::Text(_)))))
}

fn fn_islogical(args: &[CellValue]) -> Result<CellValue, CellError> {
    Ok(CellValue::Bool(matches!(args.first(), Some(CellValue::Bool(_)))))
}

fn fn_iserror(args: &[CellValue]) -> Result<CellValue, CellError> {
    Ok(CellValue::Bool(matches!(args.first(), Some(CellValue::Error(_)))))
}

fn fn_isna(args: &[CellValue]) -> Result<CellValue, CellError> {
    Ok(CellValue::Bool(matches!(args.first(), Some(CellValue::Error(CellError::NA)))))
}

fn fn_n(args: &[CellValue]) -> Result<CellValue, CellError> {
    match args.first() {
        Some(CellValue::Number(n)) => Ok(CellValue::Number(*n)),
        Some(CellValue::Bool(true)) => Ok(CellValue::Number(1.0)),
        _ => Ok(CellValue::Number(0.0)),
    }
}

fn fn_t(args: &[CellValue]) -> Result<CellValue, CellError> {
    match args.first() {
        Some(CellValue::Text(s)) => Ok(CellValue::Text(s.clone())),
        _ => Ok(CellValue::Text(String::new())),
    }
}

fn fn_type(args: &[CellValue]) -> Result<CellValue, CellError> {
    match args.first() {
        Some(CellValue::Number(_)) => Ok(CellValue::Number(1.0)),
        Some(CellValue::Text(_)) => Ok(CellValue::Number(2.0)),
        Some(CellValue::Bool(_)) => Ok(CellValue::Number(4.0)),
        Some(CellValue::Error(_)) => Ok(CellValue::Number(16.0)),
        _ => Ok(CellValue::Number(1.0)),
    }
}

// ---------------- Text Operations ----------------

fn fn_replace(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 4 {
        return Err(CellError::Value);
    }
    let orig = args[0].as_string();
    let start = args[1].as_number().ok_or(CellError::Value)? as usize;
    let count = args[2].as_number().ok_or(CellError::Value)? as usize;
    let replacement = args[3].as_string();

    if start == 0 {
        return Err(CellError::Value);
    }
    let chars: Vec<char> = orig.chars().collect();
    let s_idx = (start - 1).min(chars.len());
    let e_idx = (s_idx + count).min(chars.len());

    let mut result = String::new();
    result.extend(&chars[..s_idx]);
    result.push_str(&replacement);
    result.extend(&chars[e_idx..]);
    Ok(CellValue::Text(result))
}

fn fn_substitute(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 3 {
        return Err(CellError::Value);
    }
    let orig = args[0].as_string();
    let old_s = args[1].as_string();
    let new_s = args[2].as_string();
    let instance = args.get(3).and_then(|v| v.as_number()).map(|n| n as usize);

    if old_s.is_empty() {
        return Ok(CellValue::Text(orig));
    }

    if let Some(target_inst) = instance {
        if target_inst == 0 {
            return Err(CellError::Value);
        }
        let mut curr_inst = 0;
        let mut last_end = 0;
        let mut result = String::new();
        while let Some(idx) = orig[last_end..].find(&old_s) {
            let match_pos = last_end + idx;
            curr_inst += 1;
            if curr_inst == target_inst {
                result.push_str(&orig[last_end..match_pos]);
                result.push_str(&new_s);
                result.push_str(&orig[match_pos + old_s.len()..]);
                return Ok(CellValue::Text(result));
            }
            result.push_str(&orig[last_end..match_pos + old_s.len()]);
            last_end = match_pos + old_s.len();
        }
        Ok(CellValue::Text(orig))
    } else {
        Ok(CellValue::Text(orig.replace(&old_s, &new_s)))
    }
}

fn fn_rept(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 2 {
        return Err(CellError::Value);
    }
    let s = args[0].as_string();
    let n = args[1].as_number().ok_or(CellError::Value)?;
    if n < 0.0 || n > 32767.0 {
        return Err(CellError::Value);
    }
    Ok(CellValue::Text(s.repeat(n as usize)))
}

fn fn_find(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 2 {
        return Err(CellError::Value);
    }
    let find_text = args[0].as_string();
    let within_text = args[1].as_string();
    let start_num = args.get(2).and_then(|v| v.as_number()).unwrap_or(1.0) as usize;
    if start_num == 0 || start_num > within_text.chars().count() + 1 {
        return Err(CellError::Value);
    }
    let chars: Vec<char> = within_text.chars().collect();
    let slice: String = chars[start_num - 1..].iter().collect();
    if let Some(byte_idx) = slice.find(&find_text) {
        let char_offset = slice[..byte_idx].chars().count();
        Ok(CellValue::Number((start_num + char_offset) as f64))
    } else {
        Err(CellError::Value)
    }
}

fn fn_search(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 2 {
        return Err(CellError::Value);
    }
    let find_text = args[0].as_string().to_ascii_uppercase();
    let within_text = args[1].as_string().to_ascii_uppercase();
    let start_num = args.get(2).and_then(|v| v.as_number()).unwrap_or(1.0) as usize;
    if start_num == 0 || start_num > within_text.chars().count() + 1 {
        return Err(CellError::Value);
    }
    let chars: Vec<char> = within_text.chars().collect();
    let slice: String = chars[start_num - 1..].iter().collect();
    if let Some(byte_idx) = slice.find(&find_text) {
        let char_offset = slice[..byte_idx].chars().count();
        Ok(CellValue::Number((start_num + char_offset) as f64))
    } else {
        Err(CellError::Value)
    }
}

fn fn_exact(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 2 {
        return Err(CellError::Value);
    }
    let s1 = args[0].as_string();
    let s2 = args[1].as_string();
    Ok(CellValue::Bool(s1 == s2))
}

fn fn_textjoin(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 3 {
        return Err(CellError::Value);
    }
    let delimiter = args[0].as_string();
    let ignore_empty = args[1].as_bool().unwrap_or(true);
    let mut parts = Vec::new();
    for a in &args[2..] {
        let s = a.as_string();
        if ignore_empty && (s.is_empty() || matches!(a, CellValue::Empty)) {
            continue;
        }
        parts.push(s);
    }
    Ok(CellValue::Text(parts.join(&delimiter)))
}

fn fn_value(args: &[CellValue]) -> Result<CellValue, CellError> {
    let s = args.first().ok_or(CellError::Value)?.as_string();
    let cleaned = s.replace('$', "").replace(',', "").replace('%', "").trim().to_string();
    if let Ok(n) = cleaned.parse::<f64>() {
        if s.contains('%') {
            Ok(CellValue::Number(n / 100.0))
        } else {
            Ok(CellValue::Number(n))
        }
    } else {
        Err(CellError::Value)
    }
}

fn fn_text(args: &[CellValue]) -> Result<CellValue, CellError> {
    if args.len() < 2 {
        return Err(CellError::Value);
    }
    let num = args[0].as_number().ok_or(CellError::Value)?;
    let fmt_str = args[1].as_string();
    if fmt_str.contains('%') {
        Ok(CellValue::Text(format!("{:.1}%", num * 100.0)))
    } else if fmt_str.contains('$') {
        Ok(CellValue::Text(format!("${:.2}", num)))
    } else if fmt_str.contains(".00") {
        Ok(CellValue::Text(format!("{:.2}", num)))
    } else {
        Ok(CellValue::Text(format!("{}", num)))
    }
}

fn fn_char(args: &[CellValue]) -> Result<CellValue, CellError> {
    let code = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)? as u32;
    if let Some(ch) = char::from_u32(code) {
        Ok(CellValue::Text(ch.to_string()))
    } else {
        Err(CellError::Value)
    }
}

fn fn_code(args: &[CellValue]) -> Result<CellValue, CellError> {
    let s = args.first().ok_or(CellError::Value)?.as_string();
    if let Some(ch) = s.chars().next() {
        Ok(CellValue::Number(ch as u32 as f64))
    } else {
        Err(CellError::Value)
    }
}

// ---------------- Reference ----------------

fn fn_address(args: &[CellValue]) -> Result<CellValue, CellError> {
    let row = args.first().and_then(|v| v.as_number()).ok_or(CellError::Value)? as usize;
    let col = args.get(1).and_then(|v| v.as_number()).ok_or(CellError::Value)? as usize;
    let abs_num = args.get(2).and_then(|v| v.as_number()).unwrap_or(1.0) as i32;
    if row == 0 || col == 0 {
        return Err(CellError::Value);
    }
    let col_name = CellCoord::col_to_name(col - 1);
    let addr = match abs_num {
        1 => format!("${}${}", col_name, row),
        2 => format!("{}${}", col_name, row),
        3 => format!("${}{}", col_name, row),
        4 => format!("{}{}", col_name, row),
        _ => return Err(CellError::Value),
    };
    Ok(CellValue::Text(addr))
}

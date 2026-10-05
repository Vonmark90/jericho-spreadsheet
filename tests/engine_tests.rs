use jericho_spreadsheet::engine::dependency::{recalculate_incremental, recalculate_sheet};
use jericho_spreadsheet::engine::evaluator::Evaluator;
use jericho_spreadsheet::io::csv::{export_csv, import_csv};
use jericho_spreadsheet::io::xlsx::{load_xlsx, save_xlsx};
use jericho_spreadsheet::model::cell::{CellCoord, CellError, CellValue};
use jericho_spreadsheet::model::sheet::Sheet;
use jericho_spreadsheet::model::workbook::Workbook;

#[test]
fn test_basic_arithmetic_formulas() {
    let sheet = Sheet::new("Test");
    let c = CellCoord::new(0, 0);

    let v1 = Evaluator::evaluate_formula(&sheet, c, "=2 + 3 * 4");
    assert_eq!(v1, CellValue::Number(14.0));

    let v2 = Evaluator::evaluate_formula(&sheet, c, "=(2 + 3) * 4");
    assert_eq!(v2, CellValue::Number(20.0));

    let v3 = Evaluator::evaluate_formula(&sheet, c, "=2 ^ 3");
    assert_eq!(v3, CellValue::Number(8.0));

    let v4 = Evaluator::evaluate_formula(&sheet, c, "=10 / 2");
    assert_eq!(v4, CellValue::Number(5.0));

    let v5 = Evaluator::evaluate_formula(&sheet, c, "=10 / 0");
    assert_eq!(v5, CellValue::Error(CellError::Div0));
}

#[test]
fn test_string_concat_and_logic() {
    let sheet = Sheet::new("Test");
    let c = CellCoord::new(0, 0);

    let v1 = Evaluator::evaluate_formula(&sheet, c, "=\"Hello \" & \"World\"");
    assert_eq!(v1, CellValue::Text("Hello World".to_string()));

    let v2 = Evaluator::evaluate_formula(&sheet, c, "=IF(5 > 3, \"Yes\", \"No\")");
    assert_eq!(v2, CellValue::Text("Yes".to_string()));

    let v3 = Evaluator::evaluate_formula(&sheet, c, "=IF(2 > 10, \"Yes\", \"No\")");
    assert_eq!(v3, CellValue::Text("No".to_string()));

    let v4 = Evaluator::evaluate_formula(&sheet, c, "=AND(TRUE, 1=1)");
    assert_eq!(v4, CellValue::Bool(true));
}

#[test]
fn test_cell_references_and_aggregations() {
    let mut sheet = Sheet::new("Test");
    sheet.set_cell_input(CellCoord::new(0, 0), "10"); // A1 = 10
    sheet.set_cell_input(CellCoord::new(1, 0), "20"); // A2 = 20
    sheet.set_cell_input(CellCoord::new(2, 0), "30"); // A3 = 30
    sheet.set_cell_input(CellCoord::new(3, 0), "=SUM(A1:A3)"); // A4 = 60
    sheet.set_cell_input(CellCoord::new(4, 0), "=AVERAGE(A1:A3)"); // A5 = 20
    sheet.set_cell_input(CellCoord::new(5, 0), "=MAX(A1:A3)"); // A6 = 30
    sheet.set_cell_input(CellCoord::new(6, 0), "=MIN(A1:A3)"); // A7 = 10

    recalculate_sheet(&mut sheet);

    assert_eq!(sheet.get_cell_value(CellCoord::new(3, 0)), CellValue::Number(60.0));
    assert_eq!(sheet.get_cell_value(CellCoord::new(4, 0)), CellValue::Number(20.0));
    assert_eq!(sheet.get_cell_value(CellCoord::new(5, 0)), CellValue::Number(30.0));
    assert_eq!(sheet.get_cell_value(CellCoord::new(6, 0)), CellValue::Number(10.0));

    // Test incremental calculation when A1 changes
    sheet.set_cell_input(CellCoord::new(0, 0), "100");
    recalculate_incremental(&mut sheet, CellCoord::new(0, 0));

    assert_eq!(sheet.get_cell_value(CellCoord::new(3, 0)), CellValue::Number(150.0));
    assert_eq!(sheet.get_cell_value(CellCoord::new(4, 0)), CellValue::Number(50.0));
}

#[test]
fn test_financial_functions() {
    let sheet = Sheet::new("Financial");
    let c = CellCoord::new(0, 0);

    // PMT: 6% annual rate (0.06 / 12), 360 months, $200,000 principal loan
    // In Excel: =PMT(0.06/12, 360, 200000) -> approximately -$1199.10
    let pmt_val = Evaluator::evaluate_formula(&sheet, c, "=PMT(0.06/12, 360, 200000)");
    if let CellValue::Number(pmt) = pmt_val {
        assert!((pmt - (-1199.101)).abs() < 0.1, "PMT computed: {}", pmt);
    } else {
        panic!("PMT failed: {:?}", pmt_val);
    }

    // NPV: Rate 10%, Cash flows 100, 200, 300
    // NPV = 100/1.1 + 200/1.21 + 300/1.331 = 90.909 + 165.289 + 225.394 = 481.59
    let npv_val = Evaluator::evaluate_formula(&sheet, c, "=NPV(0.10, 100, 200, 300)");
    if let CellValue::Number(npv) = npv_val {
        assert!((npv - 481.59).abs() < 0.5, "NPV computed: {}", npv);
    } else {
        panic!("NPV failed: {:?}", npv_val);
    }
}

#[test]
fn test_lookup_functions() {
    let mut sheet = Sheet::new("Lookup");
    // Column A: ID (101, 102, 103)
    // Column B: Name ("Alice", "Bob", "Charlie")
    // Column C: Salary (50000, 75000, 90000)
    sheet.set_cell_input(CellCoord::new(0, 0), "101");
    sheet.set_cell_input(CellCoord::new(0, 1), "Alice");
    sheet.set_cell_input(CellCoord::new(0, 2), "50000");

    sheet.set_cell_input(CellCoord::new(1, 0), "102");
    sheet.set_cell_input(CellCoord::new(1, 1), "Bob");
    sheet.set_cell_input(CellCoord::new(1, 2), "75000");

    sheet.set_cell_input(CellCoord::new(2, 0), "103");
    sheet.set_cell_input(CellCoord::new(2, 1), "Charlie");
    sheet.set_cell_input(CellCoord::new(2, 2), "90000");

    sheet.set_cell_input(CellCoord::new(4, 0), "=VLOOKUP(102, A1:C3, 2, FALSE)");
    sheet.set_cell_input(CellCoord::new(5, 0), "=VLOOKUP(103, A1:C3, 3, FALSE)");
    sheet.set_cell_input(CellCoord::new(6, 0), "=INDEX(A1:C3, 1, 2)");
    sheet.set_cell_input(CellCoord::new(7, 0), "=MATCH(\"Charlie\", B1:B3, 0)");

    recalculate_sheet(&mut sheet);

    assert_eq!(sheet.get_cell_value(CellCoord::new(4, 0)), CellValue::Text("Bob".to_string()));
    assert_eq!(sheet.get_cell_value(CellCoord::new(5, 0)), CellValue::Number(90000.0));
    assert_eq!(sheet.get_cell_value(CellCoord::new(6, 0)), CellValue::Text("Alice".to_string()));
    assert_eq!(sheet.get_cell_value(CellCoord::new(7, 0)), CellValue::Number(3.0));
}

#[test]
fn test_circular_reference_cycle_detection() {
    let mut sheet = Sheet::new("Cycle");
    // A1 = B1
    // B1 = A1
    sheet.set_cell_input(CellCoord::new(0, 0), "=B1");
    sheet.set_cell_input(CellCoord::new(0, 1), "=A1");

    recalculate_sheet(&mut sheet);

    assert_eq!(
        sheet.get_cell_value(CellCoord::new(0, 0)),
        CellValue::Error(CellError::Circular)
    );
    assert_eq!(
        sheet.get_cell_value(CellCoord::new(0, 1)),
        CellValue::Error(CellError::Circular)
    );
}

#[test]
fn test_xlsx_roundtrip() {
    let mut wb = Workbook::new();
    let sheet = wb.active_sheet_mut();
    sheet.set_cell_input(CellCoord::new(0, 0), "Quarter");
    sheet.set_cell_input(CellCoord::new(0, 1), "Revenue");
    sheet.set_cell_input(CellCoord::new(1, 0), "Q1");
    sheet.set_cell_input(CellCoord::new(1, 1), "15000");
    sheet.set_cell_input(CellCoord::new(2, 0), "Q2");
    sheet.set_cell_input(CellCoord::new(2, 1), "22000");
    sheet.set_cell_input(CellCoord::new(3, 0), "Total");
    sheet.set_cell_input(CellCoord::new(3, 1), "=SUM(B2:B3)");
    recalculate_sheet(sheet);

    let temp_path = std::env::temp_dir().join("jericho_test_roundtrip.xlsx");
    let save_res = save_xlsx(&wb, &temp_path);
    assert!(save_res.is_ok(), "Saving xlsx failed: {:?}", save_res);

    let load_res = load_xlsx(&temp_path);
    assert!(load_res.is_ok(), "Loading xlsx failed: {:?}", load_res);

    let loaded_wb = load_res.unwrap();
    let loaded_sheet = loaded_wb.active_sheet();
    assert_eq!(
        loaded_sheet.get_cell_value(CellCoord::new(0, 0)),
        CellValue::Text("Quarter".to_string())
    );
    assert_eq!(
        loaded_sheet.get_cell_value(CellCoord::new(1, 1)),
        CellValue::Number(15000.0)
    );
    assert_eq!(
        loaded_sheet.get_cell_value(CellCoord::new(2, 1)),
        CellValue::Number(22000.0)
    );

    let _ = std::fs::remove_file(temp_path);
}

#[test]
fn test_csv_roundtrip() {
    let mut sheet = Sheet::new("CSVData");
    sheet.set_cell_input(CellCoord::new(0, 0), "Product");
    sheet.set_cell_input(CellCoord::new(0, 1), "Price");
    sheet.set_cell_input(CellCoord::new(1, 0), "Widget");
    sheet.set_cell_input(CellCoord::new(1, 1), "19.99");
    recalculate_sheet(&mut sheet);

    let temp_path = std::env::temp_dir().join("jericho_test_roundtrip.csv");
    let exp_res = export_csv(&sheet, &temp_path);
    assert!(exp_res.is_ok());

    let imp_res = import_csv(&temp_path);
    assert!(imp_res.is_ok());

    let imported_sheet = imp_res.unwrap();
    assert_eq!(
        imported_sheet.get_cell_value(CellCoord::new(0, 0)),
        CellValue::Text("Product".to_string())
    );
    assert_eq!(
        imported_sheet.get_cell_value(CellCoord::new(1, 1)),
        CellValue::Number(19.99)
    );

    let _ = std::fs::remove_file(temp_path);
}

#[test]
fn test_font_definitions() {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "Inter-Medium".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/fonts/Inter-Medium.ttf"))),
    );
    fonts.font_data.insert(
        "Inter-SemiBold".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/fonts/Inter-SemiBold.ttf"))),
    );
    fonts.font_data.insert(
        "SourceCodePro-Medium".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/fonts/SourceCodePro-Medium.ttf"))),
    );

    fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "Inter-Medium".to_owned());
    fonts.families.insert(
        egui::FontFamily::Name("Bold".into()),
        vec!["Inter-SemiBold".to_owned(), "Inter-Medium".to_owned()],
    );
    fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "SourceCodePro-Medium".to_owned());

    let ctx = egui::Context::default();
    ctx.set_fonts(fonts);
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        let _ = ctx.fonts(|f| f.layout_no_wrap("Test $1,234.56".to_string(), egui::FontId::proportional(14.0), egui::Color32::BLACK));
        let _ = ctx.fonts(|f| f.layout_no_wrap("=SUM(A1:B10)".to_string(), egui::FontId::monospace(14.0), egui::Color32::BLACK));
        let _ = ctx.fonts(|f| f.layout_no_wrap("Header Bold".to_string(), egui::FontId::new(14.0, egui::FontFamily::Name("Bold".into())), egui::Color32::BLACK));
    });
}

#[test]
fn test_extended_math_and_stats() {
    let sheet = Sheet::new("MathStats");
    let c = CellCoord::new(0, 0);

    // Math & Trig
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=POWER(2, 5)"), CellValue::Number(32.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=EXP(0)"), CellValue::Number(1.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=ROUND(PI(), 2)"), CellValue::Number(3.14));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=SIGN(-42)"), CellValue::Number(-1.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=FACT(5)"), CellValue::Number(120.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=COMBIN(5, 2)"), CellValue::Number(10.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=PERMUT(5, 2)"), CellValue::Number(20.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=GCD(12, 18, 24)"), CellValue::Number(6.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=LCM(4, 6)"), CellValue::Number(12.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=EVEN(3)"), CellValue::Number(4.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=ODD(4)"), CellValue::Number(5.0));

    // Stats
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=LARGE(10, 20, 30, 40, 2)"), CellValue::Number(30.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=SMALL(10, 20, 30, 40, 2)"), CellValue::Number(20.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=MODE(1, 2, 2, 3, 4)"), CellValue::Number(2.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=MODE.SNGL(1, 2, 2, 3, 4)"), CellValue::Number(2.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=RANK(30, 10, 20, 30, 40)"), CellValue::Number(2.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=RANK.EQ(30, 10, 20, 30, 40)"), CellValue::Number(2.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=PERCENTILE.INC(10, 20, 30, 40, 0.5)"), CellValue::Number(25.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=QUARTILE.INC(10, 20, 30, 40, 2)"), CellValue::Number(25.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=VAR.S(10, 20, 30)"), CellValue::Number(100.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=STDEV.S(10, 20, 30)"), CellValue::Number(10.0));
}

#[test]
fn test_extended_financial_functions() {
    let sheet = Sheet::new("FinExt");
    let c = CellCoord::new(0, 0);

    // IRR for cash flows: -1000, 300, 420, 680
    // At rate ~ 16.34%, NPV is ~ 0
    let irr_val = Evaluator::evaluate_formula(&sheet, c, "=IRR(-1000, 300, 420, 680)");
    if let CellValue::Number(irr) = irr_val {
        assert!((irr - 0.1634).abs() < 0.005, "IRR was: {}", irr);
    } else {
        panic!("IRR evaluation failed: {:?}", irr_val);
    }

    // Depreciation
    // SLN(cost, salvage, life) -> (10000 - 1000) / 5 = 1800
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=SLN(10000, 1000, 5)"), CellValue::Number(1800.0));
    // SYD(cost, salvage, life, per) -> per 1: 9000 * 5 / 15 = 3000
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=SYD(10000, 1000, 5, 1)"), CellValue::Number(3000.0));

    // EFFECT & NOMINAL
    let eff = Evaluator::evaluate_formula(&sheet, c, "=EFFECT(0.06, 12)");
    if let CellValue::Number(e) = eff {
        assert!((e - 0.061678).abs() < 0.0001, "EFFECT was: {}", e);
    } else {
        panic!("EFFECT evaluation failed: {:?}", eff);
    }
}

#[test]
fn test_multicriteria_and_reference() {
    let mut sheet = Sheet::new("MultiCrit");
    // Setup data table:
    // A: Category, B: Department, C: Amount
    // 1: Category   Dept   Amount
    // 2: Food       East   100
    // 3: Food       West   150
    // 4: Drink      East   200
    // 5: Food       East   250
    sheet.set_cell_input(CellCoord::new(1, 0), "Food");
    sheet.set_cell_input(CellCoord::new(1, 1), "East");
    sheet.set_cell_input(CellCoord::new(1, 2), "100");

    sheet.set_cell_input(CellCoord::new(2, 0), "Food");
    sheet.set_cell_input(CellCoord::new(2, 1), "West");
    sheet.set_cell_input(CellCoord::new(2, 2), "150");

    sheet.set_cell_input(CellCoord::new(3, 0), "Drink");
    sheet.set_cell_input(CellCoord::new(3, 1), "East");
    sheet.set_cell_input(CellCoord::new(3, 2), "200");

    sheet.set_cell_input(CellCoord::new(4, 0), "Food");
    sheet.set_cell_input(CellCoord::new(4, 1), "East");
    sheet.set_cell_input(CellCoord::new(4, 2), "250");

    recalculate_sheet(&mut sheet);

    let c = CellCoord::new(10, 0);

    // SUMIFS(sum_range, crit_range1, crit1, crit_range2, crit2)
    // Sum C2:C5 where A2:A5 = "Food" AND B2:B5 = "East" -> C2 (100) + C5 (250) = 350
    let sumifs_val = Evaluator::evaluate_formula(&sheet, c, "=SUMIFS(C2:C5, A2:A5, \"Food\", B2:B5, \"East\")");
    assert_eq!(sumifs_val, CellValue::Number(350.0));

    // COUNTIFS
    let countifs_val = Evaluator::evaluate_formula(&sheet, c, "=COUNTIFS(A2:A5, \"Food\", B2:B5, \"East\")");
    assert_eq!(countifs_val, CellValue::Number(2.0));

    // AVERAGEIFS -> (100 + 250) / 2 = 175
    let avgifs_val = Evaluator::evaluate_formula(&sheet, c, "=AVERAGEIFS(C2:C5, A2:A5, \"Food\", B2:B5, \"East\")");
    assert_eq!(avgifs_val, CellValue::Number(175.0));

    // MAXIFS / MINIFS
    let maxifs_val = Evaluator::evaluate_formula(&sheet, c, "=MAXIFS(C2:C5, A2:A5, \"Food\", B2:B5, \"East\")");
    assert_eq!(maxifs_val, CellValue::Number(250.0));
    let minifs_val = Evaluator::evaluate_formula(&sheet, c, "=MINIFS(C2:C5, A2:A5, \"Food\", B2:B5, \"East\")");
    assert_eq!(minifs_val, CellValue::Number(100.0));

    // CHOOSE & SWITCH
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=CHOOSE(2, \"Alpha\", \"Beta\", \"Gamma\")"), CellValue::Text("Beta".to_string()));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=SWITCH(3, 1, \"One\", 2, \"Two\", 3, \"Three\", \"Other\")"), CellValue::Text("Three".to_string()));

    // ROWS & COLUMNS
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=ROWS(A2:C5)"), CellValue::Number(4.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=COLUMNS(A2:C5)"), CellValue::Number(3.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=ADDRESS(5, 3)"), CellValue::Text("$C$5".to_string()));
}

#[test]
fn test_extended_text_and_information() {
    let sheet = Sheet::new("TextInfo");
    let c = CellCoord::new(0, 0);

    // Text functions
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=TEXTJOIN(\", \", TRUE, \"Apple\", \"Banana\", \"Cherry\")"), CellValue::Text("Apple, Banana, Cherry".to_string()));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=REPLACE(\"Jericho\", 1, 4, \"New \")"), CellValue::Text("New cho".to_string()));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=SUBSTITUTE(\"2023-01-01\", \"-\", \"/\")"), CellValue::Text("2023/01/01".to_string()));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=REPT(\"*\", 5)"), CellValue::Text("*****".to_string()));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=FIND(\"c\", \"Jericho\")"), CellValue::Number(5.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=SEARCH(\"JERICHO\", \"jericho\")"), CellValue::Number(1.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=EXACT(\"Test\", \"test\")"), CellValue::Bool(false));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=CHAR(65)"), CellValue::Text("A".to_string()));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=CODE(\"Apple\")"), CellValue::Number(65.0));

    // Information functions
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=ISNUMBER(123.45)"), CellValue::Bool(true));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=ISTEXT(\"Hello\")"), CellValue::Bool(true));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=ISLOGICAL(TRUE)"), CellValue::Bool(true));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=ISERROR(1/0)"), CellValue::Bool(true));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=TYPE(42)"), CellValue::Number(1.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=TYPE(\"hello\")"), CellValue::Number(2.0));
}

#[test]
fn test_extended_date_and_time() {
    let sheet = Sheet::new("DateTime");
    let c = CellCoord::new(0, 0);

    // DATE(2023, 10, 5) -> Days between 1899-12-30 and 2023-10-05 = 45204
    let d_val = Evaluator::evaluate_formula(&sheet, c, "=DATE(2023, 10, 5)");
    assert_eq!(d_val, CellValue::Number(45204.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=YEAR(45204)"), CellValue::Number(2023.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=MONTH(45204)"), CellValue::Number(10.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=DAY(45204)"), CellValue::Number(5.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=DAYS(DATE(2023, 10, 15), DATE(2023, 10, 5))"), CellValue::Number(10.0));

    // EDATE & EOMONTH
    let edate_val = Evaluator::evaluate_formula(&sheet, c, "=EDATE(DATE(2023, 1, 15), 3)");
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, &format!("=MONTH({})", edate_val.as_number().unwrap())), CellValue::Number(4.0));

    let eomonth_val = Evaluator::evaluate_formula(&sheet, c, "=EOMONTH(DATE(2023, 2, 1), 0)");
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, &format!("=DAY({})", eomonth_val.as_number().unwrap())), CellValue::Number(28.0));

    // DATEDIF
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=DATEDIF(DATE(2020, 1, 1), DATE(2023, 1, 1), \"Y\")"), CellValue::Number(3.0));
    assert_eq!(Evaluator::evaluate_formula(&sheet, c, "=DATEDIF(DATE(2023, 1, 1), DATE(2023, 6, 1), \"M\")"), CellValue::Number(5.0));
}

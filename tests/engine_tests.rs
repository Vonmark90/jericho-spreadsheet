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

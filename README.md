# Jericho Spreadsheet 📊

> **Fast, Lightweight, Native Desktop Spreadsheet with Industry-Standard Excel Compatibility**

Jericho Spreadsheet is an original, ultra-optimized spreadsheet application built from scratch in **Rust**. It is designed specifically for professionals in finance, accounting, data analysis, and engineering who rely on Microsoft Excel's industry-standard conventions, formulas, and performance—while remaining so lightweight and memory-efficient that it runs seamlessly on resource-constrained hardware where modern heavy spreadsheet suites struggle.

---

## ✨ Key Features & Capabilities

### ⚡ Blazing Performance & Low Footprint
- **Virtualized 2D Grid**: Viewport culling renders only the cells currently visible on screen. Smooth **60 FPS** scrolling even across **100,000 rows × 200 columns**.
- **Sparse Memory Storage**: Unallocated cells take zero space; only active data is stored. Typical memory footprint is **< 35 MB RAM**.
- **Immediate-Mode GPU Rendering**: Native window with hardware acceleration (Metal / OpenGL) via `eframe` / `egui`.
- **Zero Heavy Web Runtimes**: Pure native machine code without the bloat, battery drain, or memory leaks of Electron.

### 📐 Real Reactive Formula Engine (DAG)
- **Excel-Standard Syntax**: Formulas start with `=` (e.g., `=SUM(B6:B7)`, `=NPV(B28, B23:F23)`, `=IF(A1>0, "Profit", "Loss")`).
- **Reactive Dependency Graph**: Cells construct a Directed Acyclic Graph (DAG); editing a cell triggers fast topological recalculation only for its transitive dependents.
- **Cycle Detection**: Circular references (e.g. `A1 = =B1`, `B1 = =A1`) are detected via 3-color graph traversal and flagged as `#CIRCULAR!` without freezing or crashing.
- **Error Codes**: Full Excel error model (`#DIV/0!`, `#VALUE!`, `#REF!`, `#NAME?`, `#NUM!`, `#N/A`, `#CIRCULAR!`).

### 💼 Comprehensive Function Library
- **Financial**: `PMT`, `PV`, `FV`, `NPV`, `NPER` (loan payments, DCF modeling, valuations).
- **Lookup & Reference**: `VLOOKUP`, `XLOOKUP`, `INDEX`, `MATCH`, `ROW`, `COLUMN`.
- **Math & Statistics**: `SUM`, `AVERAGE`, `MIN`, `MAX`, `COUNT`, `COUNTA`, `PRODUCT`, `ABS`, `SQRT`, `ROUND`, `ROUNDUP`, `ROUNDDOWN`, `FLOOR`, `CEILING`, `MOD`, `POWER`, `INT`, `MEDIAN`, `STDEV`, `STDEVP`, `SUMIF`, `COUNTIF`.
- **Logic**: `IF`, `IFS`, `AND`, `OR`, `NOT`, `XOR`, `IFERROR`.
- **Text**: `CONCATENATE`, `CONCAT`, `&`, `LEFT`, `RIGHT`, `MID`, `LEN`, `TRIM`, `UPPER`, `LOWER`, `PROPER`.
- **Date & Time**: `TODAY`, `NOW`, `DATE`, `YEAR`, `MONTH`, `DAY`.

### 📂 Native Excel (.xlsx) & CSV Compatibility
- **Import XLSX**: Open existing Microsoft Excel workbooks (`.xlsx`), reading multi-sheet tabs, formulas, strings, numbers, dates, and formatting using `calamine`.
- **Export XLSX**: Save full workbooks directly into `.xlsx` using `rust_xlsxwriter`, preserving formulas, cell numbers, styles (Currency, Percent, Bold, Italic), background colors, and custom column widths.
- **CSV Support**: Fast import and export of comma-separated files.

### 🎯 Professional Excel User Interface
- **Excel Ribbon Toolbar**:
  - **File**: New, Open XLSX/CSV, Save XLSX, Save As, Export CSV, Load 5-Year Financial Model Template.
  - **Home**: Undo/Redo, Font Styles (Bold, Italic, Underline), Alignments (Left, Center, Right), Number Formatting (Currency `$#,##0.00`, Percentage `0.0%`, Number `1,234.56`, Decimal places `+ / -`), Cell Fill Colors, Text Colors, AutoSum (`Σ`), Clear.
  - **Formulas**: Quick formula insert library (`SUM`, `AVERAGE`, `VLOOKUP`, `PMT`, `NPV`...).
  - **Data**: Sort Ascending (A→Z), Sort Descending (Z→A), Insert/Delete Rows & Columns.
  - **Theme Toggle**: Instant switch between **Excel Modern Light** and **Executive Dark** modes.
- **Excel Formula Bar**:
  - Live Name Box (shows active cell coordinate e.g. `B6` or selection range e.g. `B6:E10`; jump directly to coordinates by typing).
  - `fx` button for quick formula creation.
  - Live formula input synchronized with active cell.
- **Excel Status Bar**:
  - Live aggregates computed in real time for selected ranges:
    - **Average**
    - **Count**
    - **Numerical Count**
    - **Min**
    - **Max**
    - **Sum**
- **Dynamic Row & Column Resizing**:
  - Interactive mouse boundary dragging on column letters (`↔`) and row numbers (`↕`) with real-time green alignment guidelines.
  - Double-click column divider to AutoFit width based on longest cell content.
  - Double-click row divider to reset height.
  - Ribbon Data quick-actions for AutoFit Column and Reset Row Height.
  - Prefix-sum `GridLayout` engine with binary search indexing for 60fps rendering across 100,000 rows.
- **Sleek Embedded Typography**:
  - Embedded modern typefaces: **Inter-Medium** (500 weight) for crisp, readable UI and cell text, **Inter-SemiBold** (600 weight) for headers and bold figures, and **SourceCodePro-Medium** for formula editing and monospace numbers.
  - Zero system font configuration required; looks identical and razor-sharp across all machines.
- **Autofill Handle**:
  - The classic Excel green square at the bottom-right of cell selections.
  - Drag down or right to auto-fill formulas (automatically adjusting relative references, e.g. `=B6*1.15` becomes `=B7*1.15`) or linear number series!
- **Multi-Sheet Tabs Bar**:
  - Add sheets (`＋`), switch tabs, double-click or right-click to rename, and delete sheets.
- **Keyboard Shortcuts**:
  - `Arrow Keys`: Move active cursor
  - `Shift + Arrow Keys`: Expand rectangular selection
  - `Enter` / `Shift + Enter`: Move down / up
  - `Tab` / `Shift + Tab`: Move right / left
  - `F2` or `Double-Click`: Enter in-cell edit mode
  - `Delete` / `Backspace`: Clear selected cells
  - `Cmd/Ctrl + C`: Copy selection as TSV (pasteable directly into Excel/Google Sheets)
  - `Cmd/Ctrl + V`: Paste TSV from clipboard
  - `Cmd/Ctrl + Z`: Undo
  - `Cmd/Ctrl + Y` (or `Cmd/Ctrl + Shift + Z`): Redo
  - `Cmd/Ctrl + B`: Toggle Bold
  - `Cmd/Ctrl + I`: Toggle Italic
  - `Cmd/Ctrl + U`: Toggle Underline

---

## 🏗️ Architecture & Project Structure

```
jericho_spreadsheet/
├── Cargo.toml                  # High-performance dependencies and release LTO profile
├── src/
│   ├── main.rs                 # Native window initialization & eframe entry point
│   ├── lib.rs                  # Library entry point for testing and reuse
│   ├── app.rs                  # JerichoApp coordinating ribbon, grid, formula bar, tabs, and status bar
│   ├── model/
│   │   ├── cell.rs             # CellCoord, CellValue, CellFormat, CellError, CellData
│   │   ├── selection.rs        # SelectionRange, cursor navigation, autofill tracking
│   │   ├── sheet.rs            # Sheet state, sparse cell map, row/col dimensions, undo/redo
│   │   └── workbook.rs         # Multi-sheet Workbook, sheet manager, 5-Yr Financial Model template
│   ├── engine/
│   │   ├── parser.rs           # Lexer & recursive descent parser for formula expressions
│   │   ├── functions.rs        # Math, Finance, Lookup, Logic, Text, Date library
│   │   ├── evaluator.rs        # AST evaluation, range expansion, VLOOKUP, INDEX, MATCH
│   │   └── dependency.rs       # Reactive DAG, topological sort, circular reference detection
│   ├── io/
│   │   ├── xlsx.rs             # Excel .xlsx import (calamine) and export (rust_xlsxwriter)
│   │   ├── csv.rs              # CSV import and export
│   │   └── json.rs             # JSON workbook backup & restore
│   └── ui/
│       ├── theme.rs            # Light & Dark Excel theme color tokens
│       ├── ribbon.rs           # Multi-tab ribbon toolbar
│       ├── formula_bar.rs      # Name Box and formula editor
│       ├── grid.rs             # 60 FPS Virtualized 2D canvas grid view with in-cell editor
│       ├── tabs.rs             # Multi-sheet tab switcher
│       └── status_bar.rs       # Live status aggregates (Sum, Avg, Count, Min, Max)
└── tests/
    └── engine_tests.rs         # Integration tests for formulas, DAG cycles, XLSX, and CSV
```

---

## 🚀 Getting Started

### Prerequisites
- [Rust toolchain](https://www.rust-lang.org/) (`cargo`, `rustc`)

### Running the Application
Launch the standalone native desktop application:
```bash
cargo run --release
```

### Running the Test Suite
Execute the formula engine, DAG, and file I/O tests:
```bash
cargo test
```

### Building Optimized Binary
Generate a stripped, optimized standalone binary:
```bash
cargo build --release
```
The compiled executable will be located at `target/release/jericho_spreadsheet`.

### Packaging as macOS Desktop Application (.app)
Package the application with custom Retina icon and metadata into `Jericho Spreadsheet.app` and install directly to the Desktop:
```bash
./scripts/build_app_bundle.sh
```
Or open the desktop application directly:
```bash
open "/Users/markrsadler/Desktop/Jericho Spreadsheet.app"
```

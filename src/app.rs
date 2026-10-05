use crate::engine::dependency::recalculate_sheet;
use crate::model::workbook::Workbook;
use crate::ui::formula_bar::{render_formula_bar, FormulaBarState};
use crate::ui::grid::{render_grid, GridState};
use crate::ui::ribbon::{render_ribbon, RibbonState};
use crate::ui::status_bar::render_status_bar;
use crate::ui::tabs::{render_tabs, TabsState};
use crate::ui::theme::{SpreadsheetTheme, ThemeMode};
use eframe::App;
use egui::{CentralPanel, Context, TopBottomPanel, Vec2};

pub struct JerichoApp {
    pub workbook: Workbook,
    pub theme: SpreadsheetTheme,
    pub ribbon_state: RibbonState,
    pub formula_bar_state: FormulaBarState,
    pub grid_state: GridState,
    pub tabs_state: TabsState,
}

impl Default for JerichoApp {
    fn default() -> Self {
        let mut wb = Workbook::create_sample_financial_model();
        recalculate_sheet(wb.active_sheet_mut());
        Self {
            workbook: wb,
            theme: SpreadsheetTheme::light(),
            ribbon_state: RibbonState::default(),
            formula_bar_state: FormulaBarState::default(),
            grid_state: GridState::default(),
            tabs_state: TabsState::default(),
        }
    }
}

impl JerichoApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::configure_fonts(&cc.egui_ctx);
        Self::default()
    }

    pub fn configure_fonts(ctx: &Context) {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "Inter-Medium".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/fonts/Inter-Medium.ttf"
            ))),
        );
        fonts.font_data.insert(
            "Inter-SemiBold".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/fonts/Inter-SemiBold.ttf"
            ))),
        );
        fonts.font_data.insert(
            "SourceCodePro-Medium".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/fonts/SourceCodePro-Medium.ttf"
            ))),
        );

        // Primary proportional: Inter-Medium (crisp, modern, medium weight / "somebold")
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Inter-Medium".to_owned());

        // Dedicated Bold family: Inter-SemiBold
        fonts.families.insert(
            egui::FontFamily::Name("Bold".into()),
            vec!["Inter-SemiBold".to_owned(), "Inter-Medium".to_owned()],
        );

        // Monospace family: SourceCodePro-Medium
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "SourceCodePro-Medium".to_owned());

        ctx.set_fonts(fonts);

        // Configure typography styles
        ctx.style_mut(|s| {
            s.text_styles = [
                (
                    egui::TextStyle::Heading,
                    egui::FontId::new(17.5, egui::FontFamily::Name("Bold".into())),
                ),
                (
                    egui::TextStyle::Body,
                    egui::FontId::new(14.0, egui::FontFamily::Proportional),
                ),
                (
                    egui::TextStyle::Button,
                    egui::FontId::new(13.0, egui::FontFamily::Proportional),
                ),
                (
                    egui::TextStyle::Monospace,
                    egui::FontId::new(14.0, egui::FontFamily::Monospace),
                ),
                (
                    egui::TextStyle::Small,
                    egui::FontId::new(12.0, egui::FontFamily::Proportional),
                ),
            ]
            .into();
        });
    }
}

impl App for JerichoApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Set egui theme visuals
        let mut visuals = if self.theme.mode == ThemeMode::Dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.override_text_color = Some(self.theme.cell_text);
        visuals.panel_fill = self.theme.ribbon_bg;
        visuals.window_fill = self.theme.cell_bg;
        ctx.set_visuals(visuals);

        // --- Top Ribbon Panel ---
        TopBottomPanel::top("top_ribbon_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
                render_ribbon(ui, &mut self.theme, &mut self.ribbon_state, &mut self.workbook);
            });

        // --- Formula Bar Panel ---
        TopBottomPanel::top("formula_bar_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(4.0, 2.0);
                render_formula_bar(
                    ui,
                    &self.theme,
                    &mut self.formula_bar_state,
                    &mut self.workbook,
                );
            });

        // --- Bottom Status Bar Panel ---
        TopBottomPanel::bottom("status_bar_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.style_mut().visuals.panel_fill = self.theme.status_bar_bg;
                render_status_bar(
                    ui,
                    &self.theme,
                    &self.workbook,
                    self.grid_state.is_cell_editing || self.formula_bar_state.is_editing,
                    self.ribbon_state.status_message.as_ref(),
                );
            });

        // --- Bottom Sheet Tabs Panel ---
        TopBottomPanel::bottom("sheet_tabs_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.style_mut().visuals.panel_fill = self.theme.tab_bar_bg;
                render_tabs(
                    ui,
                    &self.theme,
                    &mut self.tabs_state,
                    &mut self.workbook,
                );
            });

        // --- Main Central Spreadsheet Grid ---
        CentralPanel::default()
            .frame(egui::Frame::NONE.fill(self.theme.cell_bg))
            .show(ctx, |ui| {
                render_grid(ui, &self.theme, &mut self.grid_state, &mut self.workbook);
            });
    }
}

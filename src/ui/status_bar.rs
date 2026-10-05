use super::theme::SpreadsheetTheme;
use crate::model::workbook::Workbook;
use egui::{RichText, Ui, Vec2};

pub fn render_status_bar(
    ui: &mut Ui,
    theme: &SpreadsheetTheme,
    wb: &Workbook,
    is_editing: bool,
    status_msg: Option<&(String, std::time::Instant)>,
) {
    let sheet = wb.active_sheet();
    let selection = &sheet.selection;
    let aggregates = sheet.calculate_range_aggregates(selection.range);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(12.0, 0.0);

        // --- Mode Pill ---
        let mode_text = if is_editing { "EDIT" } else { "READY" };
        let mode_color = if is_editing {
            egui::Color32::from_rgb(220, 130, 20)
        } else {
            theme.ribbon_accent
        };
        ui.label(
            RichText::new(mode_text)
                .font(egui::FontId::new(12.0, egui::FontFamily::Name("Bold".into())))
                .color(mode_color),
        );

        // --- Status Message (if any within last 5 seconds) ---
        if let Some((msg, instant)) = status_msg {
            if instant.elapsed().as_secs() < 5 {
                ui.label(
                    RichText::new(msg)
                        .font(egui::FontId::new(12.0, egui::FontFamily::Proportional))
                        .italics()
                        .color(theme.status_bar_fg),
                );
            }
        }

        // --- Real-time Excel Aggregates on the right ---
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(14.0, 0.0);

            // Active cell
            ui.label(
                RichText::new(format!("Cell: {}", selection.cursor.to_a1()))
                    .font(egui::FontId::new(12.0, egui::FontFamily::Name("Bold".into())))
                    .color(theme.status_bar_fg),
            );

            // Only show aggregates if more than 1 cell is selected or non-empty
            if selection.range.cell_count() > 1 || aggregates.non_empty_count > 0 {
                if let Some(sum) = aggregates.sum {
                    ui.label(
                        RichText::new(format!("Sum: {}", format_compact(sum)))
                            .font(egui::FontId::new(12.0, egui::FontFamily::Name("Bold".into())))
                            .color(theme.status_bar_fg),
                    );
                }

                if let Some(max) = aggregates.max {
                    ui.label(
                        RichText::new(format!("Max: {}", format_compact(max)))
                            .font(egui::FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(theme.status_bar_fg),
                    );
                }

                if let Some(min) = aggregates.min {
                    ui.label(
                        RichText::new(format!("Min: {}", format_compact(min)))
                            .font(egui::FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(theme.status_bar_fg),
                    );
                }

                if aggregates.num_count > 0 {
                    ui.label(
                        RichText::new(format!("Numerical Count: {}", aggregates.num_count))
                            .font(egui::FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(theme.status_bar_fg),
                    );
                }

                if aggregates.non_empty_count > 0 {
                    ui.label(
                        RichText::new(format!("Count: {}", aggregates.non_empty_count))
                            .font(egui::FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(theme.status_bar_fg),
                    );
                }

                if let Some(avg) = aggregates.average {
                    ui.label(
                        RichText::new(format!("Average: {}", format_compact(avg)))
                            .font(egui::FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(theme.status_bar_fg),
                    );
                }
            }
        });
    });
}

fn format_compact(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e12 {
        insert_commas(&(n as i64).to_string())
    } else {
        let s = format!("{:.2}", n);
        let parts: Vec<&str> = s.split('.').collect();
        format!("{}.{}", insert_commas(parts[0]), parts[1])
    }
}

fn insert_commas(s: &str) -> String {
    let is_neg = s.starts_with('-');
    let raw = if is_neg { &s[1..] } else { s };
    let mut result = String::new();
    let len = raw.len();
    for (i, c) in raw.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    if is_neg {
        format!("-{}", result)
    } else {
        result
    }
}

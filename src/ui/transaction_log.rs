// transaction_log.rs
use eframe::{egui, App};
use egui::{Color32, Label, Rect, ScrollArea, StrokeKind};
use crate::modmanager::SyncReport;

pub struct TransactionLogApp {
    report: SyncReport,
    scroll_offsets: [usize; 4], // track how many items to skip per column
    human_readable: bool,
}

impl TransactionLogApp {
    pub fn new(report: SyncReport) -> Self {
        Self {
            report,
            scroll_offsets: [0; 4],
            human_readable: true,
        }
    }

    fn display_filename(&self, filename: &str) -> String {
        if !self.human_readable {
            filename.to_string()
        } else {
            let parts: Vec<&str> = filename.split('-').collect();
            let mut result = String::new();

            for (i, part) in parts.iter().enumerate() {
                // Check if this part starts with a digit
                if let Some(first_char) = part.chars().next() {
                    if first_char.is_digit(10) {
                        // Part starts with a digit, stop here
                        break;
                    }
                }

                // Add the part to result
                if i > 0 {
                    result.push('-');
                }
                result.push_str(part);
            }

            result
        }
    }

    fn draw_transaction_log(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut column_rects: Vec<(Rect, Color32)> = vec![];

            let hrn_height = 10.0;
            let hrn_spacing = 10.0;
            let button_height = 40.0;
            let button_spacing = 20.0;
            let header_height = 80.0;

            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("Transaction Log")
                        .size(24.0)
                        .color(Color32::from_rgb(0x2F, 0x36, 0x99))
                );
                ui.add_space(25.0);

                let available_width = ui.available_width();
                let available_height = ui.available_height() - button_height - button_spacing * 2.0 - header_height;
                let column_height = available_height.max(80.0);
                let column_width = available_width / 4.0;
                let item_height = 20.0;

                // Create 4 columns for the transaction log
                ui.columns(4, |columns| {
                    let categories = ["Downloaded", "Unchanged", "Removed", "Failed"];
                    let symbols = ["+", "~", "-", "!"];
                    let colors = [
                        Color32::from_rgb(0x00, 0xFF, 0x00),
                        Color32::from_rgb(0xFF, 0xFF, 0x00),
                        Color32::from_rgb(0xFF, 0xA5, 0x00),
                        Color32::from_rgb(0xFF, 0x00, 0x00),
                    ];

                    for (i, column) in columns.iter_mut().enumerate() {
                        column.vertical(|ui| {
                            ui.add_space(15.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    egui::RichText::new(categories[i])
                                        .color(colors[i])
                                        .size(16.0),
                                );
                            });
                            ui.add_space(5.0);
                            ui.separator();
                            ui.add_space(5.0);

                            // Total items and how many fit
                            let total_items = match i {
                                0 => self.report.downloaded.len(),
                                1 => self.report.unchanged.len(),
                                2 => self.report.removed.len(),
                                3 => self.report.failed.len(),
                                _ => 0,
                            };
                            let items_fit = (column_height / item_height).floor() as usize;

                            // Auto scroll back if more space is available
                            if self.scroll_offsets[i] + items_fit > total_items {
                                if total_items > items_fit {
                                    self.scroll_offsets[i] = total_items - items_fit;
                                } else {
                                    self.scroll_offsets[i] = 0;
                                }
                            }

                            // Determine visible range
                            let mut visible_range_start = self.scroll_offsets[i];
                            let mut visible_range_end = (self.scroll_offsets[i] + items_fit).min(total_items);

                            let hide_top = visible_range_start > 0;
                            let hide_bottom = visible_range_end < total_items;

                            // Adjust range for "..."
                            if hide_top && visible_range_end > visible_range_start {
                                visible_range_start += 1; // reserve first visible line for "..."
                            }
                            if hide_bottom && visible_range_end > visible_range_start {
                                visible_range_end -= 1; // reserve last visible line for "..."
                            }

                            ScrollArea::vertical()
                                .max_height(column_height)
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    // Draw visible items
                                    match i {
                                        0 => {
                                            if hide_top {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new("...").color(colors[i])).truncate());
                                                });
                                            }
                                            for entry in self.report.downloaded.iter().skip(visible_range_start).take(visible_range_end - visible_range_start) {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new(symbols[i]).color(colors[i])).truncate());
                                                    ui.add_space(5.0);
                                                    ui.add(Label::new(self.display_filename(&*entry.filename)).truncate());
                                                });
                                            }
                                            if hide_bottom {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new("...").color(colors[i])).truncate());
                                                });
                                            }
                                            if self.report.downloaded.is_empty() {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new("None").color(Color32::from_gray(136)).size(12.0)).truncate());
                                                });
                                            }
                                        }
                                        1 => {
                                            if hide_top {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new("...").color(colors[i])).truncate());
                                                });
                                            }
                                            for entry in self.report.unchanged.iter().skip(visible_range_start).take(visible_range_end - visible_range_start) {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new(symbols[i]).color(colors[i])).truncate());
                                                    ui.add_space(5.0);
                                                    ui.add(Label::new(self.display_filename(&*entry.filename)).truncate());
                                                });
                                            }
                                            if hide_bottom {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new("...").color(colors[i])).truncate());
                                                });
                                            }
                                            if self.report.unchanged.is_empty() {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new("None").color(Color32::from_gray(136)).size(12.0)).truncate());
                                                });
                                            }
                                        }
                                        2 => {
                                            if hide_top {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new("...").color(colors[i])).truncate());
                                                });
                                            }
                                            for entry in self.report.removed.iter().skip(visible_range_start).take(visible_range_end - visible_range_start) {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new(symbols[i]).color(colors[i])).truncate());
                                                    ui.add_space(5.0);
                                                    ui.add(Label::new(self.display_filename(&*entry.filename)).truncate());
                                                });
                                            }
                                            if hide_bottom {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new("...").color(colors[i])).truncate());
                                                });
                                            }
                                            if self.report.removed.is_empty() {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new("None").color(Color32::from_gray(136)).size(12.0)).truncate());
                                                });
                                            }
                                        }
                                        3 => {
                                            if hide_top {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new("...").color(colors[i])).truncate());
                                                });
                                            }
                                            for (entry, error) in self.report.failed.iter().skip(visible_range_start).take(visible_range_end - visible_range_start) {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new(symbols[i]).color(colors[i])).truncate());
                                                    ui.add_space(5.0);
                                                    ui.add(Label::new(format!("{}: {}", self.display_filename(&*entry.filename), error)).truncate());
                                                });
                                            }
                                            if hide_bottom {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new("...").color(colors[i])).truncate());
                                                });
                                            }
                                            if self.report.failed.is_empty() {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(10.0);
                                                    ui.add(Label::new(egui::RichText::new("None").color(Color32::from_gray(136)).size(12.0)).truncate());
                                                });
                                            }
                                        }
                                        _ => unreachable!(),
                                    }
                                });
                        });
                        column_rects.push((column.min_rect(), colors[i]));
                    }
                });

                // Determine which column the scroll was meant for
                let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
                if scroll_delta != 0.0 {
                    if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                        for (i, (rect, _color)) in column_rects.iter().enumerate() {
                            if rect.contains(pos) {
                                // Total items and how many fit in this column
                                let total_items = match i {
                                    0 => self.report.downloaded.len(),
                                    1 => self.report.unchanged.len(),
                                    2 => self.report.removed.len(),
                                    3 => self.report.failed.len(),
                                    _ => 0,
                                };
                                let items_fit = (column_height / item_height).floor() as usize;

                                if total_items > items_fit {
                                    if scroll_delta < 0.0 {
                                        self.scroll_offsets[i] = (self.scroll_offsets[i] + 1).min(total_items - items_fit);
                                    } else if scroll_delta > 0.0 {
                                        self.scroll_offsets[i] = self.scroll_offsets[i].saturating_sub(1);
                                    }
                                }
                                break; // Only scroll one column
                            }
                        }
                    }
                }


                // Button at bottom
                ui.add_space(ui.available_height() - button_height - button_spacing * 2.0);

                if ui
                    .add(egui::Label::new(
                        egui::RichText::new(if self.human_readable { "Human Readable Names: ON" } else { "Human Readable Names: OFF" })
                            .color(Color32::from_gray(180))
                            .underline(),
                    ).sense(egui::Sense::click()))
                    .clicked()
                {
                    self.human_readable = !self.human_readable;
                }

                ui.add_space(10.0);


                let button_response = ui.vertical_centered(|ui| {
                    let button = egui::Button::new(
                        egui::RichText::new("Launch Game")
                            .size(18.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                        .min_size(egui::vec2(180.0, button_height))
                        .fill(Color32::from_rgb(0x10, 0x10, 0x10))
                        .stroke(egui::Stroke::new(2.0, Color32::from_rgb(0x00, 0xFF, 0x00)));

                    let response = ui.add(button);
                    if response.clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    response
                });

                // Draw outlines and separators after content
                for (i, r) in column_rects.iter().enumerate() {
                    let (column_rect, color) = r;
                    let mut rect = column_rect.clone();
                    rect.set_bottom(button_response.response.rect.top() - button_spacing * 2.0);

                    let stroke = egui::Stroke::new(1.5, *color);
                    ui.painter().rect_stroke(rect, 0.0, stroke, StrokeKind::Middle);
                }
            });
        });

        ctx.request_repaint();
    }
}

impl App for TransactionLogApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw_transaction_log(ctx);
    }
}

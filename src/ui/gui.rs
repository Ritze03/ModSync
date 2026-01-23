// modsync_app.rs
use eframe::{egui, App};
use std::sync::Arc;
use std::time::Instant;
use egui::{Direction, Vec2, ColorImage, TextureHandle};
use tokio::sync::mpsc::UnboundedReceiver;
use crate::modmanager::{SyncProgress, SyncEvent, SyncReport};
use crate::ui::theme::{setup_dark_theme, setup_fonts};

pub struct ModSyncApp {
    progress: Arc<SyncProgress>,
    events: UnboundedReceiver<SyncEvent>,

    // Splash / timeout state
    splash_finished: bool,
    splash_start: Option<Instant>,
    splash_timeout_secs: f32,

    // New state to track if we should launch transaction log window
    has_changes: bool,
    transaction_report: Option<SyncReport>,
    show_transaction_log: bool,
    report_sender: std::sync::mpsc::Sender<SyncReport>,

    // Logo image
    logo_texture: Option<TextureHandle>,
}

impl ModSyncApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        progress: Arc<SyncProgress>,
        events: UnboundedReceiver<SyncEvent>,
        timeout_secs: u64,
        report_sender: std::sync::mpsc::Sender<SyncReport>,
    ) -> Self {
        setup_fonts(&cc.egui_ctx);
        setup_dark_theme(&cc.egui_ctx);

        let logo_texture = load_logo(&cc.egui_ctx, 100, 100);

        Self {
            progress,
            events,
            splash_finished: false,
            splash_start: None,
            splash_timeout_secs: timeout_secs as f32,
            show_transaction_log: false,
            has_changes: false,
            transaction_report: None,
            logo_texture,
            report_sender, // Add this
        }
    }

    pub fn draw_splash(&mut self, ui: &mut egui::Ui) {
        // Drain events
        while let Ok(event) = self.events.try_recv() {
            match event {
                SyncEvent::Finished(report) => {
                    if !self.splash_finished {
                        self.splash_finished = true;
                        self.splash_start = Some(Instant::now());

                        // Store the report
                        self.transaction_report = Some(report.clone());

                        // Check if there were any changes
                        self.has_changes = !report.downloaded.is_empty()
                            || !report.removed.is_empty()
                            || !report.failed.is_empty();
                    }
                }
                // Handle other events if needed
                _ => {}
            }
        }

        ui.vertical_centered(|ui| {
            // Logo
            if let Some(texture) = &self.logo_texture {
                ui.image(texture);
            } else {
                ui.add_space(25.0);
                ui.label(
                    egui::RichText::new("ModSync")
                        .size(50.0)
                        .color(egui::Color32::from_rgb(0xF0, 0xF0, 0xF0)),
                );
                ui.add_space(25.0);
            }

            ui.add_space(20.0);

            use egui::Layout;

            let status_numbers_margin = 20.0;
            let status_numbers_size =
                Vec2::new(ui.available_width() - (status_numbers_margin * 2.0), 60.0);

            ui.allocate_ui_with_layout(
                status_numbers_size,
                Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    let stats = self.progress.stats();

                    ui.columns(4, |columns| {
                        for (i, column) in columns.iter_mut().enumerate() {
                            column.with_layout(
                                Layout::top_down(egui::Align::Center)
                                    .with_cross_align(egui::Align::Center),
                                |col| {
                                    col.spacing_mut().item_spacing.x = 0.0;

                                    let (num, color, label) = match i {
                                        0 => (
                                            stats.downloaded.to_string(),
                                            egui::Color32::from_rgb(0x00, 0xFF, 0x00),
                                            "Downloaded",
                                        ),
                                        1 => (
                                            stats.unchanged.to_string(),
                                            egui::Color32::from_rgb(0xFF, 0xFF, 0x00),
                                            "Unchanged",
                                        ),
                                        2 => (
                                            stats.removed.to_string(),
                                            egui::Color32::from_rgb(0xFF, 0xA5, 0x00),
                                            "Removed",
                                        ),
                                        3 => (
                                            stats.failed.to_string(),
                                            egui::Color32::from_rgb(0xFF, 0x00, 0x00),
                                            "Failed",
                                        ),
                                        _ => unreachable!(),
                                    };

                                    col.label(
                                        egui::RichText::new(num)
                                            .color(color)
                                            .size(32.0),
                                    );
                                    col.label(
                                        egui::RichText::new(label)
                                            .color(color)
                                            .size(12.0),
                                    );
                                },
                            );
                        }
                    });
                },
            );

            ui.add_space(10.0);


            // Handle different states
            if !self.splash_finished {
                // Still processing
                let total = self.progress.total as f32;
                let done = self.progress.processed() as f32;
                let fraction = if total > 0.0 { done / total } else { 0.0 };
                let text = format!("Processed {}/{} mods", done as usize, total as usize);

                draw_squared_progress_bar(ui, fraction, &text, true);

                ui.add_space(10.0);

                let last_mod = self.progress
                    .last_processed()
                    .unwrap_or_else(|| "Waiting…".to_string());
                ui.label(
                    egui::RichText::new(last_mod)
                        .color(egui::Color32::from_rgb(0xF0, 0xF0, 0xF0)),
                );
            } else {
                // Finished, show countdown or ready message
                if self.has_changes {
                    // Check for click to open transaction log
                    if self.splash_finished && self.has_changes && ui.input(|i| i.pointer.any_click()) {
                        if let Some(report) = self.transaction_report.take() {
                            // Send the report to main thread to open transaction log window
                            let _ = self.report_sender.send(report);
                            // Close this window
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }

                    // Show countdown bar for changes
                    let remaining = self.time_remaining();
                    let fraction = 1.0 - (remaining / self.splash_timeout_secs).clamp(0.0, 1.0);
                    let text = format!("Launching in {:.0}s…", remaining.ceil());

                    draw_squared_progress_bar(ui, fraction, &text, false);

                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("Click to view Changes")
                            .color(egui::Color32::from_rgb(0xF0, 0xF0, 0xF0)),
                    );
                } else {
                    // No changes, show "Ready to play!" for 2 seconds
                    let elapsed = self.splash_start.unwrap().elapsed().as_secs_f32();

                    if elapsed < 2.0 {
                        // Show "Ready to play!" for 2 seconds
                        ui.add_space(7.5);
                        ui.label(
                            egui::RichText::new("Ready to play!")
                                .size(18.0)
                                .color(egui::Color32::from_rgb(0x00, 0xFF, 0x00)),
                        );
                        ui.add_space(5.0);
                        ui.label(
                            egui::RichText::new(format!("Launching in {:.0}s…", (2.0 - elapsed).ceil()))
                                .color(egui::Color32::from_rgb(0x88, 0x88, 0x88)),
                        );
                    } else {
                        // After 2 seconds, force close
                        self.splash_start = Some(Instant::now() - std::time::Duration::from_secs_f32(self.splash_timeout_secs));
                    }
                }
            }
        });

        ui.ctx().request_repaint();
    }

    fn time_remaining(&self) -> f32 {
        if let Some(start) = self.splash_start {
            let elapsed = start.elapsed().as_secs_f32();
            (self.splash_timeout_secs - elapsed).max(0.0)
        } else {
            self.splash_timeout_secs
        }
    }
}

fn draw_squared_progress_bar(
    ui: &mut egui::Ui,
    progress_fraction: f32,
    progress_text: &str,
    is_processing: bool,
) {
    let progress_bar_height = 20.0;
    let horizontal_margin = 20.0;
    let progress_bar_width = ui.available_width() - (horizontal_margin * 2.0);

    let start_pos = ui.cursor().min + egui::vec2(horizontal_margin, 0.0);

    // Background
    let bg_rect = egui::Rect::from_min_size(
        start_pos,
        egui::vec2(progress_bar_width, progress_bar_height)
    );

    ui.painter().rect_filled(
        bg_rect,
        0.0,
        egui::Color32::from_rgb(0x1B, 0x1B, 0x1B),
    );

    // Progress fill
    if progress_fraction > 0.0 {
        let fill_width = progress_bar_width * progress_fraction.clamp(0.0, 1.0);
        let fill_rect = egui::Rect::from_min_size(
            start_pos,
            egui::vec2(fill_width, progress_bar_height)
        );

        let fill_color = if is_processing {
            egui::Color32::from_rgb(0x20, 0x25, 0x6A) // Processing color
        } else {
            egui::Color32::from_rgb(0x00, 0x7A, 0x00) // Countdown color
        };

        ui.painter().rect_filled(
            fill_rect,
            0.0,
            fill_color,
        );
    }

    // Text
    let text_pos = bg_rect.center();
    ui.painter().text(
        text_pos,
        egui::Align2::CENTER_CENTER,
        progress_text,
        egui::FontId::new(12.0, egui::FontFamily::Proportional),
        egui::Color32::from_rgb(0xF0, 0xF0, 0xF0),
    );

    ui.add_space(progress_bar_height + 4.0);
}

fn load_logo(ctx: &egui::Context, target_width: u32, target_height: u32) -> Option<TextureHandle> {
    let image_bytes = include_bytes!("../../assets/images/logo.png");

    let image = match image::load_from_memory(image_bytes) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Failed to load logo: {}", e);
            return None;
        }
    };

    let resized_image = image.resize_exact(
        target_width,
        target_height,
        image::imageops::FilterType::Nearest,
    );

    let image_buffer = resized_image.to_rgba8();
    let size = [target_width as usize, target_height as usize];
    let pixels = image_buffer.as_flat_samples();

    let color_image = ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    );

    Some(ctx.load_texture(
        "logo",
        color_image,
        egui::TextureOptions::default(),
    ))
}

impl App for ModSyncApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_splash(ui);
        });

        // Close window when countdown finishes
        if self.splash_finished && self.time_remaining() <= 0.0 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        ctx.request_repaint();
    }
}
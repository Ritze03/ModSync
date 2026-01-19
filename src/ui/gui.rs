use eframe::{egui, App};
use std::sync::Arc;
use egui::{Direction, Vec2, ColorImage, TextureHandle, FontFamily, FontData, FontDefinitions, Color32, CornerRadius};
use tokio::sync::mpsc::UnboundedReceiver;
use crate::modmanager::{SyncProgress, SyncEvent};

pub struct ModSyncApp {
    progress: Arc<SyncProgress>,
    events: UnboundedReceiver<SyncEvent>,

    // Splash state
    splash_finished: bool,
    logo_texture: Option<TextureHandle>,
}

impl ModSyncApp {
    pub fn new(cc: &eframe::CreationContext<'_>, progress: Arc<SyncProgress>, events: UnboundedReceiver<SyncEvent>) -> Self {
        // Set up custom font FIRST
        setup_fonts(&cc.egui_ctx);

        // Apply dark theme styling
        setup_dark_theme(&cc.egui_ctx);

        // Try to load logo image
        let logo_texture = load_logo(&cc.egui_ctx, 100, 100);

        Self {
            progress,
            events,
            splash_finished: false,
            logo_texture,
        }
    }

    pub fn draw_splash(&mut self, ui: &mut egui::Ui) {
        // Drain events - when Finished is received, mark as done
        while let Ok(event) = self.events.try_recv() {
            if let SyncEvent::Finished = event {
                self.splash_finished = true;
            }
        }

        // Vertical layout
        ui.vertical_centered(|ui| {
            // Logo - image or text fallback
            if let Some(texture) = &self.logo_texture {
                // Display the logo with a fixed size
                ui.image(texture); // Adjust size as needed
            } else {
                // Fallback text if image fails to load
                ui.add_space(25.0);
                ui.label(egui::RichText::new("ModSync").size(50.0).color(Color32::from_rgb(0xF0, 0xF0, 0xF0)));
                ui.add_space(25.0);
            }

            ui.add_space(20.0);

            use egui::{Layout};

            let status_numbers_margin = 20.0;
            let status_numbers_size = Vec2::new(ui.available_width() - (status_numbers_margin * 2.0), 60.0);

            ui.allocate_ui_with_layout(status_numbers_size, Layout::centered_and_justified(Direction::LeftToRight), |ui| {
                let stats = self.progress.stats();

                // Use columns for equal spacing
                ui.columns(4, |columns| {
                    for (i, column) in columns.iter_mut().enumerate() {
                        // Center content both horizontally and vertically within the column
                        column.with_layout(
                            Layout::top_down(egui::Align::Center).with_cross_align(egui::Align::Center),
                            |col| {
                                col.spacing_mut().item_spacing.x = 0.0;

                                let (number_text, color, label_text) = match i {
                                    0 => (stats.downloaded.to_string(), Color32::from_rgb(0x00, 0xFF, 0x00), "Downloaded"),
                                    1 => (stats.skipped.to_string(), Color32::from_rgb(0xFF, 0xFF, 0x00), "Skipped"),
                                    2 => (stats.removed.to_string(), Color32::from_rgb(0xFF, 0xA5, 0x00), "Removed"),
                                    3 => (stats.failed.to_string(), Color32::from_rgb(0xFF, 0x00, 0x00), "Failed"),
                                    _ => unreachable!(),
                                };

                                // Number with large font
                                col.label(
                                    egui::RichText::new(number_text)
                                        .color(color)
                                        .size(32.0),
                                );

                                // Label with small font
                                col.label(
                                    egui::RichText::new(label_text)
                                        .color(color)
                                        .size(12.0),
                                );
                            }
                        );
                    }
                });
            });

            ui.add_space(10.0);

            // Progress bar - only show during processing
            if !self.splash_finished {
                let total = self.progress.total as f32;
                let done = self.progress.processed() as f32;
                let fraction = if total > 0.0 { done / total } else { 0.0 };
                let text = format!("Processed {}/{} mods", done as usize, total as usize);

                draw_squared_progress_bar(ui, fraction, &text, true);

                ui.add_space(10.0);

                // Status line - only show during processing
                let last_mod = self.progress.last_processed().unwrap_or("Waiting...".to_string());
                let status_text = format!("{}", last_mod);
                ui.label(egui::RichText::new(status_text).color(Color32::from_rgb(0xF0, 0xF0, 0xF0)));
            } else {
                // When finished, show completion message briefly
                ui.add_space(20.0);
                ui.label(egui::RichText::new("Ready to play!").size(18.0).color(Color32::from_rgb(0x00, 0xFF, 0x00)));
                //ui.add_space(10.0);
                //ui.label(egui::RichText::new("Closing window...").color(Color32::from_rgb(0x88, 0x88, 0x88)));
            }
        });

        ui.ctx().request_repaint();
    }
}

fn draw_squared_progress_bar(
    ui: &mut egui::Ui,
    progress_fraction: f32,
    progress_text: &str,
    _is_processing: bool, // Keep parameter for compatibility but not used
) {
    // Configuration
    let progress_bar_height = 20.0;
    let horizontal_margin = 20.0;
    let progress_bar_width = ui.available_width() - (horizontal_margin * 2.0);

    // Get starting position (accounting for margin)
    let start_pos = ui.cursor().min + egui::vec2(horizontal_margin, 0.0);

    // Create background rect
    let bg_rect = egui::Rect::from_min_size(
        start_pos,
        egui::vec2(progress_bar_width, progress_bar_height)
    );

    // Paint the background - Color #1b1b1b
    ui.painter().rect_filled(
        bg_rect,
        0.0,
        Color32::from_rgb(0x1B, 0x1B, 0x1B),
    );

    // Paint the progress fill (if any progress)
    if progress_fraction > 0.0 {
        let fill_width = progress_bar_width * progress_fraction.clamp(0.0, 1.0);
        let fill_rect = egui::Rect::from_min_size(
            start_pos,
            egui::vec2(fill_width, progress_bar_height)
        );

        // Always use processing color since there's no timeout anymore
        let fill_color = Color32::from_rgb(0x20, 0x25, 0x6A); // #20256a for processing

        ui.painter().rect_filled(
            fill_rect,
            0.0,
            fill_color,
        );
    }

    // Add progress text in the middle
    let text_pos = bg_rect.center();
    ui.painter().text(
        text_pos,
        egui::Align2::CENTER_CENTER,
        progress_text,
        egui::FontId::new(12.0, FontFamily::Proportional),
        Color32::from_rgb(0xF0, 0xF0, 0xF0), // #f0f0f0
    );

    // Advance cursor (height + some vertical spacing)
    ui.add_space(progress_bar_height + 4.0);
}

// Helper function to load the logo
fn load_logo(ctx: &egui::Context, target_width: u32, target_height: u32) -> Option<TextureHandle> {
    // Load the image bytes
    let image_bytes = include_bytes!("../../assets/images/logo.png");

    // Decode the PNG
    let image = match image::load_from_memory(image_bytes) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Failed to load logo: {}", e);
            return None;
        }
    };

    // Resize the image
    let resized_image = image.resize_exact(
        target_width,
        target_height,
        image::imageops::FilterType::Nearest,
    );

    // Convert to RGBA
    let image_buffer = resized_image.to_rgba8();
    let size = [target_width as usize, target_height as usize];
    let pixels = image_buffer.as_flat_samples();

    // Create egui ColorImage
    let color_image = ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    );

    // Load as texture
    Some(ctx.load_texture(
        "logo",
        color_image,
        egui::TextureOptions::default(),
    ))
}

// Function to set up custom fonts
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // Load the Monocraft font
    let font_bytes = include_bytes!("../../assets/fonts/Monocraft.otf");

    fonts.font_data.insert(
        "monocraft".to_owned(),
        Arc::from(FontData::from_static(font_bytes)),
    );

    // Use Monocraft for both proportional and monospace
    fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "monocraft".to_owned());
    fonts.families.get_mut(&FontFamily::Monospace).unwrap().insert(0, "monocraft".to_owned());

    ctx.set_fonts(fonts);
}

// Function to set up dark theme
fn setup_dark_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Main background color: #0E0E0E
    style.visuals.panel_fill = Color32::from_rgb(0x0E, 0x0E, 0x0E);
    style.visuals.window_fill = Color32::from_rgb(0x0E, 0x0E, 0x0E);
    style.visuals.faint_bg_color = Color32::from_rgb(0x0E, 0x0E, 0x0E);
    style.visuals.extreme_bg_color = Color32::from_rgb(0x0E, 0x0E, 0x0E);

    // Text colors: #F0F0F0 for primary, #888888 for secondary
    style.visuals.override_text_color = Option::from(Color32::from_rgb(0xF0, 0xF0, 0xF0));
    style.visuals.weak_text_color = Option::from(Color32::from_rgb(0x88, 0x88, 0x88));
    style.visuals.hyperlink_color = Color32::from_rgb(0x4A, 0x9C, 0xFF);

    // Widget styling
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(0x1B, 0x1B, 0x1B);
    style.visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(0xF0, 0xF0, 0xF0);

    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(0x1B, 0x1B, 0x1B);
    style.visuals.widgets.inactive.fg_stroke.color = Color32::from_rgb(0xF0, 0xF0, 0xF0);

    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x25, 0x25, 0x25);
    style.visuals.widgets.hovered.fg_stroke.color = Color32::from_rgb(0xFF, 0xFF, 0xFF);

    style.visuals.widgets.active.bg_fill = Color32::from_rgb(0x30, 0x30, 0x30);
    style.visuals.widgets.active.fg_stroke.color = Color32::from_rgb(0xFF, 0xFF, 0xFF);

    // Remove rounding for squared look
    style.visuals.widgets.noninteractive.corner_radius = CornerRadius::ZERO;
    style.visuals.widgets.inactive.corner_radius = CornerRadius::ZERO;
    style.visuals.widgets.hovered.corner_radius = CornerRadius::ZERO;
    style.visuals.widgets.active.corner_radius = CornerRadius::ZERO;

    // Selection colors
    style.visuals.selection.bg_fill = Color32::from_rgb(0x20, 0x25, 0x6A); // #20256a
    style.visuals.selection.stroke.color = Color32::from_rgb(0x2F, 0x36, 0x99); // #2f3699

    ctx.set_style(style);
}

impl App for ModSyncApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Draw splash panel
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_splash(ui);
        });

        // Close window immediately when processing is finished
        if self.splash_finished {
            // Small delay to show completion message (optional)
            let ctx = ctx.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(2000)); // 2 second delay
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            });
        }

        ctx.request_repaint(); // keep UI live
    }
}
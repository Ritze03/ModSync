// theme.rs
use egui::{FontFamily, FontData, FontDefinitions, Color32, CornerRadius, Context};
use std::sync::Arc;

pub fn setup_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    let font_bytes = include_bytes!("../../assets/fonts/Monocraft.otf");

    fonts.font_data.insert(
        "monocraft".to_owned(),
        Arc::from(FontData::from_static(font_bytes)),
    );

    fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "monocraft".to_owned());
    fonts.families.get_mut(&FontFamily::Monospace).unwrap().insert(0, "monocraft".to_owned());

    ctx.set_fonts(fonts);
}

pub fn setup_dark_theme(ctx: &Context) {
    let mut style = (*ctx.style()).clone();

    style.visuals.panel_fill = Color32::from_rgb(0x0E, 0x0E, 0x0E);
    style.visuals.window_fill = Color32::from_rgb(0x0E, 0x0E, 0x0E);
    style.visuals.faint_bg_color = Color32::from_rgb(0x0E, 0x0E, 0x0E);
    style.visuals.extreme_bg_color = Color32::from_rgb(0x0E, 0x0E, 0x0E);

    style.visuals.override_text_color = Some(Color32::from_rgb(0xF0, 0xF0, 0xF0));
    style.visuals.weak_text_color = Some(Color32::from_rgb(0x88, 0x88, 0x88));
    style.visuals.hyperlink_color = Color32::from_rgb(0x4A, 0x9C, 0xFF);

    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(0x1B, 0x1B, 0x1B);
    style.visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(0xF0, 0xF0, 0xF0);

    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(0x1B, 0x1B, 0x1B);
    style.visuals.widgets.inactive.fg_stroke.color = Color32::from_rgb(0xF0, 0xF0, 0xF0);

    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x25, 0x25, 0x25);
    style.visuals.widgets.hovered.fg_stroke.color = Color32::from_rgb(0xFF, 0xFF, 0xFF);

    style.visuals.widgets.active.bg_fill = Color32::from_rgb(0x30, 0x30, 0x30);
    style.visuals.widgets.active.fg_stroke.color = Color32::from_rgb(0xFF, 0xFF, 0xFF);

    style.visuals.widgets.noninteractive.corner_radius = CornerRadius::ZERO;
    style.visuals.widgets.inactive.corner_radius = CornerRadius::ZERO;
    style.visuals.widgets.hovered.corner_radius = CornerRadius::ZERO;
    style.visuals.widgets.active.corner_radius = CornerRadius::ZERO;

    style.visuals.selection.bg_fill = Color32::from_rgb(0x20, 0x25, 0x6A);
    style.visuals.selection.stroke.color = Color32::from_rgb(0x2F, 0x36, 0x99);

    ctx.set_style(style);
}
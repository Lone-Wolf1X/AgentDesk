// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Light Blue / Clean White Theme — matching the custom icon
// Made in Nepal 🇳🇵

use egui::{Color32, Frame, Margin, Rounding, Stroke};

// ─── Color Palette (Light Blue / White theme) ─────────────────────────────────
pub const COLOR_BG:           Color32 = Color32::from_rgb(235, 242, 252); // soft sky blue-white bg
pub const COLOR_PANEL_BG:     Color32 = Color32::from_rgb(255, 255, 255); // pure white cards
pub const COLOR_CARD_BG:      Color32 = COLOR_PANEL_BG;                    // alias for compatibility
pub const COLOR_SIDEBAR_BG:   Color32 = Color32::from_rgb(224, 236, 250); // slightly deeper left panel
pub const COLOR_PRIMARY:      Color32 = Color32::from_rgb(37,  99,  235); // vivid blue (#2563EB)
pub const COLOR_PRIMARY_DARK: Color32 = Color32::from_rgb(29,  78,  216); // hover / pressed blue
pub const COLOR_PRIMARY_LIGHT:Color32 = Color32::from_rgb(219, 234, 254); // very light blue tint
pub const COLOR_ACCENT:       Color32 = Color32::from_rgb(14,  165, 233); // sky-400 accent
pub const COLOR_SUCCESS:      Color32 = Color32::from_rgb(16,  185, 129); // emerald-500
pub const COLOR_WARNING:      Color32 = Color32::from_rgb(245, 158, 11);  // amber-500
pub const COLOR_ERROR:        Color32 = Color32::from_rgb(239, 68,  68);  // red-500
pub const COLOR_TEXT:         Color32 = Color32::from_rgb(15,  23,  42);  // slate-900
pub const COLOR_TEXT_MUTED:   Color32 = Color32::from_rgb(100, 116, 139); // slate-500
pub const COLOR_TEXT_LIGHT:   Color32 = Color32::from_rgb(148, 163, 184); // slate-400
pub const COLOR_BORDER:       Color32 = Color32::from_rgb(203, 213, 225); // slate-300
pub const COLOR_BORDER_FOCUS: Color32 = Color32::from_rgb(147, 197, 253); // blue-300

/// Apply the global light-blue visual style to egui context.
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // — Background colours
    style.visuals.window_fill       = COLOR_BG;
    style.visuals.panel_fill        = COLOR_BG;
    style.visuals.faint_bg_color    = COLOR_SIDEBAR_BG;
    style.visuals.extreme_bg_color  = COLOR_PANEL_BG;

    // — Widget fill colours
    style.visuals.widgets.noninteractive.bg_fill  = COLOR_PANEL_BG;
    style.visuals.widgets.inactive.bg_fill         = Color32::from_rgb(241, 245, 249); // slate-100
    style.visuals.widgets.hovered.bg_fill          = COLOR_PRIMARY_LIGHT;
    style.visuals.widgets.active.bg_fill           = Color32::from_rgb(191, 219, 254); // blue-200
    style.visuals.widgets.open.bg_fill             = Color32::from_rgb(239, 246, 255);

    // — Stroke colours (borders)
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER);
    style.visuals.widgets.inactive.bg_stroke       = Stroke::new(1.0_f32, COLOR_BORDER);
    style.visuals.widgets.hovered.bg_stroke        = Stroke::new(1.5_f32, COLOR_PRIMARY);
    style.visuals.widgets.active.bg_stroke         = Stroke::new(2.0_f32, COLOR_PRIMARY_DARK);

    // — Foreground colours (text on widgets)
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT);
    style.visuals.widgets.inactive.fg_stroke       = Stroke::new(1.0_f32, COLOR_TEXT);
    style.visuals.widgets.hovered.fg_stroke        = Stroke::new(1.5_f32, COLOR_PRIMARY_DARK);
    style.visuals.widgets.active.fg_stroke         = Stroke::new(2.0_f32, COLOR_PRIMARY_DARK);

    // — Rounding (all widgets rounded)
    let rounding = Rounding::same(8.0);
    style.visuals.widgets.noninteractive.rounding = rounding;
    style.visuals.widgets.inactive.rounding       = rounding;
    style.visuals.widgets.hovered.rounding        = rounding;
    style.visuals.widgets.active.rounding         = rounding;
    style.visuals.window_rounding   = Rounding::same(12.0);
    style.visuals.menu_rounding     = Rounding::same(10.0);

    // — Selection colour
    style.visuals.selection.bg_fill = COLOR_PRIMARY_LIGHT;
    style.visuals.selection.stroke  = Stroke::new(1.0_f32, COLOR_PRIMARY);

    // — Override text colour so labels show on white bg
    style.visuals.override_text_color = Some(COLOR_TEXT);

    // — Separator colour
    style.visuals.window_stroke = Stroke::new(1.0_f32, COLOR_BORDER);

    // — Hyperlink colour
    style.visuals.hyperlink_color = COLOR_PRIMARY;

    // — Spacing
    style.spacing.item_spacing   = egui::vec2(8.0, 6.0);
    style.spacing.button_padding  = egui::vec2(14.0, 8.0);
    style.spacing.window_margin   = Margin::same(16.0);
    style.spacing.indent          = 20.0;
    style.spacing.scroll.bar_width = 6.0;

    ctx.set_style(style);
}

/// Card-style frame (white bg, rounded, light shadow border).
pub fn card_frame() -> Frame {
    Frame::none()
        .fill(COLOR_PANEL_BG)
        .rounding(Rounding::same(12.0))
        .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
        .inner_margin(Margin::same(20.0))
}

/// Accent/highlight card (light blue tint).
pub fn accent_card_frame() -> Frame {
    Frame::none()
        .fill(Color32::from_rgb(239, 246, 255)) // blue-50
        .rounding(Rounding::same(12.0))
        .stroke(Stroke::new(1.5_f32, COLOR_BORDER_FOCUS))
        .inner_margin(Margin::same(20.0))
}

/// Warning/alert card frame (orange tint for incoming connections).
pub fn alert_card_frame() -> Frame {
    Frame::none()
        .fill(Color32::from_rgb(255, 251, 235)) // amber-50
        .rounding(Rounding::same(12.0))
        .stroke(Stroke::new(2.0_f32, COLOR_WARNING))
        .inner_margin(Margin::same(16.0))
}

/// A large prominent button styled with solid primary blue.
pub fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(label)
            .color(Color32::WHITE)
            .size(14.0)
            .strong(),
    )
    .fill(COLOR_PRIMARY)
    .stroke(Stroke::new(1.0_f32, COLOR_PRIMARY_DARK))
    .rounding(Rounding::same(8.0))
    .min_size(egui::vec2(160.0, 40.0));

    let resp = ui.add(button);
    resp
}

/// A ghost/outline button for secondary actions.
pub fn secondary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(label)
            .color(COLOR_PRIMARY)
            .size(13.0),
    )
    .fill(Color32::TRANSPARENT)
    .stroke(Stroke::new(1.5_f32, COLOR_PRIMARY))
    .rounding(Rounding::same(8.0))
    .min_size(egui::vec2(120.0, 36.0));

    ui.add(button)
}

/// A danger red button (reject/disconnect).
pub fn danger_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(label)
            .color(Color32::WHITE)
            .size(13.0)
            .strong(),
    )
    .fill(COLOR_ERROR)
    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(185, 28, 28)))
    .rounding(Rounding::same(8.0))
    .min_size(egui::vec2(100.0, 36.0));

    ui.add(button)
}

/// Top app header bar.
pub fn draw_header(ui: &mut egui::Ui, subtitle: &str) {
    Frame::none()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
        .inner_margin(Margin::symmetric(20.0, 12.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Logo + name
                ui.label(egui::RichText::new("🇳🇵").size(26.0));
                ui.add_space(4.0);
                ui.heading(
                    egui::RichText::new("AgentDesk")
                        .size(20.0)
                        .color(COLOR_PRIMARY)
                        .strong(),
                );
                // Version badge
                Frame::none()
                    .fill(COLOR_PRIMARY_LIGHT)
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::symmetric(6.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("v0.1.0")
                                .size(10.0)
                                .color(COLOR_PRIMARY)
                                .strong(),
                        );
                    });
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(subtitle)
                        .color(COLOR_TEXT_MUTED)
                        .size(13.0),
                );

                // Right side: company pill
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    Frame::none()
                        .fill(Color32::from_rgb(239, 246, 255))
                        .stroke(Stroke::new(1.0_f32, COLOR_BORDER_FOCUS))
                        .rounding(Rounding::same(20.0))
                        .inner_margin(Margin::symmetric(12.0, 5.0))
                        .show(ui, |ui| {
                            ui.hyperlink_to(
                                egui::RichText::new("🏢 Next Gen Innovations")
                                    .color(COLOR_PRIMARY)
                                    .size(12.0)
                                    .strong(),
                                "https://nextgeninnovations.com.np/",
                            );
                        });
                });
            });
        });
}

/// Bottom status bar / footer.
pub fn draw_footer(ui: &mut egui::Ui) {
    ui.add_space(6.0);
    Frame::none()
        .fill(Color32::from_rgb(248, 250, 252)) // slate-50
        .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
        .inner_margin(Margin::symmetric(20.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("🇳🇵  AgentDesk — 100% Free & Open Source (MIT)  •  No Limits  •  No Telemetry")
                        .color(COLOR_TEXT_MUTED)
                        .size(11.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to(
                        egui::RichText::new("nextgeninnovations.com.np")
                            .color(COLOR_PRIMARY)
                            .size(11.0),
                        "https://nextgeninnovations.com.np/",
                    );
                    ui.label(
                        egui::RichText::new("by Abhishek Kumar Paswan  |  ")
                            .color(COLOR_TEXT_LIGHT)
                            .size(11.0),
                    );
                });
            });
        });
}

/// About strip shown on the main idle screen.
pub fn draw_about_box(ui: &mut egui::Ui) {
    Frame::none()
        .fill(Color32::from_rgb(239, 246, 255)) // blue-50
        .rounding(Rounding::same(10.0))
        .stroke(Stroke::new(1.0_f32, COLOR_BORDER_FOCUS))
        .inner_margin(Margin::symmetric(16.0, 10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🇳🇵").size(16.0));
                ui.label(
                    egui::RichText::new("Developed by  Next Gen Innovations Private Limited  &  Abhishek Kumar Paswan")
                        .color(COLOR_TEXT)
                        .size(12.0)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to(
                        egui::RichText::new("🌐 nextgeninnovations.com.np")
                            .color(COLOR_PRIMARY)
                            .size(12.0),
                        "https://nextgeninnovations.com.np/",
                    );
                });
            });
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(
                    "100% Free  •  Open Source (MIT)  •  No Paid Tiers  •  No Time Limits  •  No Device Limits  •  No Telemetry  •  Made in Nepal 🇳🇵",
                )
                .color(COLOR_TEXT_MUTED)
                .size(10.5),
            );
        });
}

/// A labelled info row inside a card (label: value with copy button pattern).
pub fn info_row(ui: &mut egui::Ui, label: &str, value: &str, value_color: Color32, font_size: f32) -> bool {
    let mut copied = false;
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .color(COLOR_TEXT_MUTED)
                .size(12.0),
        );
    });
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(value)
                .size(font_size)
                .strong()
                .color(value_color),
        );
        Frame::none()
            .fill(COLOR_PRIMARY_LIGHT)
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(8.0, 4.0))
            .show(ui, |ui| {
                if ui.add(egui::Button::new(
                    egui::RichText::new("⎘ Copy").size(11.0).color(COLOR_PRIMARY)
                ).frame(false)).clicked() {
                    ui.output_mut(|o| o.copied_text = value.to_string());
                    copied = true;
                }
            });
    });
    copied
}

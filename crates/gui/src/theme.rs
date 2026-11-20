// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

use egui::{Color32, Frame, Margin, Rounding, Stroke};

pub const COLOR_PRIMARY: Color32 = Color32::from_rgb(220, 20, 60); // Nepal Crimson
pub const COLOR_BG_DARK: Color32 = Color32::from_rgb(20, 20, 24);
pub const COLOR_CARD_BG: Color32 = Color32::from_rgb(30, 30, 36);
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(160, 160, 175);
pub const COLOR_SUCCESS: Color32 = Color32::from_rgb(46, 204, 113);
pub const COLOR_WARNING: Color32 = Color32::from_rgb(241, 196, 15);
pub const COLOR_GOLD: Color32 = Color32::from_rgb(255, 215, 0);

pub fn draw_header(ui: &mut egui::Ui, title: &str) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🇳🇵").size(24.0));
            ui.heading(
                egui::RichText::new("AgentDesk")
                    .size(22.0)
                    .color(COLOR_PRIMARY)
                    .strong(),
            );
            ui.label(
                egui::RichText::new("v0.1.0")
                    .size(11.0)
                    .color(COLOR_GOLD)
                    .background_color(Color32::from_rgb(45, 45, 55)),
            );
            ui.label(egui::RichText::new(format!("— {title}")).color(COLOR_TEXT_MUTED));
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            Frame::none()
                .fill(Color32::from_rgb(40, 25, 30))
                .stroke(Stroke::new(1.0, COLOR_PRIMARY))
                .rounding(Rounding::same(12.0))
                .inner_margin(Margin::symmetric(10.0, 4.0))
                .show(ui, |ui| {
                    ui.hyperlink_to(
                        egui::RichText::new("🏢 Next Gen Innovations")
                            .color(Color32::WHITE)
                            .small()
                            .strong(),
                        "https://nextgeninnovations.com.np/",
                    );
                });
        });
    });
    ui.add_space(4.0);
    ui.separator();
}

pub fn draw_about_box(ui: &mut egui::Ui) {
    Frame::none()
        .fill(Color32::from_rgb(24, 24, 30))
        .rounding(Rounding::same(6.0))
        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 45, 55)))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🇳🇵").size(18.0));
                ui.label(
                    egui::RichText::new("Developed by Next Gen Innovations Private Limited & Abhishek Kumar Paswan")
                        .color(Color32::WHITE)
                        .small()
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to(
                        egui::RichText::new("🌐 nextgeninnovations.com.np")
                            .color(COLOR_PRIMARY)
                            .small(),
                        "https://nextgeninnovations.com.np/",
                    );
                });
            });
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("100% Free & Open Source (MIT) • No Paid Tiers • No Time Limits • No Telemetry • Made in Nepal 🇳🇵")
                    .color(COLOR_TEXT_MUTED)
                    .size(10.5),
            );
        });
}

pub fn draw_footer(ui: &mut egui::Ui) {
    ui.separator();
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("AgentDesk 🇳🇵 — Free & Open Source Remote Desktop")
                .color(COLOR_TEXT_MUTED)
                .small(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.hyperlink_to(
                egui::RichText::new("Next Gen Innovations").color(COLOR_PRIMARY).small(),
                "https://nextgeninnovations.com.np/",
            );
            ui.label(
                egui::RichText::new("by Abhishek Kumar Paswan |")
                    .color(COLOR_TEXT_MUTED)
                    .small(),
            );
        });
    });
    ui.add_space(2.0);
}

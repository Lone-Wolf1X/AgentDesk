// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

use egui::Color32;

pub const COLOR_PRIMARY: Color32 = Color32::from_rgb(220, 20, 60);
pub const COLOR_BG_DARK: Color32 = Color32::from_rgb(24, 24, 28);
pub const COLOR_CARD_BG: Color32 = Color32::from_rgb(34, 34, 40);
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(160, 160, 170);
pub const COLOR_SUCCESS: Color32 = Color32::from_rgb(46, 204, 113);
pub const COLOR_WARNING: Color32 = Color32::from_rgb(241, 196, 15);

pub fn draw_header(ui: &mut egui::Ui, title: &str) {
    ui.horizontal(|ui| {
        ui.heading(format!("🇳🇵 AgentDesk — {title}"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new("Next Gen Innovations").color(COLOR_PRIMARY).strong());
        });
    });
    ui.separator();
}

pub fn draw_footer(ui: &mut egui::Ui) {
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Free & Open Source — Made in Nepal 🇳🇵").color(COLOR_TEXT_MUTED).small());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.hyperlink_to("nextgeninnovations.com.np", "https://nextgeninnovations.com.np/");
            ui.label(egui::RichText::new("by Abhishek Kumar Paswan |").color(COLOR_TEXT_MUTED).small());
        });
    });
}

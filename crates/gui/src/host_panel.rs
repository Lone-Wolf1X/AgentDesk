// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

use crate::theme::{draw_footer, draw_header, COLOR_CARD_BG, COLOR_PRIMARY, COLOR_SUCCESS, COLOR_WARNING};
use egui::{Align2, Frame, Margin, Rounding, Vec2};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// State for the Host Control Window
pub struct HostGuiState {
    pub device_id: String,
    pub password: String,
    pub status: String,
    pub is_connected: bool,
    pub connected_peer: Option<String>,
    pub incoming_request: Option<String>,
    pub input_paused: Arc<AtomicBool>,
    pub accept_signal: Option<bool>,
    pub disconnect_requested: bool,
}

impl HostGuiState {
    pub fn new(device_id: String, password: String) -> Self {
        Self {
            device_id,
            password,
            status: "Ready for connection".to_string(),
            is_connected: false,
            connected_peer: None,
            incoming_request: None,
            input_paused: Arc::new(AtomicBool::new(false)),
            accept_signal: None,
            disconnect_requested: false,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            draw_header(ui, "Host Control");

            ui.add_space(10.0);

            // Display Device ID card
            Frame::none()
                .fill(COLOR_CARD_BG)
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(16.0))
                .show(ui, |ui| {
                    ui.label("Your Connection ID (Share with controller):");
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&self.device_id)
                                .size(28.0)
                                .color(COLOR_PRIMARY)
                                .strong(),
                        );
                        if ui.button("📋 Copy").clicked() {
                            ui.output_mut(|o| o.copied_text = self.device_id.clone());
                        }
                    });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("Access Password:");
                        ui.text_edit_singleline(&mut self.password);
                    });
                });

            ui.add_space(15.0);

            // Connection Status
            ui.horizontal(|ui| {
                ui.label("Status:");
                let color = if self.is_connected {
                    COLOR_SUCCESS
                } else {
                    COLOR_WARNING
                };
                ui.label(egui::RichText::new(&self.status).color(color).strong());
            });

            if self.is_connected {
                if let Some(ref peer) = self.connected_peer {
                    ui.label(format!("Connected to remote viewer: {peer}"));
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    let paused = self.input_paused.load(Ordering::Relaxed);
                    let pause_label = if paused { "▶ Resume Input" } else { "⏸ Pause Remote Input" };
                    if ui.button(pause_label).clicked() {
                        self.input_paused.store(!paused, Ordering::Relaxed);
                    }

                    if ui.button("❌ Terminate Session").clicked() {
                        self.disconnect_requested = true;
                    }
                });
            }

            // Incoming Connection Dialog Popup
            if let Some(ref peer) = self.incoming_request.clone() {
                egui::Window::new("Incoming Connection Request")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                    .show(ctx, |ui| {
                        ui.label(format!("Viewer '{peer}' requests access to control this desktop."));
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.button("✅ Allow").clicked() {
                                self.accept_signal = Some(true);
                                self.incoming_request = None;
                            }
                            if ui.button("❌ Deny").clicked() {
                                self.accept_signal = Some(false);
                                self.incoming_request = None;
                            }
                        });
                    });
            }

            ui.add_space(20.0);
            draw_footer(ui);
        });
    }
}

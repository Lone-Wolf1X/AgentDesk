// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

use crate::app_state::{AppState, SessionStats};
use crate::theme::{draw_footer, draw_header, COLOR_CARD_BG, COLOR_PRIMARY, COLOR_SUCCESS};
use agentdesk_core::types::{InputEvent, MouseButton};
use egui::{ColorImage, Frame, Margin, Rounding, TextureOptions};
use std::net::IpAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

/// Action emitted by the UI for background network execution.
#[derive(Debug, Clone)]
pub enum UiAction {
    Connect { target: String, password: String },
    Disconnect,
    AcceptIncoming,
    RejectIncoming,
    UpdatePassword(String),
}

/// Unified Single-Window AgentDesk Application
pub struct AgentDeskApp {
    pub state: AppState,
    pub remote_target_input: String,
    pub remote_password_input: String,
    pub recent_connections: Vec<String>,
    pub action_sender: Option<Sender<UiAction>>,
    pub frame_receiver: Option<Receiver<(u32, u32, Vec<u8>)>>,
    pub input_sender: Option<tokio::sync::mpsc::Sender<InputEvent>>,
    pub remote_screen_size: (f64, f64),
}

impl AgentDeskApp {
    pub fn new(
        local_ip: Option<IpAddr>,
        local_port: u16,
        my_id: Option<String>,
        password: String,
        action_sender: Option<Sender<UiAction>>,
    ) -> Self {
        Self {
            state: AppState::Idle {
                local_ip,
                local_port,
                my_id,
                password,
                host_listening: true,
                incoming_request: None,
            },
            remote_target_input: String::new(),
            remote_password_input: String::new(),
            recent_connections: Vec::new(),
            action_sender,
            frame_receiver: None,
            input_sender: None,
            remote_screen_size: (1920.0, 1080.0),
        }
    }

    /// Transitions to viewing mode once network client successfully connects.
    pub fn set_viewing(
        &mut self,
        remote_addr: String,
        frame_rx: Receiver<(u32, u32, Vec<u8>)>,
        input_tx: tokio::sync::mpsc::Sender<InputEvent>,
        screen_size: (f64, f64),
    ) {
        if !self.recent_connections.contains(&remote_addr) {
            self.recent_connections.insert(0, remote_addr.clone());
            if self.recent_connections.len() > 5 {
                self.recent_connections.pop();
            }
        }
        self.frame_receiver = Some(frame_rx);
        self.input_sender = Some(input_tx);
        self.remote_screen_size = screen_size;
        self.state = AppState::Viewing {
            remote_addr,
            texture: None,
            connected_since: Instant::now(),
            stats: SessionStats::default(),
        };
    }

    /// Transitions back to Idle state.
    pub fn set_idle(&mut self, local_ip: Option<IpAddr>, local_port: u16, my_id: Option<String>, password: String) {
        self.frame_receiver = None;
        self.input_sender = None;
        self.state = AppState::Idle {
            local_ip,
            local_port,
            my_id,
            password,
            host_listening: true,
            incoming_request: None,
        };
    }
}

impl eframe::App for AgentDeskApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match &mut self.state {
            AppState::Viewing {
                remote_addr,
                texture,
                connected_since,
                stats,
            } => {
                // Poll for incoming video frames
                if let Some(ref rx) = self.frame_receiver {
                    while let Ok((w, h, rgba)) = rx.try_recv() {
                        let color_image = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
                        *texture = Some(ctx.load_texture("remote_screen", color_image, TextureOptions::LINEAR));
                        stats.frames_received += 1;
                        stats.bytes_received += rgba.len() as u64;
                    }
                }

                // Top control bar
                egui::TopBottomPanel::top("viewing_toolbar").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🔴 LIVE SESSION").color(COLOR_PRIMARY).strong());
                        ui.separator();
                        ui.label(format!("Connected to: {remote_addr}"));
                        ui.separator();
                        let elapsed = connected_since.elapsed().as_secs();
                        ui.label(format!("Duration: {:02}:{:02}", elapsed / 60, elapsed % 60));
                        ui.separator();
                        ui.label(format!("Frames: {}", stats.frames_received));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("❌ Disconnect").clicked() {
                                if let Some(ref tx) = self.action_sender {
                                    let _ = tx.send(UiAction::Disconnect);
                                }
                            }
                        });
                    });
                });

                // Central remote desktop canvas
                egui::CentralPanel::default().show(ctx, |ui| {
                    if let Some(tex) = texture {
                        let available_size = ui.available_size();
                        let image_response = ui.image(egui::load::SizedTexture::new(tex.id(), available_size));

                        // Capture user mouse & keyboard inputs over remote viewport
                        if image_response.hovered() {
                            let rect = image_response.rect;
                            if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                                let normalized_x = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                                let normalized_y = ((pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
                                let remote_x = normalized_x as f64 * self.remote_screen_size.0;
                                let remote_y = normalized_y as f64 * self.remote_screen_size.1;

                                if let Some(ref tx) = self.input_sender {
                                    let _ = tx.try_send(InputEvent::MouseMove {
                                        x: remote_x,
                                        y: remote_y,
                                    });

                                    if ctx.input(|i| i.pointer.primary_clicked()) {
                                        let _ = tx.try_send(InputEvent::MouseDown {
                                            button: MouseButton::Left,
                                            x: remote_x,
                                            y: remote_y,
                                        });
                                        let _ = tx.try_send(InputEvent::MouseUp {
                                            button: MouseButton::Left,
                                            x: remote_x,
                                            y: remote_y,
                                        });
                                    }

                                    if ctx.input(|i| i.pointer.secondary_clicked()) {
                                        let _ = tx.try_send(InputEvent::MouseDown {
                                            button: MouseButton::Right,
                                            x: remote_x,
                                            y: remote_y,
                                        });
                                        let _ = tx.try_send(InputEvent::MouseUp {
                                            button: MouseButton::Right,
                                            x: remote_x,
                                            y: remote_y,
                                        });
                                    }
                                }
                            }
                        }
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("Awaiting Remote Display Stream...").size(22.0));
                        });
                    }
                });
            }

            AppState::Hosting {
                viewer_id,
                connected_since,
                stats,
            } => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    draw_header(ui, "Active Host Session");
                    ui.add_space(20.0);

                    Frame::none()
                        .fill(COLOR_CARD_BG)
                        .rounding(Rounding::same(8.0))
                        .inner_margin(Margin::same(20.0))
                        .show(ui, |ui| {
                            ui.heading(egui::RichText::new("🟢 Sharing Screen With Remote Viewer").color(COLOR_SUCCESS));
                            ui.add_space(10.0);
                            ui.label(format!("Connected Viewer: {viewer_id}"));
                            let elapsed = connected_since.elapsed().as_secs();
                            ui.label(format!("Session Duration: {:02}:{:02}", elapsed / 60, elapsed % 60));
                            ui.label(format!("Stream Packets Sent: {}", stats.frames_received));
                            ui.add_space(15.0);

                            if ui.button("❌ Terminate Remote Access").clicked() {
                                if let Some(ref tx) = self.action_sender {
                                    let _ = tx.send(UiAction::Disconnect);
                                }
                            }
                        });

                    draw_footer(ui);
                });
            }

            AppState::Connecting { target, error } => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    draw_header(ui, "Connecting to Remote");
                    ui.add_space(30.0);

                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(format!("Connecting to {target}..."));
                            ui.add_space(10.0);
                            ui.spinner();

                            if let Some(err) = error {
                                ui.add_space(15.0);
                                ui.label(egui::RichText::new(format!("Error: {err}")).color(COLOR_PRIMARY));
                            }

                            ui.add_space(20.0);
                            if ui.button("Cancel").clicked() {
                                if let Some(ref tx) = self.action_sender {
                                    let _ = tx.send(UiAction::Disconnect);
                                }
                            }
                        });
                    });
                    draw_footer(ui);
                });
            }

            AppState::Idle {
                local_ip,
                local_port,
                my_id,
                password,
                host_listening: _,
                incoming_request,
            } => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    draw_header(ui, "Free & Open Source Remote Desktop");
                    ui.add_space(10.0);

                    // Incoming connection popup modal/card if present
                    if let Some(req) = incoming_request {
                        Frame::none()
                            .fill(COLOR_CARD_BG)
                            .rounding(Rounding::same(8.0))
                            .stroke(egui::Stroke::new(2.0_f32, COLOR_PRIMARY))
                            .inner_margin(Margin::same(16.0))
                            .show(ui, |ui| {
                                ui.heading(egui::RichText::new("⚠️ Incoming Connection Request").color(COLOR_PRIMARY));
                                ui.label(format!("A remote viewer ({}) wants to view and control your desktop.", req.viewer_addr));
                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    if ui.button("✅ Accept Access").clicked() {
                                        if let Some(ref tx) = self.action_sender {
                                            let _ = tx.send(UiAction::AcceptIncoming);
                                        }
                                    }
                                    if ui.button("❌ Reject").clicked() {
                                        if let Some(ref tx) = self.action_sender {
                                            let _ = tx.send(UiAction::RejectIncoming);
                                        }
                                    }
                                });
                            });
                        ui.add_space(15.0);
                    }

                    // Main dual-panel layout
                    ui.columns(2, |columns| {
                        // Left Column: THIS DEVICE (Host Info)
                        columns[0].vertical(|ui| {
                            Frame::none()
                                .fill(COLOR_CARD_BG)
                                .rounding(Rounding::same(8.0))
                                .inner_margin(Margin::same(16.0))
                                .show(ui, |ui| {
                                    ui.heading("🖥️ This Device");
                                    ui.add_space(8.0);

                                    let ip_str = match local_ip {
                                        Some(ip) => format!("{}:{}", ip, local_port),
                                        None => format!("127.0.0.1:{}", local_port),
                                    };

                                    ui.label("Your IP Address:");
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(&ip_str).size(20.0).strong().color(COLOR_SUCCESS));
                                        if ui.button("📋 Copy").clicked() {
                                            ui.output_mut(|o| o.copied_text = ip_str.clone());
                                        }
                                    });

                                    if let Some(id) = my_id {
                                        ui.add_space(6.0);
                                        ui.label("Your Connection ID:");
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(id.as_str()).size(18.0).color(COLOR_PRIMARY).strong());
                                            if ui.button("📋 Copy").clicked() {
                                                ui.output_mut(|o| o.copied_text = id.clone());
                                            }
                                        });
                                    }

                                    ui.add_space(8.0);
                                    ui.label("Access Password:");
                                    ui.horizontal(|ui| {
                                        let text_resp = ui.text_edit_singleline(password);
                                        if text_resp.changed() {
                                            if let Some(ref tx) = self.action_sender {
                                                let _ = tx.send(UiAction::UpdatePassword(password.clone()));
                                            }
                                        }
                                        if ui.button("📋 Copy").clicked() {
                                            ui.output_mut(|o| o.copied_text = password.clone());
                                        }
                                    });

                                    ui.add_space(10.0);
                                    ui.label(egui::RichText::new("🟢 Status: Waiting for connections").color(COLOR_SUCCESS).small());
                                });
                        });

                        // Right Column: CONNECT TO REMOTE
                        columns[1].vertical(|ui| {
                            Frame::none()
                                .fill(COLOR_CARD_BG)
                                .rounding(Rounding::same(8.0))
                                .inner_margin(Margin::same(16.0))
                                .show(ui, |ui| {
                                    ui.heading("🔗 Connect to Remote");
                                    ui.add_space(8.0);

                                    ui.label("Remote IP or 9-Digit ID:");
                                    ui.add(egui::TextEdit::singleline(&mut self.remote_target_input).hint_text("e.g. 192.168.1.50:9090"));

                                    ui.add_space(6.0);
                                    ui.label("Remote Password:");
                                    ui.add(egui::TextEdit::singleline(&mut self.remote_password_input).password(true));

                                    ui.add_space(12.0);
                                    if ui.button("🚀 Connect to Remote").clicked() && !self.remote_target_input.trim().is_empty() {
                                        if let Some(ref tx) = self.action_sender {
                                            let _ = tx.send(UiAction::Connect {
                                                target: self.remote_target_input.trim().to_string(),
                                                password: self.remote_password_input.clone(),
                                            });
                                        }
                                    }

                                    if !self.recent_connections.is_empty() {
                                        ui.add_space(14.0);
                                        ui.label(egui::RichText::new("Recent Connections:").small().color(egui::Color32::GRAY));
                                        for recent in &self.recent_connections {
                                            if ui.small_button(format!("• {recent}")).clicked() {
                                                self.remote_target_input = recent.clone();
                                            }
                                        }
                                    }
                                });
                        });
                    });

                    ui.add_space(15.0);
                    draw_footer(ui);
                });
            }
        }

        // Repaint continuously when viewing or hosting for smooth frame rates
        if matches!(self.state, AppState::Viewing { .. } | AppState::Hosting { .. }) {
            ctx.request_repaint();
        }
    }
}

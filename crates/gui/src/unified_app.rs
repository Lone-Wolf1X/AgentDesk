// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

use crate::app_state::{AppState, IncomingRequest, SessionStats};
use crate::theme::{draw_about_box, draw_footer, draw_header, COLOR_CARD_BG, COLOR_PRIMARY, COLOR_SUCCESS, COLOR_WARNING};
use agentdesk_core::protocol::SessionPermissions;
use agentdesk_core::types::{InputEvent, MouseButton};
use egui::{ColorImage, Frame, Margin, Rounding, TextureOptions};
use std::net::IpAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

/// Action emitted by the UI for background network execution.
#[derive(Debug, Clone)]
pub enum UiAction {
    Connect { target: String },
    Disconnect,
    AcceptIncoming { permissions: SessionPermissions },
    RejectIncoming,
}

/// Events sent from background networking tasks into the GUI.
pub enum AppEvent {
    Connected {
        remote_addr: String,
        permissions: SessionPermissions,
        screen_size: (f64, f64),
        input_tx: tokio::sync::mpsc::Sender<InputEvent>,
    },
    ConnectionFailed {
        error: String,
    },
    IncomingRequest(IncomingRequest),
    Disconnected,
}

/// Unified Single-Window AgentDesk Application
pub struct AgentDeskApp {
    pub local_ip: Option<IpAddr>,
    pub local_port: u16,
    pub my_id: Option<String>,
    pub state: AppState,
    pub remote_target_input: String,
    pub recent_connections: Vec<String>,
    pub action_sender: Option<Sender<UiAction>>,
    pub event_receiver: Option<Receiver<AppEvent>>,
    pub frame_receiver: Option<Receiver<(u32, u32, Vec<u8>)>>,
    pub input_sender: Option<tokio::sync::mpsc::Sender<InputEvent>>,
    pub remote_screen_size: (f64, f64),
}

impl AgentDeskApp {
    pub fn new(
        local_ip: Option<IpAddr>,
        local_port: u16,
        my_id: Option<String>,
        action_sender: Option<Sender<UiAction>>,
        event_receiver: Option<Receiver<AppEvent>>,
        frame_receiver: Option<Receiver<(u32, u32, Vec<u8>)>>,
    ) -> Self {
        Self {
            local_ip,
            local_port,
            my_id: my_id.clone(),
            state: AppState::Idle {
                local_ip,
                local_port,
                my_id,
                host_listening: true,
                incoming_request: None,
            },
            remote_target_input: String::new(),
            recent_connections: Vec::new(),
            action_sender,
            event_receiver,
            frame_receiver,
            input_sender: None,
            remote_screen_size: (1920.0, 1080.0),
        }
    }
}

impl eframe::App for AgentDeskApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Process background network events
        if let Some(ref rx) = self.event_receiver {
            while let Ok(event) = rx.try_recv() {
                match event {
                    AppEvent::Connected {
                        remote_addr,
                        permissions,
                        screen_size,
                        input_tx,
                    } => {
                        self.input_sender = Some(input_tx);
                        self.remote_screen_size = screen_size;
                        if !self.recent_connections.contains(&remote_addr) {
                            self.recent_connections.insert(0, remote_addr.clone());
                            if self.recent_connections.len() > 5 {
                                self.recent_connections.pop();
                            }
                        }
                        self.state = AppState::Viewing {
                            remote_addr,
                            texture: None,
                            connected_since: Instant::now(),
                            permissions,
                            stats: SessionStats::default(),
                        };
                    }
                    AppEvent::ConnectionFailed { error } => {
                        let target = match &self.state {
                            AppState::Connecting { target, .. } => target.clone(),
                            _ => self.remote_target_input.clone(),
                        };
                        self.state = AppState::Connecting {
                            target,
                            error: Some(error),
                        };
                    }
                    AppEvent::IncomingRequest(req) => {
                        if let AppState::Idle { ref mut incoming_request, .. } = self.state {
                            *incoming_request = Some(req);
                        }
                    }
                    AppEvent::Disconnected => {
                        self.state = AppState::Idle {
                            local_ip: self.local_ip,
                            local_port: self.local_port,
                            my_id: self.my_id.clone(),
                            host_listening: true,
                            incoming_request: None,
                        };
                    }
                }
            }
        }

        // 2. Render state-specific interface
        let mut next_state = None;
        match &mut self.state {
            AppState::Viewing {
                remote_addr,
                texture,
                connected_since,
                permissions,
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
                        ui.separator();

                        if permissions.keyboard_mouse {
                            ui.label(egui::RichText::new("🖱️ Remote Control Active").color(COLOR_SUCCESS).small());
                        } else {
                            ui.label(egui::RichText::new("👁️ View Only").color(COLOR_WARNING).small());
                        }

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

                        // Capture user mouse & keyboard inputs over remote viewport if control permitted
                        if permissions.keyboard_mouse && image_response.hovered() {
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
                permissions: _,
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
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("Waiting for remote host to accept connection request...").color(COLOR_WARNING));
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
                local_port: _,
                my_id,
                host_listening: _,
                incoming_request,
            } => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    draw_header(ui, "Free & Open Source Remote Desktop");
                    ui.add_space(10.0);

                    // Incoming connection popup with granular permission toggles and dual Accept buttons
                    if let Some(req) = incoming_request {
                        Frame::none()
                            .fill(COLOR_CARD_BG)
                            .rounding(Rounding::same(8.0))
                            .stroke(egui::Stroke::new(2.0_f32, COLOR_PRIMARY))
                            .inner_margin(Margin::same(16.0))
                            .show(ui, |ui| {
                                ui.heading(egui::RichText::new("⚠️ Incoming Connection Request").color(COLOR_PRIMARY));
                                ui.label(format!("Remote peer ({}) wants to view and control your desktop.", req.viewer_addr));
                                ui.add_space(8.0);

                                ui.label(egui::RichText::new("Session Permissions:").strong());
                                ui.horizontal_wrapped(|ui| {
                                    ui.checkbox(&mut req.permissions.keyboard_mouse, "Keyboard & Mouse");
                                    ui.checkbox(&mut req.permissions.clipboard, "Clipboard Sync");
                                    ui.checkbox(&mut req.permissions.sound, "Sound Audio");
                                    ui.checkbox(&mut req.permissions.whiteboard, "Whiteboard");
                                    ui.checkbox(&mut req.permissions.restart, "Restart PC");
                                    ui.checkbox(&mut req.permissions.file_transfer, "File Transfer");
                                    ui.checkbox(&mut req.permissions.file_manager, "File Manager");
                                });

                                ui.add_space(12.0);
                                ui.horizontal(|ui| {
                                    if ui.button("✅ Accept").clicked() {
                                        if let Some(ref tx) = self.action_sender {
                                            let _ = tx.send(UiAction::AcceptIncoming {
                                                permissions: req.permissions.clone(),
                                            });
                                        }
                                    }
                                    if ui.button("🛡️ Accept with Full Control").clicked() {
                                        if let Some(ref tx) = self.action_sender {
                                            let _ = tx.send(UiAction::AcceptIncoming {
                                                permissions: SessionPermissions::full_control(),
                                            });
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
                                        Some(ip) => ip.to_string(),
                                        None => "127.0.0.1".to_string(),
                                    };

                                    ui.label("Your IP Address:");
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(&ip_str).size(24.0).strong().color(COLOR_SUCCESS));
                                        if ui.button("📋 Copy").clicked() {
                                            ui.output_mut(|o| o.copied_text = ip_str.clone());
                                        }
                                    });

                                    if let Some(id) = my_id {
                                        ui.add_space(8.0);
                                        let formatted_id = if id.len() == 9 {
                                            format!("{} {} {}", &id[0..3], &id[3..6], &id[6..9])
                                        } else {
                                            id.clone()
                                        };

                                        ui.label("Your AnyDesk ID (Direct):");
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(&formatted_id).size(26.0).color(COLOR_PRIMARY).strong());
                                            if ui.button("📋 Copy").clicked() {
                                                ui.output_mut(|o| o.copied_text = formatted_id.clone());
                                            }
                                        });
                                    }

                                    ui.add_space(12.0);
                                    ui.label(egui::RichText::new("🟢 Status: Ready & Waiting for incoming connection requests").color(COLOR_SUCCESS).small());
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

                                    ui.label("Remote IP or Connection ID:");
                                    ui.add(egui::TextEdit::singleline(&mut self.remote_target_input).hint_text("e.g. 192.168.1.15 or 482 195 730"));

                                    ui.add_space(12.0);
                                    if ui.button("🚀 Connect to Remote").clicked() && !self.remote_target_input.trim().is_empty() {
                                        let target = self.remote_target_input.trim().to_string();
                                        next_state = Some(AppState::Connecting {
                                            target: target.clone(),
                                            error: None,
                                        });
                                        if let Some(ref tx) = self.action_sender {
                                            let _ = tx.send(UiAction::Connect { target });
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

                    ui.add_space(14.0);
                    draw_about_box(ui);
                    ui.add_space(8.0);
                    draw_footer(ui);
                });
            }
        }

        if let Some(ns) = next_state {
            self.state = ns;
        }

        // Repaint continuously when viewing or hosting for smooth frame rates
        if matches!(self.state, AppState::Viewing { .. } | AppState::Hosting { .. } | AppState::Connecting { .. }) {
            ctx.request_repaint();
        }
    }
}

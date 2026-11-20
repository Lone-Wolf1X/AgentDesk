// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

use crate::app_state::{AppState, IncomingRequest, SessionStats};
use crate::theme::{
    apply_theme, card_frame, alert_card_frame,
    primary_button, secondary_button, danger_button,
    draw_header, draw_footer, draw_about_box,
    COLOR_PRIMARY, COLOR_SUCCESS, COLOR_ERROR,
    COLOR_TEXT, COLOR_TEXT_MUTED, COLOR_BORDER, COLOR_BG,
    COLOR_PRIMARY_LIGHT, COLOR_BORDER_FOCUS,
};
use agentdesk_core::protocol::SessionPermissions;
use agentdesk_core::types::{InputEvent, MouseButton};
use egui::{ColorImage, Frame, Margin, Rounding, Stroke, TextureOptions};
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
    theme_applied: bool,
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
            theme_applied: false,
        }
    }
}

impl eframe::App for AgentDeskApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme once on first render
        if !self.theme_applied {
            apply_theme(ctx);
            self.theme_applied = true;
        }

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
            // ─── VIEWING: Remote session live ────────────────────────────────
            AppState::Viewing {
                remote_addr,
                texture,
                connected_since,
                permissions,
                stats,
            } => {
                // Poll for video frames
                if let Some(ref rx) = self.frame_receiver {
                    while let Ok((w, h, rgba)) = rx.try_recv() {
                        let color_image = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
                        *texture = Some(ctx.load_texture("remote_screen", color_image, TextureOptions::LINEAR));
                        stats.frames_received += 1;
                        stats.bytes_received += rgba.len() as u64;
                    }
                }

                // Top control bar — dark blue strip for contrast in viewing mode
                egui::TopBottomPanel::top("viewing_toolbar")
                    .frame(
                        Frame::none()
                            .fill(COLOR_PRIMARY)
                            .inner_margin(Margin::symmetric(16.0, 8.0)),
                    )
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            // Live dot
                            ui.label(
                                egui::RichText::new("● LIVE")
                                    .color(egui::Color32::from_rgb(239, 68, 68))
                                    .strong()
                                    .size(13.0),
                            );
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!("🖥  {remote_addr}"))
                                    .color(egui::Color32::WHITE)
                                    .size(13.0),
                            );
                            ui.separator();
                            let elapsed = connected_since.elapsed().as_secs();
                            ui.label(
                                egui::RichText::new(format!("⏱  {:02}:{:02}", elapsed / 60, elapsed % 60))
                                    .color(egui::Color32::WHITE)
                                    .size(12.0),
                            );
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!("📦  {} frames", stats.frames_received))
                                    .color(egui::Color32::from_rgb(186, 230, 253))
                                    .size(12.0),
                            );

                            if permissions.keyboard_mouse {
                                ui.separator();
                                ui.label(
                                    egui::RichText::new("🖱 Control Active")
                                        .color(egui::Color32::from_rgb(167, 243, 208))
                                        .size(12.0),
                                );
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let btn = egui::Button::new(
                                    egui::RichText::new("✕  Disconnect")
                                        .color(egui::Color32::WHITE)
                                        .size(13.0)
                                        .strong(),
                                )
                                .fill(egui::Color32::from_rgb(239, 68, 68))
                                .stroke(Stroke::new(1.0_f32, egui::Color32::from_rgb(185, 28, 28)))
                                .rounding(Rounding::same(8.0))
                                .min_size(egui::vec2(130.0, 30.0));

                                if ui.add(btn).clicked() {
                                    if let Some(ref tx) = self.action_sender {
                                        let _ = tx.send(UiAction::Disconnect);
                                    }
                                }
                            });
                        });
                    });

                // Remote screen canvas
                egui::CentralPanel::default()
                    .frame(Frame::none().fill(egui::Color32::from_rgb(10, 15, 30)))
                    .show(ctx, |ui| {
                        if let Some(tex) = texture {
                            let available_size = ui.available_size();
                            let image_response = ui.image(egui::load::SizedTexture::new(tex.id(), available_size));

                            if permissions.keyboard_mouse && image_response.hovered() {
                                let rect = image_response.rect;
                                if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                                    let nx = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                                    let ny = ((pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
                                    let rx = nx as f64 * self.remote_screen_size.0;
                                    let ry = ny as f64 * self.remote_screen_size.1;

                                    if let Some(ref tx) = self.input_sender {
                                        let _ = tx.try_send(InputEvent::MouseMove { x: rx, y: ry });

                                        if ctx.input(|i| i.pointer.primary_clicked()) {
                                            let _ = tx.try_send(InputEvent::MouseDown { button: MouseButton::Left, x: rx, y: ry });
                                            let _ = tx.try_send(InputEvent::MouseUp   { button: MouseButton::Left, x: rx, y: ry });
                                        }
                                        if ctx.input(|i| i.pointer.secondary_clicked()) {
                                            let _ = tx.try_send(InputEvent::MouseDown { button: MouseButton::Right, x: rx, y: ry });
                                            let _ = tx.try_send(InputEvent::MouseUp   { button: MouseButton::Right, x: rx, y: ry });
                                        }
                                    }
                                }
                            }
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(60.0);
                                    ui.spinner();
                                    ui.add_space(12.0);
                                    ui.label(
                                        egui::RichText::new("Waiting for remote display stream…")
                                            .color(egui::Color32::from_rgb(148, 163, 184))
                                            .size(16.0),
                                    );
                                });
                            });
                        }
                    });
            }

            // ─── HOSTING: Sharing screen ─────────────────────────────────────
            AppState::Hosting { viewer_id, connected_since, permissions: _, stats } => {
                egui::CentralPanel::default()
                    .frame(Frame::none().fill(COLOR_BG).inner_margin(Margin::same(0.0)))
                    .show(ctx, |ui| {
                        draw_header(ui, "Active Host Session");
                        ui.add_space(0.0);

                        Frame::none()
                            .fill(COLOR_BG)
                            .inner_margin(Margin::symmetric(32.0, 24.0))
                            .show(ui, |ui| {
                                // Session card
                                card_frame().show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Green pulse dot
                                        Frame::none()
                                            .fill(egui::Color32::from_rgb(209, 250, 229))
                                            .rounding(Rounding::same(20.0))
                                            .inner_margin(Margin::symmetric(10.0, 6.0))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    egui::RichText::new("● Sharing Screen")
                                                        .color(COLOR_SUCCESS)
                                                        .strong()
                                                        .size(14.0),
                                                );
                                            });
                                    });

                                    ui.add_space(16.0);

                                    // Stats grid
                                    egui::Grid::new("host_stats")
                                        .num_columns(2)
                                        .spacing([32.0, 10.0])
                                        .show(ui, |ui| {
                                            ui.label(egui::RichText::new("Connected Viewer").color(COLOR_TEXT_MUTED).size(12.0));
                                            ui.label(egui::RichText::new(viewer_id.as_str()).color(COLOR_TEXT).strong().size(13.0));
                                            ui.end_row();

                                            let elapsed = connected_since.elapsed().as_secs();
                                            ui.label(egui::RichText::new("Session Duration").color(COLOR_TEXT_MUTED).size(12.0));
                                            ui.label(
                                                egui::RichText::new(format!("{:02}:{:02}", elapsed / 60, elapsed % 60))
                                                    .color(COLOR_TEXT).strong().size(13.0),
                                            );
                                            ui.end_row();

                                            ui.label(egui::RichText::new("Frames Streamed").color(COLOR_TEXT_MUTED).size(12.0));
                                            ui.label(
                                                egui::RichText::new(format!("{}", stats.frames_received))
                                                    .color(COLOR_TEXT).size(13.0),
                                            );
                                            ui.end_row();
                                        });

                                    ui.add_space(20.0);
                                    if danger_button(ui, "✕  Stop Sharing & Disconnect").clicked() {
                                        if let Some(ref tx) = self.action_sender {
                                            let _ = tx.send(UiAction::Disconnect);
                                        }
                                    }
                                });
                            });

                        ui.add_space(4.0);
                        draw_footer(ui);
                    });
            }

            // ─── CONNECTING: Spinner / waiting ───────────────────────────────
            AppState::Connecting { target, error } => {
                egui::CentralPanel::default()
                    .frame(Frame::none().fill(COLOR_BG).inner_margin(Margin::same(0.0)))
                    .show(ctx, |ui| {
                        draw_header(ui, "Connecting…");

                        Frame::none()
                            .fill(COLOR_BG)
                            .inner_margin(Margin::symmetric(32.0, 60.0))
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    // Spinner card
                                    card_frame().show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.add_space(20.0);
                                            ui.spinner();
                                            ui.add_space(16.0);
                                            ui.label(
                                                egui::RichText::new(format!("Connecting to {target}"))
                                                    .size(18.0)
                                                    .strong()
                                                    .color(COLOR_TEXT),
                                            );
                                            ui.add_space(6.0);
                                            ui.label(
                                                egui::RichText::new("Waiting for the remote host to accept your request…")
                                                    .color(COLOR_TEXT_MUTED)
                                                    .size(13.0),
                                            );

                                            if let Some(err) = error {
                                                ui.add_space(16.0);
                                                Frame::none()
                                                    .fill(egui::Color32::from_rgb(254, 242, 242))
                                                    .rounding(Rounding::same(8.0))
                                                    .stroke(Stroke::new(1.0_f32, COLOR_ERROR))
                                                    .inner_margin(Margin::same(12.0))
                                                    .show(ui, |ui| {
                                                        ui.label(
                                                            egui::RichText::new(format!("⚠  {err}"))
                                                                .color(COLOR_ERROR)
                                                                .size(13.0),
                                                        );
                                                    });
                                            }

                                            ui.add_space(20.0);
                                            if secondary_button(ui, "✕  Cancel").clicked() {
                                                if let Some(ref tx) = self.action_sender {
                                                    let _ = tx.send(UiAction::Disconnect);
                                                }
                                            }
                                            ui.add_space(20.0);
                                        });
                                    });
                                });
                            });

                        draw_footer(ui);
                    });
            }

            // ─── IDLE: Main dashboard ─────────────────────────────────────────
            AppState::Idle {
                local_ip,
                local_port: _,
                my_id,
                host_listening: _,
                incoming_request,
            } => {
                egui::CentralPanel::default()
                    .frame(Frame::none().fill(COLOR_BG).inner_margin(Margin::same(0.0)))
                    .show(ctx, |ui| {
                        draw_header(ui, "Free & Open Source Remote Desktop");

                        // Scrollable body
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                Frame::none()
                                    .fill(COLOR_BG)
                                    .inner_margin(Margin::symmetric(28.0, 20.0))
                                    .show(ui, |ui| {
                                        // ── Incoming connection alert ──────────────
                                        if let Some(req) = incoming_request {
                                            alert_card_frame().show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new("🔔").size(22.0));
                                                    ui.vertical(|ui| {
                                                        ui.label(
                                                            egui::RichText::new("Incoming Connection Request")
                                                                .strong()
                                                                .size(16.0)
                                                                .color(COLOR_TEXT),
                                                        );
                                                        ui.label(
                                                            egui::RichText::new(format!(
                                                                "{}  wants to view and control your desktop.",
                                                                req.viewer_addr
                                                            ))
                                                            .color(COLOR_TEXT_MUTED)
                                                            .size(13.0),
                                                        );
                                                    });
                                                });

                                                ui.add_space(14.0);
                                                ui.label(
                                                    egui::RichText::new("Allow permissions:")
                                                        .color(COLOR_TEXT_MUTED)
                                                        .size(12.0),
                                                );
                                                ui.add_space(6.0);

                                                // Permission checkboxes in a 3-column grid
                                                egui::Grid::new("perms_grid")
                                                    .num_columns(3)
                                                    .spacing([20.0, 8.0])
                                                    .show(ui, |ui| {
                                                        ui.checkbox(&mut req.permissions.keyboard_mouse, "🖱  Keyboard & Mouse");
                                                        ui.checkbox(&mut req.permissions.clipboard,      "📋  Clipboard Sync");
                                                        ui.checkbox(&mut req.permissions.sound,          "🔊  Audio");
                                                        ui.end_row();
                                                        ui.checkbox(&mut req.permissions.file_transfer,  "📂  File Transfer");
                                                        ui.checkbox(&mut req.permissions.file_manager,   "🗂  File Manager");
                                                        ui.checkbox(&mut req.permissions.restart,        "🔄  Restart PC");
                                                        ui.end_row();
                                                    });

                                                ui.add_space(14.0);
                                                ui.horizontal(|ui| {
                                                    if primary_button(ui, "✅  Accept").clicked() {
                                                        if let Some(ref tx) = self.action_sender {
                                                            let _ = tx.send(UiAction::AcceptIncoming {
                                                                permissions: req.permissions.clone(),
                                                            });
                                                        }
                                                    }
                                                    ui.add_space(8.0);
                                                    let full_btn = egui::Button::new(
                                                        egui::RichText::new("🛡  Full Control").color(egui::Color32::WHITE).size(13.0).strong()
                                                    )
                                                    .fill(COLOR_SUCCESS)
                                                    .rounding(Rounding::same(8.0))
                                                    .min_size(egui::vec2(130.0, 38.0));
                                                    if ui.add(full_btn).clicked() {
                                                        if let Some(ref tx) = self.action_sender {
                                                            let _ = tx.send(UiAction::AcceptIncoming {
                                                                permissions: SessionPermissions::full_control(),
                                                            });
                                                        }
                                                    }
                                                    ui.add_space(8.0);
                                                    if danger_button(ui, "✕  Reject").clicked() {
                                                        if let Some(ref tx) = self.action_sender {
                                                            let _ = tx.send(UiAction::RejectIncoming);
                                                        }
                                                    }
                                                });
                                            });
                                            ui.add_space(20.0);
                                        }

                                        // ── Main two-column layout ─────────────────
                                        let available = ui.available_width();
                                        let col_w = (available - 20.0) / 2.0;

                                        ui.horizontal_top(|ui| {
                                            // LEFT: This Device card
                                            ui.allocate_ui(egui::vec2(col_w, 0.0), |ui| {
                                                card_frame().show(ui, |ui| {
                                                    ui.set_min_width(col_w - 20.0);

                                                    // Card header strip
                                                    Frame::none()
                                                        .fill(COLOR_PRIMARY_LIGHT)
                                                        .rounding(Rounding::same(8.0))
                                                        .inner_margin(Margin::symmetric(12.0, 8.0))
                                                        .show(ui, |ui| {
                                                            ui.label(
                                                                egui::RichText::new("🖥   This Device")
                                                                    .strong()
                                                                    .size(14.0)
                                                                    .color(COLOR_PRIMARY),
                                                            );
                                                        });

                                                    ui.add_space(16.0);

                                                    // 9-Digit AgentDesk ID (No IP exposed - complete privacy)
                                                    let id_str = my_id.clone().unwrap_or_else(|| "---------".to_string());
                                                    let formatted_id = if id_str.len() == 9 {
                                                        format!("{} {} {}", &id_str[0..3], &id_str[3..6], &id_str[6..9])
                                                    } else {
                                                        id_str.clone()
                                                    };

                                                    ui.label(
                                                        egui::RichText::new("Your AgentDesk ID")
                                                            .color(COLOR_TEXT)
                                                            .size(13.0)
                                                            .strong(),
                                                    );
                                                    ui.label(
                                                        egui::RichText::new("Share this 9-digit ID with the remote person to connect")
                                                            .color(COLOR_TEXT_MUTED)
                                                            .size(11.0),
                                                    );
                                                    ui.add_space(4.0);
                                                    ui.horizontal(|ui| {
                                                        Frame::none()
                                                            .fill(COLOR_BG)
                                                            .rounding(Rounding::same(8.0))
                                                            .stroke(Stroke::new(1.5_f32, COLOR_BORDER_FOCUS))
                                                            .inner_margin(Margin::symmetric(16.0, 10.0))
                                                            .show(ui, |ui| {
                                                                ui.label(
                                                                    egui::RichText::new(&formatted_id)
                                                                        .size(26.0)
                                                                        .strong()
                                                                        .color(COLOR_PRIMARY),
                                                                );
                                                            });
                                                        ui.add_space(6.0);
                                                        let copy_btn = egui::Button::new(
                                                            egui::RichText::new("⎘  Copy ID")
                                                                .size(13.0)
                                                                .strong()
                                                                .color(COLOR_PRIMARY)
                                                        )
                                                        .fill(COLOR_PRIMARY_LIGHT)
                                                        .rounding(Rounding::same(8.0))
                                                        .min_size(egui::vec2(90.0, 42.0));
                                                        if ui.add(copy_btn).on_hover_text("Copy your 9-digit ID to clipboard").clicked() {
                                                            ui.output_mut(|o| o.copied_text = id_str.clone());
                                                        }
                                                    });

                                                    ui.add_space(10.0);
                                                    ui.label(
                                                        egui::RichText::new("🔒 Privacy Protected: Your IP is hidden. Direct connection via 9-digit ID.")
                                                            .color(COLOR_TEXT_MUTED)
                                                            .size(11.0),
                                                    );

                                                    ui.add_space(16.0);
                                                    // Status badge
                                                    Frame::none()
                                                        .fill(egui::Color32::from_rgb(209, 250, 229))
                                                        .rounding(Rounding::same(20.0))
                                                        .inner_margin(Margin::symmetric(10.0, 5.0))
                                                        .show(ui, |ui| {
                                                            ui.label(
                                                                egui::RichText::new("● Ready — Waiting for connections")
                                                                    .color(COLOR_SUCCESS)
                                                                    .size(11.0)
                                                                    .strong(),
                                                            );
                                                        });
                                                });
                                            });

                                            ui.add_space(20.0);

                                            // RIGHT: Connect to Remote card
                                            ui.allocate_ui(egui::vec2(col_w, 0.0), |ui| {
                                                card_frame().show(ui, |ui| {
                                                    ui.set_min_width(col_w - 20.0);

                                                    // Card header strip
                                                    Frame::none()
                                                        .fill(egui::Color32::from_rgb(240, 253, 250)) // teal-50
                                                        .rounding(Rounding::same(8.0))
                                                        .inner_margin(Margin::symmetric(12.0, 8.0))
                                                        .show(ui, |ui| {
                                                            ui.label(
                                                                egui::RichText::new("🔗   Connect to Remote")
                                                                    .strong()
                                                                    .size(14.0)
                                                                    .color(egui::Color32::from_rgb(5, 150, 105)), // emerald
                                                            );
                                                        });

                                                    ui.add_space(16.0);
                                                    ui.label(egui::RichText::new("Remote AgentDesk ID").color(COLOR_TEXT).size(13.0).strong());
                                                    ui.label(egui::RichText::new("Enter the 9-digit ID of the computer you want to control").color(COLOR_TEXT_MUTED).size(11.0));
                                                    ui.add_space(4.0);

                                                    // Large input field
                                                    let text_edit = egui::TextEdit::singleline(&mut self.remote_target_input)
                                                        .hint_text("e.g.  482 195 730")
                                                        .font(egui::FontId::proportional(16.0))
                                                        .desired_width(f32::INFINITY)
                                                        .margin(egui::vec2(12.0, 10.0));
                                                    ui.add(text_edit);

                                                    ui.add_space(14.0);

                                                    // Connect button (full width)
                                                    let connect_active = !self.remote_target_input.trim().is_empty();
                                                    let btn_fill = if connect_active { COLOR_PRIMARY } else { egui::Color32::from_rgb(203, 213, 225) };
                                                    let btn_text_color = if connect_active { egui::Color32::WHITE } else { egui::Color32::from_rgb(148, 163, 184) };

                                                    let conn_btn = egui::Button::new(
                                                        egui::RichText::new("🚀   Connect")
                                                            .color(btn_text_color)
                                                            .size(15.0)
                                                            .strong(),
                                                    )
                                                    .fill(btn_fill)
                                                    .rounding(Rounding::same(10.0))
                                                    .min_size(egui::vec2(ui.available_width(), 44.0));

                                                    if ui.add(conn_btn).clicked() && connect_active {
                                                        let target = self.remote_target_input.trim().to_string();
                                                        next_state = Some(AppState::Connecting {
                                                            target: target.clone(),
                                                            error: None,
                                                        });
                                                        if let Some(ref tx) = self.action_sender {
                                                            let _ = tx.send(UiAction::Connect { target });
                                                        }
                                                    }

                                                    // Recent connections
                                                    if !self.recent_connections.is_empty() {
                                                        ui.add_space(16.0);
                                                        ui.label(
                                                            egui::RichText::new("Recent")
                                                                .color(COLOR_TEXT_MUTED)
                                                                .size(11.0),
                                                        );
                                                        ui.add_space(4.0);
                                                        let recent_clone: Vec<String> = self.recent_connections.clone();
                                                        for recent in &recent_clone {
                                                            let rb = egui::Button::new(
                                                                egui::RichText::new(format!("🕐  {recent}"))
                                                                    .color(COLOR_TEXT_MUTED)
                                                                    .size(12.0),
                                                            )
                                                            .fill(egui::Color32::from_rgb(248, 250, 252))
                                                            .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                                                            .rounding(Rounding::same(6.0))
                                                            .min_size(egui::vec2(ui.available_width(), 30.0));
                                                            if ui.add(rb).clicked() {
                                                                self.remote_target_input = recent.clone();
                                                            }
                                                        }
                                                    }
                                                });
                                            });
                                        });

                                        ui.add_space(20.0);
                                        draw_about_box(ui);
                                    });
                            });

                        draw_footer(ui);
                    });
            }
        }

        if let Some(ns) = next_state {
            self.state = ns;
        }

        // Continuous repaint for active sessions
        if matches!(
            self.state,
            AppState::Viewing { .. } | AppState::Hosting { .. } | AppState::Connecting { .. }
        ) {
            ctx.request_repaint();
        }
    }
}

// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

use crate::theme::{draw_footer, draw_header, COLOR_SUCCESS, COLOR_WARNING};
use agentdesk_core::types::{InputEvent, Key, MouseButton};
use egui::{ColorImage, TextureHandle, TextureOptions};
use std::sync::mpsc::{Receiver, Sender};

/// State for the Viewer Interface
pub struct ViewerGuiState {
    pub target_id: String,
    pub password: String,
    pub status: String,
    pub is_connected: bool,
    pub texture: Option<TextureHandle>,
    pub frame_receiver: Option<Receiver<(u32, u32, Vec<u8>)>>,
    pub input_sender: Option<Sender<InputEvent>>,
    pub connect_requested: bool,
    pub disconnect_requested: bool,
}

impl ViewerGuiState {
    pub fn new() -> Self {
        Self {
            target_id: String::new(),
            password: String::new(),
            status: "Enter Host ID to connect".to_string(),
            is_connected: false,
            texture: None,
            frame_receiver: None,
            input_sender: None,
            connect_requested: false,
            disconnect_requested: false,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        // Poll for incoming decoded video frames
        if let Some(ref rx) = self.frame_receiver {
            while let Ok((w, h, rgba)) = rx.try_recv() {
                let color_image = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
                self.texture = Some(ctx.load_texture("remote_screen", color_image, TextureOptions::LINEAR));
            }
        }

        egui::TopBottomPanel::top("viewer_toolbar").show(ctx, |ui| {
            draw_header(ui, "Viewer Client");

            ui.horizontal(|ui| {
                if !self.is_connected {
                    ui.label("Remote ID / IP:");
                    ui.text_edit_singleline(&mut self.target_id);

                    ui.label("Password:");
                    ui.text_edit_singleline(&mut self.password);

                    if ui.button("🔗 Connect").clicked() {
                        self.connect_requested = true;
                    }
                } else {
                    ui.label(format!("Connected to: {}", self.target_id));
                    if ui.button("❌ Disconnect").clicked() {
                        self.disconnect_requested = true;
                    }
                }

                ui.separator();
                let color = if self.is_connected {
                    COLOR_SUCCESS
                } else {
                    COLOR_WARNING
                };
                ui.label(egui::RichText::new(&self.status).color(color));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref texture) = self.texture {
                let available_size = ui.available_size();
                let image_response = ui.image(egui::load::SizedTexture::new(texture.id(), available_size));

                // Capture mouse inputs when interacting with the remote viewport
                if image_response.hovered() {
                    let rect = image_response.rect;
                    if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                        let normalized_x = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                        let normalized_y = ((pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);

                        if let Some(ref tx) = self.input_sender {
                            // Emit mouse move
                            let _ = tx.send(InputEvent::MouseMove {
                                x: normalized_x as f64 * 1920.0,
                                y: normalized_y as f64 * 1080.0,
                            });

                            // Emit clicks
                            if ctx.input(|i| i.pointer.primary_clicked()) {
                                let _ = tx.send(InputEvent::MouseDown {
                                    button: MouseButton::Left,
                                    x: normalized_x as f64 * 1920.0,
                                    y: normalized_y as f64 * 1080.0,
                                });
                                let _ = tx.send(InputEvent::MouseUp {
                                    button: MouseButton::Left,
                                    x: normalized_x as f64 * 1920.0,
                                    y: normalized_y as f64 * 1080.0,
                                });
                            }
                        }
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new("Awaiting Remote Display Stream...").size(20.0));
                });
            }
        });

        egui::TopBottomPanel::bottom("viewer_footer").show(ctx, |ui| {
            draw_footer(ui);
        });
    }
}

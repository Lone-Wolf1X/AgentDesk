// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

use egui::IconData;

/// Generates a crisp, branded 64x64 RGBA icon for AgentDesk:
/// Dark rounded emblem with Nepal Crimson `#DC143C` border, monitor silhouette,
/// and glowing remote connection signals.
pub fn generate_icon_rgba(size: u32) -> (u32, u32, Vec<u8>) {
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let center = size as f32 / 2.0;
    let radius = size as f32 / 2.0 - 2.0;

    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            // Circular badge with anti-aliasing
            if dist <= radius {
                let border_dist = radius - dist;
                let is_border = border_dist < 3.5;

                // Base emblem color
                let (mut r, mut g, mut b, mut a) = if is_border {
                    (220, 20, 60, 255) // Nepal Crimson Red (#DC143C)
                } else {
                    let grad = ((y as f32 / size as f32) * 20.0) as u8;
                    (24 + grad, 24 + grad, 32 + grad, 255) // Dark modern slate
                };

                // Draw central monitor / remote desktop glyph
                let norm_x = (x as f32 - center) / (size as f32 * 0.4);
                let norm_y = (y as f32 - center) / (size as f32 * 0.4);

                // Screen frame: [-0.7..0.7, -0.6..0.4]
                let in_screen_outer = norm_x >= -0.75 && norm_x <= 0.75 && norm_y >= -0.65 && norm_y <= 0.35;
                let in_screen_inner = norm_x >= -0.62 && norm_x <= 0.62 && norm_y >= -0.52 && norm_y <= 0.22;

                // Stand: norm_y between 0.35 and 0.65
                let in_stand = norm_x.abs() <= 0.12 && norm_y >= 0.35 && norm_y <= 0.6;
                let in_base = norm_x.abs() <= 0.38 && norm_y >= 0.55 && norm_y <= 0.68;

                if in_screen_outer && !in_screen_inner {
                    r = 240;
                    g = 240;
                    b = 250; // White monitor bezel
                } else if in_stand || in_base {
                    r = 200;
                    g = 200;
                    b = 210;
                } else if in_screen_inner {
                    // Inside screen: Draw remote signal wave / Nepal double triangle in Crimson & Gold
                    let is_wave_1 = (norm_x * norm_x + (norm_y + 0.15) * (norm_y + 0.15)).sqrt() < 0.22;
                    let is_bolt = (norm_x * 1.5 - norm_y).abs() < 0.12 && norm_x >= -0.35 && norm_x <= 0.35;

                    if is_wave_1 {
                        r = 255;
                        g = 215;
                        b = 0; // Gold
                    } else if is_bolt {
                        r = 220;
                        g = 20;
                        b = 60; // Crimson
                    } else {
                        r = 16;
                        g = 18;
                        b = 24; // Deep display background
                    }
                }

                // Smooth outer edge anti-aliasing
                if border_dist < 1.0 {
                    a = (border_dist * 255.0).clamp(0.0, 255.0) as u8;
                }

                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = a;
            }
        }
    }

    (size, size, rgba)
}

/// Returns the egui IconData to set as window and taskbar icon.
pub fn get_app_icon() -> IconData {
    let (width, height, rgba) = generate_icon_rgba(64);
    IconData {
        rgba,
        width,
        height,
    }
}

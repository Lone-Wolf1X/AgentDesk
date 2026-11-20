// AgentDesk — Installation Wizard for Windows
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// Made in Nepal 🇳🇵

use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

static AGENTDESK_EXE: &[u8] = include_bytes!("../../../target/release/agentdesk.exe");
static ICON_ICO: &[u8] = include_bytes!("../../../assets/icon.ico");
static ICON_PNG: &[u8] = include_bytes!("../../../assets/icon.png");

#[derive(PartialEq, Clone, Copy)]
enum WizardStep {
    Welcome,
    Options,
    Installing,
    Finished,
    Error,
}

struct InstallerApp {
    step: WizardStep,
    install_path: String,
    create_desktop_shortcut: bool,
    create_start_menu_shortcut: bool,
    launch_after_install: bool,
    error_message: Option<String>,
    status_text: String,
}

impl Default for InstallerApp {
    fn default() -> Self {
        let default_dir = if let Ok(local) = std::env::var("LOCALAPPDATA") {
            format!(r"{}\Programs\AgentDesk", local)
        } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
            format!(r"{}\AppData\Local\Programs\AgentDesk", userprofile)
        } else {
            r"C:\AgentDesk".to_string()
        };

        Self {
            step: WizardStep::Welcome,
            install_path: default_dir,
            create_desktop_shortcut: true,
            create_start_menu_shortcut: true,
            launch_after_install: true,
            error_message: None,
            status_text: String::new(),
        }
    }
}

impl InstallerApp {
    fn perform_installation(&mut self) -> Result<(), String> {
        let dest = PathBuf::from(&self.install_path);
        fs::create_dir_all(&dest)
            .map_err(|e| format!("Failed to create destination folder: {e}"))?;

        let exe_path = dest.join("agentdesk.exe");
        fs::write(&exe_path, AGENTDESK_EXE)
            .map_err(|e| format!("Failed to write agentdesk.exe: {e}"))?;

        let ico_path = dest.join("icon.ico");
        let _ = fs::write(&ico_path, ICON_ICO);

        let png_path = dest.join("icon.png");
        let _ = fs::write(&png_path, ICON_PNG);

        let exe_str = exe_path.to_string_lossy().replace('\\', "\\\\");
        let ico_str = ico_path.to_string_lossy().replace('\\', "\\\\");
        let dest_str = dest.to_string_lossy().replace('\\', "\\\\");

        // Create Desktop Shortcut
        if self.create_desktop_shortcut {
            if let Ok(userprofile) = std::env::var("USERPROFILE") {
                let desktop_lnk = format!(r"{}\Desktop\AgentDesk.lnk", userprofile).replace('\\', "\\\\");
                let ps_script = format!(
                    "$ws = New-Object -ComObject WScript.Shell; \
                     $s = $ws.CreateShortcut('{}'); \
                     $s.TargetPath = '{}'; \
                     $s.IconLocation = '{}'; \
                     $s.WorkingDirectory = '{}'; \
                     $s.Save()",
                    desktop_lnk, exe_str, ico_str, dest_str
                );
                let _ = Command::new("powershell")
                    .args(["-NoProfile", "-Command", &ps_script])
                    .output();
            }
        }

        // Create Start Menu Shortcut
        if self.create_start_menu_shortcut {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let start_lnk = format!(r"{}\Microsoft\Windows\Start Menu\Programs\AgentDesk.lnk", appdata).replace('\\', "\\\\");
                let ps_script = format!(
                    "$ws = New-Object -ComObject WScript.Shell; \
                     $s = $ws.CreateShortcut('{}'); \
                     $s.TargetPath = '{}'; \
                     $s.IconLocation = '{}'; \
                     $s.WorkingDirectory = '{}'; \
                     $s.Save()",
                    start_lnk, exe_str, ico_str, dest_str
                );
                let _ = Command::new("powershell")
                    .args(["-NoProfile", "-Command", &ps_script])
                    .output();
            }
        }

        // Create Uninstaller script
        let uninstaller_path = dest.join("uninstall.bat");
        let uninstaller_content = format!(
            "@echo off\r\n\
             echo Uninstalling AgentDesk...\r\n\
             taskkill /F /IM agentdesk.exe >nul 2>&1\r\n\
             del /Q \"%USERPROFILE%\\Desktop\\AgentDesk.lnk\" >nul 2>&1\r\n\
             del /Q \"%APPDATA%\\Microsoft\\Windows\\Start Menu\\Programs\\AgentDesk.lnk\" >nul 2>&1\r\n\
             reg delete \"HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\AgentDesk\" /f >nul 2>&1\r\n\
             echo AgentDesk files uninstalled.\r\n\
             pause\r\n"
        );
        let _ = fs::write(&uninstaller_path, uninstaller_content);

        // Register in Windows Add/Remove Programs (Registry)
        let _ = Command::new("reg")
            .args(["add", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\AgentDesk", "/v", "DisplayName", "/d", "AgentDesk", "/f"])
            .output();
        let _ = Command::new("reg")
            .args(["add", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\AgentDesk", "/v", "DisplayVersion", "/d", "0.1.0", "/f"])
            .output();
        let _ = Command::new("reg")
            .args(["add", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\AgentDesk", "/v", "Publisher", "/d", "Next Gen Innovations Private Limited", "/f"])
            .output();
        let _ = Command::new("reg")
            .args(["add", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\AgentDesk", "/v", "DisplayIcon", "/d", &exe_path.to_string_lossy(), "/f"])
            .output();
        let _ = Command::new("reg")
            .args(["add", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\AgentDesk", "/v", "UninstallString", "/d", &uninstaller_path.to_string_lossy(), "/f"])
            .output();

        Ok(())
    }
}

impl eframe::App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🇳🇵").size(24.0));
                ui.heading(
                    egui::RichText::new("AgentDesk Setup Wizard")
                        .size(20.0)
                        .color(egui::Color32::from_rgb(220, 20, 60))
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("Next Gen Innovations").small().color(egui::Color32::LIGHT_GRAY));
                });
            });
            ui.separator();
            ui.add_space(10.0);

            match self.step {
                WizardStep::Welcome => {
                    ui.heading("Welcome to the AgentDesk Setup Wizard");
                    ui.add_space(12.0);

                    ui.label(
                        "This will install AgentDesk v0.1.0 on your computer.\n\n\
                         AgentDesk is a 100% Free and Open Source Remote Desktop application,\n\
                         crafted in Nepal 🇳🇵 by Next Gen Innovations Private Limited.\n\n\
                         • Free & Open Source (MIT License)\n\
                         • No time limits, no device count limits, no subscriptions\n\
                         • No account required, no telemetry, direct P2P connections\n\n\
                         Developer & Maintainer: Abhishek Kumar Paswan\n\
                         Website: https://nextgeninnovations.com.np/\n\n\
                         Click 'Next' to choose installation options.",
                    );

                    ui.add_space(20.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new(" Next > ").size(15.0).strong()).clicked() {
                            self.step = WizardStep::Options;
                        }
                    });
                }

                WizardStep::Options => {
                    ui.heading("Select Destination & Tasks");
                    ui.add_space(10.0);

                    ui.label("Install Folder:");
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.install_path).desired_width(450.0));
                    });

                    ui.add_space(15.0);
                    ui.label(egui::RichText::new("Shortcuts:").strong());
                    ui.checkbox(&mut self.create_desktop_shortcut, "Create a Desktop shortcut");
                    ui.checkbox(&mut self.create_start_menu_shortcut, "Create a Start Menu shortcut");

                    ui.add_space(25.0);
                    ui.horizontal(|ui| {
                        if ui.button("< Back").clicked() {
                            self.step = WizardStep::Welcome;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(egui::RichText::new(" Install ").size(15.0).strong()).clicked() {
                                self.step = WizardStep::Installing;
                                self.status_text = "Installing files and registering shortcuts...".to_string();
                            }
                        });
                    });
                }

                WizardStep::Installing => {
                    ui.heading("Installing AgentDesk...");
                    ui.add_space(20.0);
                    ui.label(&self.status_text);
                    ui.add_space(15.0);
                    ui.spinner();

                    // Perform install
                    match self.perform_installation() {
                        Ok(()) => {
                            self.step = WizardStep::Finished;
                        }
                        Err(err) => {
                            self.error_message = Some(err);
                            self.step = WizardStep::Error;
                        }
                    }
                }

                WizardStep::Finished => {
                    ui.heading(egui::RichText::new("Installation Complete! 🎉").color(egui::Color32::from_rgb(46, 204, 113)));
                    ui.add_space(15.0);

                    ui.label(
                        "AgentDesk has been successfully installed on your computer.\n\n\
                         You can now start remote controlling other machines or share your screen,\n\
                         just like AnyDesk / TeamViewer with complete privacy and zero restrictions.",
                    );

                    ui.add_space(15.0);
                    ui.checkbox(&mut self.launch_after_install, "Launch AgentDesk now");

                    ui.add_space(25.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new(" Finish ").size(15.0).strong()).clicked() {
                            if self.launch_after_install {
                                let exe = Path::new(&self.install_path).join("agentdesk.exe");
                                let _ = Command::new(exe).spawn();
                            }
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                }

                WizardStep::Error => {
                    ui.heading(egui::RichText::new("Installation Failed").color(egui::Color32::RED));
                    ui.add_space(10.0);

                    if let Some(ref err) = self.error_message {
                        ui.label(format!("An error occurred during installation:\n\n{err}"));
                    }

                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if ui.button("< Back").clicked() {
                            self.step = WizardStep::Options;
                        }
                        if ui.button("Close").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                }
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let icon_data = if let Ok(img) = image::load_from_memory(ICON_PNG) {
        let resized = img.resize(64, 64, image::imageops::FilterType::Lanczos3);
        let rgba = resized.to_rgba8();
        let (width, height) = rgba.dimensions();
        egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        }
    } else {
        egui::IconData {
            rgba: vec![220, 20, 60, 255].repeat(64 * 64),
            width: 64,
            height: 64,
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([580.0, 420.0])
            .with_resizable(false)
            .with_icon(icon_data)
            .with_title("AgentDesk Setup Wizard — Next Gen Innovations 🇳🇵"),
        ..Default::default()
    };

    eframe::run_native(
        "AgentDesk Setup",
        native_options,
        Box::new(|_cc| Ok(Box::new(InstallerApp::default()))),
    )
}

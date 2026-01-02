use std::{fs, path::PathBuf};

use crate::cpu::Cpu;
use eframe::egui::{self, Color32, Event};
use tracing::{Level, event};

pub struct GameSelect {
    pub filepaths: Vec<PathBuf>,
    pub selected_game: Option<PathBuf>,
}

impl GameSelect {
    pub fn new(folder: PathBuf) -> Self {
        let mut filepaths = Vec::new();
        for filepath in folder.read_dir().unwrap().flatten() {
            filepaths.push(filepath.path());
        }
        Self {
            filepaths,
            selected_game: None,
        }
    }
}

pub struct MyApp {
    cpu: Option<Cpu>,
    paused: bool,
    tty_output: bool,
    game_select: GameSelect,
    screen_texture: egui::TextureHandle,
    frame_buffer: [Color32; 524288],
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>, folder: PathBuf) -> Self {
        Self {
            cpu: None,
            paused: false,
            tty_output: true,
            game_select: GameSelect::new(folder),
            screen_texture: cc.egui_ctx.load_texture(
                "Noise",
                egui::ColorImage::example(),
                egui::TextureOptions::NEAREST,
            ),
            frame_buffer: [Color32::WHITE; 524288],
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(cpu) = &mut self.cpu {
            if !self.paused {
                cpu.step_instruction();

                if self.tty_output {
                    cpu.check_for_tty_output();
                }
            }

            // user input
            ctx.input(|i| {
                for event in &i.events {
                    match event {
                        Event::Key {
                            key: egui::Key::Escape,
                            ..
                        } => std::process::exit(0),
                        Event::Key {
                            key: egui::Key::P, ..
                        } => self.paused = !self.paused,
                        _ => {}
                    }
                }
            });

            if cpu.bus.gpu.frame_is_ready {
                // render frame to screen, get user input, etc

                cpu.bus.gpu.render_vram(&mut self.frame_buffer);

                self.screen_texture.set(
                    egui::ColorImage {
                        size: [1024, 512],
                        source_size: egui::Vec2 {
                            x: 1024.0,
                            y: 512.0,
                        },
                        pixels: self.frame_buffer.to_vec(),
                    },
                    egui::TextureOptions::NEAREST,
                );
            }

            ctx.request_repaint();
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ctx.input(|i| {
                    for event in &i.events {
                        match event {
                            Event::Key {
                                key: egui::Key::Escape,
                                ..
                            } => std::process::exit(0),
                            _ => {}
                        }
                    }
                });

                if let Some(game) = &self.game_select.selected_game {
                    // Load BIOS from folder
                    let bios_path = match fs::read_dir("bios/").unwrap().next() {
                        Some(Ok(path)) => path.path(),
                        _ => panic!("BIOS not found"),
                    };

                    let bios = fs::read(bios_path).unwrap();
                    // Create CPU
                    let mut cpu = Cpu::new();

                    // Load BIOS
                    event!(Level::INFO, "BIOS size is {:08X}", bios.len());
                    cpu.load_bios(&bios);

                    // Load exe
                    let exe = fs::read(game).unwrap();
                    event!(Level::INFO, "Exe size (including header): {:08X}", exe.len());
                    cpu.sideload_exe(&exe, self.tty_output);

                    self.cpu = Some(cpu);
                } else {
                    // Offer game selection option
                    egui::ComboBox::from_label("Select a Game: ").show_ui(ui, |ui| {
                        for file in &self.game_select.filepaths {
                            ui.selectable_value(
                                &mut self.game_select.selected_game,
                                Some(file.clone()),
                                file.to_string_lossy(),
                            );
                        }
                    });
                }
            });
        };
    }
}

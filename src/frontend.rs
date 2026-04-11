use std::{fs, path::PathBuf, time::Instant};

use crate::cpu::Cpu;
use crate::tracing_setup;
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
    cpu: Cpu,
    cpu_rom_loaded: bool,
    paused: bool,
    //reload_handle: Handle<Filtered<Filtered<Layer<..., ..., ..., ...>, ..., ...>, ..., ...>, ...>,
    tty_output: bool,
    game_select: GameSelect,
    screen_texture: egui::TextureHandle,
    frame_buffer: [Color32; 524288],
    tracing_start_pc: u32,
    logging_enabled: bool,
    timing_baseline: Instant,
    frame_count: usize,
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>, folder: PathBuf, tracing_start_pc: u32) -> Self {
        Self {
            cpu: Cpu::new(),
            cpu_rom_loaded: false,
            paused: false,
            //reload_handle,
            tty_output: true,
            game_select: GameSelect::new(folder),
            screen_texture: cc.egui_ctx.load_texture(
                "Noise",
                egui::ColorImage::example(),
                egui::TextureOptions::NEAREST,
            ),
            frame_buffer: [Color32::WHITE; 524288],
            tracing_start_pc,
            logging_enabled: false,
            timing_baseline: Instant::now(),
            frame_count: 0,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Run CPU and associated steps
        if self.cpu_rom_loaded {
            while !self.paused && !self.cpu.bus.gpu.frame_is_ready {
                if !self.logging_enabled
                    && self.cpu.registers.program_counter == self.tracing_start_pc
                {
                    println!("Begin logging...");
                    self.logging_enabled = true;
                    tracing_setup::init_tracing();
                }

                self.cpu.step_instruction();

                if self.tty_output {
                    self.cpu.check_for_tty_output();
                }
            }

            //user input
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

            if self.cpu.bus.gpu.frame_is_ready {
                event!(Level::TRACE, "Frame is ready");
                // render frame to screen, get user input, etc

                self.cpu.bus.gpu.render_vram(&mut self.frame_buffer);

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

                self.cpu.bus.gpu.frame_is_ready = false;
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

                    // Load BIOS
                    println!("BIOS size is {:08X}", bios.len());
                    self.cpu.load_bios(&bios);

                    // Load exe
                    let exe = fs::read(game).unwrap();
                    println!("Exe size (including header): {:08X}", exe.len());
                    //println!("At 0x00040CE8 is 0x{:02X}{:02X}{:02X}{:02X}", exe[])
                    // Runs CPU until exe can be loaded
                    self.cpu.sideload_exe(&exe, self.tty_output);

                    self.cpu_rom_loaded = true;
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

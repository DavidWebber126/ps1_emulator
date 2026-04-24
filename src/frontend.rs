use std::{fs, path::PathBuf, time::Instant};

use crate::cpu::Cpu;
use crate::tracing_setup;
use eframe::egui::{self, Event, RichText};

//use tracing::{Level, event};

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
    tty_output: bool,
    game_select: GameSelect,
    screen_texture: egui::TextureHandle,
    tracing_start_pc: Option<u32>,
    logging_enabled: bool,
    timing_baseline: Instant,
    frame_count: usize,
    fps: f32,
}

impl MyApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        folder: PathBuf,
        tty_output: bool,
        tracing_start_pc: Option<u32>,
    ) -> Self {
        Self {
            cpu: Cpu::new(),
            cpu_rom_loaded: false,
            paused: false,
            tty_output,
            game_select: GameSelect::new(folder),
            screen_texture: cc.egui_ctx.load_texture(
                "Noise",
                egui::ColorImage::example(),
                egui::TextureOptions::NEAREST,
            ),
            tracing_start_pc,
            logging_enabled: false,
            timing_baseline: Instant::now(),
            frame_count: 0,
            fps: 0.0,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Run CPU and associated steps
        if self.cpu_rom_loaded {
            while !self.paused && !self.cpu.bus.gpu.frame_is_ready {
                if let Some(tracing_pc) = self.tracing_start_pc
                    && !self.logging_enabled
                    && tracing_pc == self.cpu.registers.program_counter
                {
                    println!("Begin logging...");
                    self.logging_enabled = true;
                    tracing_setup::init_tracing();
                }

                // if self.logging_enabled {
                //     event!(Level::TRACE, "In While Loop");
                // }

                self.cpu.step_instruction(self.tty_output);
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
                            key: egui::Key::P, 
                            pressed: true,
                            ..
                        } => {
                            self.paused = !self.paused;
                        }
                        _ => {}
                    }
                }
            });

            // Frame Timings
            if self.frame_count == 0 {
                // let frame_time = self.timing_baseline.elapsed().as_secs_f32();
                // self.fps = frame_time;
                self.frame_count = 1;
                self.timing_baseline = Instant::now();
            } else if self.frame_count == 5 {
                let five_frame_time = self.timing_baseline.elapsed().as_secs_f32();
                self.frame_count = 1;
                self.timing_baseline = Instant::now();
                if !self.paused {
                    self.fps = 5.0 / five_frame_time;
                }
            }

            self.frame_count += 1;

            let pixel_bytes = bytemuck::cast_slice(&(*self.cpu.bus.gpu.gp0.vram));
            self.screen_texture.set(
                /*
                egui::ColorImage {
                    size: [1024, 512],
                    source_size: egui::Vec2 {
                        x: 1024.0,
                        y: 512.0,
                    },
                    pixels: &self.cpu.bus.gpu.gp0.vram[0..524288],
                },
                */
                egui::ColorImage::from_rgba_unmultiplied([1024, 512], pixel_bytes),
                egui::TextureOptions::NEAREST,
            );

            self.cpu.bus.gpu.frame_is_ready = false;

            let sized_texture =
                egui::load::SizedTexture::new(self.screen_texture.id(), [1024.0, 512.0]);

            // Render current frame

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading(RichText::new(format!("FPS is {}", self.fps)));

                ui.add(
                    egui::Image::new(sized_texture).fit_to_exact_size(egui::vec2(1024.0, 512.0)),
                );
            });

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

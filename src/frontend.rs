use crate::cpu::Cpu;
use crate::opcodes;
use crate::opcodes::Op;
use crate::tracing_setup;
use eframe::egui::{self, Event, RichText};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::thread::JoinHandle;
use std::{fs, path::PathBuf, thread, time::Instant};

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
        filepaths.sort();
        Self {
            filepaths,
            selected_game: None,
        }
    }
}

pub struct MyApp {
    cpu: Cpu,
    cpu_rom_loaded: bool,
    play_bios: bool,
    paused: bool,
    tty_output: bool,
    game_select: GameSelect,
    screen_texture: egui::TextureHandle,
    //tracing_start_pc: Option<u32>,
    logging_enabled: bool,
    logging_thread: Option<JoinHandle<()>>,
    breakpoint: BreakpointDebug,
    timing_baseline: Instant,
    frame_count: usize,
    fps: f32,
}

impl MyApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        folder: PathBuf,
        tty_output: bool,
        breakpoint: BreakpointDebug,
    ) -> Self {
        Self {
            cpu: Cpu::new(),
            cpu_rom_loaded: false,
            play_bios: false,
            paused: false,
            tty_output,
            game_select: GameSelect::new(folder),
            screen_texture: cc.egui_ctx.load_texture(
                "Noise",
                egui::ColorImage::example(),
                egui::TextureOptions::NEAREST,
            ),
            logging_thread: None,
            breakpoint,
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
                if self.breakpoint.enabled
                    && !self.logging_enabled
                    && self.breakpoint.addr == self.cpu.registers.program_counter
                {
                    self.breakpoint.visit_counter += 1;
                    if self.breakpoint.visit_counter >= self.breakpoint.record_after {
                        println!("Begin logging...");
                        self.logging_enabled = true;
                        self.logging_thread = Some(init_logging());
                    }
                }

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
                        Event::Key {
                            key: egui::Key::L,
                            pressed: true,
                            ..
                        } if self.paused => {
                            println!("PC is 0x{:08X}", self.cpu.registers.program_counter);
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

            let vram_bytes = &self.cpu.bus.gpu.render_vram()[..];
            let sized_texture = if self.cpu.bus.gpu.gp1.color_depth {
                // VRAM in 24 bit mode.
                self.screen_texture.set(
                    egui::ColorImage::from_rgb([682, 512], vram_bytes),
                    egui::TextureOptions::NEAREST,
                );

                egui::load::SizedTexture::new(self.screen_texture.id(), [682.0, 512.0])
            } else {
                self.screen_texture.set(
                    egui::ColorImage::from_rgb([1024, 512], vram_bytes),
                    egui::TextureOptions::NEAREST,
                );

                egui::load::SizedTexture::new(self.screen_texture.id(), [1024.0, 512.0])
            };

            self.cpu.bus.gpu.frame_is_ready = false;

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
                        if let Event::Key {
                            key: egui::Key::Escape,
                            ..
                        } = event
                        {
                            std::process::exit(0)
                        }
                    }
                });

                if self.play_bios || self.game_select.selected_game.is_some() {
                    // Load BIOS from folder
                    let bios_path = match fs::read_dir("bios/").unwrap().next() {
                        Some(Ok(path)) => path.path(),
                        _ => panic!("BIOS not found"),
                    };

                    let bios = fs::read(bios_path).unwrap();

                    // Load BIOS
                    println!("BIOS size is {:08X}", bios.len());
                    self.cpu.load_bios(&bios);

                    if let Some(game) = &self.game_select.selected_game {
                        // Load exe
                        let exe = fs::read(game).unwrap();
                        println!("Exe size (including header): {:08X}", exe.len());

                        // Runs CPU until exe can be loaded
                        self.cpu.sideload_exe(&exe, self.tty_output);
                    }

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

                    ui.checkbox(&mut self.play_bios, "Play BIOS");
                }
            });
        };
    }
}

pub struct BreakpointDebug {
    enabled: bool,
    addr: u32,
    visit_counter: u32,
    record_after: u32,
    //registers: [Option<u32>; 32],
}

impl BreakpointDebug {
    pub fn new(enabled: bool, addr: u32, record_after: u32) -> Self {
        BreakpointDebug {
            enabled,
            addr,
            visit_counter: 0,
            record_after,
        }
    }
}

fn init_logging() -> JoinHandle<()> {
    let rx = tracing_setup::init_tracing();

    let worker = thread::spawn(move || {
        let log_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("logs/dbg.log")
            .unwrap_or(File::create("logs/dbg.log").unwrap());

        let mut writer = BufWriter::new(log_file);
        for entry in rx {
            if entry.target.contains("ASM") {
                let asm = asm_log(entry.pc as u32, entry.opcode as u32);
                let reg = format_regs(entry.regs);
                let _ = writeln!(
                    writer,
                    "{} {:<20}  PC: {:08X} {}",
                    entry.target, asm, entry.pc, reg
                );
            } else {
                let _ = writeln!(writer, "{} {}", entry.target, entry.msg);
            }
        }
        writer.flush().unwrap();
    });

    worker
}

fn format_regs(regs: [u32; 31]) -> String {
    let mut fmt = String::new();
    for (i, val) in regs.iter().enumerate() {
        fmt.push_str(&format!("r{:02}:{:08X}", i+1, val));
        if i != 30 {
            fmt.push_str(" ")
        }
    }
    fmt
}
fn asm_log(pc: u32, op: u32) -> String {
    let opcode = opcodes::parse_opcode(op);
    match opcode.op {
        Op::ADDI => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let imm = opcode.imm();
            format!("ADDI ${rt}, ${rs}, {:04X}", imm)
        }
        Op::ADDIU => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let imm = opcode.imm();
            format!("ADDIU ${rt}, ${rs}, {:04X}", imm)
        }
        Op::ANDI => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let imm = opcode.imm();
            format!("ANDI ${rt}, ${rs}, {:04X}", imm)
        }
        Op::BEQ => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let imm = opcode.imm();
            format!("BEQ ${rs}, ${rt}, {:04X}", imm)
        }
        Op::BGEZ => {
            let rs = opcode.rs();
            let imm = opcode.imm();
            format!("BGEZ ${rs}, {:04X}", imm)
        }
        Op::BGEZAL => {
            let rs = opcode.rs();
            let imm = opcode.imm();
            format!("BGEZAL ${rs}, {:04X}", imm)
        }
        Op::BLTZ => {
            let rs = opcode.rs();
            let imm = opcode.imm();
            format!("BLTZ ${rs}, {:04X}", imm)
        }
        Op::BLTZAL => {
            let rs = opcode.rs();
            let imm = opcode.imm();
            format!("BLTZAL ${rs}, {:04X}", imm)
        }
        Op::BGTZ => {
            let rs = opcode.rs();
            let imm = opcode.imm();
            format!("BGTZ ${rs}, {:04X}", imm)
        }
        Op::BLEZ => {
            let rs = opcode.rs();
            let imm = opcode.imm();
            format!("BLEZ ${rs}, {:04X}", imm)
        }
        Op::BNE => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let imm = opcode.imm();
            format!("BNE ${rs}, ${rt}, {:04X}", imm)
        }
        Op::JUMP => {
            let target = opcode.target();
            let calc_target = (pc & 0xF0000000) | (target << 2);
            format!("JUMP {:08X}", calc_target)
        }
        Op::JAL => {
            let target = opcode.target();
            let calc_target = (pc & 0xF0000000) | (target << 2);
            format!("JAL {:08X}", calc_target)
        }
        Op::LB => {
            let base = opcode.rs();
            let rt = opcode.rt();
            let offset = opcode.imm();
            format!("LB ${rt}, {:04X}(${:02})", offset, base)
        }
        Op::LBU => {
            let base = opcode.rs();
            let rt = opcode.rt();
            let offset = opcode.imm();
            format!("LBU ${rt}, {:04X}(${:02})", offset, base)
        }
        Op::LH => {
            let base = opcode.rs();
            let rt = opcode.rt();
            let offset = opcode.imm();
            format!("LH ${rt}, {:04X}(${:02})", offset, base)
        }
        Op::LHU => {
            let base = opcode.rs();
            let rt = opcode.rt();
            let offset = opcode.imm();
            format!("LHU ${rt}, {:04X}(${:02})", offset, base)
        }
        Op::LUI => {
            let rt = opcode.rt();
            let imm = opcode.imm();
            format!("LUI ${rt}, {:04X}", imm)
        }
        Op::LW => {
            let base = opcode.rs();
            let rt = opcode.rt();
            let offset = opcode.imm();
            format!("LW ${rt}, {:04X}(${:02})", offset, base)
        }
        Op::LWL => {
            let base = opcode.rs();
            let rt = opcode.rt();
            let offset = opcode.imm();
            format!("LWL ${rt}, {:04X}(${:02})", offset, base)
        }
        Op::LWR => {
            let base = opcode.rs();
            let rt = opcode.rt();
            let offset = opcode.imm();
            format!("LWR ${rt}, {:04X}(${:02})", offset, base)
        }
        Op::ORI => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let imm = opcode.imm();
            format!("ORI ${rt}, ${rs}, {:04X}", imm)
        }
        Op::SB => {
            let base = opcode.rs();
            let rt = opcode.rt();
            let offset = opcode.imm();
            format!("SB ${rt}, {:04X}(${:02})", offset, base)
        }
        Op::SH => {
            let base = opcode.rs();
            let rt = opcode.rt();
            let offset = opcode.imm();
            format!("SH ${rt}, {:04X}(${:02})", offset, base)
        }
        Op::SLTI => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let imm = opcode.imm();
            format!("SLTI ${rt}, ${rs}, {:04X}", imm)
        }
        Op::SLTIU => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let imm = opcode.imm();
            format!("SLTI ${rt}, ${rs}, {:04X}", imm)
        }
        Op::SW => {
            let base = opcode.rs();
            let rt = opcode.rt();
            let offset = opcode.imm();
            format!("SW ${rt}, {:04X}(${:02})", offset, base)
        }
        Op::SWL => {
            let base = opcode.rs();
            let rt = opcode.rt();
            let offset = opcode.imm();
            format!("SWL ${rt}, {:04X}(${:02})", offset, base)
        }
        Op::SWR => {
            let base = opcode.rs();
            let rt = opcode.rt();
            let offset = opcode.imm();
            format!("SH ${rt}, {:04X}(${:02})", offset, base)
        }
        Op::XORI => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let imm = opcode.imm();
            format!("XORI ${rt}, ${rs}, {:04X}", imm)
        }
        Op::CFC2 => {
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("CFC2 ${rt}, ${rd}")
        }
        Op::RFE => "COP0 RFE".to_string(),
        Op::TLB => "COP0 TLBP/TLBR/TLBWI/TLBWR".to_string(),
        Op::COP2 => {
            let cofun = opcode.cofun();
            format!("COP2 {:08X}", cofun)
        }
        Op::CTC2 => {
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("TC2 ${rt}, ${rd}")
        }
        Op::LWC2 => {
            let base = opcode.rs();
            let rt = opcode.rt();
            let offset = opcode.imm();
            format!("LWC2 ${rt}, {:04X}({:02X})", offset, base)
        }
        Op::MFC0 => {
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("MFC0 ${rt}, ${rd}")
        }
        Op::MFC2 => {
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("MFC2 ${rt}, ${rd}")
        }
        Op::MTC0 => {
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("MTC0 ${rt}, ${rd}")
        }
        Op::MTC2 => {
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("MTC2 ${rt}, ${rd}")
        }
        Op::SWC2 => {
            let base = opcode.rs();
            let rt = opcode.rt();
            let offset = opcode.imm();
            format!("SWC2 ${rt}, {:04X}({:02X})", offset, base)
        }
        Op::ADD => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("ADD ${rd}, ${rs}, ${rt}")
        }
        Op::ADDU => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("ADDU ${rd}, ${rs}, ${rt}")
        }
        Op::AND => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("AND ${rd}, ${rs}, ${rt}")
        }
        Op::BREAK => "BREAK".to_string(),
        Op::DIV => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            format!("DIV ${rs}, ${rt}")
        }
        Op::DIVU => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            format!("DIVU ${rs}, ${rt}")
        }
        Op::JALR => {
            let rs = opcode.rs();
            let rd = opcode.rd();
            format!("JALR ${rd}, ${rs}")
        }
        Op::JR => {
            let rs = opcode.rs();
            format!("JR ${rs}")
        }
        Op::MFHI => {
            let rd = opcode.rd();
            format!("MFHI ${rd}")
        }
        Op::MFLO => {
            let rd = opcode.rd();
            format!("MFLO ${rd}")
        }
        Op::MTHI => {
            let rs = opcode.rs();
            format!("MTHI ${rs}")
        }
        Op::MTLO => {
            let rs = opcode.rs();
            format!("MTLO ${rs}")
        }
        Op::MULT => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            format!("MULT ${rs}, ${rt}")
        }
        Op::MULTU => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            format!("MULTU ${rs}, ${rt}")
        }
        Op::NOR => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("NOR ${rd}, ${rs}, ${rt}")
        }
        Op::OR => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("OR ${rd}, ${rs}, ${rt}")
        }
        Op::SLL => {
            let rt = opcode.rt();
            let rd = opcode.rd();
            let sa = opcode.sa();
            format!("SLL ${rd}, ${rt}, {sa}")
        }
        Op::SLLV => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("SLLV ${rd}, ${rt}, ${rs}")
        }
        Op::SLT => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("SLT ${rd}, ${rs}, ${rt}")
        }
        Op::SLTU => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("SLTU ${rd}, ${rt}, ${rs}")
        }
        Op::SRA => {
            let rt = opcode.rt();
            let rd = opcode.rd();
            let sa = opcode.sa();
            format!("SRA ${rd}, ${rt}, {sa}")
        }
        Op::SRAV => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("SRAV ${rd}, ${rt}, ${rs}")
        }
        Op::SRL => {
            let rt = opcode.rt();
            let rd = opcode.rd();
            let sa = opcode.sa();
            format!("SRL ${rd}, ${rt}, {sa}")
        }
        Op::SRLV => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("SRLV ${rd}, ${rt}, ${rs}")
        }
        Op::SUB => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("SUB ${rd}, ${rs}, ${rt}")
        }
        Op::SUBU => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("SUB ${rd}, ${rs}, ${rt}")
        }
        Op::SYSCALL => "SYSCALL".to_string(),
        Op::XOR => {
            let rs = opcode.rs();
            let rt = opcode.rt();
            let rd = opcode.rd();
            format!("XOR ${rd}, ${rs}, ${rt}")
        }
    }
}

// fn close_logging(worker: JoinHandle<()>) {
//     worker.join().unwrap();
// }

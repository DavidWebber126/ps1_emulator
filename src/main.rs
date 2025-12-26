mod bus;
mod cop0;
mod cpu;
mod frontend;
mod gpu;
mod interrupts;
mod timer;

use eframe::egui;
use std::{env, path::PathBuf};

//use cpu::Cpu;
use frontend::MyApp;

fn main() {
    let _args: String = env::args().collect();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([992.0, 558.0]),
        ..Default::default()
    };

    let folder: PathBuf = PathBuf::from("roms/");

    let _ = eframe::run_native(
        "PS1 Emulator",
        options,
        Box::new(|cc| Ok(Box::<MyApp>::new(MyApp::new(cc, folder)))),
    );

    // let mut cpu = Cpu::new();
    // cpu.step_instruction();
}

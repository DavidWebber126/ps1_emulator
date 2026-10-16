mod bus;
mod cop0;
mod cpu;
mod dma;
mod frontend;
mod gpu;
mod gte;
mod interrupts;
mod mdec;
mod opcodes;
mod timer;
mod tracing_setup;

use frontend::BreakpointDebug;
use frontend::MyApp;
use std::path::PathBuf;

use eframe::egui;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1040.0, 560.0]),
        ..Default::default()
    };

    //let folder: PathBuf = PathBuf::from("roms/tests/tests-Jaczekanski/gte/test-all/");
    //let folder: PathBuf = PathBuf::from("roms/tests/tests-Jaczekanski/gte-fuzz/");
    let folder: PathBuf = PathBuf::from("roms/tests/tests-Jaczekanski/gpu/animated-triangle/");
    //let folder: PathBuf = PathBuf::from("roms/tests/tests/");

    let breakpoint = BreakpointDebug::new(true, 0x80059DFC, 8000000);

    let _ = eframe::run_native(
        "PS1 Emulator",
        options,
        Box::new(|cc| Ok(Box::<MyApp>::new(MyApp::new(cc, folder, true, breakpoint)))),
    );

    //drop(tracing::dispatcher::get_default(|d| d.clone()));
    println!("Done running PS1 Emulator!");
}

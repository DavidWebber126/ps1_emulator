mod bus;
mod cop0;
mod cpu;
mod dma;
mod frontend;
mod gpu;
mod gte;
mod interrupts;
mod mdec;
mod timer;
mod tracing_setup;

use eframe::egui;
use frontend::MyApp;
use std::path::PathBuf;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1040.0, 560.0]),
        ..Default::default()
    };

    //let folder: PathBuf = PathBuf::from("roms/tests/tests-Jaczekanski/gpu/gp0-e1/");
    let folder: PathBuf = PathBuf::from("roms/tests/tests/");

    let _ = eframe::run_native(
        "PS1 Emulator",
        options,
        Box::new(|cc| {
            Ok(Box::<MyApp>::new(MyApp::new(
                cc,
                folder,
                true,
                Some(/*0x800507B8*/ 0x8001166C),
            )))
        }),
    );
}

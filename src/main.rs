mod bus;
mod cop0;
mod cpu;
mod frontend;
mod gpu;
mod interrupts;
mod timer;
mod tracing_setup;

use eframe::egui;
use frontend::MyApp;
use std::path::PathBuf;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([992.0, 558.0]),
        ..Default::default()
    };

    let folder: PathBuf = PathBuf::from("roms/");

    let _ = eframe::run_native(
        "PS1 Emulator",
        options,
        Box::new(|cc| {
            Ok(Box::<MyApp>::new(MyApp::new(
                cc, folder, true, Some(0x80010000 /*0x80015880*/) ,
            )))
        }),
    );
}

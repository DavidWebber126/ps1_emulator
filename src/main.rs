mod bus;
mod cop0;
mod cpu;
mod frontend;
mod gpu;
mod interrupts;
mod timer;

use eframe::egui;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, filter};

//use cpu::Cpu;
use frontend::MyApp;

fn main() {
    // let mut max_level = tracing::Level::DEBUG;
    // for arg in env::args() {
    //     match arg.as_str() {
    //         "info" => max_level = tracing::Level::INFO,
    //         "debug" => max_level = tracing::Level::DEBUG,
    //         "trace" => max_level = tracing::Level::TRACE,
    //         "error" => max_level = tracing::Level::ERROR,
    //         "warn" => max_level = tracing::Level::WARN,
    //         _ => {}
    //     }
    // }

    let log_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("logs/dbg.log")
        .unwrap_or(File::create("logs/dbg.log").unwrap());

    let dbg_layer = layer()
        .with_writer(log_file)
        .with_ansi(false)
        .without_time()
        .with_filter(EnvFilter::from_default_env())
        .with_filter(filter::filter_fn(|metadata| {
            (*metadata).target().contains("ps1_emulator")
        }));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_filter(filter::LevelFilter::INFO)
        .with_filter(filter::filter_fn(|metadata| {
            (*metadata).target().contains("ps1_emulator")
        }));

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(dbg_layer)
        .init();

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

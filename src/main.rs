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
use tracing_subscriber::{EnvFilter, Layer, filter, reload};

//use cpu::Cpu;
use frontend::MyApp;

fn main() {
    let log_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("logs/dbg.log")
        .unwrap_or(File::create("logs/dbg.log").unwrap());

    // Layer to write to debug file
    let dbg_layer = layer()
        .with_writer(log_file)
        .with_ansi(false)
        .without_time()
        .with_filter(EnvFilter::from_default_env())
        .with_filter(filter::filter_fn(|metadata| {
            (*metadata).target().contains("ps1_emulator")
        }));

    //let (dbg_filter, reload_handle) = reload::Layer::new(dbg_layer);
    //print_type_of(&reload_handle);

    // Layer to write to stdout
    let fmt_layer = layer()
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
}

// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>());
// }

use std::fs::{File, OpenOptions};

use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, filter};

pub fn init_tracing() {
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
            (*metadata).target().contains("ps1_emulator") && ((*metadata).name() != "BIOS")
        }));

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
}

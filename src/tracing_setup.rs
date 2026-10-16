use std::fmt::Debug;
use std::sync::mpsc;

use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, filter};

#[derive(Default)]
struct CPULog {
    pc: u64,
    opcode: u64,
    regs: [u32; 31],
    msg: String,
}

impl Visit for CPULog {
    fn record_u64(&mut self, field: &Field, value: u64) {
        match field.name() {
            "pc" => self.pc = value,
            "op" => self.opcode = value,
            _ => {}
        }
    }

    fn record_bytes(&mut self, _field: &Field, value: &[u8]) {
        for i in 0..31 {
            self.regs[i] = u32::from_le_bytes([
                value[4 * i],
                value[4 * i + 1],
                value[4 * i + 2],
                value[4 * i + 3],
            ]);
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        //if field.name() != "pc" || field.name() != "op" || field.name() != "regs" {
        self.msg = format!("{:?}", value);
        //}
    }
}
pub struct MyLayer {
    tx: mpsc::Sender<LogEntry>,
}

impl<S: Subscriber> Layer<S> for MyLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();

        let mut visitor = CPULog::default();
        event.record(&mut visitor);

        let entry = LogEntry {
            target: metadata.target().to_string(),
            pc: visitor.pc,
            opcode: visitor.opcode,
            regs: visitor.regs,
            msg: visitor.msg,
        };

        self.tx.send(entry).unwrap();
    }
}

pub struct LogEntry {
    pub target: String,
    pub pc: u64,
    pub opcode: u64,
    pub regs: [u32; 31],
    pub msg: String,
}

pub fn init_tracing() -> mpsc::Receiver<LogEntry> {
    let (tx, rx) = mpsc::channel();

    // Layer to write to stdout
    let fmt_layer = layer()
        .with_filter(filter::LevelFilter::INFO)
        .with_filter(filter::filter_fn(|metadata| {
            (*metadata).target().contains("ps1_emulator")
        }));

    tracing_subscriber::registry()
        .with(fmt_layer)
        //.with(dbg_layer)
        .with(MyLayer { tx }.with_filter(EnvFilter::from_default_env()))
        .init();

    rx
}

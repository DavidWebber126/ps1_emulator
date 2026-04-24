mod gp0;
mod gp1;

use gp0::Gp0;
use gp1::Gp1;

use tracing::{Level, event};

pub struct Gpu {
    pub gp0: Gp0,
    pub gp1: Gp1,
    pub frame_is_ready: bool,
    pub counter: u64,
}

impl Gpu {
    pub fn new() -> Self {
        Self {
            gp0: Gp0::new(),
            gp1: Gp1::new(),
            frame_is_ready: false,
            counter: 0,
        }
    }

    pub fn gpuread(&mut self) -> u32 {
        event!(target: "ps1_emulator::GPU", Level::DEBUG, "Reading GPUREAD");

        // If GP0 is in VRAM to CPU Blit then return that value first
        if self.gp0.is_sending_data() {
            return self.gp0.vram_to_cpu_process();
        }

        match self.gp1.gpu_read_register {
            0x00 | 0x01 | 0x06 => 0,
            0x07 => 0x2,
            _ => panic!("Impossible state for GP1 Read Register"),
        }
    }

    pub fn gpustat(&mut self) -> u32 {
        event!(target: "ps1_emulator::GPU", Level::DEBUG, "Reading GPUSTAT");

        let command_ready = (self.gp0.ready_for_cmd() as u32) << 26;
        let vram_data_ready = (self.gp0.is_sending_data() as u32) << 27;
        let dma_ready = (self.gp0.dma_ready() as u32) << 28;

        dma_ready + vram_data_ready + command_ready
    }

    pub fn tick(&mut self, cycles: u32) {
        self.counter += cycles as u64;

        if self.counter >= 564480 {
            event!(target: "ps1_emulator::GPU", Level::DEBUG, "Render Frame");
            self.frame_is_ready = true;
            self.counter -= 564480;
        } else {
            self.frame_is_ready = false;
        }
    }
}

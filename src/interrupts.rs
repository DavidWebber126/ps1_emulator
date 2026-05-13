use tracing::{Level, event};

#[derive(Default)]
pub struct Interrupt {
    pub stat: u32,
    pub mask: u32,
}

impl Interrupt {
    pub fn new() -> Self {
        Self { stat: 0, mask: 0 }
    }

    pub fn write_stat_low_byte(&mut self, val: u8) {
        self.stat &= 0xFFFFFF00 | (val as u32);
    }

    pub fn write_stat_hi_byte(&mut self, val: u8) {
        self.stat &= 0xFFFF00FF | ((val as u32) << 8)
    }

    pub fn set_vblank_irq(&mut self) {
        event!(target: "ps1_emulator::INT", Level::TRACE, "VBlank Interrupt Set");
        self.stat |= 0x1;
    }

    pub fn _set_gpu_irq(&mut self) {
        event!(target: "ps1_emulator::INT", Level::TRACE, "GPU Interrupt Set");
        self.stat |= 0x2;
    }

    pub fn set_dma_irq(&mut self) {
        event!(target: "ps1_emulator::INT", Level::TRACE, "DMA Interrupt Set");
        self.stat |= 0x8;
    }

    pub fn set_tmr0_irq(&mut self) {
        event!(target: "ps1_emulator::INT", Level::TRACE, "Timer 0 Interrupt Set");
        self.stat |= 0x10;
    }

    pub fn set_tmr1_irq(&mut self) {
        event!(target: "ps1_emulator::INT", Level::TRACE, "Timer 1 Interrupt Set");
        self.stat |= 0x20;
    }

    pub fn set_tmr2_irq(&mut self) {
        event!(target: "ps1_emulator::INT", Level::TRACE, "Timer 2 Interrupt Set");
        self.stat |= 0x40;
    }
}

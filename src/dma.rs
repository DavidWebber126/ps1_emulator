use tracing::{Level, event};

pub enum SyncMode {
    Burst,
    Slice,
    LinkedList,
}

pub struct Dma {
    pub enabled: bool,
    pub madr: u32,
    pub block_control: u32,
    pub channel_control: u32,
    pub sync_mode: SyncMode,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            enabled: false,
            madr: 0,
            block_control: 0,
            channel_control: 0,
            sync_mode: SyncMode::Burst,
        }
    }

    pub fn madr_write(&mut self, val: u32) {
        self.madr = val & 0x0FFFFFC;
    }

    pub fn madr_read(&self) -> u32 {
        self.madr
    }

    pub fn block_control_write(&mut self, val: u32) {
        self.block_control = val;
    }

    pub fn block_control_read(&self) -> u32 {
        self.block_control
    }

    // Returns true if transfer has been enabled
    pub fn channel_control_write(&mut self, val: u32) -> bool {
        let prev_control = self.channel_control;

        match (val >> 9) & 0b11 {
            0 => self.sync_mode = SyncMode::Burst,
            1 => self.sync_mode = SyncMode::Slice,
            2 => self.sync_mode = SyncMode::LinkedList,
            _ => {
                event!(target: "ps1_emulator::DMA", Level::WARN, "Sync Mode write not valid")
            }
        }

        self.channel_control = val;

        (prev_control & 0x1000000 == 0) && (val & 0x1000000 > 0) && self.enabled
    }

    pub fn channel_control_read(&self) -> u32 {
        self.channel_control
    }

    // true is decrement, false is increment
    pub fn increment_direction(&self) -> bool {
        self.channel_control & 0b10 > 0
    }

    // true is RAM to device, false is device to RAM
    pub fn dma_direction(&self) -> bool {
        self.channel_control & 1 > 0
    }

    pub fn start_dma(&mut self) {
        self.channel_control &= 0xEFFFFFFF;
    }

    pub fn finish_dma(&mut self) {
        self.channel_control &= 0xFEFFFFFF;
    }
}

pub struct Dicr(u32);

impl Dicr {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn read(&self) -> u32 {
        self.0
    }

    pub fn write(&mut self, val: u32) {
        event!(target: "ps1_emulator::DMA", Level::DEBUG, "Write DICR {:08X}", val);
        self.0 &= !(val & 0x7F000000);

        self.0 = val & 0x00FFFFFF;

        self.master_interrupt_calc();
    }

    fn master_interrupt_calc(&mut self) {
        if self.0 & 0x8000 > 0 {
            event!(target: "ps1_emulator::DMA", Level::TRACE, "Master Interrupt Set");
            self.0 |= 0x80000000;
            return;
        }

        if self.0 & 0x800000 > 0 && self.0 & 0x7F000000 > 0 {
            event!(target: "ps1_emulator::DMA", Level::TRACE, "Master Interrupt Set");
            self.0 |= 0x80000000;
            return;
        }

        event!(target: "ps1_emulator::DMA", Level::TRACE, "Master Interrupt Not Set");
        self.0 &= 0x7FFFFFFF;
    }

    pub fn master_interrupt_set(&self) -> bool {
        self.0 & 0x80000000 > 0
    }

    pub fn dma2_mask_set(&self) -> bool {
        self.0 & 0x40000 > 0
    }

    pub fn dma2_set_interrupt_flag(&mut self) {
        self.0 |= 0x4000000;
        self.master_interrupt_calc();
    }

    pub fn dma6_mask_set(&self) -> bool {
        self.0 & 0x400000 > 0
    }

    pub fn dma6_set_interrupt_flag(&mut self) {
        self.0 |= 0x40000000;
        self.master_interrupt_calc();
    }
}

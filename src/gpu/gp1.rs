use tracing::{Level, event, span};

pub struct Gp1 {
    pub display_enable: bool,
    pub irq: bool,
    pub dma_direction: u8,
    pub display_x: u16,             // 0-1023, 10 bits
    pub display_y: u16,             // 0-511, 9 bits
    pub horizon_range: (u16, u16),  // 12 bits each
    pub vertical_range: (u16, u16), // 10 bits each
    pub display_mode: u8,
    pub gpu_read_register: u8,
    pub vram_size: bool,
}

impl Gp1 {
    pub fn new() -> Self {
        Self {
            display_enable: false,
            irq: false,
            dma_direction: 0,
            display_x: 0,
            display_y: 0,
            horizon_range: (0, 0),
            vertical_range: (0, 0),
            display_mode: 0,
            gpu_read_register: 0,
            vram_size: false,
        }
    }

    pub fn write(&mut self, val: u32) {
        let span = span!(target: "ps1_emulator::GPU", Level::DEBUG, "GP1", cmd=val);
        let _ = span.enter();
        event!(target: "ps1_emulator::GPU", Level::DEBUG, "Write to GP1 with {:08X}", val);

        match val >> 24 {
            0x00 => {
                // Reset GPU
                self.display_enable = false;
                self.irq = false;
                self.dma_direction = 0;
                self.display_x = 0;
                self.display_y = 0;
                self.horizon_range = (0x200, 0xC00);
                self.vertical_range = (0x10, 0x100);
                self.display_mode = 0;
            }
            0x01 => {
                // Reset Command Buffer
            }
            0x02 => {
                // Acknowledge GPU Interrupt
                self.irq = false;
            }
            0x03 => {
                // Display enable
                self.display_enable = val & 0x1 > 0;
            }
            0x04 => {
                // DMA Direction/Data Request
                self.dma_direction = (val & 0b11) as u8;
            }
            0x05 => {
                // Start of Display Area
                self.display_x = (val & 0x3FF) as u16;
                self.display_y = ((val >> 10) & 0x1FF) as u16;
            }
            0x06 => {
                // Horizontal Display Range
                self.horizon_range.0 = (val & 0xFFF) as u16;
                self.horizon_range.1 = ((val >> 12) & 0xFFF) as u16;
            }
            0x07 => {
                // Vertical Display Range
                self.vertical_range.0 = (val & 0x3FF) as u16;
                self.vertical_range.1 = ((val >> 10) & 0x3FF) as u16;
            }
            0x08 => {
                // Display Mode
                self.display_mode = (val & 0xFF) as u8;
            }
            0x09 => {
                // VRAM Size v2
                self.vram_size = val & 0x1 > 0;
            }
            0x10..=0x1F => {
                // Read GPU Internal Register
                let register = (val & 0x0F) as u8;
                match register {
                    0x00 | 0x01 | 0x06 => {
                        // do nothing
                    }
                    _ => self.gpu_read_register = register,
                }
            }
            0x20 => {
                // VRAM Size v1 -- Probably not used but check to confirm
            }
            _ => panic!("Invalid GP1 command"),
        }
    }
}

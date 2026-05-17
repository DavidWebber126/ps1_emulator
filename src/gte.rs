use tracing::{Level, event};

pub struct Gte {
    pub enabled: bool,
    data_registers: [u32; 32],
    control_registers: [u32; 32],
}

impl Gte {
    pub fn new() -> Self {
        Self {
            enabled: false,
            data_registers: [0; 32],
            control_registers: [0; 32],
        }
    }

    pub fn control_reg_read(&self, reg: u32) -> u32 {
        if self.enabled {
            self.control_registers[reg as usize]
        } else {
            0
        }
    }

    pub fn control_reg_write(&mut self, reg: u32, val: u32) {
        if self.enabled {
            self.control_registers[reg as usize] = val;
        }
    }

    pub fn data_reg_read(&self, reg: u32) -> u32 {
        if self.enabled {
            self.data_registers[reg as usize]
        } else {
            0
        }
    }

    pub fn data_reg_write(&mut self, reg: u32, val: u32) {
        if self.enabled {
            self.data_registers[reg as usize] = val;
        }
    }

    pub fn write_command(&mut self, cmd: u32) {
        if !self.enabled {
            return;
        }

        match cmd & 0x1F {
            0x01 => {
                // Perspective Transformation Single: RTPS
                todo!()
            }
            0x06 => {
                // Normal Clipping
                todo!()
            }
            _ => {
                event!(target: "ps1_emulator::GTE", Level::ERROR, "No GTE command for {:02X}", cmd & 0x1F);
            }
        }
    }
}

fn multiply_fixed_point(arg1: u16, arg2: u16) -> u16 {
    let sign = (arg1 ^ arg2) & 0x8000;
    let arg1 = (arg1 & 0x7FFF) as u32;
    let arg2 = (arg2 & 0x7FFF) as u32;
    let product = ((arg1 * arg2) >> 12) as u16;

    sign | product
}

fn add_fixed_point(arg1: u16, arg2: u16) -> u16 {
    let sign1 = arg1 & 0x8000 > 0;
    let sign2 = arg2 & 0x8000 > 0;
    let arg1 = (arg1 & 0x7FFF) as u32;
    let arg2 = (arg2 & 0x7FFF) as u32;
    let (sum, sign) = match (sign1, sign2) {
        (false, false) => (arg1.wrapping_add(arg2), false),
        (true, true) => (arg1.wrapping_add(arg2), true),
        (true, false) => (arg1.wrapping_sub(arg2), arg1 > arg2),
        (false, true) => (arg2.wrapping_sub(arg1), arg2 > arg1),
    };

    ((sign as u16) << 15) | (sum as u16)
}

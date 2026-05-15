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
        self.control_registers[reg as usize]
    }

    pub fn control_reg_write(&mut self, reg: u32, val: u32) {
        self.control_registers[reg as usize] = val;
    }

    pub fn data_reg_read(&self, reg: u32) -> u32 {
        self.data_registers[reg as usize]
    }

    pub fn data_reg_write(&mut self, reg: u32, val: u32) {
        self.data_registers[reg as usize] = val;
    }

    pub fn write_command(&mut self, cmd: u32) {
        todo!()
    }
}

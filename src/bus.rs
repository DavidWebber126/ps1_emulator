pub struct Bus {
    pub ram: [u8; 2 ^ 32],
}

impl Bus {
    pub fn new() -> Self {
        Self { ram: [0; 2 ^ 32] }
    }

    pub fn mem_read_word(&mut self, addr: u32) -> u32 {
        let addr = addr as usize;
        u32::from_le_bytes([
            self.ram[addr],
            self.ram[addr + 1],
            self.ram[addr + 2],
            self.ram[addr + 3],
        ])
    }

    pub fn mem_write_word(&mut self, addr: u32, val: u32) {
        let addr = addr as usize;
        let [b0, b1, b2, b3] = val.to_le_bytes();
        self.ram[addr] = b0;
        self.ram[addr + 1] = b1;
        self.ram[addr + 2] = b2;
        self.ram[addr + 3] = b3;
    }

    pub fn mem_read_byte(&mut self, addr: u32) -> u8 {
        self.ram[addr as usize]
    }

    pub fn mem_write_byte(&mut self, addr: u32, val: u8) {
        self.ram[addr as usize] = val;
    }

    pub fn mem_read_halfword(&mut self, addr: u32) -> u16 {
        let addr = addr as usize;
        u16::from_le_bytes([self.ram[addr], self.ram[addr + 1]])
    }

    pub fn mem_write_halfword(&mut self, addr: u32, val: u16) {
        let addr = addr as usize;
        let [lo, hi] = val.to_le_bytes();
        self.ram[addr] = lo;
        self.ram[addr + 1] = hi;
    }
}

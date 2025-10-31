use crate::cpu::ExceptionType;

pub struct Bus {
    pub ram: [u8; 2097152], // 2^21
    pub scratchpad: [u8; 1024],
}

impl Bus {
    pub fn new() -> Self {
        Self {
            ram: [0; 2097152],
            scratchpad: [0; 1024],
        }
    }

    pub fn mem_read_byte(&mut self, addr: u32) -> Result<u8, ExceptionType> {
        match addr {
            // KUSEG Kernel
            0x00000000..=0x0000FFFF => {
                todo!()
            }
            // KSEG0 Kernel
            0x80000000..=0x8000FFFF => {
                todo!()
            }
            // KSEG1 Kernel
            0xA0000000..=0xA000FFFF => {
                todo!()
            }
            // KUSEG Main RAM - Cache enabled
            0x00100000..=0x001FFFFF => {
                // mirror address to between 0x00100000 and 0x001FFFFF
                let addr = addr - 0x1FFFFF;
                Ok(self.ram[addr as usize])
            }
            // KSEG0 - Cache enabled
            0x80100000..=0x801FFFFF => {
                let addr = addr - 0x80100000;
                Ok(self.ram[addr as usize])
            }
            // KSEG1 - No Cache
            0xA0100000..=0xA01FFFFF => {
                let addr = addr - 0xA0100000;
                Ok(self.ram[addr as usize])
            }
            // KUSEG ROM
            0x1F000000..=0x1F00FFFF => {
                todo!()
            }
            // KSEG0 ROM
            0x9F000000..=0x9F00FFFF => {
                todo!()
            }
            // KSEG1 ROM
            0xBF000000..=0xBF00FFFF => {
                todo!()
            }
            // KUSEG Scratchpad
            0x1F800000..=0x1F8003FF => {
                let addr = addr - 0x1F800000;
                Ok(self.scratchpad[addr as usize])
            }
            // KSEG0 Scratchpad
            0x9F800000..=0x9F8003FF => {
                let addr = addr - 0x9F800000;
                Ok(self.scratchpad[addr as usize])
            }
            // KUSEG BIOS ROM
            0x1FC00000..=0x1FC7FFFF => {
                todo!()
            }
            // KSEG0 BIOS ROM
            0x9FC00000..=0x9FC7FFFF => {
                todo!()
            }
            // KSEG1 BIOS ROM
            0xBFC00000..=0xBFC7FFFF => {
                todo!()
            }
            // CPU Control Register
            0xFFFE0000..=0xFFFE01FF => {
                todo!()
            }
            _ => Err(ExceptionType::BusErrorLoad),
        }
    }

    pub fn mem_write_byte(&mut self, addr: u32, val: u8) -> Result<(), ExceptionType> {
        match addr {
            // KUSEG Kernel
            0x00000000..=0x0000FFFF => {
                todo!()
            }
            // KSEG0 Kernel
            0x80000000..=0x8000FFFF => {
                todo!()
            }
            // KSEG1 Kernel
            0xA0000000..=0xA000FFFF => {
                todo!()
            }
            // KUSEG Main RAM - Cache enabled
            0x00100000..=0x001FFFFF => {
                // mirror address to between 0x00100000 and 0x001FFFFF
                let addr = addr - 0x1FFFFF;
                self.ram[addr as usize] = val;
                Ok(())
            }
            // KSEG0 - Cache enabled
            0x80100000..=0x801FFFFF => {
                let addr = addr - 0x80100000;
                self.ram[addr as usize] = val;
                Ok(())
            }
            // KSEG1 - No Cache
            0xA0100000..=0xA01FFFFF => {
                let addr = addr - 0xA0100000;
                self.ram[addr as usize] = val;
                Ok(())
            }
            // KUSEG ROM
            0x1F000000..=0x1F00FFFF => {
                todo!()
            }
            // KSEG0 ROM
            0x9F000000..=0x9F00FFFF => {
                todo!()
            }
            // KSEG1 ROM
            0xBF000000..=0xBF00FFFF => {
                todo!()
            }
            // KUSEG Scratchpad
            0x1F800000..=0x1F8003FF => {
                let addr = addr - 0x1F800000;
                self.scratchpad[addr as usize] = val;
                Ok(())
            }
            // KSEG0 Scratchpad
            0x9F800000..=0x9F8003FF => {
                let addr = addr - 0x9F800000;
                self.scratchpad[addr as usize] = val;
                Ok(())
            }
            // KUSEG BIOS ROM
            0x1FC00000..=0x1FC7FFFF => {
                todo!()
            }
            // KSEG0 BIOS ROM
            0x9FC00000..=0x9FC7FFFF => {
                todo!()
            }
            // KSEG1 BIOS ROM
            0xBFC00000..=0xBFC7FFFF => {
                todo!()
            }
            // CPU Control Register
            0xFFFE0000..=0xFFFE01FF => {
                todo!()
            }
            _ => Err(ExceptionType::BusErrorLoad),
        }
    }

    pub fn mem_read_word(&mut self, addr: u32) -> Result<u32, ExceptionType> {
        let b0 = self.mem_read_byte(addr)?;
        let b1 = self.mem_read_byte(addr + 1)?;
        let b2 = self.mem_read_byte(addr + 2)?;
        let b3 = self.mem_read_byte(addr + 3)?;
        Ok(u32::from_le_bytes([b0, b1, b2, b3]))
    }

    pub fn mem_write_word(&mut self, addr: u32, val: u32) -> Result<(), ExceptionType> {
        let [b0, b1, b2, b3] = val.to_le_bytes();
        self.mem_write_byte(addr, b0)?;
        self.mem_write_byte(addr, b1)?;
        self.mem_write_byte(addr, b2)?;
        self.mem_write_byte(addr, b3)?;
        Ok(())
    }

    pub fn mem_read_halfword(&mut self, addr: u32) -> Result<u16, ExceptionType> {
        Ok(u16::from_le_bytes([
            self.mem_read_byte(addr)?,
            self.mem_read_byte(addr + 1)?,
        ]))
    }

    pub fn mem_write_halfword(&mut self, addr: u32, val: u16) -> Result<(), ExceptionType> {
        let [lo, hi] = val.to_le_bytes();
        self.mem_write_byte(addr, lo)?;
        self.mem_write_byte(addr, hi)?;
        Ok(())
    }
}

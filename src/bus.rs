use crate::cpu::ExceptionType;
use crate::timer::Timer;

pub struct Bus {
    pub kernel: [u8; 65536],       // 64 KB
    pub ram: [u8; 2097152],        // 2 MB
    pub scratchpad: [u8; 1024],    // 1 KB
    pub kernel_rom: [u8; 4194304], // 4 MB
    pub interrupt_status: u32,
    pub interrupt_mask: u32,
    pub timer0: Timer,
    pub timer1: Timer,
    pub timer2: Timer,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            kernel: [0; 65536],
            ram: [0; 2097152],
            scratchpad: [0; 1024],
            kernel_rom: [0; 4194304],
            interrupt_status: 0,
            interrupt_mask: 0,
            timer0: Timer::new(),
            timer1: Timer::new(),
            timer2: Timer::new(),
        }
    }

    pub fn tick(&mut self, cycles: u32) {
        for _ in 0..cycles {
            self.timer0.tick();
            self.timer1.tick();
            self.timer2.tick();
        }
    }

    pub fn mem_read_byte(&mut self, addr: u32) -> Result<u8, ExceptionType> {
        match addr {
            // KUSEG Kernel
            0x00000000..=0x0000FFFF => Ok(self.kernel[addr as usize]),
            // KSEG0 Kernel
            0x80000000..=0x8000FFFF => {
                let addr = addr & 0xFFFF;
                Ok(self.kernel[addr as usize])
            }
            // KSEG1 Kernel
            0xA0000000..=0xA000FFFF => {
                let addr = addr & 0xFFFF;
                Ok(self.kernel[addr as usize])
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
                let addr = addr - 0x1FC00000;
                Ok(self.kernel_rom[addr as usize])
            }
            // KSEG0 BIOS ROM
            0x9FC00000..=0x9FC7FFFF => {
                let addr = addr - 0x9FC00000;
                Ok(self.kernel_rom[addr as usize])
            }
            // KSEG1 BIOS ROM
            0xBFC00000..=0xBFC7FFFF => {
                let addr = addr - 0xBFC00000;
                Ok(self.kernel_rom[addr as usize])
            }
            // IO Register
            // I_STAT - Interrupt status
            0x1F801070 => Ok((self.interrupt_status & 0xFF) as u8),
            0x1F801071 => Ok(((self.interrupt_status & 0xFF00) >> 8) as u8),
            0x1F801072 => Ok(((self.interrupt_status & 0xFF0000) >> 16) as u8),
            0x1F801073 => Ok(((self.interrupt_status & 0xFF000000) >> 24) as u8),
            // I_MASK - Interrupt Mask
            0x1F801074 => Ok((self.interrupt_mask & 0xFF) as u8),
            0x1F801075 => Ok(((self.interrupt_mask & 0xFF00) >> 8) as u8),
            0x1F801076 => Ok(((self.interrupt_mask & 0xFF00) >> 16) as u8),
            0x1F801077 => Ok(((self.interrupt_mask & 0xFF00) >> 24) as u8),
            // Timers
            // Timer 0 Counter Value
            0x1F801100 => Ok(self.timer0.counter as u8),
            0x1F801101 => Ok((self.timer0.counter >> 8) as u8),
            0x1F801102 => Ok(0),
            0x1F801103 => Ok(0),
            // Timer 0 Counter Mode
            0x1F801104 => Ok(self.timer0.mode as u8),
            0x1F801105 => Ok((self.timer0.mode >> 8) as u8),
            0x1F801106 => Ok(0),
            0x1F801107 => Ok(0),
            // Timer 0 Target
            0x1F801108 => Ok(self.timer0.target_value as u8),
            0x1F801109 => Ok((self.timer0.target_value >> 8) as u8),
            0x1F80110A => Ok(0),
            0x1F80110B => Ok(0),
            // Timer 1 Counter Value
            0x1F801110 => Ok(self.timer1.counter as u8),
            0x1F801111 => Ok((self.timer1.counter >> 8) as u8),
            0x1F801112 => Ok(0),
            0x1F801113 => Ok(0),
            // Timer 1 Counter Mode
            0x1F801114 => Ok(self.timer1.mode as u8),
            0x1F801115 => Ok((self.timer1.mode >> 8) as u8),
            0x1F801116 => Ok(0),
            0x1F801117 => Ok(0),
            // Timer 1 Target
            0x1F801118 => Ok(self.timer1.target_value as u8),
            0x1F801119 => Ok((self.timer1.target_value >> 8) as u8),
            0x1F80111A => Ok(0),
            0x1F80111B => Ok(0),
            // Timer 2 Counter Value
            0x1F801120 => Ok(self.timer2.counter as u8),
            0x1F801121 => Ok((self.timer2.counter >> 8) as u8),
            0x1F801122 => Ok(0),
            0x1F801123 => Ok(0),
            // Timer 2 Counter Mode
            0x1F801124 => Ok(self.timer2.mode as u8),
            0x1F801125 => Ok((self.timer2.mode >> 8) as u8),
            0x1F801126 => Ok(0),
            0x1F801127 => Ok(0),
            // Timer 2 Target
            0x1F801128 => Ok(self.timer2.target_value as u8),
            0x1F801129 => Ok((self.timer2.target_value >> 8) as u8),
            0x1F80112A => Ok(0),
            0x1F80112B => Ok(0),
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
                self.kernel[addr as usize] = val;
                Ok(())
            }
            // KSEG0 Kernel
            0x80000000..=0x8000FFFF => {
                let addr = addr & 0xFFFF;
                self.kernel[addr as usize] = val;
                Ok(())
            }
            // KSEG1 Kernel
            0xA0000000..=0xA000FFFF => {
                let addr = addr & 0xFFFF;
                self.kernel[addr as usize] = val;
                Ok(())
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
                let addr = addr - 0x1FC00000;
                self.kernel_rom[addr as usize] = val;
                Ok(())
            }
            // KSEG0 BIOS ROM
            0x9FC00000..=0x9FC7FFFF => {
                let addr = addr - 0x9FC00000;
                self.kernel_rom[addr as usize] = val;
                Ok(())
            }
            // KSEG1 BIOS ROM
            0xBFC00000..=0xBFC7FFFF => {
                let addr = addr - 0xBFC00000;
                self.kernel_rom[addr as usize] = val;
                Ok(())
            }
            // Timers
            // Timer 0 Counter Value
            0x1F801100 => {
                self.timer0.counter = self.timer0.counter & 0xFF00 + val as u16;
                Ok(())
            }
            0x1F801101 => {
                self.timer0.counter = self.timer0.counter & 0xFF + ((val as u16) << 8);
                Ok(())
            }
            0x1F801102 => Ok(()),
            0x1F801103 => Ok(()),
            // Timer 0 Counter Mode
            0x1F801104 => {
                self.timer0.mode = self.timer0.mode & 0xFF00 + val as u16;
                Ok(())
            }
            0x1F801105 => {
                self.timer0.mode = self.timer0.mode & 0xFF + ((val as u16) << 8);
                Ok(())
            }
            0x1F801106 => Ok(()),
            0x1F801107 => Ok(()),
            // Timer 0 Target
            0x1F801108 => {
                self.timer0.target_value = self.timer0.target_value & 0xFF00 + val as u16;
                Ok(())
            }
            0x1F801109 => {
                self.timer0.target_value = self.timer0.target_value & 0xFF + ((val as u16) << 8);
                Ok(())
            }
            0x1F80110A => Ok(()),
            0x1F80110B => Ok(()),
            // Timer 1 Counter Value
            0x1F801110 => {
                self.timer1.counter = self.timer1.counter & 0xFF00 + val as u16;
                Ok(())
            }
            0x1F801111 => {
                self.timer1.counter = self.timer1.counter & 0xFF + ((val as u16) << 8);
                Ok(())
            }
            0x1F801112 => Ok(()),
            0x1F801113 => Ok(()),
            // Timer 1 Counter Mode
            0x1F801114 => {
                self.timer1.mode = self.timer1.mode & 0xFF00 + val as u16;
                Ok(())
            }
            0x1F801115 => {
                self.timer1.mode = self.timer1.mode & 0xFF + ((val as u16) << 8);
                Ok(())
            }
            0x1F801116 => Ok(()),
            0x1F801117 => Ok(()),
            // Timer 1 Target
            0x1F801118 => {
                self.timer1.target_value = self.timer1.target_value & 0xFF00 + val as u16;
                Ok(())
            }
            0x1F801119 => {
                self.timer1.target_value = self.timer1.target_value & 0xFF + ((val as u16) << 8);
                Ok(())
            }
            0x1F80111A => Ok(()),
            0x1F80111B => Ok(()),
            // Timer 2 Counter Value
            0x1F801120 => {
                self.timer2.counter = self.timer2.counter & 0xFF00 + val as u16;
                Ok(())
            }
            0x1F801121 => {
                self.timer2.counter = self.timer2.counter & 0xFF + ((val as u16) << 8);
                Ok(())
            }
            0x1F801122 => Ok(()),
            0x1F801123 => Ok(()),
            // Timer 2 Counter Mode
            0x1F801124 => {
                self.timer2.mode = self.timer2.mode & 0xFF00 + val as u16;
                Ok(())
            }
            0x1F801125 => {
                self.timer2.mode = self.timer2.mode & 0xFF + ((val as u16) << 8);
                Ok(())
            }
            0x1F801126 => Ok(()),
            0x1F801127 => Ok(()),
            // Timer 2 Target
            0x1F801128 => {
                self.timer2.target_value = self.timer2.target_value & 0xFF00 + val as u16;
                Ok(())
            }
            0x1F801129 => {
                self.timer1.target_value = self.timer1.target_value & 0xFF00 + val as u16;
                Ok(())
            }
            0x1F80112A => Ok(()),
            0x1F80112B => Ok(()),
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

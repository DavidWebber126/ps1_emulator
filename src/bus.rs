use crate::cop0::Cop0;
use crate::cpu::ExceptionType;
use crate::gpu::Gpu;
use crate::interrupts::Interrupt;
use crate::timer::Timer;

use tracing::{Level, event};

pub struct Bus {
    pub kernel: Box<[u8; 65536]>,      // 64 KB
    pub ram: Box<[u8; 2097152]>,       // 2 MB - Box needed due to large array size
    pub expansion1: Box<[u8; 65536]>,  // 64 KB
    pub scratchpad: [u8; 1024],        // 1 KB
    pub kernel_rom: Box<[u8; 524288]>, // 512 KB - Box needed due to large array size
    pub cop0: Cop0,
    pub interrupts: Interrupt,
    pub timer0: Timer,
    pub timer1: Timer,
    pub timer2: Timer,
    pub gpu: Gpu,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            kernel: Box::new([0; 65536]),
            ram: Box::new([0; 2097152]),
            expansion1: Box::new([0; 65536]),
            scratchpad: [0; 1024],
            kernel_rom: Box::new([0; 524288]),
            cop0: Cop0::new(),
            interrupts: Interrupt::new(),
            timer0: Timer::new(),
            timer1: Timer::new(),
            timer2: Timer::new(),
            gpu: Gpu::new(),
        }
    }

    pub fn tick(&mut self, cycles: u32) {
        self.gpu.tick(cycles);

        for _ in 0..cycles {
            if self.timer0.tick() {
                self.interrupts.stat |= 0x00000010
            }
            if self.timer1.tick() {
                self.interrupts.stat |= 0x00000020
            }
            if self.timer2.tick() {
                self.interrupts.stat |= 0x00000040
            }
        }
    }

    pub fn mem_read_byte(&mut self, addr: u32) -> Result<u8, ExceptionType> {
        event!(
            target: "ps1_emulator::BUS",
            Level::TRACE,
            "Attempt to read at address: {:08X} (actual address used {:08X})",
            addr & 0x1FFFFFFF,
            addr
        );

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
            0x00010000..=0x001FFFFF => {
                // mirror address to between 0x00010000 and 0x001FFFFF
                let addr = addr - 0x00010000;
                Ok(self.ram[addr as usize])
            }
            // KSEG0 Main RAM - Cache enabled
            0x80010000..=0x801FFFFF => {
                let addr = addr - 0x80010000;
                Ok(self.ram[addr as usize])
            }
            // KSEG1 Main RAM - No Cache
            0xA0010000..=0xA01FFFFF => {
                let addr = addr - 0xA0010000;
                Ok(self.ram[addr as usize])
            }
            // KUSEG ROM
            0x1F000000..=0x1F00FFFF => {
                let addr = addr - 0x1F000000;
                Ok(self.expansion1[addr as usize])
            }
            // KSEG0 ROM
            0x9F000000..=0x9F00FFFF => {
                let addr = addr - 0x9F000000;
                Ok(self.expansion1[addr as usize])
            }
            // KSEG1 ROM
            0xBF000000..=0xBF00FFFF => {
                let addr = addr - 0xBF000000;
                Ok(self.expansion1[addr as usize])
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
            // Expansion 1 Base Address
            0x1F801000 => Ok(0x00),
            0x1F801001 => Ok(0x00),
            0x1F801002 => Ok(0x00),
            0x1F801003 => Ok(0x1F),
            // Expansion 2 Base
            0x1F801004 => Ok(0x00),
            0x1F801005 => Ok(0x20),
            0x1F801006 => Ok(0x80),
            0x1F801007 => Ok(0x1F),
            // Expansion 1 Delay/Size
            0x1F801008 => Ok(0x3F),
            0x1F801009 => Ok(0x24),
            0x1F80100A => Ok(0x13),
            0x1F80100B => Ok(0x00),
            // Expansion 3 Delay/Size
            0x1F80100C => Ok(0x22),
            0x1F80100D => Ok(0x30),
            0x1F80100E => Ok(0x00),
            0x1F80100F => Ok(0x00),
            // BIOS ROM
            0x1F801010 => Ok(0x3F),
            0x1F801011 => Ok(0x24),
            0x1F801012 => Ok(0x13),
            0x1F801013 => Ok(0x00),
            // SPU DELAY
            0x1F801014 => Ok(0xE1),
            0x1F801015 => Ok(0x31),
            0x1F801016 => Ok(0x09),
            0x1F801017 => Ok(0x20),
            // CDROM DELAY
            0x1F801018 => Ok(0x43),
            0x1F801019 => Ok(0x08),
            0x1F80101A => Ok(0x02),
            0x1F80101B => Ok(0x00),
            // Expansion 2 Delay/Size
            0x1F80101C => Ok(0x77),
            0x1F80101D => Ok(0x07),
            0x1F80101E => Ok(0x07),
            0x1F80101F => Ok(0x00),
            // COMMON Delay
            0x1F801020 => Ok(0x25),
            0x1F801021 => Ok(0x11),
            0x1F801022 => Ok(0x03),
            0x1F801023 => Ok(0x00),
            // RAM SIZE
            0x1F801060 => Ok(0x88),
            0x1F801061 => Ok(0x0B),
            0x1F801062 => Ok(0x00),
            0x1F801063 => Ok(0x00),
            // I_STAT - Interrupt status
            0x1F801070 => Ok((self.interrupts.stat & 0xFF) as u8),
            0x1F801071 => Ok(((self.interrupts.stat & 0xFF00) >> 8) as u8),
            0x1F801072 => Ok(0),
            0x1F801073 => Ok(0),
            // I_MASK - Interrupt Mask
            0x1F801074 => Ok((self.interrupts.mask & 0xFF) as u8),
            0x1F801075 => Ok(((self.interrupts.mask & 0xFF00) >> 8) as u8),
            0x1F801076 => Ok(0),
            0x1F801077 => Ok(0),
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
            // SPU Control Registers
            // Main Volume
            0x1F801D80 => Ok(0),
            0x1F801D81 => Ok(0),
            0x1F801D82 => Ok(0),
            0x1F801D83 => Ok(0),
            // Reverb Output Volume
            0x1F801D84 => Ok(0),
            0x1F801D85 => Ok(0),
            0x1F801D86 => Ok(0),
            0x1F801D87 => Ok(0),
            // Expansion Region 2 Int/Dip/Post
            0x1F802041 => Ok(0),
            // CPU Control Register
            // 0xFFFE0000..=0xFFFE01FF => {
            //     todo!()
            // }
            0xFFFE0130..=0xFFFE0133 => Ok(0),
            _ => {
                event!(
                    target: "ps1_emulator::BUS",
                    Level::WARN,
                    "Address {:08X} not implemented yet (read)",
                    addr
                );
                Err(ExceptionType::BusErrorLoad(addr))
            }
        }
    }

    pub fn mem_write_byte(&mut self, addr: u32, val: u8) -> Result<(), ExceptionType> {
        let isc_set = self.cop0.sr.get_isc();

        event!(
            target: "ps1_emulator::BUS",
            Level::TRACE,
            "Attempt to write at address: {:08X} with {:02X}, (actual address used: {:08X}). IsC set: {isc_set}",
            addr & 0x1FFFFFFF,
            val,
            addr
        );

        // If IsC is set, loads and stores go to data cache and not main memory
        if isc_set {
            return Ok(());
        }

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
            0x0010000..=0x001FFFFF => {
                // mirror address to between 0x00100000 and 0x001FFFFF
                let addr = addr - 0x10000;
                self.ram[addr as usize] = val;
                Ok(())
            }
            // KSEG0 Main RAM - Cache enabled
            0x80010000..=0x801FFFFF => {
                let addr = addr - 0x80010000;
                self.ram[addr as usize] = val;
                Ok(())
            }
            // KSEG1 Main RAM - No Cache
            0xA0010000..=0xA01FFFFF => {
                let addr = addr - 0xA0010000;
                self.ram[addr as usize] = val;
                Ok(())
            }
            // KUSEG ROM
            0x1F000000..=0x1F00FFFF => {
                // Don't write to ROM?
                Ok(())
            }
            // KSEG0 ROM
            0x9F000000..=0x9F00FFFF => {
                // Don't write to ROM?
                Ok(())
            }
            // KSEG1 ROM
            0xBF000000..=0xBF00FFFF => {
                // Don't write to ROM?
                Ok(())
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
            // IO Registers
            // Expansion 1 Base Address
            0x1F801000 => Ok(()),
            0x1F801001 => Ok(()),
            0x1F801002 => Ok(()),
            0x1F801003 => Ok(()),
            // Expansion 2 Base
            0x1F801004 => Ok(()),
            0x1F801005 => Ok(()),
            0x1F801006 => Ok(()),
            0x1F801007 => Ok(()),
            // Expansion 1 Delay/Size
            0x1F801008 => Ok(()),
            0x1F801009 => Ok(()),
            0x1F80100A => Ok(()),
            0x1F80100B => Ok(()),
            // Expansion 3 Delay/Size
            0x1F80100C => Ok(()),
            0x1F80100D => Ok(()),
            0x1F80100E => Ok(()),
            0x1F80100F => Ok(()),
            // BIOS ROM
            0x1F801010 => Ok(()),
            0x1F801011 => Ok(()),
            0x1F801012 => Ok(()),
            0x1F801013 => Ok(()),
            // SPU DELAY
            0x1F801014 => Ok(()),
            0x1F801015 => Ok(()),
            0x1F801016 => Ok(()),
            0x1F801017 => Ok(()),
            // CDROM DELAY
            0x1F801018 => Ok(()),
            0x1F801019 => Ok(()),
            0x1F80101A => Ok(()),
            0x1F80101B => Ok(()),
            // Expansion 2 Delay/Size
            0x1F80101C => Ok(()),
            0x1F80101D => Ok(()),
            0x1F80101E => Ok(()),
            0x1F80101F => Ok(()),
            // COMMON DELAY
            0x1F801020 => Ok(()),
            0x1F801021 => Ok(()),
            0x1F801022 => Ok(()),
            0x1F801023 => Ok(()),
            // RAM SIZE
            0x1F801060 => Ok(()),
            0x1F801061 => Ok(()),
            0x1F801062 => Ok(()),
            0x1F801063 => Ok(()),
            // I_STAT
            0x1F801070 => {
                self.interrupts.stat = (self.interrupts.stat & 0xFFFFFF00) + val as u32;
                Ok(())
            }
            0x1F801071 => {
                self.interrupts.stat = (self.interrupts.stat & 0xFFFF00FF) + ((val as u32) << 8);
                Ok(())
            }
            0x1F801072 => Ok(()),
            0x1F801073 => Ok(()),
            // I_MASK
            0x1F801074 => {
                self.interrupts.mask = (self.interrupts.mask & 0xFFFFFF00) + val as u32;
                Ok(())
            }
            0x1F801075 => {
                self.interrupts.mask = (self.interrupts.mask & 0xFFFF00FF) + ((val as u32) << 8);
                Ok(())
            }
            0x1F801076 => Ok(()),
            0x1F801077 => Ok(()),
            // Timers
            // Timer 0 Counter Value
            0x1F801100 => {
                self.timer0.counter = (self.timer0.counter & 0xFF00) + val as u16;
                Ok(())
            }
            0x1F801101 => {
                self.timer0.counter = (self.timer0.counter & 0xFF) + ((val as u16) << 8);
                Ok(())
            }
            0x1F801102 => Ok(()),
            0x1F801103 => Ok(()),
            // Timer 0 Counter Mode
            0x1F801104 => {
                self.timer0
                    .write_to_mode((self.timer0.mode & 0xFF00) + val as u16);
                Ok(())
            }
            0x1F801105 => {
                self.timer0
                    .write_to_mode((self.timer0.mode & 0xFF) + ((val as u16) << 8));
                Ok(())
            }
            0x1F801106 => {
                self.timer0.counter = 0;
                Ok(())
            }
            0x1F801107 => {
                self.timer0.counter = 0;
                Ok(())
            }
            // Timer 0 Target
            0x1F801108 => {
                self.timer0.target_value = (self.timer0.target_value & 0xFF00) + val as u16;
                Ok(())
            }
            0x1F801109 => {
                self.timer0.target_value = (self.timer0.target_value & 0xFF) + ((val as u16) << 8);
                Ok(())
            }
            0x1F80110A => Ok(()),
            0x1F80110B => Ok(()),
            // Timer 1 Counter Value
            0x1F801110 => {
                self.timer1.counter = (self.timer1.counter & 0xFF00) + val as u16;
                Ok(())
            }
            0x1F801111 => {
                self.timer1.counter = (self.timer1.counter & 0xFF) + ((val as u16) << 8);
                Ok(())
            }
            0x1F801112 => Ok(()),
            0x1F801113 => Ok(()),
            // Timer 1 Counter Mode
            0x1F801114 => {
                self.timer1
                    .write_to_mode((self.timer1.mode & 0xFF00) + val as u16);
                Ok(())
            }
            0x1F801115 => {
                self.timer1
                    .write_to_mode((self.timer1.mode & 0xFF) + ((val as u16) << 8));
                Ok(())
            }
            0x1F801116 => {
                self.timer1.counter = 0;
                Ok(())
            }
            0x1F801117 => {
                self.timer1.counter = 0;
                Ok(())
            }
            // Timer 1 Target
            0x1F801118 => {
                self.timer1.target_value = (self.timer1.target_value & 0xFF00) + val as u16;
                Ok(())
            }
            0x1F801119 => {
                self.timer1.target_value = (self.timer1.target_value & 0xFF) + ((val as u16) << 8);
                Ok(())
            }
            0x1F80111A => Ok(()),
            0x1F80111B => Ok(()),
            // Timer 2 Counter Value
            0x1F801120 => {
                self.timer2.counter = (self.timer2.counter & 0xFF00) + val as u16;
                Ok(())
            }
            0x1F801121 => {
                self.timer2.counter = (self.timer2.counter & 0xFF) + ((val as u16) << 8);
                Ok(())
            }
            0x1F801122 => Ok(()),
            0x1F801123 => Ok(()),
            // Timer 2 Counter Mode
            0x1F801124 => {
                self.timer2
                    .write_to_mode((self.timer2.mode & 0xFF00) + val as u16);
                Ok(())
            }
            0x1F801125 => {
                self.timer2
                    .write_to_mode((self.timer2.mode & 0xFF) + ((val as u16) << 8));
                Ok(())
            }
            0x1F801126 => {
                self.timer2.counter = 0;
                Ok(())
            }
            0x1F801127 => {
                self.timer2.counter = 0;
                Ok(())
            }
            // Timer 2 Target
            0x1F801128 => {
                self.timer2.target_value = (self.timer2.target_value & 0xFF00) + val as u16;
                Ok(())
            }
            0x1F801129 => {
                self.timer1.target_value = (self.timer1.target_value & 0xFF00) + val as u16;
                Ok(())
            }
            0x1F80112A => Ok(()),
            0x1F80112B => Ok(()),
            // SPU Control Registers
            // Main Volume
            0x1F801D80 => Ok(()),
            0x1F801D81 => Ok(()),
            0x1F801D82 => Ok(()),
            0x1F801D83 => Ok(()),
            // Reverb Output Volume
            0x1F801D84 => Ok(()),
            0x1F801D85 => Ok(()),
            0x1F801D86 => Ok(()),
            0x1F801D87 => Ok(()),
            // Expansion Region 2 Int/Dip/Post
            0x1F802041 => Ok(()),
            // CPU Control Register
            // 0xFFFE0000..=0xFFFE01FF => {
            //     println!("Write to {:08X} with {:02X}", addr, val);
            //     todo!()
            // }
            0xFFFE0130..=0xFFFE0133 => Ok(()),
            _ => {
                event!(
                    target: "ps1_emulator::BUS",
                    Level::WARN,
                    "Address {:08X} not implemented yet (write with {:02X})",
                    addr,
                    val
                );
                Err(ExceptionType::BusErrorLoad(addr))
            }
        }
    }

    pub fn mem_read_word(&mut self, addr: u32) -> Result<u32, ExceptionType> {
        if addr & 0b11 > 0 {
            return Err(ExceptionType::AddressErrorLoad(addr))
        }

        match addr {
            0x1F801810 => Ok(self.gpu.gpuread()),
            0x1F801814 => Ok(self.gpu.gpustat()),
            _ => {
                let b0 = self.mem_read_byte(addr)?;
                let b1 = self.mem_read_byte(addr + 1)?;
                let b2 = self.mem_read_byte(addr + 2)?;
                let b3 = self.mem_read_byte(addr + 3)?;
                Ok(u32::from_le_bytes([b0, b1, b2, b3]))
            }
        }
    }

    pub fn mem_write_word(&mut self, addr: u32, val: u32) -> Result<(), ExceptionType> {
        if addr & 0b11 > 0 {
            return Err(ExceptionType::AddressErrorLoad(addr))
        }

        // If isc is set, loads and stores go to data cache and not main memory
        if self.cop0.sr.get_isc() {
            return Ok(());
        }

        match addr {
            0x1F801810 => {
                event!(target: "ps1_emulator::BUS", Level::TRACE, "Write to GP0 with {:08X}", val);
                self.gpu.gp0.write(val);
                Ok(())
            }
            0x1F801814 => {
                event!(target: "ps1_emulator::BUS", Level::TRACE, "Write to GP1 with {:08X}", val);
                self.gpu.gp1.write(val);
                Ok(())
            }
            _ => {
                let [b0, b1, b2, b3] = val.to_le_bytes();
                self.mem_write_byte(addr, b0)?;
                self.mem_write_byte(addr + 1, b1)?;
                self.mem_write_byte(addr + 2, b2)?;
                self.mem_write_byte(addr + 3, b3)?;
                Ok(())
            }
        }
    }

    pub fn mem_read_halfword(&mut self, addr: u32) -> Result<u16, ExceptionType> {
        if addr & 0b1 > 0 {
            return Err(ExceptionType::AddressErrorLoad(addr))
        }

        Ok(u16::from_le_bytes([
            self.mem_read_byte(addr)?,
            self.mem_read_byte(addr + 1)?,
        ]))
    }

    pub fn mem_write_halfword(&mut self, addr: u32, val: u16) -> Result<(), ExceptionType> {
        if addr & 0b1 > 0 {
            return Err(ExceptionType::AddressErrorLoad(addr))
        }
        
        // If isc is set, loads and stores go to data cache and not main memory
        if self.cop0.sr.get_isc() {
            return Ok(());
        }

        let [lo, hi] = val.to_le_bytes();
        self.mem_write_byte(addr, lo)?;
        self.mem_write_byte(addr + 1, hi)?;
        Ok(())
    }
}

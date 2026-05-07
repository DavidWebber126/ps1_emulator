use crate::cop0::Cop0;
use crate::cpu::ExceptionType;
use crate::dma::{Dma, SyncMode};
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
    pub dma2: Dma,
    pub dma6: Dma,
    pub dpcr: u32,
    pub dicr: u32,
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
            dma2: Dma::new(),
            dma6: Dma::new(),
            dpcr: 0x07654321,
            dicr: 0,
        }
    }

    pub fn tick(&mut self, cycles: u32) {
        self.gpu.tick(cycles);

        for _ in 0..2 {
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
            // JOY DATA
            0x1F801040 => Ok(0),
            0x1F801041 => Ok(0),
            0x1F801042 => Ok(0),
            0x1F801043 => Ok(0),
            // JOY CTRL
            0x1F80104A => Ok(0),
            0x1F80104B => Ok(0),
            0x1F80104C => Ok(0),
            0x1F80104D => Ok(0),
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
            // Voice Registers
            0x1F801C00..=0x1F801D7F => Ok(0),
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
            // Voice 0..23 Key ON
            0x1F801D88 => Ok(0),
            0x1F801D89 => Ok(0),
            0x1F801D8A => Ok(0),
            0x1F801D8B => Ok(0),
            // Voice 0..23 Key OFF
            0x1F801D8C => Ok(0),
            0x1F801D8D => Ok(0),
            0x1F801D8E => Ok(0),
            0x1F801D8F => Ok(0),
            // SPU Control Register (SPUCNT)
            0x1F801DAA => Ok(0),
            0x1F801DAB => Ok(0),
            // Sound RAM Data Transfer Control
            0x1F801DAC => Ok(0),
            0x1F801DAD => Ok(0),
            // SPU Status Register (SPUSTAT)
            0x1F801DAE => Ok(0),
            0x1F801DAF => Ok(0),
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
                    Level::TRACE,
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
            // JOY_DATA
            0x1F801040 => Ok(()),
            0x1F801041 => Ok(()),
            0x1F801042 => Ok(()),
            0x1F801043 => Ok(()),
            // JOY_CTRL
            0x1F80104A => Ok(()),
            0x1F80104B => Ok(()),
            0x1F80104C => Ok(()),
            0x1F80104D => Ok(()),
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
            // Voice Registers
            0x1F801C00..=0x1F801D7F => Ok(()),
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
            // Voice 0..23 Key ON
            0x1F801D88 => Ok(()),
            0x1F801D89 => Ok(()),
            0x1F801D8A => Ok(()),
            0x1F801D8B => Ok(()),
            // Voice 0..23 Key OFF
            0x1F801D8C => Ok(()),
            0x1F801D8D => Ok(()),
            0x1F801D8E => Ok(()),
            0x1F801D8F => Ok(()),
            // Voice 0..23 Channel FM
            0x1F801D90 => Ok(()),
            0x1F801D91 => Ok(()),
            0x1F801D92 => Ok(()),
            0x1F801D93 => Ok(()),
            // Voice 0..23 Channel Noise Mode
            0x1F801D94 => Ok(()),
            0x1F801D95 => Ok(()),
            0x1F801D96 => Ok(()),
            0x1F801D97 => Ok(()),
            // Voice 0..23 Channel Reverb Mode
            0x1F801D98 => Ok(()),
            0x1F801D99 => Ok(()),
            0x1F801D9A => Ok(()),
            0x1F801D9B => Ok(()),
            // Sound RAM Reverb Work Area
            0x1F801DA2 => Ok(()),
            0x1F801DA3 => Ok(()),
            // Sound RAM Data Transfer Address
            0x1F801DA6 => Ok(()),
            0x1F801DA7 => Ok(()),
            // Sound RAM Data Transfer FIFO
            0x1F801DA8 => Ok(()),
            0x1F801DA9 => Ok(()),
            // SPU Control Register (SPUCNT)
            0x1F801DAA => Ok(()),
            0x1F801DAB => Ok(()),
            // Sound RAM Data Transfer Control
            0x1F801DAC => Ok(()),
            0x1F801DAD => Ok(()),
            // SPU Status Register (SPUSTAT)
            0x1F801DAE => Ok(()),
            0x1F801DAF => Ok(()),
            // CD Volume Left/Right
            0x1F801DB0 => Ok(()),
            0x1F801DB1 => Ok(()),
            0x1F801DB2 => Ok(()),
            0x1F801DB3 => Ok(()),
            // External Volume Left/Right
            0x1F801DB4 => Ok(()),
            0x1F801DB5 => Ok(()),
            0x1F801DB6 => Ok(()),
            0x1F801DB7 => Ok(()),
            0x1F801DC0..=0x1F801DFF => Ok(()),

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
                    Level::TRACE,
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
            return Err(ExceptionType::AddressErrorLoad(addr));
        }

        match addr {
            // DMA 2 - GPU
            0x1F8010A0 => Ok(self.dma2.madr_read()),
            0x1F8010A4 => Ok(self.dma2.block_control_read()),
            0x1F8010A8 => Ok(self.dma2.channel_control_read()),
            // DMA 6 - OTC
            0x1F8010E0 => Ok(self.dma6.madr_read()),
            0x1F8010E4 => Ok(self.dma6.block_control_read()),
            0x1F8010E8 => Ok(self.dma6.channel_control_read()),
            // DPCR
            0x1F8010F0 => {
                event!(target: "ps1_emulator::BUS", Level::TRACE, "DPCR DMA Unimplemented");
                Ok(self.dpcr)
            }
            // DICR
            0x1F8010F4 => {
                event!(target: "ps1_emulator::BUS", Level::TRACE, "DICR DMA Unimplemented");
                Ok(self.dicr)
            }
            //
            // GPU
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
            return Err(ExceptionType::AddressErrorLoad(addr));
        }

        // If isc is set, loads and stores go to data cache and not main memory
        if self.cop0.sr.get_isc() {
            return Ok(());
        }

        match addr {
            // DMA 2 - GPU
            0x1F8010A0 => {
                event!(target: "ps1_emulator::BUS", Level::TRACE, "DMA 2 MADR write {:08X}", val);
                self.dma2.madr_write(val);
                Ok(())
            }
            0x1F8010A4 => {
                event!(target: "ps1_emulator::BUS", Level::TRACE, "DMA 2 BCH write {:08X}", val);
                self.dma2.block_control_write(val);
                Ok(())
            }
            0x1F8010A8 => {
                event!(target: "ps1_emulator::BUS", Level::TRACE, "DMA 2 CHCR write {:08X}", val);
                if self.dma2.channel_control_write(val) {
                    let mut address = self.dma2.madr_read();
                    self.dma2.start_dma();
                    match self.dma2.sync_mode {
                        SyncMode::Burst => {
                            panic!("Sync Mode Burst not implemented")
                        }
                        SyncMode::Slice => {
                            let block_ctrl = self.dma2.block_control_read();
                            let block_size = block_ctrl & 0xFFFF;
                            let num_blocks = (block_ctrl >> 16) & 0xFFFF;
                            let dma_len = block_size * num_blocks;

                            for _ in 0..dma_len {
                                if self.dma2.dma_direction() {
                                    let val = self.mem_read_word(address).unwrap();
                                    self.gpu.gp0.write(val);
                                }

                                if self.dma2.increment_direction() {
                                    address -= 4;
                                } else {
                                    address += 4;
                                }
                            }

                            self.dma2.madr_write(address);
                            self.dma2.block_control_write(0);
                        }
                        SyncMode::LinkedList => {
                            loop {
                                let header = self.mem_read_word(address).unwrap();

                                let data_words = header >> 24;

                                for i in 0..data_words {
                                    let addr = address + 4 * (i + 1);
                                    let data = self.mem_read_word(addr).unwrap();
                                    self.gpu.gp0.write(data);
                                }

                                let next_address = header & 0xFFFFFF;

                                if next_address & 0x800000 > 0 {
                                    break;
                                }

                                address = next_address;
                            }
                            self.dma2.madr_write(address);
                        }
                    }
                    self.dma2.finish_dma();
                }

                Ok(())
            }
            // DMA 6 - OTC
            0x1F8010E0 => {
                event!(target: "ps1_emulator::BUS", Level::TRACE, "DMA 6 MADR write {:08X}", val);
                self.dma6.madr_write(val);
                Ok(())
            }
            0x1F8010E4 => {
                event!(target: "ps1_emulator::BUS", Level::TRACE, "DMA 6 BCR write {:08X}", val);
                self.dma6.block_control_write(val);
                Ok(())
            }
            0x1F8010E8 => {
                event!(target: "ps1_emulator::BUS", Level::TRACE, "DMA 6 CHCR write {:08X}", val);
                if self.dma6.channel_control_write(val) {
                    self.dma6.start_dma();
                    let mut address = self.dma6.madr_read();
                    match self.dma6.sync_mode {
                        SyncMode::Burst => {
                            let dma_len = self.dma6.block_control_read();
                            for i in 0..dma_len {
                                let header = if i == dma_len - 1 {
                                    0xFFFFFF
                                } else {
                                    address - 4
                                };

                                self.mem_write_word(address, header).unwrap();
                                address -= 4;
                            }
                        }
                        SyncMode::Slice => {
                            todo!("Slice mode not implemented")
                        }
                        SyncMode::LinkedList => {
                            panic!("LinkedList shouldn't happen for DMA 6");
                        }
                    }

                    self.dma6.finish_dma();
                }

                Ok(())
            }
            // DPCR - DMA Control Register
            0x1F8010F0 => {
                event!(target: "ps1_emulator::BUS", Level::TRACE, "DPCR DMA Write {:08X}", val);
                self.dma2.enabled = val & 0x800 > 0;
                self.dma6.enabled = val & 0x8000000 > 0;
                self.dpcr = val;
                Ok(())
            }
            // DICR - DMA Interrupt Register
            0x1F8010F4 => {
                event!(target: "ps1_emulator::BUS", Level::TRACE, "DICR DMA Unimplemented {:08X}", val);
                self.dicr = val;
                Ok(())
            }
            0x1F801810 => {
                self.gpu.gp0.write(val);
                Ok(())
            }
            0x1F801814 => {
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
            return Err(ExceptionType::AddressErrorLoad(addr));
        }

        Ok(u16::from_le_bytes([
            self.mem_read_byte(addr)?,
            self.mem_read_byte(addr + 1)?,
        ]))
    }

    pub fn mem_write_halfword(&mut self, addr: u32, val: u16) -> Result<(), ExceptionType> {
        if addr & 0b1 > 0 {
            return Err(ExceptionType::AddressErrorLoad(addr));
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

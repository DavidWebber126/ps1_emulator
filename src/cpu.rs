use core::fmt;

use crate::bus::Bus;

use tracing::{Level, event, span};

pub struct Registers {
    pub registers: [u32; 32],
    pub program_counter: u32,
    pub hi: u32,
    pub lo: u32,
    pub delayed_branch: Option<u32>,
    pub delayed_load: (u32, u32),
    pub delayed_load_next: (u32, u32),
}

impl Registers {
    pub fn new() -> Self {
        Self {
            registers: [0; 32],
            program_counter: 0xBFC00000,
            hi: 0,
            lo: 0,
            delayed_branch: None,
            delayed_load: (0, 0),
            delayed_load_next: (0, 0),
        }
    }

    fn read(&self, reg: u32) -> u32 {
        self.registers[reg as usize]
    }

    fn read_lwl_lwr(&self, reg: u32) -> u32 {
        // LWL and LWR can read in flight delayed loads
        if reg == self.delayed_load.0 {
            self.delayed_load.1
        } else {
            self.registers[reg as usize]
        }
    }

    fn write(&mut self, reg: u32, val: u32) {
        match reg {
            0 => {}
            1..=31 => self.registers[reg as usize] = val,
            _ => panic!("Impossible register value"),
        }

        // If register that load was going to write to gets updated then cancel the load
        if reg == self.delayed_load.0 {
            self.delayed_load = (0, 0);
        }
    }

    fn write_delayed(&mut self, reg: u32, val: u32) {
        if reg == 0 {
            return;
        }

        self.delayed_load_next = (reg, val);

        // If there was a previous delayed load to same register then cancel it
        if self.delayed_load.0 == reg {
            self.delayed_load = (0, 0);
        }
    }

    fn process_loads(&mut self) {
        let (register, value) = self.delayed_load;
        self.registers[register as usize] = value;

        self.delayed_load = self.delayed_load_next;
        self.delayed_load_next = (0, 0);
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PC: {:08X}   ", self.program_counter)?;
        for (i, val) in self.registers.iter().enumerate() {
            write!(f, "r{:02}:{:08X}", i, val)?;
            if i != 31 {
                write!(f, " ")?;
            }
        }
        Ok(())
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ExceptionType {
    Interrupt, // External Interrupt
    //TLBMod,              // TLB Modification
    //TLBLoad,             // TLB Load
    //TLBStore,            // TLB Store
    AddressErrorLoad(u32),  // Address Error, data load or instruction fetch
    AddressErrorStore(u32), // Address Error, data store
    //BusErrorFetch,       // Bus error on instruction fetch
    BusErrorLoad(u32),   // Bus error on data load/store
    Syscall,             // Syscall
    Break,               // Breakpoint
    Reserved,            // Reserved Instruction
    CoprocessorUnusable, // Coprocessor Unusable
    ArithmeticOverflow,  // Arithmetic Overflow
}

pub struct Cpu {
    pub registers: Registers,
    pub bus: Bus,
}

impl Cpu {
    pub fn new() -> Self {
        let registers = Registers::new();
        let bus = Bus::new();

        Self { registers, bus }
    }

    pub fn load_bios(&mut self, bios: &[u8]) {
        self.bus.kernel_rom[0..0x80000].clone_from_slice(bios);
    }

    pub fn sideload_exe(&mut self, exe: &[u8], tty_check: bool) {
        let bios_span = span!(target: "ps1_emulator::BIOS", Level::DEBUG, "BIOS").entered();
        bios_span.in_scope(|| {
            while self.registers.program_counter != 0x80030000 {
                self.step_instruction(tty_check);
            }
        });

        bios_span.exit();

        let initial_pc = u32::from_le_bytes(exe[0x10..0x14].try_into().unwrap());
        let initial_r28 = u32::from_le_bytes(exe[0x14..0x18].try_into().unwrap());
        let exe_ram_addr = u32::from_le_bytes(exe[0x18..0x1C].try_into().unwrap()) & 0x1FFFFF;
        let exe_size = u32::from_le_bytes(exe[0x1C..0x20].try_into().unwrap());
        let initial_sp = u32::from_le_bytes(exe[0x30..0x34].try_into().unwrap());

        println!(
            "Initial PC: 0x{:08X}, Initial r28: 0x{:08X}, Initial SP: 0x{:08X}, EXE RAM ADDR: 0x{:08X}, EXE Size: 0x{:08X}",
            initial_pc, initial_r28, initial_sp, exe_ram_addr, exe_size
        );

        let ram_start_addr = exe_ram_addr - 0x10000;
        let ram_end_addr = exe_ram_addr + exe_size - 0x10000;
        self.bus.ram[ram_start_addr as usize..ram_end_addr as usize]
            .copy_from_slice(&exe[2048..2048 + exe_size as usize]);

        self.registers.registers[28] = initial_r28;
        if initial_sp != 0 {
            self.registers.registers[29] = initial_sp;
            self.registers.registers[30] = initial_sp;
        }

        self.registers.program_counter = initial_pc;
    }

    pub fn check_for_tty_output(&self) {
        let pc = self.registers.program_counter & 0x1FFFFFFF;
        if (pc == 0xA0 && self.registers.registers[9] == 0x3C)
            || (pc == 0xB0 && self.registers.registers[9] == 0x3D)
        {
            let ch = self.registers.registers[4] as u8 as char;
            event!(target: "ps1_emulator::CPU", Level::TRACE, "TTY Output: {ch}");
            print!("{ch}");
        }
    }

    fn handle_exception(&mut self, exception: ExceptionType, in_delay_slot: bool) {
        event!(target: "ps1_emulator::CPU", Level::TRACE, "Exception Occured: {:?}", exception);
        // Store PC in EPC register (unless currently in Branch Delay in which case store PC - 4)
        if in_delay_slot {
            self.bus.cop0.epc = self.registers.program_counter - 4;
            self.bus.cop0.cause.set_branch_delay(true);
        } else {
            self.bus.cop0.epc = self.registers.program_counter;
            self.bus.cop0.cause.set_branch_delay(false);
        }

        // Store exception code in Cause register
        self.bus.cop0.cause.set_exception_code(exception);

        // Push previous interrupt/kernel bits and turn off interrupts and enable kernel mode
        self.bus.cop0.sr.push_interrupt();
        self.bus.cop0.sr.set_interrupt(false);
        self.bus.cop0.sr.set_kernel_mode(true);

        // Set BadVaddr on Address Error Exception to the problematic address
        match exception {
            ExceptionType::AddressErrorLoad(addr) | ExceptionType::AddressErrorStore(addr) => {
                self.bus.cop0.badvaddr = addr;
            }
            _ => {} // do nothing
        }

        // Jump to Exception Vector. If BEV is set then 0xBFC00180, otherwise 0x80000080
        if self.bus.cop0.sr.get_bev() {
            self.registers.program_counter = 0xBFC00180;
        } else {
            self.registers.program_counter = 0x80000080;
        }
    }

    pub fn step_instruction(&mut self, tty_check: bool) {
        let span = span!(
            Level::DEBUG,
            "CPU Step",
            pc = self.registers.program_counter
        );
        let _enter = span.enter();

        // Check for interrupts
        // Set cause bit (or clear it) if a hardware interrupt is ready
        event!(target: "ps1_emulator::CPU", Level::TRACE, "Check Interrupt");
        self.bus
            .cop0
            .cause
            .set_interrupt_pending(self.bus.interrupts.stat & self.bus.interrupts.mask > 0);

        if tty_check {
            self.check_for_tty_output();
        }

        // Execute interrupt if SR allows
        if self.bus.cop0.sr.interrupt_enabled()
            && ((self.bus.cop0.sr.interrupt_mask() & self.bus.cop0.cause.interrupt_pending()) > 0)
        {
            self.handle_exception(
                ExceptionType::Interrupt,
                self.registers.delayed_branch.is_some(),
            );
        }

        // Unaligned address exception
        if !self.registers.program_counter.is_multiple_of(4) {
            self.handle_exception(
                ExceptionType::AddressErrorLoad(self.registers.program_counter),
                false,
            );
            return
        }

        let opcode = self
            .bus
            .mem_read_word(self.registers.program_counter)
            .unwrap();

        event!(target: "ps1_emulator::CPU", Level::TRACE, "Got opcode: {:08X}", opcode);

        // If there is a branch delay, go to branch. Otherwise go to next instruction word
        let (next_pc, in_delay_slot) = match self.registers.delayed_branch.take() {
            Some(addr) => (addr, true),
            None => (self.registers.program_counter + 4, false),
        };

        self.registers.process_loads();
        
        // Let each instruction take two ticks
        // Perform before exception handler bc instruction was already executed
        self.bus.tick(2);

        // Handle Exception if something happened, otherwise go to next instruction
        if let Err(exception) = self.execute_opcode(opcode) {
            self.handle_exception(exception, in_delay_slot);
        } else {
            self.registers.program_counter = next_pc;
        }
    }

    fn execute_opcode(&mut self, opcode: u32) -> Result<(), ExceptionType> {
        match opcode {
            // ADDI
            0x20000000..=0x23FFFFFF => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let imm = (opcode & 0x0000FFFF) as i16;

                let (sum, err) = Cpu::add(self.registers.read(rs), (imm as i32) as u32);

                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", format!("ADDI ${rt}, ${rs}, {:04X}", imm), self.registers);

                if err {
                    Err(ExceptionType::ArithmeticOverflow)
                } else {
                    self.registers.write(rt, sum);
                    Ok(())
                }
            }
            // ADDIU
            0x24000000..=0x27FFFFFF => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let imm = (opcode & 0x0000FFFF) as i16;

                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", format!("ADDIU ${rt}, ${rs}, {:04X}", imm), self.registers);

                self.registers
                    .write(rt, Cpu::addu(self.registers.read(rs), (imm as i32) as u32));

                Ok(())
            }
            // ANDI
            0x30000000..=0x33FFFFFF => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let imm = opcode & 0x0000FFFF;

                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", format!("ANDI ${rt}, ${rs}, {:04X}", imm), self.registers);

                self.registers.write(rt, self.registers.read(rs) & imm);

                Ok(())
            }
            // BEQ - Branch on equal
            0x10000000..=0x13FFFFFF => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let imm = (opcode & 0x0000FFFF) as i16;

                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", format!("BEQ ${rs}, ${rt}, {:04X}", imm), self.registers);

                if self.registers.read(rs) == self.registers.read(rt) {
                    let offset = (imm as i32) << 2;
                    let offset = offset.wrapping_add(4);
                    self.registers.delayed_branch =
                        Some(self.registers.program_counter.wrapping_add(offset as u32));
                }

                Ok(())
            }
            // BGEZ - Branch on greater than or equal to zero. Name = 0b00001
            // BGEZAL - Branch on greater than or equal to zero and link. Name = 0b10001
            // BLTZ - Branch on less than zero. Name = 0b00000
            // BLTZAL - Branch on less than zero and link. Name = 0b10000
            0x04000000..=0x07FFFFFF => {
                let rs = (opcode >> 21) & 0x1F;
                let name = (opcode >> 16) & 0x1F;
                let imm = (opcode & 0x0000FFFF) as i16;

                let rs_val = self.registers.read(rs);

                match name {
                    0x10 => {
                        self.registers.registers[31] = self.registers.program_counter + 8;
                        if rs_val & 0x80000000 > 0 {
                            let offset = (imm as i32) << 2;
                            let offset = offset.wrapping_add(4);
                            self.registers.delayed_branch =
                                Some(self.registers.program_counter.wrapping_add(offset as u32));
                        }
                        event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", format!("BLTZAL ${rs}, {:04X}", imm), self.registers)
                    }
                    0x11 => {
                        self.registers.registers[31] = self.registers.program_counter + 8;
                        if rs_val & 0x80000000 == 0 {
                            let offset = (imm as i32) << 2;
                            let offset = offset.wrapping_add(4);
                            self.registers.delayed_branch =
                                Some(self.registers.program_counter.wrapping_add(offset as u32));
                        }
                        event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", format!("BGEZAL ${rs}, {:04X}", imm), self.registers)
                    }
                    _ => {
                        // Both conditions true then BGEZ, if both false then BLTZ
                        if (name & 0x1 > 0) == (rs_val & 0x80000000 == 0) {
                            let offset = (imm as i32) << 2;
                            let offset = offset.wrapping_add(4);
                            self.registers.delayed_branch =
                                Some(self.registers.program_counter.wrapping_add(offset as u32));
                        }

                        if name & 0x1 > 0 {
                            event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20} {}", format!("BGEZ ${rs}, {:04X}", imm), self.registers);
                        } else {
                            event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", format!("BLTZ ${rs}, {:04X}", imm), self.registers);
                        }
                    }
                }

                Ok(())
            }
            // BGTZ - Branch on greater than zero
            0x1C000000..=0x1FFFFFFF => {
                let rs = (opcode >> 21) & 0x1F;
                let imm = (opcode & 0x0000FFFF) as i16;

                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", format!("BGTZ ${rs}, {:04X}", imm), self.registers);

                if (self.registers.read(rs) as i32) > 0 {
                    let offset = (imm as i32) << 2;
                    let offset = offset.wrapping_add(4);
                    self.registers.delayed_branch =
                        Some(self.registers.program_counter.wrapping_add(offset as u32));
                }

                Ok(())
            }
            // BLEZ - Branch on Less than or equal to zero
            0x18000000..=0x1BFFFFFF => {
                let rs = (opcode >> 21) & 0x1F;
                let imm = (opcode & 0x0000FFFF) as i16;

                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", format!("BLEZ ${rs}, {:04X}", imm), self.registers);

                if (self.registers.read(rs) as i32) <= 0 {
                    let offset = (imm as i32) << 2;
                    let offset = offset.wrapping_add(4);
                    self.registers.delayed_branch =
                        Some(self.registers.program_counter.wrapping_add(offset as u32));
                }

                Ok(())
            }
            // BNE
            0x14000000..=0x17FFFFFF => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let imm = (opcode & 0x0000FFFF) as i16;

                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", format!("BNE ${rs}, ${rt}, {:04X}", imm), self.registers);

                if self.registers.read(rs) != self.registers.read(rt) {
                    let offset = (imm as i32) << 2;
                    let offset = offset.wrapping_add(4);
                    self.registers.delayed_branch =
                        Some(self.registers.program_counter.wrapping_add(offset as u32));
                }

                Ok(())
            }
            // JUMP
            0x08000000..=0x0BFFFFFF => {
                let target = opcode & 0x03FFFFFF;

                let calc_target = (self.registers.program_counter & 0xF0000000) | (target << 2);

                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", format!("JUMP {:08X}", calc_target), self.registers);

                self.registers.delayed_branch = Some(calc_target);

                Ok(())
            }
            // JAL - Jump and Link
            0x0C000000..=0x0FFFFFFF => {
                let target = opcode & 0x03FFFFFF;

                let calc_target = (self.registers.program_counter & 0xF0000000) | (target << 2);

                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", format!("JAL {:08X}", calc_target), self.registers);

                self.registers.registers[31] = self.registers.program_counter + 8;
                self.registers.delayed_branch = Some(calc_target);

                Ok(())
            }
            // LB - Load Byte
            0x80000000..=0x83FFFFFF => {
                let base = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let offset = (opcode & 0x0000FFFF) as i16;

                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", format!("LB ${rt}, {:04X}(${:02})", offset, base), self.registers);

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                let data = self.bus.mem_read_byte(addr)? as i8;
                self.registers.write_delayed(rt, data as i32 as u32);

                Ok(())
            }
            // LBU - Load Byte Unsigned
            0x90000000..=0x93FFFFFF => {
                let base = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let offset = (opcode & 0x0000FFFF) as i16;

                let asm = format!("LBU ${rt}, {:04X}(${:02X})", offset, base);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                let data = self.bus.mem_read_byte(addr)?;
                self.registers.write_delayed(rt, data as u32);

                Ok(())
            }
            // LH - Load Halfword
            0x84000000..=0x87FFFFFF => {
                let base = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let offset = (opcode & 0x0000FFFF) as i16;

                let asm = format!("LH ${rt}, {:04X}({:02X})", offset, base);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);

                let halfword = self.bus.mem_read_halfword(addr)? as i16;
                self.registers.write_delayed(rt, halfword as i32 as u32);

                Ok(())
            }
            // LHU - Load Halfword Unsigned
            0x94000000..=0x97FFFFFF => {
                let base = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let offset = (opcode & 0x0000FFFF) as i16;

                let asm = format!("LHU ${rt}, {:04X}({:02X})", offset, base);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                self.registers
                    .write_delayed(rt, self.bus.mem_read_halfword(addr)? as u32);

                Ok(())
            }
            // LUI - Load Upper Immediate
            0x3C000000..=0x3C1FFFFF => {
                let rt = (opcode >> 16) & 0x1F;
                let imm = opcode & 0x0000FFFF;

                let asm = format!("LUI ${rt}, {:04X}", imm);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                self.registers.write(rt, imm << 16);

                Ok(())
            }
            // LW - Load Word
            0x8C000000..=0x8FFFFFFF => {
                let base = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let offset = (opcode & 0x0000FFFF) as i16;

                let asm = format!("LW ${rt}, {:04X}(${base})", offset);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                self.registers
                    .write_delayed(rt, self.bus.mem_read_word(addr)?);

                Ok(())
            }
            // LWL - Load Word Left
            0x88000000..=0x8BFFFFFF => {
                let base = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let offset = (opcode & 0x0000FFFF) as i16;

                let asm = format!("LWL ${rt}, {:04X}({:02X})", offset, base);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let addr = self
                    .registers
                    .read_lwl_lwr(base)
                    .wrapping_add_signed(offset as i32) as usize;
                let [b0, b1, b2, b3] = self
                    .bus
                    .mem_read_word(addr as u32 & 0xFFFFFFFC)?
                    .to_le_bytes();
                let [r0, r1, r2, _] = self.registers.read_lwl_lwr(rt).to_le_bytes();
                let reg_value = match addr % 4 {
                    0 => u32::from_le_bytes([r0, r1, r2, b0]),
                    1 => u32::from_le_bytes([r0, r1, b0, b1]),
                    2 => u32::from_le_bytes([r0, b0, b1, b2]),
                    3 => u32::from_le_bytes([b0, b1, b2, b3]),
                    _ => panic!("Impossible"),
                };

                self.registers.write_delayed(rt, reg_value);

                Ok(())
            }
            // LWR - Load Word Right
            0x98000000..=0x9BFFFFFF => {
                let base = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let offset = (opcode & 0x0000FFFF) as i16;

                let asm = format!("LWR ${rt}, {:04X}(${base})", offset);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let addr = self
                    .registers
                    .read_lwl_lwr(base)
                    .wrapping_add_signed(offset as i32) as usize;
                let [b0, b1, b2, b3] = self
                    .bus
                    .mem_read_word(addr as u32 & 0xFFFFFFFC)?
                    .to_le_bytes();
                let [_, r1, r2, r3] = self.registers.read_lwl_lwr(rt).to_le_bytes();
                let reg_value = match addr % 4 {
                    0 => u32::from_le_bytes([b0, b1, b2, b3]),
                    1 => u32::from_le_bytes([b1, b2, b3, r3]),
                    2 => u32::from_le_bytes([b2, b3, r2, r3]),
                    3 => u32::from_le_bytes([b3, r1, r2, r3]),
                    _ => panic!("Impossible"),
                };

                self.registers.write_delayed(rt, reg_value);

                Ok(())
            }
            // ORI - Or Immediate
            0x34000000..=0x37FFFFFF => {
                let rs = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let imm = opcode & 0x0000FFFF;

                let asm = format!("ORI ${rt}, ${rs}, {:04X}", imm);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                self.registers.write(rt, self.registers.read(rs) | imm);

                Ok(())
            }
            // SB - Store Byte
            0xA0000000..=0xA3FFFFFF => {
                let base = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let offset = (opcode & 0x0000FFFF) as i16;

                let asm = format!("SB ${rt}, {:04X}(${base})", offset);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                let byte = (self.registers.read(rt) & 0x000000FF) as u8;
                self.bus.mem_write_byte(addr, byte)?;

                Ok(())
            }
            // SH - Store Halfword
            0xA4000000..=0xA7FFFFFF => {
                let base = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let offset = (opcode & 0x0000FFFF) as i16;

                let asm = format!("SH ${rt}, {:04X}(${base})", offset);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                if addr.is_multiple_of(2) {
                    let halfbyte = (self.registers.read(rt) & 0x0000FFFF) as u16;
                    self.bus.mem_write_halfword(addr, halfbyte)?;
                    Ok(())
                } else {
                    Err(ExceptionType::AddressErrorStore(addr))
                }
            }
            // SLTI - Set on Less Than Immediate
            0x28000000..=0x2BFFFFFF => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let imm = (opcode & 0x0000FFFF) as i16;

                let asm = format!("SLTI ${rt}, ${rs}, {:04X}", imm);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                if (self.registers.read(rs) as i32) < imm as i32 {
                    self.registers.write(rt, 1);
                } else {
                    self.registers.write(rt, 0);
                }

                Ok(())
            }
            // SLTIU
            0x2C000000..=0x2FFFFFFF => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let imm = (opcode & 0x0000FFFF) as i16;

                let asm = format!("SLTIU ${rt}, ${rs}, {:04X}", imm);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                if self.registers.read(rs) < (imm as i32) as u32 {
                    self.registers.write(rt, 1);
                } else {
                    self.registers.write(rt, 0);
                }

                Ok(())
            }
            // SW - Store Word
            0xAC000000..=0xAFFFFFFF => {
                let base = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let offset = (opcode & 0x0000FFFF) as i16;

                let asm = format!("SW ${rt}, {:04X}(${})", offset, base);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                if addr.is_multiple_of(4) {
                    self.bus.mem_write_word(addr, self.registers.read(rt))?;
                    Ok(())
                } else {
                    Err(ExceptionType::AddressErrorStore(addr))
                }
            }
            // SWL - Store Word Left
            0xA8000000..=0xABFFFFFF => {
                let base = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let offset = (opcode & 0x0000FFFF) as i16;

                let asm = format!("SWL ${rt}, {:04X}({:02X})", offset, base);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                let [b0, b1, b2, b3] = self.registers.read(rt).to_le_bytes();
                match addr % 4 {
                    0 => {
                        self.bus.mem_write_byte(addr, b3)?;
                    }
                    1 => {
                        self.bus.mem_write_byte(addr, b3)?;
                        self.bus.mem_write_byte(addr - 1, b2)?;
                    }
                    2 => {
                        self.bus.mem_write_byte(addr, b3)?;
                        self.bus.mem_write_byte(addr - 1, b2)?;
                        self.bus.mem_write_byte(addr - 2, b1)?;
                    }
                    3 => {
                        self.bus.mem_write_byte(addr, b3)?;
                        self.bus.mem_write_byte(addr - 1, b2)?;
                        self.bus.mem_write_byte(addr - 2, b1)?;
                        self.bus.mem_write_byte(addr - 3, b0)?;
                    }
                    _ => panic!("Impossible"),
                };

                Ok(())
            }
            // SWR - Store Word Right
            0xB8000000..=0xBBFFFFFF => {
                let base = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let offset = (opcode & 0x0000FFFF) as i16;

                let asm = format!("SWR ${rt}, {:04X}({:02X})", offset, base);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                let [b0, b1, b2, b3] = self.registers.read(rt).to_le_bytes();
                match addr % 4 {
                    0 => {
                        self.bus.mem_write_byte(addr, b0)?;
                        self.bus.mem_write_byte(addr + 1, b1)?;
                        self.bus.mem_write_byte(addr + 2, b2)?;
                        self.bus.mem_write_byte(addr + 3, b3)?;
                    }
                    1 => {
                        // self.bus.mem_write_byte(addr, b3)?;
                        // self.bus.mem_write_byte(addr - 1, b2)?;
                        self.bus.mem_write_byte(addr, b0)?;
                        self.bus.mem_write_byte(addr + 1, b1)?;
                        self.bus.mem_write_byte(addr + 2, b2)?;
                    }
                    2 => {
                        // self.bus.mem_write_byte(addr, b3)?;
                        // self.bus.mem_write_byte(addr - 1, b2)?;
                        // self.bus.mem_write_byte(addr - 2, b1)?;
                        self.bus.mem_write_byte(addr, b0)?;
                        self.bus.mem_write_byte(addr + 1, b1)?;
                    }
                    3 => {
                        // self.bus.mem_write_byte(addr, b3)?;
                        // self.bus.mem_write_byte(addr - 1, b2)?;
                        // self.bus.mem_write_byte(addr - 2, b1)?;
                        // self.bus.mem_write_byte(addr - 3, b0)?;
                        self.bus.mem_write_byte(addr, b0)?;
                    }
                    _ => panic!("Impossible"),
                };

                Ok(())
            }
            // XORI
            0x38000000..=0x3BFFFFFF => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let imm = opcode & 0x0000FFFF;

                let asm = format!("SLTIU ${rt}, ${rs}, {:04X}", imm);
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                self.registers.write(rt, self.registers.read(rs) ^ imm);

                Ok(())
            }
            // Coprocessor
            // CFC0 - Move Control From Coprocessor 0
            0x40400000..=0x405FFFFF => {
                panic!("CFC is invalid for Coprocessor 0")
            }
            // CFC1 - Move Control From Coprocessor 1
            0x44400000..=0x445FFFFF => {
                panic!("No Coprocessor 1")
            }
            // CFC2 - Move Control From Coprocessor 2
            0x48400000..=0x485FFFFF => {
                todo!()
            }
            // CFC3 - Move Control From Coprocessor 3
            0x4C400000..=0x4C5FFFFF => {
                panic!("No Coprocessor 3")
            }
            // COP0 - Coprocessor Operation 0
            // RFE - Return from Exception
            0x42000010 => {
                let asm = "COP0 RFE";
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);
                self.bus.cop0.sr.pop_interrupt();
                Ok(())
            }
            // TLBP, TLBR, TLBWI, TLBWR - Returns Reserved Instruction Exception
            0x42000008 | 0x42000001 | 0x42000002 | 0x42000006 => {
                let asm = "COP0 TLBP/TLBR/TLBWI/TLBWR";
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);
                Err(ExceptionType::Reserved)
            }
            // COP1 - Coprocessor Operation 1
            0x46000000..=0x47FFFFFF => {
                panic!("No Coprocessor 1")
            }
            // COP2 - Coprocessor Operation 2
            0x4A000000..=0x4BFFFFFF => {
                todo!()
            }
            // COP3 - Coprocessor Operation 3
            0x4E000000..=0x4FFFFFFF => {
                panic!("No Coprocessor 3")
            }
            // CTC0 - Move Control To Coprocessor 0
            0x40C00000..=0x40DFFFFF => {
                panic!("CTC is invalid for Coprocessor 0")
            }
            // CTC1 - Move Control To Coprocessor 1
            0x44C00000..=0x44DFFFFF => {
                panic!("No Coprocessor 1")
            }
            // CTC2 - Move Control To Coprocessor 2
            0x48C00000..=0x48DFFFFF => {
                todo!()
            }
            // CTC3 - Move Control To Coprocessor 3
            0x4CC00000..=0x4CDFFFFF => {
                panic!("No Coprocessor 3")
            }
            // LWC0 - Load Word to Coprocessor 0
            0xC0000000..=0xC3FFFFFF => {
                panic!("LWC is invalid for Coprocessor 0")
            }
            // LWC1 - Load Word to Coprocessor 1
            0xC4000000..=0xC7FFFFFF => {
                panic!("No Coprocessor 1")
            }
            // LWC2 - Load Word to Coprocessor 2
            0xC8000000..=0xCBFFFFFF => {
                todo!()
            }
            // LWC3 - Load Word to Coprocessor 3
            0xCC000000..=0xCFFFFFFF => {
                panic!("No Coprocessor 3")
            }
            // MFC0 - Move From Coprocessor 0
            0x40000000..=0x401FFFFF if opcode & 0x7FF == 0 => {
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("MFC0 ${rt}, ${rd}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                if let Ok(val) = self.bus.cop0.register_read(rd) {
                    self.registers.write(rt, val);
                    Ok(())
                } else {
                    Err(ExceptionType::CoprocessorUnusable)
                }
            }
            // MFC1 - Move From Coprocessor 1
            0x44000000..=0x441FFFFF => {
                panic!("No Coprocessor 1")
            }
            // MFC2 - Move From Coprocessor 2
            0x48000000..=0x481FFFFF => {
                todo!()
            }
            // MFC3 - Move From Coprocesor 3
            0x4C000000..=0x4C1FFFFF => {
                panic!("No Coprocessor 3")
            }
            // MTC0 - Move To Coprocessor 0
            0x40800000..=0x409FFFFF if opcode & 0x7FF == 0 => {
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("MTC0 ${rt}, ${rd}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                self.bus.cop0.register_write(rd, self.registers.read(rt))?;

                Ok(())
            }
            // MTC1 - Move to Coprocessor 1
            0x44800000..=0x449FFFFF => {
                panic!("No Coprocessor 1")
            }
            // MTC2 - Move to Coprocessor 2
            0x48800000..=0x489FFFFF => {
                todo!()
            }
            // MTC3 - Move to Coprocessor 3
            0x4C800000..=0x4C9FFFFF => {
                panic!("No Coprocessor 3")
            }
            // SWC0 - Store Word from Coprocessor 0
            0xE0000000..=0xE3FFFFFF => Err(ExceptionType::Reserved),
            // SWC1 - Store Word from Coprocessor 1
            0xE4000000..=0xE7FFFFFF => {
                panic!("No Coprocessor 1")
            }
            // SWC2 - Store Word from Coprocessor 2
            0xE8000000..=0xEBFFFFFF => {
                todo!()
            }
            // SWC3 - Store Word from Coprocessor 3
            0xEC000000..=0xEFFFFFFF => {
                panic!("No Coprocessor 3")
            }
            // Special
            // ADD
            op if op & 0xFC00003F == 0x00000020 => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("ADD ${rd}, ${rs}, ${rt}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let (sum, err) = Cpu::add(self.registers.read(rs), self.registers.read(rt));

                if err {
                    Err(ExceptionType::ArithmeticOverflow)
                } else {
                    self.registers.write(rd, sum);
                    Ok(())
                }
            }
            // ADDU
            op if op & 0xFC00003F == 0x00000021 => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("ADDU ${rd}, ${rs}, ${rt}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let sum = Cpu::addu(self.registers.read(rs), self.registers.read(rt));
                self.registers.write(rd, sum);

                Ok(())
            }
            // AND
            op if op & 0xFC00003F == 0x00000024 => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("AND ${rd}, ${rs}, ${rt}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                self.registers
                    .write(rd, self.registers.read(rs) & self.registers.read(rt));

                Ok(())
            }
            // BREAK
            op if op & 0xFC00003F == 0x0000000D => {
                let asm = "BREAK";
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);
                Err(ExceptionType::Break)
            }
            // DIV
            op if op & 0xFC00003F == 0x0000001A => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;

                let asm = format!("DIV ${rs}, ${rt}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let dividend = self.registers.read(rs) as i32;
                let divisor = self.registers.read(rt) as i32;
                if divisor == 0 {
                    self.registers.hi = dividend as u32;
                    if dividend >= 0 {
                        self.registers.lo = 0xFFFFFFFF;
                    } else {
                        self.registers.lo = 1;
                    }
                } else if dividend == i32::MIN && divisor == -1 {
                    self.registers.lo = 0x80000000;
                    self.registers.hi = 0;
                } else {
                    self.registers.lo = (dividend / divisor) as u32;
                    self.registers.hi = (dividend % divisor) as u32;
                }

                Ok(())
            }
            // DIVU
            op if op & 0xFC00003F == 0x0000001B => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;

                let asm = format!("DIVU ${rs}, ${rt}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let dividend = self.registers.read(rs);
                let divisor = self.registers.read(rt);

                if divisor == 0 {
                    self.registers.hi = dividend;
                    self.registers.lo = 0xFFFFFFFF;
                } else {
                    self.registers.lo = dividend / divisor;
                    self.registers.hi = dividend % divisor;
                }

                Ok(())
            }
            // JALR - Jump and Link Register
            op if op & 0xFC00003F == 0x00000009 => {
                let rs = (opcode >> 21) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("JALR ${rd}, ${rs}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let addr = self.registers.read(rs);
                self.registers.write(rd, self.registers.program_counter + 8);
                self.registers.delayed_branch = Some(addr);

                Ok(())
            }
            // JR
            op if op & 0xFC00003F == 0x00000008 => {
                let rs = (opcode >> 21) & 0x1F;
                let target = self.registers.read(rs);

                let asm = format!("JR ${rs}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                self.registers.delayed_branch = Some(target);

                Ok(())
            }
            // MFHI - Move From HI
            op if op & 0xFFFF07FF == 0x00000010 => {
                let rd = (opcode >> 11) & 0x1F;
                self.registers.write(rd, self.registers.hi);

                let asm = format!("MFHI ${rd}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                Ok(())
            }
            // MFLO - Move From LO
            op if op & 0xFFFF07FF == 0x00000012 => {
                let rd = (opcode >> 11) & 0x1F;
                self.registers.write(rd, self.registers.lo);

                let asm = format!("MFLO ${rd}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                Ok(())
            }
            // MTHI - Move To HI
            op if op & 0xFC1FFFFF == 0x00000011 => {
                let rs = (opcode >> 21) & 0x1F;
                self.registers.hi = self.registers.read(rs);

                let asm = format!("MTHI ${rs}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                Ok(())
            }
            // MTLO - Move To LO
            op if op & 0xFC1FFFFF == 0x00000013 => {
                let rs = (opcode >> 21) & 0x1F;
                self.registers.lo = self.registers.read(rs);

                let asm = format!("MTLO ${rs}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                Ok(())
            }
            // MULT - Multiply Word
            op if op & 0xFC00FFFF == 0x00000018 => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;

                let asm = format!("MULT ${rs}, ${rt}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let arg1 = self.registers.read(rs) as i32;
                let arg2 = self.registers.read(rt) as i32;
                let product = (arg1 as i64 * arg2 as i64) as u64;

                self.registers.lo = (product & 0x00000000FFFFFFFF) as u32;
                self.registers.hi = ((product & 0xFFFFFFFF00000000) >> 32) as u32;

                Ok(())
            }
            // MULTU - Multiply Unsigned Word
            op if op & 0xFC00FFFF == 0x00000019 => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;

                let asm = format!("MULTU ${rs}, ${rt}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let arg1 = self.registers.read(rs) as u64;
                let arg2 = self.registers.read(rt) as u64;
                let product = arg1 * arg2;

                self.registers.lo = (product & 0x00000000FFFFFFFF) as u32;
                self.registers.hi = ((product & 0xFFFFFFFF00000000) >> 32) as u32;

                Ok(())
            }
            // NOR
            op if op & 0xFC0007FF == 0x00000027 => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("NOR ${rd}, ${rs}, ${rt}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                self.registers
                    .write(rd, !(self.registers.read(rs) | self.registers.read(rt)));

                Ok(())
            }
            // OR
            op if op & 0xFC0007FF == 0x00000025 => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("OR ${rd}, ${rs}, ${rt}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                self.registers
                    .write(rd, self.registers.read(rs) | self.registers.read(rt));

                Ok(())
            }
            // SLL - Shift Word Left Logical
            op if op & 0xFFE0003F == 0x00000000 => {
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;
                let sa = (opcode >> 6) & 0x1F;

                let asm = format!("SLL ${rd}, ${rt}, {sa}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                self.registers.write(rd, self.registers.read(rt) << sa);

                Ok(())
            }
            // SLLV - Shift Word Left Logical Variable
            op if op & 0xFC0007FF == 0x00000004 => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("SLLV ${rd}, ${rt}, ${rs}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let shift = self.registers.read(rs) & 0x1F;
                self.registers.write(rd, self.registers.read(rt) << shift);

                Ok(())
            }
            // SLT - Set on Less Than
            op if op & 0xFC0007FF == 0x0000002A => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("SLT ${rd}, ${rs}, ${rt}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let result = (self.registers.read(rs) as i32) < self.registers.read(rt) as i32;
                self.registers.write(rd, result as u32);

                Ok(())
            }
            // SLTU - Set on Less Than Unsigned
            op if op & 0xFC0007FF == 0x0000002B => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("SLTU ${rd}, ${rs}, ${rt}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let result = self.registers.read(rs) < self.registers.read(rt);
                self.registers.write(rd, result as u32);

                Ok(())
            }
            // SRA - Shift Word Right Arithmetic
            op if op & 0xFFE0003F == 0x00000003 => {
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;
                let sa = (opcode >> 6) & 0x1F;

                let asm = format!("SRA ${rd}, ${rt}, {sa}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                self.registers
                    .write(rd, ((self.registers.read(rt) as i32) >> sa) as u32);

                Ok(())
            }
            // SRAV - Shift Word Right Arithmetic Variable
            op if op & 0xFC0007FF == 0x00000007 => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("SRAV ${rd}, ${rt}, ${rs}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let shift = self.registers.read(rs) & 0b11111;
                self.registers
                    .write(rd, ((self.registers.read(rt) as i32) >> shift) as u32);

                Ok(())
            }
            // SRL - Shift Word Right Logical
            op if op & 0xFFE0003F == 0x00000002 => {
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;
                let sa = (opcode >> 6) & 0x1F;

                let asm = format!("SRL ${rd}, ${rt}, {sa}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                self.registers.write(rd, self.registers.read(rt) >> sa);

                Ok(())
            }
            // SRLV - Shift Word Right Logical Variable
            op if op & 0xFC0007FF == 0x00000006 => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("SRLV ${rd}, ${rt}, ${rs}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let shift = self.registers.read(rs) & 0b11111;
                self.registers.write(rd, self.registers.read(rt) >> shift);

                Ok(())
            }
            // SUB - Subtract Word
            op if op & 0xFC0007FF == 0x00000022 => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("SUB ${rd}, ${rs}, {rt}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                let lhs = self.registers.read(rs);
                let rhs = self.registers.read(rt);
                let (diff, err) = (lhs as i32).overflowing_sub(rhs as i32); //Cpu::add(lhs, (!rhs).wrapping_add(1));

                if err {
                    Err(ExceptionType::ArithmeticOverflow)
                } else {
                    self.registers.write(rd, diff as u32);
                    Ok(())
                }
            }
            // SUBU - Subtract Unsigned Word
            op if op & 0xFC0007FF == 0x00000023 => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("SUBU ${rd}, ${rs}, {rt}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                self.registers.write(
                    rd,
                    Cpu::addu(
                        self.registers.read(rs),
                        (!self.registers.read(rt)).wrapping_add(1),
                    ),
                );

                Ok(())
            }
            // SYSCALL
            op if op & 0xFC00003F == 0x0000000C => {
                let asm = "SYSCALL";
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);
                Err(ExceptionType::Syscall)
            }
            // XOR
            op if op & 0xFC0007FF == 0x00000026 => {
                let rs = (opcode >> 21) & 0x1F;
                let rt = (opcode >> 16) & 0x1F;
                let rd = (opcode >> 11) & 0x1F;

                let asm = format!("XOR ${rd}, ${rs}, {rt}");
                event!(target: "ps1_emulator::CPU", Level::DEBUG, "{:<20}  {}", asm, self.registers);

                self.registers
                    .write(rd, self.registers.read(rs) ^ self.registers.read(rt));

                Ok(())
            }
            _ => {
                event!(target: "ps1_emulator::CPU",
                    Level::ERROR,
                    "Received {:08X} as opcode but no matching instruction",
                    opcode
                );
                panic!()
            }
        }
    }

    // Causes an exception on overflow, indicated by true in bool
    fn add(arg1: u32, arg2: u32) -> (u32, bool) {
        let lhs = arg1 as i32;
        let rhs = arg2 as i32;
        let (result, err) = lhs.overflowing_add(rhs);
        (result as u32, err)
    }

    // Does not cause an exception on overflow
    fn addu(arg1: u32, arg2: u32) -> u32 {
        arg1.wrapping_add(arg2)
    }
}

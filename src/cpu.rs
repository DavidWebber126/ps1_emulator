use core::panic;

use crate::bus::Bus;

pub struct Registers {
    registers: [u32; 32],
    program_counter: u32,
    hi: u32,
    lo: u32,
    delayed_branch: Option<u32>,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            registers: [0; 32],
            program_counter: 0,
            hi: 0,
            lo: 0,
            delayed_branch: None,
        }
    }

    fn read(&self, reg: u32) -> u32 {
        match reg {
            0 => 0,
            1..31 => self.registers[reg as usize],
            _ => panic!("Impossible"),
        }
    }

    fn write(&mut self, reg: u32, val: u32) {
        match reg {
            0 => {}
            1..31 => self.registers[reg as usize] = val,
            _ => panic!("Impossible"),
        }
    }
}

pub struct Cpu {
    // Thirty Two 32-bit registers in an array
    registers: Registers,
    bus: Bus,
}

impl Cpu {
    pub fn new() -> Self {
        let registers = Registers::new();
        let bus = Bus::new();
        Self { registers, bus }
    }

    pub fn step_instruction(&mut self) {
        let opcode = self.bus.mem_read_word(self.registers.program_counter);

        let (next_pc, in_delay_slot) = match self.registers.delayed_branch.take() {
            Some(addr) => (addr, true),
            None => (self.registers.program_counter + 4, false),
        };

        self.execute_opcode(opcode);

        self.registers.program_counter = next_pc;
    }

    fn execute_opcode(&mut self, opcode: u32) {
        match opcode {
            // ADDI
            0x20000000..=0x20FFFFFF => {
                let reg = (opcode & 0x03E00000) >> 21;
                let imm = (opcode & 0x0000FFFF) as i16;
                let target = (opcode & 0x001F0000) >> 16;

                self.registers.write(
                    target,
                    Cpu::add(self.registers.read(reg), (imm as i32) as u32),
                );
            }
            // ADDIU
            0x21000000..=0x21FFFFFF => {
                let reg = (opcode & 0x03E00000) >> 21;
                let imm = (opcode & 0x0000FFFF) as i16;
                let target = (opcode & 0x001F0000) >> 16;

                self.registers.write(
                    target,
                    Cpu::addu(self.registers.read(reg), (imm as i32) as u32),
                );
            }
            // ANDI
            0x30000000..=0x33FFFFFF => {
                let reg = (opcode & 0x03E00000) >> 21;
                let imm = (opcode & 0x0000FFFF) as i16;
                let target = (opcode & 0x001F0000) >> 16;

                self.registers
                    .write(target, self.registers.read(reg) & ((imm as i32) as u32));
            }
            // BEQ - Branch on equal
            0x10000000..=0x13000000 => {
                let source = (opcode & 0x03E00000) >> 21;
                let imm = (opcode & 0x0000FFFF) as i16;
                let target = (opcode & 0x001F0000) >> 16;

                if self.registers.read(source) == self.registers.read(target) {
                    let offset = (imm as i32) << 2;
                    self.registers.program_counter =
                        self.registers.program_counter.wrapping_add(offset as u32);
                }
            }
            // BGEZ - Branch on greater than or equal to zero. Name = 0b00001
            // BGEZAL - Branch on greater than or equal to zero and link. Name = 0b10001
            // BLTZ - Branch on less than zero. Name = 0b00000
            // BLTZAL - Branch on less than zero and link. Name = 0b10000
            0x04000000..=0x07FFFFFF => {
                let reg = (opcode & 0x03E00000) >> 21;
                let name = (opcode & 0x001F0000) >> 16;
                let imm = (opcode & 0x0000FFFF) as i16;

                // BGEZAL and BLTZAL unconditionally set register 31
                if name & 0b10000 > 0 {
                    self.registers.registers[31] = self.registers.program_counter + 8;
                }

                // Both conditions true then BGEZ/BGEZAL, if both false then BLTZ/BLTZAL
                if (name & 0x1 > 0) == (self.registers.read(reg) & 0x80000000 == 0) {
                    let offset = (imm as i32) << 2;
                    self.registers.program_counter =
                        self.registers.program_counter.wrapping_add(offset as u32);
                }
            }
            // BGTZ - Branch on greater than or equal to zero
            0x1C000000..=0x1FFFFFFF => {
                let reg = (opcode & 0x03E00000) >> 21;
                let imm = (opcode & 0x0000FFFF) as i16;

                if self.registers.read(reg) & 0x80000000 == 0 && self.registers.read(reg) > 0 {
                    let offset = (imm as i32) << 2;
                    self.registers.program_counter =
                        self.registers.program_counter.wrapping_add(offset as u32);
                }
            }
            // BNE
            0x14000000..=0x17FFFFFF => {
                let source = (opcode & 0x03E00000) >> 21;
                let imm = (opcode & 0x0000FFFF) as i16;
                let target = (opcode & 0x001F0000) >> 16;

                if self.registers.read(source) != self.registers.read(target) {
                    let offset = (imm as i32) << 2;
                    self.registers.program_counter =
                        self.registers.program_counter.wrapping_add(offset as u32);
                }
            }
            // JUMP
            0x08000000..=0x0BFFFFFF => {
                let target = (opcode & 0x03FFFFFF) << 2;

                self.registers.program_counter =
                    (self.registers.program_counter & 0x0FFFFFFF) | target;
            }
            // JAL - Jump and Link
            0x0C000000..=0x0FFFFFFF => {
                let target = (opcode & 0x03FFFFFF) << 2;

                self.registers.registers[31] = self.registers.program_counter + 8;
                self.registers.program_counter =
                    (self.registers.program_counter & 0x0FFFFFFF) | target;
            }
            // LB - Load Byte
            0x80000000..=0x83FFFFFF => {
                let base = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let offset = (opcode & 0x0000FFFF) as i16;

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                let data = self.bus.mem_read_byte(addr) as i8;
                self.registers.write(rt, data as i32 as u32);
            }
            // LBU - Load Byte Unsigned
            0x90000000..=0x93FFFFFF => {
                let base = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let offset = (opcode & 0x0000FFFF) as i16;

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                self.registers
                    .write(rt, self.bus.mem_read_byte(addr) as u32);
            }
            // LH - Load Halfword
            0x84000000..=0x87FFFFFF => {
                let base = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let offset = (opcode & 0x0000FFFF) as i16;

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                let halfword = self.bus.mem_read_halfword(addr) as i16;
                self.registers.write(rt, halfword as i32 as u32);
            }
            // LHU - Load Halfword Unsigned
            0x94000000..=0x97FFFFFF => {
                let base = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let offset = (opcode & 0x0000FFFF) as i16;

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                self.registers
                    .write(rt, self.bus.mem_read_halfword(addr) as u32);
            }
            // LUI - Load Upper Immediate
            0x3C000000..=0x3C1FFFFF => {
                let target = (opcode & 0x001F0000) >> 16;
                let imm = (opcode & 0x0000FFFF) << 16;

                self.registers.write(target, imm);
            }
            // LW - Load Word
            0x8C000000..=0x8FFFFFFF => {
                let base = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let offset = (opcode & 0x0000FFFF) as i16;

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                self.registers.write(rt, self.bus.mem_read_word(addr));
            }
            // LWL - Load Word Left
            0x88000000..=0x8BFFFFFF => {
                let base = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let offset = (opcode & 0x0000FFFF) as i16;

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32) as usize;
                let [b0, b1, b2, b3] = self
                    .bus
                    .mem_read_word(addr as u32 & 0xFFFFFFFC)
                    .to_le_bytes();
                let [_, r1, r2, r3] = self.registers.read(rt).to_le_bytes();
                match addr % 4 {
                    0 => self
                        .registers
                        .write(rt, u32::from_le_bytes([b0, b1, b2, b3])),
                    1 => self
                        .registers
                        .write(rt, u32::from_le_bytes([b1, b2, b3, r3])),
                    2 => self
                        .registers
                        .write(rt, u32::from_le_bytes([b2, b3, r2, r3])),
                    3 => self
                        .registers
                        .write(rt, u32::from_le_bytes([b3, r1, r2, r3])),
                    _ => panic!("Impossible"),
                }
            }
            // LWR - Load Word Right
            0x98000000..=0x9BFFFFFF => {
                let base = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let offset = (opcode & 0x0000FFFF) as i16;

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32) as usize;
                let [b0, b1, b2, b3] = self
                    .bus
                    .mem_read_word(addr as u32 & 0xFFFFFFFC)
                    .to_le_bytes();
                let [r0, r1, r2, _] = self.registers.read(rt).to_le_bytes();
                match addr % 4 {
                    0 => self
                        .registers
                        .write(rt, u32::from_le_bytes([r0, r1, r2, b0])),
                    1 => self
                        .registers
                        .write(rt, u32::from_le_bytes([r0, r1, b0, b1])),
                    2 => self
                        .registers
                        .write(rt, u32::from_le_bytes([r0, b0, b1, b2])),
                    3 => self
                        .registers
                        .write(rt, u32::from_le_bytes([b0, b1, b2, b3])),
                    _ => panic!("Impossible"),
                }
            }
            // ORI - Or Immediate
            0x34000000..=0x37FFFFFF => {
                let source = (opcode & 0x03E00000) >> 21;
                let target = (opcode & 0x001F0000) >> 16;
                let imm = opcode & 0x0000FFFF;

                self.registers
                    .write(target, self.registers.read(source) | imm);
            }
            // SB - Store Byte
            0xA0000000..=0xA3FFFFFF => {
                let base = (opcode & 0x03E00000) >> 21;
                let target = (opcode & 0x001F0000) >> 16;
                let offset = (opcode & 0x0000FFFF) as i16;

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                let byte = (self.registers.read(target) & 0x000000FF) as u8;
                self.bus.mem_write_byte(addr, byte);
            }
            // SH - Store Halfword
            0xA4000000..=0xA7FFFFFF => {
                let base = (opcode & 0x03E00000) >> 21;
                let target = (opcode & 0x001F0000) >> 16;
                let offset = (opcode & 0x0000FFFF) as i16;

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                let halfbyte = (self.registers.read(target) & 0x0000FFFF) as u16;
                self.bus.mem_write_halfword(addr, halfbyte);
            }
            // SLTI - Set on Less Than Immediate
            0x28000000..=0x2BFFFFFF => {
                let rs = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let imm = (opcode & 0x0000FFFF) as i16;

                if (self.registers.read(rs) as i32) < imm as i32 {
                    self.registers.write(rt, 1);
                } else {
                    self.registers.write(rt, 0);
                }
            }
            // SLTIU
            0x2C000000..=0x2FFFFFFF => {
                let rs = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let imm = (opcode & 0x0000FFFF) as i16;

                if self.registers.read(rs) < (imm as i32) as u32 {
                    self.registers.write(rt, 1);
                } else {
                    self.registers.write(rt, 0);
                }
            }
            // SW - Store Word
            0xAC000000..=0xAFFFFFFF => {
                let base = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let offset = (opcode & 0x0000FFFF) as i16;

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32);
                self.bus.mem_write_word(addr, self.registers.read(rt));
            }
            // SWL - Store Word Left
            0xA8000000..=0xABFFFFFF => {
                let base = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let offset = (opcode & 0x0000FFFF) as i16;

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32) as usize;
                let [b0, b1, b2, b3] = self.registers.read(rt).to_le_bytes();
                match addr % 4 {
                    0 => {
                        self.bus.ram[addr] = b0;
                        self.bus.ram[addr + 1] = b1;
                        self.bus.ram[addr + 2] = b2;
                        self.bus.ram[addr + 3] = b3;
                    }
                    1 => {
                        self.bus.ram[addr] = b0;
                        self.bus.ram[addr + 1] = b1;
                        self.bus.ram[addr + 2] = b2;
                    }
                    2 => {
                        self.bus.ram[addr] = b0;
                        self.bus.ram[addr + 1] = b1;
                    }
                    3 => {
                        self.bus.ram[addr] = b0;
                    }
                    _ => panic!("Impossible"),
                }
            }
            // SWR - Store Word Right
            0xB8000000..=0xBBFFFFFF => {
                let base = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let offset = (opcode & 0x0000FFFF) as i16;

                let addr = self.registers.read(base).wrapping_add_signed(offset as i32) as usize;
                let [b0, b1, b2, b3] = self.registers.read(rt).to_le_bytes();
                match addr % 4 {
                    0 => {
                        self.bus.ram[addr] = b3;
                    }
                    1 => {
                        self.bus.ram[addr] = b3;
                        self.bus.ram[addr - 1] = b2;
                    }
                    2 => {
                        self.bus.ram[addr] = b3;
                        self.bus.ram[addr - 1] = b2;
                        self.bus.ram[addr - 2] = b1;
                    }
                    3 => {
                        self.bus.ram[addr] = b3;
                        self.bus.ram[addr - 1] = b2;
                        self.bus.ram[addr - 2] = b1;
                        self.bus.ram[addr - 3] = b0;
                    }
                    _ => panic!("Impossible"),
                }
            }
            // XORI
            0x38000000..=0x3BFFFFFF => {
                let rs = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let imm = opcode & 0x0000FFFF;

                self.registers.write(rt, self.registers.read(rs) ^ imm);
            }
            // Coprocessor
            // CFC0 - Move Control From Coprocessor 0
            
            // LWC0 - Load Word to Coprocessor 0
            0xC0000000..=0xC3FFFFFF => {
                todo!()
            }
            // LWC1 - Load Word to Coprocessor 1
            0xC4000000..=0xC7FFFFFF => {
                todo!()
            }
            // LWC2 - Load Word to Coprocessor 2
            0xC8000000..=0xCBFFFFFF => {
                todo!()
            }
            // LWC3 - Load Word to Coprocessor 3
            0xCC000000..=0xCFFFFFFF => {
                todo!()
            }
            // MFC0 - Move From Coprocessor 0
            0x40000000..=0x401FFFFF => {
                todo!()
            }
            // MFC1 - Move From Coprocessor 1
            0x44000000..=0x441FFFFF => {
                todo!()
            }
            // MFC2 - Move From Coprocessor 2
            0x48000000..=0x481FFFFF => {
                todo!()
            }
            // MFC3 - Move From Coprocesor 3
            0x4C000000..=0x4C1FFFFF => {
                todo!()
            }
            // MTC0 - Move To Coprocessor 0
            0x40800000..=0x409FFFFF => {
                todo!()
            }
            // MTC1 - Move to Coprocessor 1
            0x44800000..=0x449FFFFF => {
                todo!()
            }
            // MTC2 - Move to Coprocessor 2
            0x48800000..=0x489FFFFF => {
                todo!()
            }
            // MTC3 - Move to Coprocessor 3
            0x4C800000..=0x4C9FFFFF => {
                todo!()
            }
            // SWC0 - Store Word from Coprocessor 0
            0xE0000000..=0xE3FFFFFF => {
                todo!()
            }
            // SWC1 - Store Word from Coprocessor 1
            0xE4000000..=0xE7FFFFFF => {
                todo!()
            }
            // SWC2 - Store Word from Coprocessor 2
            0xE8000000..=0xEBFFFFFF => {
                todo!()
            }
            // SWC3 - Store Word from Coprocessor 3
            0xEC000000..=0xEFFFFFFF => {
                todo!()
            }
            // Special
            // ADD
            op if op & 0xFC00003F == 0x00000020 => {
                let reg1 = (opcode & 0x03E00000) >> 21;
                let reg2 = (opcode & 0x001F0000) >> 16;
                let target = (opcode & 0x0000F800) >> 11;

                let sum = Cpu::add(self.registers.read(reg1), self.registers.read(reg2));
                self.registers.write(target, sum);
            }
            // ADDU
            op if op & 0xFC00003F == 0x00000021 => {
                let reg1 = (opcode & 0x03E00000) >> 21;
                let reg2 = (opcode & 0x001F0000) >> 16;
                let target = (opcode & 0x0000F800) >> 11;

                let sum = Cpu::addu(self.registers.read(reg1), self.registers.read(reg2));
                self.registers.write(target, sum);
            }
            // AND
            op if op & 0xFC00003F == 0x00000024 => {
                let reg1 = (opcode & 0x03E00000) >> 21;
                let reg2 = (opcode & 0x001F0000) >> 16;
                let target = (opcode & 0x0000F800) >> 11;

                self.registers.write(
                    target,
                    self.registers.read(reg1) & self.registers.read(reg2),
                );
            }
            // BREAK
            op if op & 0xFC00003F == 0x0000000D => {
                todo!()
            }
            // DIV
            op if op & 0xFC00003F == 0x0000001A => {
                let reg1 = (opcode & 0x03E00000) >> 21;
                let reg2 = (opcode & 0x001F0000) >> 16;

                let dividend = self.registers.read(reg1) as i32;
                let divisor = self.registers.read(reg2) as i32;
                if divisor == 0 {
                    self.registers.hi = dividend as u32;
                } else {
                    self.registers.lo = (dividend / divisor) as u32;
                    self.registers.hi = (dividend % divisor) as u32;
                }
            }
            // DIVU
            op if op & 0xFC00003F == 0x0000001B => {
                let reg1 = (opcode & 0x03E00000) >> 21;
                let reg2 = (opcode & 0x001F0000) >> 16;

                let dividend = self.registers.read(reg1);
                let divisor = self.registers.read(reg2);
                self.registers.lo = dividend / divisor;
                self.registers.hi = dividend % divisor;
            }
            // JALR - Jump and Link Register
            op if op & 0xFC00003F == 0x00000009 => {
                let source_reg = (opcode & 0x03E00000) >> 21;
                let delay_reg = (opcode & 0x0000F800) >> 11;

                let addr = self.registers.read(source_reg);
                self.registers
                    .write(delay_reg, self.registers.program_counter + 8);
                self.registers.program_counter = addr;
            }
            // JR
            op if op & 0xFC00003F == 0x00000008 => {
                let source_reg = (opcode & 0x03E00000) >> 21;
                let target = self.registers.read(source_reg);
                if target & 0b11 == 0 {
                    self.registers.program_counter = target;
                }
            }
            // MFHI - Move From HI
            op if op & 0xFFFF07FF == 0x00000010 => {
                let reg = (opcode & 0x0000F800) >> 11;
                self.registers.write(reg, self.registers.hi);
            }
            // MFLO - Move From LO
            op if op & 0xFFFF07FF == 0x00000012 => {
                let reg = (opcode & 0x0000F800) >> 11;
                self.registers.write(reg, self.registers.lo);
            }
            // MTHI - Move To HI
            op if op & 0xFC1FFFFF == 0x00000011 => {
                let reg = (opcode & 0x03E00000) >> 21;
                self.registers.hi = self.registers.read(reg);
            }
            // MTLO - Move To LO
            op if op & 0xFC1FFFFF == 0x00000013 => {
                let reg = (opcode & 0x03E00000) >> 21;
                self.registers.lo = self.registers.read(reg);
            }
            // MULT - Multiply Word
            op if op & 0xFC00FFFF == 0x00000018 => {
                let reg1 = (opcode & 0x03E00000) >> 21;
                let reg2 = (opcode & 0x001F0000) >> 16;

                let arg1 = self.registers.read(reg1) as i32;
                let arg2 = self.registers.read(reg2) as i32;
                let product = (arg1 as i64 * arg2 as i64) as u64;

                self.registers.lo = (product & 0x00000000FFFFFFFF) as u32;
                self.registers.hi = ((product & 0xFFFFFFFF00000000) >> 32) as u32;
            }
            // MULTU - Multiply Unsigned Word
            op if op & 0xFC00FFFF == 0x00000019 => {
                let reg1 = (opcode & 0x03E00000) >> 21;
                let reg2 = (opcode & 0x001F0000) >> 16;

                let arg1 = self.registers.read(reg1) as u64;
                let arg2 = self.registers.read(reg2) as u64;
                let product = arg1 * arg2;

                self.registers.lo = (product & 0x00000000FFFFFFFF) as u32;
                self.registers.hi = ((product & 0xFFFFFFFF00000000) >> 32) as u32;
            }
            // NOR
            op if op & 0xFC0007FF == 0x00000027 => {
                let reg1 = (opcode & 0x03E00000) >> 21;
                let reg2 = (opcode & 0x001F0000) >> 16;
                let target = (opcode & 0x0000F800) >> 11;

                self.registers.write(
                    target,
                    !(self.registers.read(reg1) | self.registers.read(reg2)),
                );
            }
            // OR
            op if op & 0xFC0007FF == 0x00000025 => {
                let reg1 = (opcode & 0x03E00000) >> 21;
                let reg2 = (opcode & 0x001F0000) >> 16;
                let target = (opcode & 0x0000F800) >> 11;

                self.registers.write(
                    target,
                    self.registers.read(reg1) | self.registers.read(reg2),
                );
            }
            // SLL - Shift Word Left Logical
            op if op & 0xFFE0003F == 0x00000000 => {
                let rt = (opcode & 0x001F0000) >> 16;
                let rd = (opcode & 0x0000F800) >> 11;
                let sa = (opcode & 0x000007C0) >> 6;

                self.registers.write(rd, self.registers.read(rt) << sa);
            }
            // SLLV - Shift Word Left Logical Variable
            op if op & 0xFC0007FF == 0x00000004 => {
                let rs = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let rd = (opcode & 0x0000F800) >> 11;

                let shift = self.registers.read(rs) & 0x7;
                self.registers.write(rd, self.registers.read(rt) << shift);
            }
            // SLT - Set on Less Than
            op if op & 0xFC0007FF == 0x0000002A => {
                let rs = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let rd = (opcode & 0x0000F800) >> 11;

                let result = (self.registers.read(rs) as i32) < self.registers.read(rt) as i32;
                self.registers.write(rd, result as u32);
            }
            // SLTU - Set on Less Than Unsigned
            op if op & 0xFC0007FF == 0x0000004B => {
                let rs = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let rd = (opcode & 0x0000F800) >> 11;

                let result = self.registers.read(rs) < self.registers.read(rt);
                self.registers.write(rd, result as u32);
            }
            // SRA - Shift Word Right Arithmetic
            op if op & 0xFFE0003F == 0x00000003 => {
                let rt = (opcode & 0x001F0000) >> 16;
                let rd = (opcode & 0x0000F800) >> 11;
                let sa = (opcode & 0x000007C0) >> 6;

                self.registers
                    .write(rd, ((self.registers.read(rt) as i32) >> sa) as u32);
            }
            // SRAV - Shift Word Right Arithmetic Variable
            op if op & 0xFC0007FF == 0x00000007 => {
                let rs = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let rd = (opcode & 0x0000F800) >> 11;

                let shift = self.registers.read(rs) & 0b11111;
                self.registers
                    .write(rd, ((self.registers.read(rt) as i32) >> shift) as u32);
            }
            // SRL - Shift Word Right Logical
            op if op & 0xFFE0003F == 0x00000002 => {
                let rt = (opcode & 0x001F0000) >> 16;
                let rd = (opcode & 0x0000F800) >> 11;
                let sa = (opcode & 0x000007C0) >> 6;

                self.registers.write(rd, self.registers.read(rt) >> sa);
            }
            // SRLV - Shift Word Right Logical Variable
            op if op & 0xFC0007FF == 0x00000006 => {
                let rs = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let rd = (opcode & 0x0000F800) >> 11;

                let shift = self.registers.read(rs) & 0b11111;
                self.registers.write(rd, self.registers.read(rt) >> shift);
            }
            // SUB - Subtract Word
            op if op & 0xFC0007FF == 0x00000022 => {
                let rs = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let rd = (opcode & 0x0000F800) >> 11;

                self.registers.write(
                    rd,
                    Cpu::add(
                        self.registers.read(rs),
                        (!self.registers.read(rt)).wrapping_add(1),
                    ),
                );
            }
            // SUBU - Subtract Unsigned Word
            op if op & 0xFC0007FF == 0x00000023 => {
                let rs = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let rd = (opcode & 0x0000F800) >> 11;

                self.registers.write(
                    rd,
                    Cpu::addu(
                        self.registers.read(rs),
                        (!self.registers.read(rt)).wrapping_add(1),
                    ),
                );
            }
            // SYSCALL
            op if op & 0xFC00003F == 0x0000000C => {
                todo!()
            }
            // XOR
            op if op & 0xFC0007FF == 0x00000026 => {
                let rs = (opcode & 0x03E00000) >> 21;
                let rt = (opcode & 0x001F0000) >> 16;
                let rd = (opcode & 0x0000F800) >> 11;

                self.registers
                    .write(rd, self.registers.read(rs) ^ self.registers.read(rt));
            }
            _ => panic!(),
        }
    }

    // Casues an exception on overflow
    fn add(arg1: u32, arg2: u32) -> u32 {
        let (sum, _carry) = arg1.overflowing_add(arg2);
        // logic to handle overflow
        sum
    }

    // Does not cause an exception on overflow
    fn addu(arg1: u32, arg2: u32) -> u32 {
        arg1.wrapping_add(arg2)
    }
}

pub struct Opcode {
    pub op: Op,
    pub val: u32,
}

impl Opcode {
    fn new(op: Op, val: u32) -> Opcode {
        Opcode { op, val }
    }
    pub fn rs(&self) -> u32 {
        (self.val >> 21) & 0x1F
    }
    pub fn rt(&self) -> u32 {
        (self.val >> 16) & 0x1F
    }
    pub fn rd(&self) -> u32 {
        (self.val >> 11) & 0x1F
    }
    pub fn imm(&self) -> i16 {
        (self.val & 0xFFFF) as i16
    }

    pub fn lower_nib(&self) -> u32 {
        self.val & 0xFFFF
    }
    pub fn target(&self) -> u32 {
        self.val & 0x03FFFFFF
    }
    pub fn cofun(&self) -> u32 {
        self.val & 0x1FFFFFF
    }
    pub fn sa(&self) -> u32 {
        (self.val >> 6) & 0x1F
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Op {
    ADDI,
    ADDIU,
    ANDI,
    BEQ,
    BGEZ,
    BGEZAL,
    BLTZ,
    BLTZAL,
    BGTZ,
    BLEZ,
    BNE,
    JUMP,
    JAL,
    LB,
    LBU,
    LH,
    LHU,
    LUI,
    LW,
    LWL,
    LWR,
    ORI,
    SB,
    SH,
    SLTI,
    SLTIU,
    SW,
    SWL,
    SWR,
    XORI,
    // Special
    ADD,
    ADDU,
    AND,
    BREAK,
    DIV,
    DIVU,
    JALR,
    JR,
    MFHI,
    MFLO,
    MTHI,
    MTLO,
    MULT,
    MULTU,
    NOR,
    OR,
    SLL,
    SLLV,
    SLT,
    SLTU,
    SRA,
    SRAV,
    SRL,
    SRLV,
    SUB,
    SUBU,
    SYSCALL,
    XOR,
    // Coprocessor
    CFC2,
    RFE,
    TLB,
    COP2,
    CTC2,
    LWC2,
    MFC0,
    MFC2,
    MTC0,
    MTC2,
    SWC2,
}

pub fn parse_opcode(op: u32) -> Opcode {
    let top_bits = op >> 26;
    let low_bits = op & 0x3F;
    match top_bits {
        0b001000 => Opcode::new(Op::ADDI, op),
        0b001001 => Opcode::new(Op::ADDIU, op),
        0b001100 => Opcode::new(Op::ANDI, op),
        0b000100 => Opcode::new(Op::BEQ, op),
        0b000001 => {
            let name_lo = ((op >> 16) & 0x1) > 0;
            let name_hi = ((op >> 17) & 0xF) == 0b1000;
            match (name_lo, name_hi) {
                (false, false) => Opcode::new(Op::BLTZ, op),
                (true, false) => Opcode::new(Op::BGEZ, op),
                (false, true) => Opcode::new(Op::BLTZAL, op),
                (true, true) => Opcode::new(Op::BGEZAL, op),
            }
        }
        0b000111 => Opcode::new(Op::BGTZ, op),
        0b000110 => Opcode::new(Op::BLEZ, op),
        0b000101 => Opcode::new(Op::BNE, op),
        0b000010 => Opcode::new(Op::JUMP, op),
        0b000011 => Opcode::new(Op::JAL, op),
        0b100000 => Opcode::new(Op::LB, op),
        0b100100 => Opcode::new(Op::LBU, op),
        0b100001 => Opcode::new(Op::LH, op),
        0b100101 => Opcode::new(Op::LHU, op),
        0b001111 => Opcode::new(Op::LUI, op),
        0b100011 => Opcode::new(Op::LW, op),
        0b100010 => Opcode::new(Op::LWL, op),
        0b100110 => Opcode::new(Op::LWR, op),
        0b001101 => Opcode::new(Op::ORI, op),
        0b101000 => Opcode::new(Op::SB, op),
        0b101001 => Opcode::new(Op::SH, op),
        0b001010 => Opcode::new(Op::SLTI, op),
        0b001011 => Opcode::new(Op::SLTIU, op),
        0b101011 => Opcode::new(Op::SW, op),
        0b101010 => Opcode::new(Op::SWL, op),
        0b101110 => Opcode::new(Op::SWR, op),
        0b001110 => Opcode::new(Op::XORI, op),
        // Coprocessor
        0b010010 => {
            if op & 0x2000000 > 0 {
                Opcode::new(Op::COP2, op)
            } else if (op >> 21) & 0x1F == 0b00110 {
                Opcode::new(Op::CTC2, op)
            } else if (op >> 21) & 0x1F == 0b00010 {
                Opcode::new(Op::CFC2, op)
            } else if (op >> 21) & 0x1F == 0b00000 {
                Opcode::new(Op::MFC2, op)
            } else if (op >> 21) & 0x1F == 0b00100 {
                Opcode::new(Op::MTC2, op)
            } else {
                panic!("Invalid Coprocessor 2 op {:08X}", op)
            }
        }
        0b110010 => Opcode::new(Op::LWC2, op),
        0b111010 => Opcode::new(Op::SWC2, op),
        0b010000 => {
            if op == 0x42000010 {
                return Opcode::new(Op::RFE, op);
            };

            if op == 0x42000008 || op == 0x42000001 || op == 0x42000002 || op == 0x42000006 {
                return Opcode::new(Op::TLB, op);
            }

            let name = (op >> 21) & 0x1F;
            match name {
                0b00000 => Opcode::new(Op::MFC0, op),
                0b00100 => Opcode::new(Op::MTC0, op),
                _ => panic!("Invalid Coprocessor 0 op {:08X}", op),
            }
        }
        // Special
        0b000000 => match low_bits {
            0b100000 => Opcode::new(Op::ADD, op),
            0b100001 => Opcode::new(Op::ADDU, op),
            0b100100 => Opcode::new(Op::AND, op),
            0b001101 => Opcode::new(Op::BREAK, op),
            0b011010 => Opcode::new(Op::DIV, op),
            0b011011 => Opcode::new(Op::DIVU, op),
            0b001001 => Opcode::new(Op::JALR, op),
            0b001000 => Opcode::new(Op::JR, op),
            0b010000 => Opcode::new(Op::MFHI, op),
            0b010010 => Opcode::new(Op::MFLO, op),
            0b010001 => Opcode::new(Op::MTHI, op),
            0b010011 => Opcode::new(Op::MTLO, op),
            0b011000 => Opcode::new(Op::MULT, op),
            0b011001 => Opcode::new(Op::MULTU, op),
            0b100111 => Opcode::new(Op::NOR, op),
            0b100101 => Opcode::new(Op::OR, op),
            0b000000 => Opcode::new(Op::SLL, op),
            0b000100 => Opcode::new(Op::SLLV, op),
            0b101010 => Opcode::new(Op::SLT, op),
            0b101011 => Opcode::new(Op::SLTU, op),
            0b000011 => Opcode::new(Op::SRA, op),
            0b000111 => Opcode::new(Op::SRAV, op),
            0b000010 => Opcode::new(Op::SRL, op),
            0b000110 => Opcode::new(Op::SRLV, op),
            0b100010 => Opcode::new(Op::SUB, op),
            0b100011 => Opcode::new(Op::SUBU, op),
            0b001100 => Opcode::new(Op::SYSCALL, op),
            0b100110 => Opcode::new(Op::XOR, op),
            _ => panic!("Unknown opcode {:08X}", op),
        },
        _ => panic!("Unknown opcode {:08X}", op),
    }
}

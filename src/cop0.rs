use crate::cpu::ExceptionType;

pub struct Cop0 {
    pub sr: StatusRegister,
    pub cause: CauseRegister,
    pub epc: u32,
    pub badvaddr: u32,
}

impl Cop0 {
    pub fn new() -> Self {
        Self {
            sr: StatusRegister(0),
            cause: CauseRegister(0),
            epc: 0,
            badvaddr: 0,
        }
    }

    pub fn read_register(&self, reg: u32) -> u32 {
        match reg {
            8 => self.badvaddr,
            12 => self.sr.0,
            13 => self.cause.0,
            14 => self.epc,
            15 => 0x00000002,
            _ => 0,
        }
    }
}

pub struct CauseRegister(u32);

impl CauseRegister {
    pub fn set_exception_code(&mut self, exception: ExceptionType) {
        let code = match exception {
            ExceptionType::Interrupt => 0,
            ExceptionType::AddressErrorLoad(_) => 4,
            ExceptionType::AddressErrorStore(_) => 5,
            ExceptionType::Syscall => 8,
            ExceptionType::Break => 9,
            ExceptionType::ArithmeticOverflow => 12,
        };

        self.0 = (self.0 & 0xFFFFFF83) | (code << 2);
    }

    pub fn set_branch_delay(&mut self, bd: bool) {
        if bd {
            self.0 |= 0x80000000;
        } else {
            self.0 &= 0x7FFFFFFF;
        }
    }
}

pub struct StatusRegister(u32);

impl StatusRegister {
    pub fn set_interrupt(&mut self, ie: bool) {
        if ie {
            self.0 |= 0x1;
        } else {
            self.0 &= 0xFFFFFFFE;
        }
    }

    pub fn set_kernel_mode(&mut self, ku: bool) {
        if ku {
            self.0 |= 0x2;
        } else {
            self.0 &= 0xFFFFFFFD;
        }
    }

    pub fn push_interrupt(&mut self) {
        self.0 = (self.0 & 0xFFFFFFC3) + ((self.0 & 0x0000000F) << 2);
    }

    pub fn pop_interrupt(&mut self) {
        self.0 = (self.0 & 0xFFFFFFF0) + ((self.0 & 0x0000003C) >> 2);
    }

    pub fn bev_is_set(&self) -> bool {
        self.0 & 0x00400000 > 0
    }
}

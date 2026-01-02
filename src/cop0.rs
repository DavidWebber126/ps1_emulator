use crate::cpu::ExceptionType;

pub struct Cop0 {
    pub sr: StatusRegister,
    pub cause: CauseRegister,
    pub epc: u32,
    pub badvaddr: u32,
    pub target: u32,
    pub breakpoint_program_counter: u32,
    pub breakpoint_data_address: u32,
    pub breakpoint_data_address_mask: u32,
    pub breakpoint_program_counter_mask: u32,
    pub debug: u32,
}

impl Cop0 {
    pub fn new() -> Self {
        Self {
            sr: StatusRegister(0),
            cause: CauseRegister(0),
            epc: 0,
            badvaddr: 0,
            target: 0,
            breakpoint_program_counter: 0,
            breakpoint_data_address: 0,
            breakpoint_data_address_mask: 0,
            breakpoint_program_counter_mask: 0,
            debug: 0,
        }
    }

    pub fn register_read(&self, reg: u32) -> Result<u32, ExceptionType> {
        match reg {
            3 => Ok(self.breakpoint_program_counter),
            5 => Ok(self.breakpoint_data_address),
            6 => Ok(self.target),
            7 => Ok(self.debug),
            8 => Ok(self.badvaddr),
            9 => Ok(self.breakpoint_data_address_mask),
            11 => Ok(self.breakpoint_program_counter_mask),
            12 => Ok(self.sr.0),
            13 => Ok(self.cause.0),
            14 => Ok(self.epc),
            15 => Ok(0x00000002),
            16..=31 => Ok(0),
            _ => Err(ExceptionType::Reserved),
        }
    }

    pub fn register_write(&mut self, reg: u32, val: u32) -> Result<(), ExceptionType> {
        match reg {
            3 => {
                self.breakpoint_program_counter = val;
                Ok(())
            }
            5 => {
                self.breakpoint_data_address = val;
                Ok(())
            }
            7 => {
                self.debug = val;
                Ok(())
            }
            9 => {
                self.breakpoint_data_address_mask = val;
                Ok(())
            }
            11 => {
                self.breakpoint_program_counter_mask = val;
                Ok(())
            }
            12 => {
                self.sr.write(val);
                Ok(())
            }
            13 => {
                self.cause.write(val);
                Ok(())
            }
            6 | 8 | 14 | 15 => Ok(()),
            _ => Err(ExceptionType::Reserved),
        }
    }
}

pub struct CauseRegister(u32);

impl CauseRegister {
    // Only bits 8, 9 can be written to
    pub fn write(&mut self, val: u32) {
        self.0 = (self.0 & !(0x300)) + (val & 0x300);
    }

    // Cause register setters and getters
    pub fn set_exception_code(&mut self, exception: ExceptionType) {
        let code = match exception {
            ExceptionType::Interrupt => 0,
            ExceptionType::AddressErrorLoad(_) => 4,
            ExceptionType::AddressErrorStore(_) => 5,
            ExceptionType::BusErrorLoad(_) => 7,
            ExceptionType::Syscall => 8,
            ExceptionType::Break => 9,
            ExceptionType::Reserved => 0xA,
            ExceptionType::CoprocessorUnusable => 0xB,
            ExceptionType::ArithmeticOverflow => 0xC,
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

    pub fn set_interrupt_pending(&mut self, ip: bool) {
        if ip {
            self.0 |= 0x00000400
        } else {
            self.0 &= 0xFFFFFBFF
        }
    }

    pub fn interrupt_pending(&self) -> u32 {
        self.0 & 0x0000FF00
    }
}

pub struct StatusRegister(u32);

impl StatusRegister {
    pub fn write(&mut self, val: u32) {
        self.0 = val & 0x507FFF2F;
    }

    pub fn push_interrupt(&mut self) {
        self.0 = (self.0 & 0xFFFFFFC3) + ((self.0 & 0x0000000F) << 2);
    }

    pub fn pop_interrupt(&mut self) {
        self.0 = (self.0 & 0xFFFFFFF0) + ((self.0 & 0x0000003C) >> 2);
    }

    pub fn interrupt_mask(&self) -> u32 {
        self.0 & 0x0000FF00
    }

    pub fn interrupt_enabled(&self) -> bool {
        self.0 & 0x1 > 0
    }

    // SR setters and getters
    pub fn set_interrupt(&mut self, ie: bool) {
        if ie {
            self.0 |= 0x1;
        } else {
            self.0 &= 0xFFFFFFFE;
        }
    }

    pub fn set_kernel_mode(&mut self, ku: bool) {
        if ku {
            self.0 &= 0xFFFFFFFD;
        } else {
            self.0 |= 0x2;
        }
    }

    pub fn get_bev(&self) -> bool {
        self.0 & 0x00400000 > 0
    }
}

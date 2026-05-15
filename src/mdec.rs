#![allow(unused)]

pub struct Mdec {
    command: u32,
    data: u32,
    status: u32,
    control: u32,
}

impl Mdec {
    pub fn new() -> Self {
        Self {
            command: 0,
            data: 0,
            status: 0,
            control: 0,
        }
    }

    pub fn command_write(&mut self, val: u32) {
        match val >> 29 {
            1 => {
                // decode macroblock
                todo!()
            }
            2 => {
                // Set Quant Tables
                todo!()
            }
            3 => {
                // Set Scale Table
                todo!()
            }
            _ => {
                // do nothing
            }
        }
    }
}

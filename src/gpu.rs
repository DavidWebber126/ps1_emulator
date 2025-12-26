use std::collections::VecDeque;

use eframe::egui::Color32;

const GPUPARAMLIMITS: [u8; 8] = [0, 0, 0, 1, 2, 1, 1, 0];

#[derive(Clone, Copy)]
pub struct VramCopyFields {
    vram_x: u16,
    vram_y: u16,
    width: u16,
    height: u16,
    current_row: u16,
    current_col: u16,
}

pub enum Gp0State {
    WaitingForCommand,
    ReceivingParams { command: u8, idx: u8 },
    ReceivingData(VramCopyFields),
    //CpuBlitParams { idx: u8 },
    //ReceivingParams { command: 4, idx: u8 },
}

pub struct Gpu {
    gp0_state: Gp0State,
    pub params: [u32; 16],
    pub vram: Box<[u8; 1048576]>, // 1024 x 512 grid of pixels. Each pixel is two bytes so total space is 2 * 1024 * 512
    pub frame_is_ready: bool,
    pub counter: u64,
    pub status: u32,
    pub command_buffer: VecDeque<u32>, // Holds at most 16 words (i.e 16 u32s)
}

impl Gpu {
    pub fn new() -> Self {
        Self {
            gp0_state: Gp0State::WaitingForCommand,
            params: [0; 16],
            vram: Box::new([0; 1048576]),
            frame_is_ready: false,
            counter: 0,
            status: 0,
            command_buffer: VecDeque::with_capacity(16),
        }
    }

    pub fn gp0_write(&mut self, val: u32) {
        self.gp0_state = match self.gp0_state {
            Gp0State::WaitingForCommand => {
                match val >> 29 {
                    0 => todo!(), // misc commands
                    1 => todo!(), // polygon primitive
                    2 => todo!(), // line primitive
                    3 => {
                        // rectangle primitive
                        // Store command in params as it will be needed later
                        self.params[0] = val;
                        Gp0State::ReceivingParams { command: 3, idx: 1 }
                    }
                    4 => Gp0State::ReceivingParams { command: 4, idx: 0 }, // VRAM to VRAM blit
                    5 => Gp0State::ReceivingParams { command: 5, idx: 0 }, // CPU to VRAM blit
                    6 => Gp0State::ReceivingParams { command: 6, idx: 0 }, // VRAM to CPU blit
                    7 => todo!(),                                          // Environment commands
                    _ => panic!("Impossible GPU command {}", val),
                }
            }
            Gp0State::ReceivingParams { command, idx } => {
                let limit = GPUPARAMLIMITS[command as usize];

                if idx >= limit {
                    // All parameters received
                    match command {
                        0 => todo!(), // misc commands
                        1 => todo!(), // polygon primitive
                        2 => todo!(), // line primitive
                        3 => {
                            // rectangle primitive
                            self.draw_1x1_untextured_rectangle();
                        }
                        4 => todo!(), // VRAM to VRAM blit
                        5 => {
                            // CPU to VRAM blit
                        }
                        6 => todo!(), // VRAM to CPU blit
                        7 => todo!(), // Environment commands
                        _ => panic!("Impossible GPU command {}", val),
                    };
                    self.cpu_to_vram_init()
                } else {
                    Gp0State::ReceivingParams {
                        command,
                        idx: idx + 1,
                    }
                }
            }
            Gp0State::ReceivingData(mut fields) => self.cpu_to_vram_process(val, &mut fields),
            //_ => todo!(),
        };
    }

    fn cpu_to_vram_init(&mut self) -> Gp0State {
        let vram_x = (self.params[0] & 0x3FF) as u16;
        let vram_y = ((self.params[0] >> 16) & 0x1FF) as u16;

        let mut width = (self.params[1] & 0x3FF) as u16;
        if width == 0 {
            width = 1024;
        }

        let mut height = ((self.params[1] >> 16) & 0x1FF) as u16;
        if height == 0 {
            height = 512;
        }

        Gp0State::ReceivingData(VramCopyFields {
            vram_x,
            vram_y,
            width,
            height,
            current_row: 0,
            current_col: 0,
        })
    }

    fn cpu_to_vram_process(&mut self, word: u32, fields: &mut VramCopyFields) -> Gp0State {
        for i in 0..2 {
            let halfword = (word >> (16 * i)) as u16;
            let vram_row = ((fields.vram_x + fields.current_row) & 0x1FF) as usize;
            let vram_col = ((fields.vram_y + fields.current_col) & 0x3FF) as usize;

            let [lo, hi] = halfword.to_le_bytes();
            let vram_addr = 2 * (1024 * vram_row + vram_col);
            self.vram[vram_addr] = lo;
            self.vram[vram_addr + 1] = hi;

            fields.current_col += 1;
            if fields.current_col == fields.width {
                fields.current_col = 0;
                fields.current_row += 1;

                if fields.current_row == fields.height {
                    return Gp0State::WaitingForCommand;
                }
            }
        }

        todo!()
    }

    pub fn gp1_write(&mut self, val: u32) {
        todo!()
    }

    pub fn gpuread(&mut self) -> u32 {
        0x14000000
    }

    pub fn gpustat(&mut self) -> u32 {
        todo!()
    }

    pub fn tick(&mut self, cycles: u32) {
        self.counter += cycles as u64;

        if self.counter >= 564480 {
            self.frame_is_ready = true;
            self.counter -= 564480
        } else {
            self.frame_is_ready = false;
        }
    }

    fn draw_1x1_untextured_rectangle(&mut self) {
        let command = self.params[0];
        let parameter = self.params[1];

        let r = (command & 0xFF) >> 3;
        let g = ((command >> 8) & 0xFF) >> 3;
        let b = ((command >> 16) & 0xFF) >> 3;

        let pixel = (r | (g << 5) | (b << 10)) as u16;
        let [pixel_lo, pixel_hi] = pixel.to_le_bytes();

        let x = parameter & 0x3FF;
        let y = (parameter >> 16) & 0x1FF;

        let vram_addr = 2 * (1024 * y + x) as usize;
        self.vram[vram_addr] = pixel_lo;
        self.vram[vram_addr] = pixel_hi;
    }

    pub fn render_vram(&mut self, output_buffer: &mut [Color32; 524288]) {
        for y in 0..512 {
            for x in 0..1024 {
                let vram_addr = 2 * (1024 * y + x);
                let pixel = u16::from_le_bytes([self.vram[vram_addr], self.vram[vram_addr + 1]]);

                // RGB555
                let r = convert_5bit_to_8bit(pixel & 0x1F);
                let g = convert_5bit_to_8bit((pixel >> 5) & 0x1F);
                let b = convert_5bit_to_8bit((pixel >> 10) & 0x1F);

                output_buffer[1024 * y + x] = Color32::from_rgb(r, g, b);
            }
        }
    }
}

fn convert_5bit_to_8bit(color: u16) -> u8 {
    (f64::from(color) * 255.0 / 31.0).round() as u8
}

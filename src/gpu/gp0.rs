use std::collections::VecDeque;

use tracing::{Level, event};

const GPUPARAMLIMITS: [u8; 8] = [0, 0, 0, 1, 2, 1, 1, 0];

#[derive(Clone, Copy)]
struct VramCopyFields {
    vram_x: u16,
    vram_y: u16,
    width: u16,
    height: u16,
    current_row: u16,
    current_col: u16,
}

enum Gp0State {
    WaitingForCommand,
    ReceivingParams { command: u8, idx: u8 },
    ReceivingData(VramCopyFields),
    SendingData(VramCopyFields),
    //CpuBlitParams { idx: u8 },
    //ReceivingParams { command: 4, idx: u8 },
}

pub struct Gp0 {
    state: Gp0State,
    pub vram: Box<[u8; 1048576]>, // 1024 x 512 grid of pixels. Each pixel is two bytes so total space is 2 * 1024 * 512
    pub params: [u32; 16],
    pub command_buffer: VecDeque<u32>, // Holds at most 16 words (i.e 16 u32s)
    pub draw_mode: u32,
    pub texture_window: u32,
    pub top_left_draw_area: (u16, u16),
    pub bot_right_draw_area: (u16, u16),
    pub draw_offset: (i16, i16),
    pub mask_bits: u8,
}

impl Gp0 {
    pub fn new() -> Self {
        Self {
            state: Gp0State::WaitingForCommand,
            vram: Box::new([0; 1048576]),
            params: [0; 16],
            command_buffer: VecDeque::with_capacity(16),
            draw_mode: 0,
            texture_window: 0,
            top_left_draw_area: (0, 0),
            bot_right_draw_area: (0, 0),
            draw_offset: (0, 0),
            mask_bits: 0,
        }
    }

    pub fn write(&mut self, val: u32) {
        // let span = span!(target: "ps1_emulator::GPU", Level::DEBUG, "GP0");
        // let _ = span.enter();
        event!(target: "ps1_emulator::GPU", Level::DEBUG, "Write to GP0 with {:08X}", val);


        self.state = match self.state {
            Gp0State::WaitingForCommand => {
                match val >> 29 {
                    0 => todo!(), // misc commands
                    1 => todo!(), // polygon primitive
                    2 => todo!(), // line primitive
                    3 => {
                        // rectangle primitive
                        // Store command in params as it will be needed later
                        event!(target: "ps1_emulator::GPU", Level::TRACE, "GP0 Rectangle Primitive command received");

                        self.params[0] = val;
                        Gp0State::ReceivingParams { command: 3, idx: 1 }
                    }
                    4 => {
                        // VRAM to VRAM blit
                        event!(target: "ps1_emulator::GPU", Level::TRACE, "GP0 VRAM to VRAM BLIT received");

                        Gp0State::ReceivingParams { command: 4, idx: 0 }
                    }
                    5 => {
                        // CPU to VRAM blit
                        event!(target: "ps1_emulator::GPU", Level::TRACE, "GP0 CPU to VRAM BLIT received");

                        Gp0State::ReceivingParams { command: 5, idx: 0 }
                    }
                    6 => {
                        // VRAM to CPU blit
                        event!(target: "ps1_emulator::GPU", Level::TRACE, "GP0 VRAM to CPU BLIT received");

                        Gp0State::ReceivingParams { command: 6, idx: 0 }
                    }
                    7 => {
                        match val >> 24 {
                            0xE1 => {
                                // Draw Mode Setting
                                self.draw_mode = val;

                                Gp0State::WaitingForCommand
                            }
                            0xE2 => {
                                // Texture Window Setting
                                self.texture_window = val;

                                Gp0State::WaitingForCommand
                            }
                            0xE3 => {
                                // Set Drawing Area Top Left (X1, Y1)
                                self.top_left_draw_area.0 = (val & 0x3FF) as u16;
                                self.top_left_draw_area.1 = ((val >> 10) & 0x3FF) as u16;

                                Gp0State::WaitingForCommand
                            }
                            0xE4 => {
                                // Set Drawing Area Bottom Right (X2, Y2)
                                self.bot_right_draw_area.0 = (val & 0x3FF) as u16;
                                self.bot_right_draw_area.1 = ((val >> 10) & 0x3FF) as u16;

                                Gp0State::WaitingForCommand
                            }
                            0xE5 => {
                                // Set Drawing Offset (X, Y)
                                self.draw_offset.0 = (val & 0x3FF) as i16;
                                self.draw_offset.1 = ((val >> 11) & 0x3FF) as i16;

                                Gp0State::WaitingForCommand
                            }
                            0xE6 => {
                                // Mask Bit Setting
                                self.mask_bits = (val & 0xF) as u8;

                                Gp0State::WaitingForCommand
                            }
                            _ => panic!("Impossible GP0 Environment Command {:02X}", val >> 24),
                        }
                    }
                    _ => {
                        event!(target: "ps1_emulator::GPU", Level::ERROR, "Impossible GP0 command {:08X}", val);
                        panic!("Impossible GPU command {}", val)
                    }
                }
            }
            Gp0State::ReceivingParams { command, idx } => {
                let limit = GPUPARAMLIMITS[command as usize];

                event!(target: "ps1_emulator::GPU", Level::TRACE, "Parameter received");

                self.params[idx as usize] = val;

                if idx >= limit {
                    event!(target: "ps1_emulator::GPU", Level::TRACE, "All Params received for command");
                    // All parameters received. Diatch to execute the command now
                    match command {
                        0 => todo!(), // misc commands
                        1 => todo!(), // polygon primitive
                        2 => todo!(), // line primitive
                        3 => {
                            // rectangle primitive
                            self.draw_1x1_untextured_rectangle();
                            Gp0State::WaitingForCommand
                        }
                        4 => todo!(), // VRAM to VRAM blit
                        5 => {
                            // CPU to VRAM blit
                            self.cpu_to_vram_init()
                        }
                        6 => {
                            // VRAM to CPU blit
                            self.vram_to_cpu_init()
                        }
                        7 => todo!(), // Environment commands
                        _ => panic!("Impossible GPU command {}", val),
                    }
                } else {
                    Gp0State::ReceivingParams {
                        command,
                        idx: idx + 1,
                    }
                }
            }
            Gp0State::ReceivingData(mut fields) => {
                event!(target: "ps1_emulator::GPU", Level::TRACE, "Received Data: {:08X}", val);

                self.cpu_to_vram_process(val, &mut fields)
            }
            Gp0State::SendingData(fields) => {
                // GPU is busy sending data. Do not change state until final data has been sent via GPUREAD
                Gp0State::SendingData(fields)
            }
        };
    }

    pub fn ready_for_cmd(&self) -> bool {
        matches!(self.state, Gp0State::WaitingForCommand)
    }

    pub fn dma_ready(&self) -> bool {
        true
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

        event!(target: "ps1_emulator::GPU", Level::TRACE, "CPU to VRAM init with vram_x: 0x{:08X}, vram_y: 0x{:08X}, width: {width}, height: {height}", vram_x, vram_y);

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
        event!(target: "ps1_emulator::GPU", Level::TRACE, "CPU to VRAM Data");

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

        Gp0State::ReceivingData(*fields)
    }

    fn vram_to_cpu_init(&mut self) -> Gp0State {
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

        event!(target: "ps1_emulator::GPU", Level::TRACE, "VRAM to CPU init with vram_x: 0x{:08X}, vram_y: 0x{:08X}, width: {width}, height: {height}", vram_x, vram_y);

        Gp0State::SendingData(VramCopyFields {
            vram_x,
            vram_y,
            width,
            height,
            current_row: 0,
            current_col: 0,
        })
    }

    pub fn vram_to_cpu_process(&mut self) -> u32 {
        let mut fields = match self.state {
            Gp0State::SendingData(fields) => fields,
            _ => panic!("VRAM to CPU only when GP0 is in sending data state"),
        };

        event!(target: "ps1_emulator::GPU", Level::TRACE, "VRAM to CPU Data");

        let mut out = [0u8; 4];
        for i in 0..2 {
            let vram_row = ((fields.vram_x + fields.current_row) & 0x1FF) as usize;
            let vram_col = ((fields.vram_y + fields.current_col) & 0x3FF) as usize;
            let vram_addr = 2 * (1024 * vram_row + vram_col);
            let vram_lo = self.vram[vram_addr];
            let vram_hi = self.vram[vram_addr + 1];

            out[2 * i] = vram_lo;
            out[2 * i + 1] = vram_hi;

            fields.current_col += 1;
            if fields.current_col == fields.width {
                fields.current_col = 0;
                fields.current_row += 1;

                if fields.current_row == fields.height {
                    self.state = Gp0State::WaitingForCommand;
                }
            }
        }

        // If we did not finish sending data then keep sending
        // If we did finish, i.e Gp0State == WaitingForCommand, then don't set state to SendingData
        if matches!(self.state, Gp0State::SendingData(_)) {
            self.state = Gp0State::SendingData(fields);
        }

        u32::from_le_bytes(out)
    }

    fn draw_1x1_untextured_rectangle(&mut self) {
        event!(target: "ps1_emulator::GPU", Level::TRACE, "Draw 1x1 Rect");
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
        self.vram[vram_addr + 1] = pixel_hi;
    }

    pub fn is_sending_data(&self) -> bool {
        matches!(self.state, Gp0State::SendingData(_))
    }
}

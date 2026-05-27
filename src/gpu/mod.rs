mod gp0;
mod gp1;
mod rasterize;

use gp0::Gp0;
use gp1::Gp1;

use tracing::{Level, event};

pub struct Gpu {
    pub gp0: Gp0,
    pub gp1: Gp1,
    pub frame_is_ready: bool,
    pub counter: u64,
}

impl Gpu {
    pub fn new() -> Self {
        Self {
            gp0: Gp0::new(),
            gp1: Gp1::new(),
            frame_is_ready: false,
            counter: 0,
        }
    }

    pub fn gp1_write(&mut self, val: u32) {
        self.gp1.write(val);
        self.gp0.vram_size_set = self.gp1.vram_size;
    }

    pub fn gpuread(&mut self) -> u32 {
        event!(target: "ps1_emulator::GPU", Level::DEBUG, "Reading GPUREAD");

        // If GP0 is in VRAM to CPU Blit then return that value first
        if self.gp0.is_sending_data() {
            return self.gp0.vram_to_cpu_process();
        }

        match self.gp1.gpu_read_register {
            0x00 | 0x01 | 0x06 => 0,
            0x07 => 0x2,
            _ => panic!("Impossible state for GP1 Read Register"),
        }
    }

    pub fn gpustat(&mut self) -> u32 {
        let command_ready = (self.gp0.ready_for_cmd() as u32) << 26;
        let vram_data_ready = (self.gp0.is_sending_data() as u32) << 27;
        let dma_ready = (self.gp0.dma_ready() as u32) << 28;

        let tex_page_x = self.gp0.tex_page_x as u32;
        let tex_page_y = (self.gp0.tex_page_y as u32) << 4;
        let semitransparency = self.gp0.transparency_mode() << 5;
        let texture_depth = self.gp0.texture_page_colors() << 7;
        let dither = (self.gp0.dither_enabled as u32) << 9;
        let display_draw = (self.gp0.draw_to_display as u32) << 10;
        let force_mask_bit = (self.gp0.mask_while_draw as u32) << 11;
        let texture_mask = (self.gp0.mask_before_draw as u32) << 12;
        let two_mb = (self.gp0.two_mb_mem as u32) << 15;

        let output = dma_ready
            + vram_data_ready
            + command_ready
            + force_mask_bit
            + texture_mask
            + display_draw
            + dither
            + texture_depth
            + semitransparency
            + tex_page_y
            + tex_page_x
            + two_mb;

        event!(target: "ps1_emulator::GPU", Level::DEBUG, "Reading GPUSTAT: {:08X}", output);

        output
    }

    pub fn tick(&mut self, cycles: u32) -> bool {
        self.counter += cycles as u64;

        if self.counter >= 564480 {
            event!(target: "ps1_emulator::GPU", Level::DEBUG, "Render Frame");
            self.counter -= 564480;
            self.frame_is_ready = true;
        } else {
            self.frame_is_ready = false;
        }
        self.frame_is_ready
    }

    pub fn render_vram(&self) -> Vec<u8> {
        if self.gp1.color_depth {
            let mut output = Vec::with_capacity(349184);
            for y in 0..512 {
                for x in 0..682 {
                    let base_addr = 2048 * y + 3 * x;
                    let r = self.gp0.vram[base_addr];
                    let g = self.gp0.vram[base_addr + 1];
                    let b = self.gp0.vram[base_addr + 2];

                    output.push(r);
                    output.push(g);
                    output.push(b);
                }
            }

            output
        } else {
            let mut output = Vec::with_capacity(1572864);
            for addr in 0..524288 {
                let pixel =
                    u16::from_le_bytes([self.gp0.vram[2 * addr], self.gp0.vram[2 * addr + 1]]);
                let r = convert_5bit_to_8bit(pixel & 0x1F);
                let g = convert_5bit_to_8bit((pixel >> 5) & 0x1F);
                let b = convert_5bit_to_8bit((pixel >> 10) & 0x1F);

                output.push(r);
                output.push(g);
                output.push(b);
            }

            output
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

fn convert_5bit_to_8bit(color: u16) -> u8 {
    match color {
        0 => 0,
        1 => 8,
        2 => 16,
        3 => 25,
        4 => 33,
        5 => 41,
        6 => 49,
        7 => 58,
        8 => 66,
        9 => 74,
        10 => 82,
        11 => 90,
        12 => 99,
        13 => 107,
        14 => 115,
        15 => 123,
        16 => 132,
        17 => 140,
        18 => 148,
        19 => 156,
        20 => 165,
        21 => 173,
        22 => 181,
        23 => 189,
        24 => 197,
        25 => 206,
        26 => 214,
        27 => 222,
        28 => 230,
        29 => 239,
        30 => 247,
        31 => 255,
        _ => panic!("Impossible"),
    }
}

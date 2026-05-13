use std::{cmp, mem};

use tracing::{Level, event};

const DITHER_TABLE: [[i8; 4]; 4] = [
    [-4, 0, -3, 1],
    [2, -2, 3, -1],
    [-3, 1, -4, 0],
    [3, -1, 2, -2],
];

enum SemiTransparency {
    Blend,
    Add,
    Subtract,
    QuarterBlend,
}

#[derive(Clone, Copy)]
enum TextureBits {
    Four,
    Eight,
    Fifteen,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub fn from(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_tuple(self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }
}

impl From<u32> for Color {
    fn from(value: u32) -> Self {
        let [_, b, g, r] = value.to_le_bytes();
        Color { r, g, b, a: 255 }
    }
}

#[derive(Clone, Copy, Debug)]
enum Commands {
    Rectangle,
    TexturedRectangle,
    SizeRectangle,
    TexturedSizeRectangle,
    VramToVram,
    CpuToVram,
    VramToCpu,
    VramFill,
}

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
    ReceivingParams {
        command: Commands,
        idx: u8,
    },
    ReceivingData(VramCopyFields),
    SendingData(VramCopyFields),
    ReceivingLineVert {
        polyline: bool,
        shaded: bool,
        idx: u8,
    },
    ReceivingPolyVert {
        size: u8,
        shaded: bool,
        textured: bool,
        idx: u8,
    },
    //ReceivingPolyVertShaded { size: u8, shaded: bool, textured: bool, idx: u8 },
    //CpuBlitParams { idx: u8 },
    //ReceivingParams { command: 4, idx: u8 },
}

pub struct Gp0 {
    state: Gp0State,
    pub vram: Box<[Color; 524288]>, // 1024 x 512 grid of pixels
    mask_field: [bool; 524288],     // 1024 x 512 simulate bit 16 mask field for each vram pixel
    pub params: [u32; 16],
    //pub command_buffer: VecDeque<u32>, // Holds at most 16 words (i.e 16 u32s)
    pub tex_page_x: u8,
    pub tex_page_y: bool,
    semitransparency: SemiTransparency,
    tex_page_colors: TextureBits,
    pub dither_enabled: bool,
    pub draw_to_display: bool,
    pub rect_x_flip: bool,
    pub rect_y_flip: bool,
    pub texture_window: u32,
    pub draw_area_top_left: (u32, u32),
    pub draw_area_bot_right: (u32, u32),
    pub draw_offset: (i16, i16),
    pub mask_while_draw: bool,
    pub mask_before_draw: bool,
}

impl Gp0 {
    pub fn new() -> Self {
        Self {
            state: Gp0State::WaitingForCommand,
            vram: Box::new([Color::from(0, 0, 0, 255); 524288]),
            mask_field: [false; 524288],
            params: [0; 16],
            //command_buffer: VecDeque::with_capacity(16),
            tex_page_x: 0,
            tex_page_y: false,
            semitransparency: SemiTransparency::Blend,
            tex_page_colors: TextureBits::Four,
            dither_enabled: false,
            draw_to_display: false,
            rect_x_flip: false,
            rect_y_flip: false,
            texture_window: 0,
            draw_area_top_left: (0, 0),
            draw_area_bot_right: (0, 0),
            draw_offset: (0, 0),
            mask_while_draw: false,
            mask_before_draw: false,
        }
    }

    pub fn vram_fill(&mut self, width: u32, height: u32, vram_x: u32, vram_y: u32, val: u16) {
        // RGB555
        let r = convert_5bit_to_8bit(val & 0x1F);
        let g = convert_5bit_to_8bit((val >> 5) & 0x1F);
        let b = convert_5bit_to_8bit((val >> 10) & 0x1F);

        let color = Color::from(r, g, b, 255);

        for y in 0..height {
            for x in 0..width {
                let vram_addr = (vram_x + x) as usize + 1024 * (vram_y + y) as usize;
                self.vram[vram_addr] = color;
            }
        }
    }

    fn write_5bit_color(&mut self, addr: usize, val: u16) {
        if self.mask_before_draw && self.mask_field[addr] {
            return;
        }

        // If Mask While Draw is set, then mask_field is forced to true. Otherwise set to bit 15
        self.mask_field[addr] = self.mask_while_draw || (val & 0x8000 > 0);

        // RGB555
        let r = convert_5bit_to_8bit(val & 0x1F);
        let g = convert_5bit_to_8bit((val >> 5) & 0x1F);
        let b = convert_5bit_to_8bit((val >> 10) & 0x1F);

        self.vram[addr] = Color::from(r, g, b, 255);
    }

    fn write_5bit_color_alpha(&mut self, addr: usize, val: u16) {
        if self.mask_before_draw && self.mask_field[addr] {
            return;
        }

        // If Mask While Draw is set, then mask_field is forced to true. Otherwise set to bit 15
        self.mask_field[addr] = self.mask_while_draw || (val & 0x8000 > 0);

        let r = convert_5bit_to_8bit(val & 0x1F);
        let g = convert_5bit_to_8bit((val >> 5) & 0x1F);
        let b = convert_5bit_to_8bit((val >> 10) & 0x1F);

        let prev_color = self.vram[addr];

        let (new_r, new_g, new_b) = match self.semitransparency {
            SemiTransparency::Blend => (
                r / 2 + prev_color.r / 2,
                g / 2 + prev_color.g / 2,
                b / 2 + prev_color.b / 2,
            ),
            SemiTransparency::Add => (r + prev_color.r, g + prev_color.g, b + prev_color.b),
            SemiTransparency::Subtract => (prev_color.r - r, prev_color.g - g, prev_color.b - b),
            SemiTransparency::QuarterBlend => (
                r / 4 + prev_color.r,
                g / 4 + prev_color.g,
                b / 4 + prev_color.b,
            ),
        };

        self.vram[addr] = Color::from(new_r, new_g, new_b, 255);
    }

    pub fn read_vram(&self, addr: usize) -> u16 {
        let (r, g, b, _) = self.vram[addr].to_tuple();
        let new_r = convert_8bit_to_5bit(r);
        let new_g = convert_8bit_to_5bit(g);
        let new_b = convert_8bit_to_5bit(b);

        let mask = self.mask_field[addr] as u16;

        new_r | (new_g << 5) | (new_b << 10) | (mask << 15)
    }

    fn modulate_5bit_color(&self, col1: u16, col2: u32) -> u16 {
        let mask = col1 & 0x8000;
        let r1 = convert_5bit_to_8bit(col1 & 0x1F) as f32;
        let g1 = convert_5bit_to_8bit((col1 >> 5) & 0x1F) as f32;
        let b1 = convert_5bit_to_8bit((col1 >> 10) & 0x1F) as f32;

        let r2 = (col2 & 0xFF) as f32;
        let g2 = ((col2 >> 8) & 0xFF) as f32;
        let b2 = ((col2 >> 16) & 0xFF) as f32;

        let new_r = (((r1 * r2) / 128.0).round() as u8) >> 3;
        let new_g = (((g1 * g2) / 128.0).round() as u8) >> 3;
        let new_b = (((b1 * b2) / 128.0).round() as u8) >> 3;

        new_r as u16 | (new_g as u16) << 5 | (new_b as u16) << 10 | mask
    }

    fn copy_vram(&mut self, source_addr: usize, dest_addr: usize) {
        self.vram[dest_addr] = self.vram[source_addr];
    }

    pub fn write(&mut self, val: u32) {
        // let span = span!(target: "ps1_emulator::GPU", Level::DEBUG, "GP0");
        // let _ = span.enter();
        event!(target: "ps1_emulator::GPU", Level::DEBUG, "Write to GP0: {:08X}", val);

        self.state = match self.state {
            Gp0State::WaitingForCommand => {
                match val >> 29 {
                    1 => {
                        // Polygon Primitive
                        event!(target: "ps1_emulator::GPU", Level::TRACE, "GP0 Polygon command received");

                        self.params[0] = val;

                        let shaded = (val >> 28) & 1 > 0;
                        let textured = (val >> 26) & 1 > 0;
                        let size = if (val >> 27) & 1 > 0 { 4 } else { 3 };

                        if shaded {
                            self.params[1] = val & 0xFFFFFF;
                            Gp0State::ReceivingPolyVert {
                                size,
                                shaded,
                                textured,
                                idx: 2,
                            }
                        } else {
                            Gp0State::ReceivingPolyVert {
                                size,
                                shaded,
                                textured,
                                idx: 1,
                            }
                        }
                    }
                    2 => {
                        // Line Primitive
                        self.params[0] = val;
                        let polyline = (self.params[0] >> 27) & 0x1 > 0;

                        if (val >> 28) & 0x1 > 0 {
                            event!(target: "ps1_emulator::GPU", Level::TRACE, "GP0 Line (Gourand) Primitive command received");
                            self.params[1] = val & 0xFFFFFF;
                            Gp0State::ReceivingLineVert {
                                polyline,
                                shaded: true,
                                idx: 2,
                            }
                        } else {
                            event!(target: "ps1_emulator::GPU", Level::TRACE, "GP0 Line Primitive command received");
                            Gp0State::ReceivingLineVert {
                                polyline,
                                shaded: false,
                                idx: 1,
                            }
                        }
                    }
                    3 => {
                        // rectangle primitive
                        // Store command in params as it will be needed later

                        let is_textured = val & 0x4000000 > 0;
                        let is_varsized = val & 0x18000000 == 0;

                        let command = match (is_varsized, is_textured) {
                            (true, true) => Commands::TexturedSizeRectangle,
                            (true, false) => Commands::SizeRectangle,
                            (false, true) => Commands::TexturedRectangle,
                            (false, false) => Commands::Rectangle,
                        };

                        event!(target: "ps1_emulator::GPU", Level::TRACE, "GP0 Rectangle ({:?}) Primitive command received", command);

                        self.params[0] = val;

                        Gp0State::ReceivingParams { command, idx: 1 }
                    }
                    4 => {
                        // VRAM to VRAM blit
                        event!(target: "ps1_emulator::GPU", Level::TRACE, "GP0 VRAM to VRAM BLIT received");

                        Gp0State::ReceivingParams {
                            command: Commands::VramToVram,
                            idx: 0,
                        }
                    }
                    5 => {
                        // CPU to VRAM blit
                        event!(target: "ps1_emulator::GPU", Level::TRACE, "GP0 CPU to VRAM BLIT received");

                        Gp0State::ReceivingParams {
                            command: Commands::CpuToVram,
                            idx: 0,
                        }
                    }
                    6 => {
                        // VRAM to CPU blit
                        event!(target: "ps1_emulator::GPU", Level::TRACE, "GP0 VRAM to CPU BLIT received");

                        Gp0State::ReceivingParams {
                            command: Commands::VramToCpu,
                            idx: 0,
                        }
                    }
                    0 | 7 => {
                        match val >> 24 {
                            0x00 => Gp0State::WaitingForCommand, // no op
                            0x01 => {
                                // Flush Texture Cache
                                // TODO
                                Gp0State::WaitingForCommand
                            }
                            0x02 => {
                                // VRAM Fill
                                event!(target: "ps1_emulator::GPU", Level::TRACE, "VRAM Fill Received");

                                self.params[0] = val;

                                Gp0State::ReceivingParams {
                                    command: Commands::VramFill,
                                    idx: 1,
                                }
                            }
                            0x03 => {
                                // Unknown?
                                Gp0State::WaitingForCommand
                            }
                            0xE1 => {
                                // Draw Mode Settings
                                self.tex_page_x = (val & 0b1111) as u8;
                                self.tex_page_y = val & 0x10 > 0;
                                self.dither_enabled = val & 0x200 > 0;
                                self.draw_to_display = val & 0x400 > 0;
                                self.rect_x_flip = val & 0x1000 > 0;
                                self.rect_y_flip = val & 0x2000 > 0;
                                match (val >> 7) & 0b11 {
                                    0 => self.tex_page_colors = TextureBits::Four,
                                    1 => self.tex_page_colors = TextureBits::Eight,
                                    2 => self.tex_page_colors = TextureBits::Fifteen,
                                    _ => {
                                        event!(target: "ps1_emulator::GPU", Level::WARN, "Texture size outside of Four, Eight and Fifteen");
                                    }
                                }
                                match (val >> 5) & 0b11 {
                                    0 => self.semitransparency = SemiTransparency::Blend,
                                    1 => self.semitransparency = SemiTransparency::Add,
                                    2 => self.semitransparency = SemiTransparency::Subtract,
                                    3 => self.semitransparency = SemiTransparency::QuarterBlend,
                                    _ => panic!("Impossible"),
                                }

                                Gp0State::WaitingForCommand
                            }
                            0xE2 => {
                                // Texture Window Setting
                                self.texture_window = val;

                                Gp0State::WaitingForCommand
                            }
                            0xE3 => {
                                // Set Drawing Area Top Left (X1, Y1)
                                self.draw_area_top_left.0 = val & 0x3FF;
                                self.draw_area_top_left.1 = (val >> 10) & 0x3FF;

                                event!(target: "ps1_emulator::GPU", Level::TRACE, "Set Draw Area Top Left to {:?}", self.draw_area_top_left);

                                Gp0State::WaitingForCommand
                            }
                            0xE4 => {
                                // Set Drawing Area Bottom Right (X2, Y2)
                                self.draw_area_bot_right.0 = val & 0x3FF;
                                self.draw_area_bot_right.1 = (val >> 10) & 0x3FF;

                                event!(target: "ps1_emulator::GPU", Level::TRACE, "Set Draw Area Bottom Right to {:?}", self.draw_area_bot_right);

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
                                self.mask_while_draw = val & 1 > 0;
                                self.mask_before_draw = val & 2 > 0;

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
                let limit = param_limits(command);

                event!(target: "ps1_emulator::GPU", Level::TRACE, "Parameter received");

                self.params[idx as usize] = val;

                if idx >= limit {
                    event!(target: "ps1_emulator::GPU", Level::TRACE, "All Params received for command");
                    // All parameters received. Diatch to execute the command now
                    match command {
                        Commands::Rectangle => {
                            // rectangle primitive
                            let dimension = match (self.params[0] >> 27) & 0b11 {
                                1 => 1,
                                2 => 8,
                                3 => 16,
                                _ => panic!("Impossible"),
                            };
                            self.draw_untextured_rectangle(dimension, dimension);
                            Gp0State::WaitingForCommand
                        }
                        Commands::SizeRectangle => {
                            let width = val & 0x3FF;
                            let height = (val >> 16) & 0x1FF;
                            self.draw_untextured_rectangle(width, height);
                            Gp0State::WaitingForCommand
                        }
                        Commands::TexturedRectangle => {
                            let dimension = match (self.params[0] >> 27) & 0b11 {
                                1 => 1,
                                2 => 8,
                                3 => 16,
                                _ => panic!("Impossible"),
                            };
                            self.draw_textured_rectangle(dimension, dimension);
                            Gp0State::WaitingForCommand
                        }
                        Commands::TexturedSizeRectangle => {
                            let width = val & 0x3FF;
                            let height = (val >> 16) & 0x1FF;
                            self.draw_textured_rectangle(width, height);
                            Gp0State::WaitingForCommand
                        }
                        Commands::VramToVram => {
                            self.vram_copy();
                            Gp0State::WaitingForCommand
                        }
                        Commands::CpuToVram => {
                            // CPU to VRAM blit
                            self.cpu_to_vram_init()
                        }
                        Commands::VramToCpu => {
                            // VRAM to CPU blit
                            self.vram_to_cpu_init()
                        }
                        Commands::VramFill => {
                            let command = self.params[0];
                            let vram_x = self.params[1] & 0x3FF;
                            let vram_y = (self.params[1] >> 16) & 0x1FF;
                            let width = self.params[2] & 0x3FF;
                            let height = (self.params[2] >> 16) & 0x1FF;

                            let r = (command & 0xFF) >> 3;
                            let g = ((command >> 8) & 0xFF) >> 3;
                            let b = ((command >> 16) & 0xFF) >> 3;

                            let pixel = (r | (g << 5) | (b << 10)) as u16;
                            self.vram_fill(width, height, vram_x, vram_y, pixel);
                            Gp0State::WaitingForCommand
                        }
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
            Gp0State::ReceivingPolyVert {
                size,
                shaded,
                textured,
                idx,
            } => {
                self.params[idx as usize] = val;

                let limit = size * (1 + shaded as u8 + textured as u8);

                if idx >= limit {
                    let mut index = 1 + shaded as usize;
                    let v0 = (
                        self.params[index] & 0x3FF,
                        (self.params[index] >> 16) & 0x1FF,
                    );
                    index += 1 + textured as usize + shaded as usize;
                    let v1 = (
                        self.params[index] & 0x3FF,
                        (self.params[index] >> 16) & 0x1FF,
                    );
                    index += 1 + textured as usize + shaded as usize;
                    let v2 = (
                        self.params[index] & 0x3FF,
                        (self.params[index] >> 16) & 0x1FF,
                    );

                    let (min, max) = self.get_bounds(v0, v1, v2);

                    match (shaded, textured) {
                        (false, false) => {
                            self.rasterize_triangle(v0, v1, v2, min, max);
                        }
                        (false, true) => {
                            let t0 = self.params[2];
                            let t1 = self.params[4];
                            let t2 = self.params[6];

                            let uv0 = (t0 & 0xFF, (t0 >> 8) & 0xFF);
                            let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
                            let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);

                            let clut_x = 16 * ((t0 >> 16) & 0x3F);
                            let clut_y = (t0 >> 22) & 0x1FF;
                            let clut = (clut_x as u16, clut_y as u16);

                            let tex_page = (t1 >> 16) & 0xFFFF;

                            self.rasterize_triangle_textured(
                                v0, v1, v2, uv0, uv1, uv2, tex_page, clut, min, max,
                            );
                        }
                        (true, false) => {
                            let c0 = self.params[1];
                            let c1 = self.params[3];
                            let c2 = self.params[5];

                            self.rasterize_triangle_shaded(v0, v1, v2, c0, c1, c2, min, max);
                        }
                        (true, true) => {
                            let c0 = self.params[1];
                            let t0 = self.params[3];
                            let c1 = self.params[4];
                            let t1 = self.params[6];
                            let c2 = self.params[7];
                            let t2 = self.params[9];

                            let uv0 = (t0 & 0xFF, (t0 >> 8) & 0xFF);
                            let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
                            let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);

                            let clut_x = 16 * ((t0 >> 16) & 0x3F);
                            let clut_y = (t0 >> 22) & 0x1FF;
                            let clut = (clut_x as u16, clut_y as u16);

                            let tex_page = (t1 >> 16) & 0xFFFF;

                            self.rasterize_triangle_textured_and_shaded(
                                v0, v1, v2, uv0, uv1, uv2, c0, c1, c2, tex_page, clut, min, max,
                            );
                        }
                    }

                    if size == 4 {
                        index += 1 + textured as usize + shaded as usize;
                        let v3 = (
                            self.params[index] & 0x3FF,
                            (self.params[index] >> 16) & 0x1FF,
                        );

                        let (min, max) = self.get_bounds(v1, v2, v3);

                        match (shaded, textured) {
                            (false, false) => {
                                self.rasterize_triangle(v1, v2, v3, min, max);
                            }
                            (false, true) => {
                                let t1 = self.params[4];
                                let t2 = self.params[6];
                                let t3 = self.params[8];

                                let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
                                let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);
                                let uv3 = (t3 & 0xFF, (t3 >> 8) & 0xFF);

                                let clut_x = 16 * ((self.params[2] >> 16) & 0x3F);
                                let clut_y = (self.params[2] >> 22) & 0x1FF;
                                let clut = (clut_x as u16, clut_y as u16);
                                let tex_page = (t1 >> 16) & 0xFFFF;

                                self.rasterize_triangle_textured(
                                    v1, v2, v3, uv1, uv2, uv3, tex_page, clut, min, max,
                                );
                            }
                            (true, false) => {
                                let c1 = self.params[3];
                                let c2 = self.params[5];
                                let c3 = self.params[7];

                                self.rasterize_triangle_shaded(v1, v2, v3, c1, c2, c3, min, max);
                            }
                            (true, true) => {
                                let c1 = self.params[4];
                                let t1 = self.params[6];
                                let c2 = self.params[7];
                                let t2 = self.params[9];
                                let c3 = self.params[10];
                                let t3 = self.params[12];

                                let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
                                let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);
                                let uv3 = (t3 & 0xFF, (t3 >> 8) & 0xFF);

                                let clut_x = 16 * ((self.params[3] >> 16) & 0x3F);
                                let clut_y = (self.params[3] >> 22) & 0x1FF;
                                let clut = (clut_x as u16, clut_y as u16);
                                let tex_page = (t1 >> 16) & 0xFFFF;

                                self.rasterize_triangle_textured_and_shaded(
                                    v1, v2, v3, uv1, uv2, uv3, c1, c2, c3, tex_page, clut, min, max,
                                );
                            }
                        }
                    }

                    Gp0State::WaitingForCommand
                } else {
                    Gp0State::ReceivingPolyVert {
                        size,
                        shaded,
                        textured,
                        idx: idx + 1,
                    }
                }
            }
            Gp0State::ReceivingLineVert {
                polyline,
                shaded,
                idx,
            } => {
                let poly_stop = val & 0xF000F000 == 0x50005000;

                self.params[idx as usize] = val;

                match (polyline, poly_stop) {
                    (true, true) => {
                        // Polyline stop signal received. Stop drawing lines
                        Gp0State::WaitingForCommand
                    }
                    (true, false) => {
                        // Polyline but no stop signal. Continue to draw
                        if idx == 2 && !shaded {
                            let x1 = self.params[1] & 0x3FF;
                            let y1 = (self.params[1] >> 16) & 0x1FF;
                            let x2 = self.params[2] & 0x3FF;
                            let y2 = (self.params[2] >> 16) & 0x1FF;

                            self.draw_line(x1, y1, x2, y2);
                            self.params[1] = self.params[2];
                            Gp0State::ReceivingLineVert {
                                polyline,
                                shaded,
                                idx: 2,
                            }
                        } else if idx == 4 {
                            let color1 = self.params[1];
                            let x1 = self.params[2] & 0x3FF;
                            let y1 = (self.params[2] >> 16) & 0x1FF;
                            let color2 = self.params[3];
                            let x2 = self.params[4] & 0x3FF;
                            let y2 = (self.params[4] >> 16) & 0x1FF;

                            self.draw_line_shaded(x1, y1, color1, x2, y2, color2);
                            self.params[1] = self.params[3];
                            self.params[2] = self.params[4];
                            Gp0State::ReceivingLineVert {
                                polyline,
                                shaded,
                                idx: 3,
                            }
                        } else {
                            Gp0State::ReceivingLineVert {
                                polyline,
                                shaded,
                                idx: idx + 1,
                            }
                        }
                    }
                    (false, _) => {
                        // Single line. Draw one line then end
                        if idx == 2 && !shaded {
                            let x1 = self.params[1] & 0x3FF;
                            let y1 = (self.params[1] >> 16) & 0x1FF;
                            let x2 = self.params[2] & 0x3FF;
                            let y2 = (self.params[2] >> 16) & 0x1FF;

                            self.draw_line(x1, y1, x2, y2);
                            Gp0State::WaitingForCommand
                        } else if idx == 4 {
                            let color1 = self.params[1];
                            let x1 = self.params[2] & 0x3FF;
                            let y1 = (self.params[2] >> 16) & 0x1FF;
                            let color2 = self.params[3];
                            let x2 = self.params[4] & 0x3FF;
                            let y2 = (self.params[4] >> 16) & 0x1FF;

                            self.draw_line_shaded(x1, y1, color1, x2, y2, color2);
                            Gp0State::WaitingForCommand
                        } else {
                            Gp0State::ReceivingLineVert {
                                polyline,
                                shaded,
                                idx: idx + 1,
                            }
                        }
                    }
                }
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
            let vram_row = ((fields.vram_y + fields.current_row) & 0x1FF) as usize;
            let vram_col = ((fields.vram_x + fields.current_col) & 0x3FF) as usize;

            let vram_addr = 1024 * vram_row + vram_col;
            self.write_5bit_color(vram_addr, halfword);

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
            let vram_row = ((fields.vram_y + fields.current_row) & 0x1FF) as usize;
            let vram_col = ((fields.vram_x + fields.current_col) & 0x3FF) as usize;
            let vram_addr = 1024 * vram_row + vram_col;
            let [vram_lo, vram_hi] = self.read_vram(vram_addr).to_le_bytes();

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

    fn vram_copy(&mut self) {
        let source_x = self.params[0] & 0x3FF;
        let source_y = (self.params[0] >> 16) & 0x1FF;
        let dest_x = self.params[1] & 0x3FF;
        let dest_y = (self.params[1] >> 16) & 0x1FF;
        let width = self.params[2] & 0x3FF;
        let height = (self.params[2] >> 16) & 0x1FF;

        for y in 0..height {
            for x in 0..width {
                let source_row = ((source_y + y) & 0x1FF) as usize;
                let source_col = ((source_x + x) & 0x3FF) as usize;
                let dest_row = ((dest_y + y) & 0x1FF) as usize;
                let dest_col = ((dest_x + x) & 0x3FF) as usize;
                let source_addr = 1024 * source_row + source_col;
                let dest_addr = 1024 * dest_row + dest_col;
                self.copy_vram(source_addr, dest_addr);
            }
        }
    }

    // returns (min_x, min_y) and (max_x, max_y) of bounding box
    fn get_bounds(
        &mut self,
        v0: (u32, u32),
        v1: (u32, u32),
        v2: (u32, u32),
    ) -> ((u32, u32), (u32, u32)) {
        let min_x = cmp::max(
            self.draw_area_top_left.0,
            cmp::min(v0.0, cmp::min(v1.0, v2.0)),
        );
        let min_y = cmp::max(
            self.draw_area_top_left.1,
            cmp::min(v0.1, cmp::min(v1.1, v2.1)),
        );
        let max_x = cmp::min(
            self.draw_area_bot_right.0,
            cmp::max(v0.0, cmp::max(v1.0, v2.0)),
        );
        let max_y = cmp::min(
            self.draw_area_bot_right.1,
            cmp::max(v0.1, cmp::max(v1.1, v2.1)),
        );

        ((min_x, min_y), (max_x, max_y))
    }

    fn rasterize_triangle(
        &mut self,
        mut v0: (u32, u32),
        mut v1: (u32, u32),
        v2: (u32, u32),
        min: (u32, u32),
        max: (u32, u32),
    ) {
        if min.0 > max.0 || min.1 > max.1 {
            return;
        }

        if cross_product(v0, v1, v2) < 0 {
            mem::swap(&mut v0, &mut v1);
        }

        let command = self.params[0];

        let r = (command & 0xFF) >> 3;
        let g = ((command >> 8) & 0xFF) >> 3;
        let b = ((command >> 16) & 0xFF) >> 3;

        let pixel = (r | (g << 5) | (b << 10)) as u16;

        let use_alpha = (self.params[0] >> 25) & 0x1 > 0;

        for y in min.1..=max.1 {
            for x in min.0..=max.0 {
                if inside_triange((x, y), v0, v1, v2).is_some() {
                    let vram_addr = 1024 * (y as usize) + x as usize;
                    if use_alpha {
                        self.write_5bit_color_alpha(vram_addr, pixel);
                    } else {
                        self.write_5bit_color(vram_addr, pixel);
                    }
                }
            }
        }
    }

    fn rasterize_triangle_textured(
        &mut self,
        mut v0: (u32, u32),
        mut v1: (u32, u32),
        v2: (u32, u32),
        mut uv0: (u32, u32),
        mut uv1: (u32, u32),
        uv2: (u32, u32),
        tex_page: u32,
        clut: (u16, u16),
        min: (u32, u32),
        max: (u32, u32),
    ) {
        if min.0 > max.0 || min.1 > max.1 {
            return;
        }

        if cross_product(v0, v1, v2) < 0 {
            mem::swap(&mut v0, &mut v1);
            mem::swap(&mut uv0, &mut uv1);
        }

        let use_alpha = (self.params[0] >> 25) & 0x1 > 0;
        let use_modulation = self.params[0] & 0x1000000 == 0;

        let texture_bits = match (tex_page >> 7) & 0b11 {
            0 => TextureBits::Four,
            1 => TextureBits::Eight,
            2 => TextureBits::Fifteen,
            _ => {
                event!(target: "ps1_emulator::GPU", Level::ERROR, "Texture size outside of Four, Eight and Fifteen");
                panic!("Impossible")
            }
        };
        let tex_page_x = 64 * (tex_page & 0xF);
        let tex_page_y = 256 * ((tex_page >> 4) & 0x1);
        let tex_page = (tex_page_x as u16, tex_page_y as u16);

        for y in min.1..=max.1 {
            for x in min.0..=max.0 {
                if let Some([a, b, c]) = inside_triange((x, y), v0, v1, v2) {
                    let u = (a * uv0.0 as f32 + b * uv1.0 as f32 + c * uv2.0 as f32).round() as u32;
                    let v = (a * uv0.1 as f32 + b * uv1.1 as f32 + c * uv2.1 as f32).round() as u32;
                    let pixel = self.get_color_from_uv(u, v, clut, tex_page, texture_bits);

                    if pixel == 0 {
                        continue;
                    }

                    let pixel = if use_modulation {
                        let color = self.params[0] & 0xFFFFFF;
                        self.modulate_5bit_color(pixel, color)
                    } else {
                        pixel
                    };

                    let vram_addr = 1024 * (y as usize) + x as usize;
                    if use_alpha {
                        self.write_5bit_color_alpha(vram_addr, pixel);
                    } else {
                        self.write_5bit_color(vram_addr, pixel);
                    }
                }
            }
        }
    }

    fn rasterize_triangle_shaded(
        &mut self,
        mut v0: (u32, u32),
        mut v1: (u32, u32),
        v2: (u32, u32),
        mut c0: u32,
        mut c1: u32,
        c2: u32,
        min: (u32, u32),
        max: (u32, u32),
    ) {
        if min.0 > max.0 || min.1 > max.1 {
            return;
        }

        if cross_product(v0, v1, v2) < 0 {
            mem::swap(&mut v0, &mut v1);
            mem::swap(&mut c0, &mut c1);
        }

        let r0 = c0 & 0xFF;
        let g0 = (c0 >> 8) & 0xFF;
        let b0 = (c0 >> 16) & 0xFF;
        let r1 = c1 & 0xFF;
        let g1 = (c1 >> 8) & 0xFF;
        let b1 = (c1 >> 16) & 0xFF;
        let r2 = c2 & 0xFF;
        let g2 = (c2 >> 8) & 0xFF;
        let b2 = (c2 >> 16) & 0xFF;

        let use_alpha = (self.params[0] >> 25) & 0x1 > 0;

        for y in min.1..=max.1 {
            for x in min.0..=max.0 {
                if let Some([a, b, c]) = inside_triange((x, y), v0, v1, v2) {
                    let r = (a * r0 as f32 + b * r1 as f32 + c * r2 as f32).round() as u8;
                    let g = (a * g0 as f32 + b * g1 as f32 + c * g2 as f32).round() as u8;
                    let b = (a * b0 as f32 + b * b1 as f32 + c * b2 as f32).round() as u8;
                    let (r, g, b) = if self.dither_enabled {
                        dither((r, g, b), (x, y))
                    } else {
                        (r, g, b)
                    };
                    let r = (r >> 3) as u16;
                    let g = (g >> 3) as u16;
                    let b = (b >> 3) as u16;
                    let pixel = r | (g << 5) | (b << 10);

                    let vram_addr = 1024 * (y as usize) + x as usize;
                    if use_alpha {
                        self.write_5bit_color_alpha(vram_addr, pixel);
                    } else {
                        self.write_5bit_color(vram_addr, pixel);
                    }
                }
            }
        }
    }

    fn rasterize_triangle_textured_and_shaded(
        &mut self,
        mut v0: (u32, u32),
        mut v1: (u32, u32),
        v2: (u32, u32),
        mut uv0: (u32, u32),
        mut uv1: (u32, u32),
        uv2: (u32, u32),
        mut c0: u32,
        mut c1: u32,
        c2: u32,
        tex_page: u32,
        clut: (u16, u16),
        min: (u32, u32),
        max: (u32, u32),
    ) {
        if min.0 > max.0 || min.1 > max.1 {
            return;
        }

        if cross_product(v0, v1, v2) < 0 {
            mem::swap(&mut v0, &mut v1);
            mem::swap(&mut uv0, &mut uv1);
            mem::swap(&mut c0, &mut c1);
        }

        let r0 = c0 & 0xFF;
        let g0 = (c0 >> 8) & 0xFF;
        let b0 = (c0 >> 16) & 0xFF;
        let r1 = c1 & 0xFF;
        let g1 = (c1 >> 8) & 0xFF;
        let b1 = (c1 >> 16) & 0xFF;
        let r2 = c2 & 0xFF;
        let g2 = (c2 >> 8) & 0xFF;
        let b2 = (c2 >> 16) & 0xFF;

        let use_alpha = (self.params[0] >> 25) & 0x1 > 0;

        let texture_bits = match (tex_page >> 7) & 0b11 {
            0 => TextureBits::Four,
            1 => TextureBits::Eight,
            2 => TextureBits::Fifteen,
            _ => {
                event!(target: "ps1_emulator::GPU", Level::ERROR, "Texture size outside of Four, Eight and Fifteen");
                panic!("Impossible")
            }
        };
        let tex_page_x = 64 * (tex_page & 0xF);
        let tex_page_y = 256 * ((tex_page >> 4) & 0x1);
        let tex_page = (tex_page_x as u16, tex_page_y as u16);

        for y in min.1..=max.1 {
            for x in min.0..=max.0 {
                if let Some([a, b, c]) = inside_triange((x, y), v0, v1, v2) {
                    let u = (a * uv0.0 as f32 + b * uv1.0 as f32 + c * uv2.0 as f32).round() as u32;
                    let v = (a * uv0.1 as f32 + b * uv1.1 as f32 + c * uv2.1 as f32).round() as u32;
                    let pixel = self.get_color_from_uv(u, v, clut, tex_page, texture_bits);

                    if pixel == 0 {
                        continue;
                    }

                    let r = (a * r0 as f32 + b * r1 as f32 + c * r2 as f32).round() as u8;
                    let g = (a * g0 as f32 + b * g1 as f32 + c * g2 as f32).round() as u8;
                    let b = (a * b0 as f32 + b * b1 as f32 + c * b2 as f32).round() as u8;

                    let (r, g, b) = if self.dither_enabled {
                        dither((r, g, b), (x, y))
                    } else {
                        (r, g, b)
                    };
                    let color = (r as u32) | ((g as u32) << 8) | ((b as u32) << 16);

                    let pixel = self.modulate_5bit_color(pixel, color as u32);

                    let vram_addr = 1024 * (y as usize) + x as usize;
                    if use_alpha {
                        self.write_5bit_color_alpha(vram_addr, pixel);
                    } else {
                        self.write_5bit_color(vram_addr, pixel);
                    }
                }
            }
        }
    }

    fn draw_line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32) {
        let command = self.params[0];

        let r = (command & 0xFF) >> 3;
        let g = ((command >> 8) & 0xFF) >> 3;
        let b = ((command >> 16) & 0xFF) >> 3;

        let pixel = (r | (g << 5) | (b << 10)) as u16;

        let use_alpha = command & 0x2000000 > 0;

        if x1.abs_diff(x2) >= y1.abs_diff(y2) {
            let slope = (y2 as f32 - y1 as f32) / (x2 as f32 - x1 as f32);
            let range = if x1 <= x2 { x1..=x2 } else { x2..=x1 };
            for x in range {
                let y = (slope * (x as f32 - x1 as f32) + y1 as f32).floor();
                if (0.0..=512.0).contains(&y) {
                    let vram_addr = 1024 * (y as usize) + x as usize;
                    if use_alpha {
                        self.write_5bit_color_alpha(vram_addr, pixel);
                    } else {
                        self.write_5bit_color(vram_addr, pixel);
                    }
                }
            }
        } else {
            let slope = (x2 as f32 - x1 as f32) / (y2 as f32 - y1 as f32);
            let range = if y1 <= y2 { y1..=y2 } else { y2..=y1 };
            for y in range {
                let x = (slope * (y as f32 - y1 as f32) + x1 as f32).floor();
                if (0.0..=1024.0).contains(&x) {
                    let vram_addr = 1024 * (y as usize) + x as usize;
                    if use_alpha {
                        self.write_5bit_color_alpha(vram_addr, pixel);
                    } else {
                        self.write_5bit_color(vram_addr, pixel);
                    }
                }
            }
        };
    }

    fn draw_line_shaded(&mut self, x1: u32, y1: u32, c1: u32, x2: u32, y2: u32, c2: u32) {
        let command = self.params[0];

        let r1 = c1 & 0xFF;
        let g1 = (c1 >> 8) & 0xFF;
        let b1 = (c1 >> 16) & 0xFF;
        let r2 = c2 & 0xFF;
        let g2 = (c2 >> 8) & 0xFF;
        let b2 = (c2 >> 16) & 0xFF;

        let use_alpha = command & 0x2000000 > 0;

        let total_dist =
            f32::sqrt((x1 as f32 - x2 as f32).powi(2) + (y1 as f32 - y2 as f32).powi(2));
        if x1.abs_diff(x2) >= y1.abs_diff(y2) {
            let slope = (y2 as f32 - y1 as f32) / (x2 as f32 - x1 as f32);
            let range = if x1 <= x2 { x1..=x2 } else { x2..=x1 };
            for x in range {
                let y_raw = slope * (x as f32 - x1 as f32) + y1 as f32;
                let y = y_raw.floor();
                let dist = f32::sqrt((x as f32 - x2 as f32).powi(2) + (y_raw - y2 as f32).powi(2));
                let pct = dist / total_dist;

                let r = (pct * (r1 as f32) + (1.0 - pct) * (r2 as f32)).round() as u16;
                let g = (pct * (g1 as f32) + (1.0 - pct) * (g2 as f32)).round() as u16;
                let b = (pct * (b1 as f32) + (1.0 - pct) * (b2 as f32)).round() as u16;
                let r = (r & 0xFF) >> 3;
                let g = (g & 0xFF) >> 3;
                let b = (b & 0xFF) >> 3;
                let pixel = r | (g << 5) | (b << 10);

                if (0.0..=512.0).contains(&y) {
                    let vram_addr = 1024 * (y as usize) + x as usize;
                    if use_alpha {
                        self.write_5bit_color_alpha(vram_addr, pixel);
                    } else {
                        self.write_5bit_color(vram_addr, pixel);
                    }
                }
            }
        } else {
            let slope = (x2 as f32 - x1 as f32) / (y2 as f32 - y1 as f32);
            let range = if y1 <= y2 { y1..=y2 } else { y2..=y1 };
            for y in range {
                let x_raw = slope * (y as f32 - y1 as f32) + x1 as f32;
                let x = x_raw.floor();
                let dist = f32::sqrt((x_raw - x2 as f32).powi(2) + (y as f32 - y2 as f32).powi(2));
                let pct = dist / total_dist;

                let r = (pct * (r1 as f32) + (1.0 - pct) * (r2 as f32)).round() as u16;
                let g = (pct * (g1 as f32) + (1.0 - pct) * (g2 as f32)).round() as u16;
                let b = (pct * (b1 as f32) + (1.0 - pct) * (b2 as f32)).round() as u16;
                let r = (r & 0xFF) >> 3;
                let g = (g & 0xFF) >> 3;
                let b = (b & 0xFF) >> 3;
                let pixel = r | (g << 5) | (b << 10);

                if (0.0..1024.0).contains(&x) {
                    let vram_addr = 1024 * (y as usize) + x as usize;
                    if use_alpha {
                        self.write_5bit_color_alpha(vram_addr, pixel);
                    } else {
                        self.write_5bit_color(vram_addr, pixel);
                    }
                }
            }
        };
    }

    fn draw_textured_rectangle(&mut self, width: u32, height: u32) {
        let command = self.params[0];

        let use_alpha = command & 0x2000000 > 0;
        let use_modulation = command & 0x1000000 == 0;

        let tex_page_base_x = ((self.tex_page_x as u32) * 64) as u16;
        let tex_page_base_y = (256 * (self.tex_page_y as u32)) as u16;
        let u_offset = self.params[2] & 0xFF;
        let v_offset = (self.params[2] >> 8) & 0xFF;
        let clut = (self.params[2] >> 16) as u16;
        let clut_x = 16 * (clut & 0x3F);
        let clut_y = (clut >> 6) & 0x1FF;

        let vram_x = self.params[1] & 0x3FF;
        let vram_y = (self.params[1] >> 16) & 0x1FF;
        for y in 0..height {
            for x in 0..width {
                let vram_row = (vram_y + y) & 0x1FF;
                let vram_col = (vram_x + x) & 0x3FF;
                if (self.draw_area_top_left.0..self.draw_area_bot_right.0).contains(&vram_col)
                    && (self.draw_area_top_left.1..self.draw_area_bot_right.1).contains(&vram_row)
                {
                    let u = if self.rect_x_flip {
                        u_offset - x
                    } else {
                        x + u_offset
                    };
                    let v = if self.rect_y_flip {
                        v_offset - y
                    } else {
                        y + v_offset
                    };

                    let pixel = self.get_color_from_uv(
                        u,
                        v,
                        (clut_x, clut_y),
                        (tex_page_base_x, tex_page_base_y),
                        self.tex_page_colors,
                    );

                    let vram_addr = 1024 * vram_row as usize + vram_col as usize;

                    if pixel == 0 {
                        continue;
                    }

                    let pixel = if use_modulation {
                        let color = command & 0xFFFFFF;
                        self.modulate_5bit_color(pixel, color)
                    } else {
                        pixel
                    };

                    if use_alpha {
                        self.write_5bit_color_alpha(vram_addr, pixel);
                    } else {
                        self.write_5bit_color(vram_addr, pixel);
                    }
                }
            }
        }
    }

    fn draw_untextured_rectangle(&mut self, width: u32, height: u32) {
        let command = self.params[0];

        let use_alpha = command & 0x2000000 > 0;

        let r = (command & 0xFF) >> 3;
        let g = ((command >> 8) & 0xFF) >> 3;
        let b = ((command >> 16) & 0xFF) >> 3;

        let pixel = (r | (g << 5) | (b << 10)) as u16;

        let vram_x = self.params[1] & 0x3FF;
        let vram_y = (self.params[1] >> 16) & 0x1FF;

        for y in 0..height {
            for x in 0..width {
                let vram_row = (vram_y + y) & 0x1FF;
                let vram_col = (vram_x + x) & 0x3FF;
                if (self.draw_area_top_left.0..self.draw_area_bot_right.0).contains(&vram_col)
                    && (self.draw_area_top_left.1..self.draw_area_bot_right.1).contains(&vram_row)
                {
                    let vram_addr = 1024 * vram_row as usize + vram_col as usize;
                    if use_alpha {
                        self.write_5bit_color_alpha(vram_addr, pixel);
                    } else {
                        self.write_5bit_color(vram_addr, pixel);
                    }
                }
            }
        }
    }

    fn get_texel_4bit(&self, x: u32, y: u32, clut: (u16, u16), tex_page: (u16, u16)) -> u16 {
        // Get texel at (x,y) relative to to the texture page
        let tex_addr =
            x as usize / 4 + tex_page.0 as usize + 1024 * (y as usize + tex_page.1 as usize);
        let texel = self.read_vram(tex_addr);
        // Get the index offset for current pixel to be used in the clut
        let index = (texel >> (4 * (x % 4))) & 0xF;
        let clut_addr = 1024 * (clut.1 as usize) + clut.0 as usize + index as usize;
        self.read_vram(clut_addr)
    }

    fn get_texel_8bit(&self, x: u32, y: u32, clut: (u16, u16), tex_page: (u16, u16)) -> u16 {
        // Get texel at (x,y) relative to to the texture page
        let tex_addr =
            x as usize / 2 + tex_page.0 as usize + 1024 * (y as usize + tex_page.1 as usize);
        let texel = self.read_vram(tex_addr);
        // Get the index offset for current pixel to be used in the clut
        let index = (texel >> (8 * (x % 2))) & 0xFF;
        let clut_addr = 1024 * (clut.1 as usize) + clut.0 as usize + index as usize;
        self.read_vram(clut_addr)
    }

    fn get_texel_15bit(&self, x: u32, y: u32, tex_page: (u16, u16)) -> u16 {
        let tex_addr = x as usize + tex_page.0 as usize + 1024 * (y as usize + tex_page.1 as usize);
        self.read_vram(tex_addr)
    }

    fn get_color_from_uv(
        &self,
        u: u32,
        v: u32,
        clut: (u16, u16),
        tex_page: (u16, u16),
        tex_page_color: TextureBits,
    ) -> u16 {
        let mask_x = self.texture_window & 0x1F;
        let mask_y = (self.texture_window >> 5) & 0x1F;
        let offset_x = (self.texture_window >> 10) & 0x1F;
        let offset_y = (self.texture_window >> 15) & 0x1F;

        let u = (u & !(mask_x * 8)) | (8 * (mask_x & offset_x));
        let v = (v & !(mask_y * 8)) | (8 * (mask_y & offset_y));

        match tex_page_color {
            TextureBits::Four => {
                self.get_texel_4bit(u, v, (clut.0, clut.1), (tex_page.0, tex_page.1))
            }
            TextureBits::Eight => {
                self.get_texel_8bit(u, v, (clut.0, clut.1), (tex_page.0, tex_page.1))
            }
            TextureBits::Fifteen => self.get_texel_15bit(u, v, (tex_page.0, tex_page.1)),
        }
    }

    pub fn is_sending_data(&self) -> bool {
        matches!(self.state, Gp0State::SendingData(_))
    }
}

// Cross product of (v1 - v0) and (v2 - v0)
fn cross_product(v0: (u32, u32), v1: (u32, u32), v2: (u32, u32)) -> i32 {
    (v1.0 as i32 - v0.0 as i32) * (v2.1 as i32 - v0.1 as i32)
        - (v1.1 as i32 - v0.1 as i32) * (v2.0 as i32 - v0.0 as i32)
}

fn inside_triange(
    p: (u32, u32),
    v0: (u32, u32),
    v1: (u32, u32),
    v2: (u32, u32),
) -> Option<[f32; 3]> {
    let mut barycentric_coords = [0.0; 3];

    let denominator = cross_product(v0, v1, v2) as f32;
    if denominator == 0.0 {
        return Some([1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]);
    }

    for (i, (a, b)) in [(v1, v2), (v2, v0), (v0, v1)].iter().enumerate() {
        let cross_product = cross_product(*a, *b, p);
        barycentric_coords[i] = (cross_product as f32) / denominator;

        if cross_product < 0 {
            return None;
        }

        if cross_product == 0 {
            if b.1 > a.1 {
                return None;
            }

            if b.1 == a.1 && b.0 < a.0 {
                return None;
            }
        }
    }

    Some(barycentric_coords)
}

// Color is in rgb
fn dither(color: (u8, u8, u8), pixel: (u32, u32)) -> (u8, u8, u8) {
    let offset = DITHER_TABLE[(pixel.0 & 0b11) as usize][(pixel.1 & 0b11) as usize];

    (
        color.0.saturating_add_signed(offset),
        color.1.saturating_add_signed(offset),
        color.2.saturating_add_signed(offset),
    )
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

fn convert_8bit_to_5bit(color: u8) -> u16 {
    match color {
        0 => 0,
        8 => 1,
        16 => 2,
        25 => 3,
        33 => 4,
        41 => 5,
        49 => 6,
        58 => 7,
        66 => 8,
        74 => 9,
        82 => 10,
        90 => 11,
        99 => 12,
        107 => 13,
        115 => 14,
        123 => 15,
        132 => 16,
        140 => 17,
        148 => 18,
        156 => 19,
        165 => 20,
        173 => 21,
        181 => 22,
        189 => 23,
        197 => 24,
        206 => 25,
        214 => 26,
        222 => 27,
        230 => 28,
        239 => 29,
        247 => 30,
        255 => 31,
        _ => panic!("Impossible"),
    }
}

fn param_limits(command: Commands) -> u8 {
    match command {
        Commands::Rectangle => 1,
        Commands::SizeRectangle => 2,
        Commands::TexturedRectangle => 2,
        Commands::TexturedSizeRectangle => 3,
        Commands::VramToVram => 2,
        Commands::CpuToVram => 1,
        Commands::VramToCpu => 1,
        Commands::VramFill => 2,
    }
}

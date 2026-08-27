use std::cmp::min;

use tracing::{Level, event};

pub struct Gte {
    pub enabled: bool,
    /* Data Registers */
    v0: [i16; 3],
    v1: [i16; 3],
    v2: [i16; 3],
    rgb: u32,
    otz: u16,
    intermediates: [i16; 4],
    screenxy: [[i16; 2]; 4],
    screenz: [u16; 4],
    characteristic_color: [u32; 3],
    res1: u32,
    mac: [i32; 4],
    // irgb: u16,
    // orgb: u16,
    lzcs: i32,
    /* Control Registers */
    rotation_matrix: [[i16; 3]; 3],
    light_matrix: [[i16; 3]; 3],
    light_color_matrix: [[i16; 3]; 3],
    translation_vec: [i32; 3],
    background_color: [i32; 3],
    far_color: [i32; 3],
    screen_offset: [i32; 2],
    h: u16,
    depth_cue_a: i16,
    depth_cue_b: i32,
    zsf3: i16,
    zsf4: i16,
    flag: Flag,
}

impl Gte {
    pub fn new() -> Self {
        Self {
            enabled: false,
            v0: [0; 3],
            v1: [0; 3],
            v2: [0; 3],
            rgb: 0,
            otz: 0,
            intermediates: [0; 4],
            screenxy: [[0; 2]; 4],
            screenz: [0; 4],
            characteristic_color: [0; 3],
            res1: 0,
            mac: [0; 4],
            // irgb: 0,
            // orgb: 0,
            lzcs: 0,
            rotation_matrix: [[0; 3]; 3],
            light_matrix: [[0; 3]; 3],
            light_color_matrix: [[0; 3]; 3],
            translation_vec: [0; 3],
            background_color: [0; 3],
            far_color: [0; 3],
            screen_offset: [0; 2],
            h: 0,
            depth_cue_a: 0,
            depth_cue_b: 0,
            zsf3: 0,
            zsf4: 0,
            flag: Flag(0),
        }
    }

    pub fn control_reg_read(&self, reg: u32) -> u32 {
        if self.enabled {
            event!(target: "ps1_emulator::GTE", Level::TRACE, "Control read reg: {reg}");
            match reg {
                0 => {
                    ((self.rotation_matrix[0][1] as u32) << 16)
                        + (self.rotation_matrix[0][0] as u32 & 0xFFFF)
                }
                1 => {
                    ((self.rotation_matrix[1][0] as u32) << 16)
                        + (self.rotation_matrix[0][2] as u32 & 0xFFFF)
                }
                2 => {
                    ((self.rotation_matrix[1][2] as u32) << 16)
                        + (self.rotation_matrix[1][1] as u32 & 0xFFFF)
                }
                3 => {
                    ((self.rotation_matrix[2][1] as u32) << 16)
                        + (self.rotation_matrix[2][0] as u32 & 0xFFFF)
                }
                4 => (self.rotation_matrix[2][2] as i32) as u32,
                5 => self.translation_vec[0] as u32,
                6 => self.translation_vec[1] as u32,
                7 => self.translation_vec[2] as u32,
                8 => {
                    ((self.light_matrix[0][1] as u32) << 16)
                        + (self.light_matrix[0][0] as u32 & 0xFFFF)
                }
                9 => {
                    ((self.light_matrix[1][0] as u32) << 16)
                        + (self.light_matrix[0][2] as u32 & 0xFFFF)
                }
                10 => {
                    ((self.light_matrix[1][2] as u32) << 16)
                        + (self.light_matrix[1][1] as u32 & 0xFFFF)
                }
                11 => {
                    ((self.light_matrix[2][1] as u32) << 16)
                        + (self.light_matrix[2][0] as u32 & 0xFFFF)
                }
                12 => (self.light_matrix[2][2] as i32) as u32,
                13 => self.background_color[0] as u32,
                14 => self.background_color[1] as u32,
                15 => self.background_color[2] as u32,
                16 => {
                    ((self.light_color_matrix[0][1] as u32) << 16)
                        + (self.light_color_matrix[0][0] as u32 & 0xFFFF)
                }
                17 => {
                    ((self.light_color_matrix[1][0] as u32) << 16)
                        + (self.light_color_matrix[0][2] as u32 & 0xFFFF)
                }
                18 => {
                    ((self.light_color_matrix[1][2] as u32) << 16)
                        + (self.light_color_matrix[1][1] as u32 & 0xFFFF)
                }
                19 => {
                    ((self.light_color_matrix[2][1] as u32) << 16)
                        + (self.light_color_matrix[2][0] as u32 & 0xFFFF)
                }
                20 => (self.light_color_matrix[2][2] as i32) as u32,
                21 => self.far_color[0] as u32,
                22 => self.far_color[1] as u32,
                23 => self.far_color[2] as u32,
                24 => self.screen_offset[0] as u32,
                25 => self.screen_offset[1] as u32,
                26 => self.h as i16 as i32 as u32,
                27 => self.depth_cue_a as u32,
                28 => self.depth_cue_b as u32,
                29 => self.zsf3 as u32,
                30 => self.zsf4 as u32,
                31 => {
                    let error_bit = (self.flag.0 & 0x7F87E000 > 0) as u32;
                    self.flag.0 | (error_bit << 31)
                }
                _ => panic!("Impossible GTE Control Register"),
            }
        } else {
            0
        }
    }

    pub fn control_reg_write(&mut self, reg: u32, val: u32) {
        if self.enabled {
            event!(target: "ps1_emulator::GTE", Level::TRACE, "Control write to reg: {reg} with {:08X}", val);
            match reg {
                0 => {
                    self.rotation_matrix[0][0] = (val & 0xFFFF) as i16;
                    self.rotation_matrix[0][1] = (val >> 16) as i16;
                }
                1 => {
                    self.rotation_matrix[0][2] = (val & 0xFFFF) as i16;
                    self.rotation_matrix[1][0] = (val >> 16) as i16;
                }
                2 => {
                    self.rotation_matrix[1][1] = (val & 0xFFFF) as i16;
                    self.rotation_matrix[1][2] = (val >> 16) as i16;
                }
                3 => {
                    self.rotation_matrix[2][0] = (val & 0xFFFF) as i16;
                    self.rotation_matrix[2][1] = (val >> 16) as i16;
                }
                4 => self.rotation_matrix[2][2] = (val & 0xFFFF) as i16,
                5 => self.translation_vec[0] = val as i32,
                6 => self.translation_vec[1] = val as i32,
                7 => self.translation_vec[2] = val as i32,
                8 => {
                    self.light_matrix[0][0] = (val & 0xFFFF) as i16;
                    self.light_matrix[0][1] = (val >> 16) as i16;
                }
                9 => {
                    self.light_matrix[0][2] = (val & 0xFFFF) as i16;
                    self.light_matrix[1][0] = (val >> 16) as i16;
                }
                10 => {
                    self.light_matrix[1][1] = (val & 0xFFFF) as i16;
                    self.light_matrix[1][2] = (val >> 16) as i16;
                }
                11 => {
                    self.light_matrix[2][0] = (val & 0xFFFF) as i16;
                    self.light_matrix[2][1] = (val >> 16) as i16;
                }
                12 => self.light_matrix[2][2] = (val & 0xFFFF) as i16,
                13 => self.background_color[0] = val as i32,
                14 => self.background_color[1] = val as i32,
                15 => self.background_color[2] = val as i32,
                16 => {
                    self.light_color_matrix[0][0] = (val & 0xFFFF) as i16;
                    self.light_color_matrix[0][1] = (val >> 16) as i16;
                }
                17 => {
                    self.light_color_matrix[0][2] = (val & 0xFFFF) as i16;
                    self.light_color_matrix[1][0] = (val >> 16) as i16;
                }
                18 => {
                    self.light_color_matrix[1][1] = (val & 0xFFFF) as i16;
                    self.light_color_matrix[1][2] = (val >> 16) as i16;
                }
                19 => {
                    self.light_color_matrix[2][0] = (val & 0xFFFF) as i16;
                    self.light_color_matrix[2][1] = (val >> 16) as i16;
                }
                20 => self.light_color_matrix[2][2] = (val & 0xFFFF) as i16,
                21 => self.far_color[0] = val as i32,
                22 => self.far_color[1] = val as i32,
                23 => self.far_color[2] = val as i32,
                24 => self.screen_offset[0] = val as i32,
                25 => self.screen_offset[1] = val as i32,
                26 => self.h = (val & 0xFFFF) as u16,
                27 => self.depth_cue_a = (val & 0xFFFF) as i16,
                28 => self.depth_cue_b = val as i32,
                29 => self.zsf3 = (val & 0xFFFF) as i16,
                30 => self.zsf4 = (val & 0xFFFF) as i16,
                31 => self.flag = Flag(val & 0x7FFFF000),
                _ => panic!("Impossible GTE Control Register"),
            }
        }
    }

    pub fn data_reg_read(&self, reg: u32) -> u32 {
        event!(target: "ps1_emulator::GTE", Level::TRACE, "Data read reg: {reg}");
        if self.enabled {
            match reg {
                0 => ((self.v0[1] as u32) << 16) + (self.v0[0] as u32 & 0xFFFF),
                1 => self.v0[2] as u32,
                2 => ((self.v1[1] as u32) << 16) + (self.v1[0] as u32 & 0xFFFF),
                3 => self.v1[2] as u32,
                4 => ((self.v2[1] as u32) << 16) + (self.v2[0] as u32 & 0xFFFF),
                5 => self.v2[2] as u32,
                6 => self.rgb,
                7 => self.otz as u32,
                8 => (self.intermediates[0] as i32) as u32,
                9 => (self.intermediates[1] as i32) as u32,
                10 => (self.intermediates[2] as i32) as u32,
                11 => (self.intermediates[3] as i32) as u32,
                12 => ((self.screenxy[0][1] as u32) << 16) + (self.screenxy[0][0] as u32 & 0xFFFF),
                13 => ((self.screenxy[1][1] as u32) << 16) + (self.screenxy[1][0] as u32 & 0xFFFF),
                14 => ((self.screenxy[2][1] as u32) << 16) + (self.screenxy[2][0] as u32 & 0xFFFF),
                15 => ((self.screenxy[3][1] as u32) << 16) + (self.screenxy[3][0] as u32 & 0xFFFF),
                16 => self.screenz[0] as u32,
                17 => self.screenz[1] as u32,
                18 => self.screenz[2] as u32,
                19 => self.screenz[3] as u32,
                20 => self.characteristic_color[0],
                21 => self.characteristic_color[1],
                22 => self.characteristic_color[2],
                23 => self.res1,
                24 => self.mac[0] as u32,
                25 => self.mac[1] as u32,
                26 => self.mac[2] as u32,
                27 => self.mac[3] as u32,
                28 => {
                    // IRGB
                    let r = (self.intermediates[1] / 0x80).clamp(0, 0x1F) as u32;
                    let g = (self.intermediates[2] / 0x80).clamp(0, 0x1F) as u32;
                    let b = (self.intermediates[3] / 0x80).clamp(0, 0x1F) as u32;

                    r | (g << 5) | (b << 10)
                }
                29 => {
                    // ORGB
                    let r = (self.intermediates[1] / 0x80).clamp(0, 0x1F) as u32;
                    let g = (self.intermediates[2] / 0x80).clamp(0, 0x1F) as u32;
                    let b = (self.intermediates[3] / 0x80).clamp(0, 0x1F) as u32;

                    r | (g << 5) | (b << 10)
                }
                30 => self.lzcs as u32,
                31 => {
                    if self.lzcs >= 0 {
                        // count leading zeroes of lzcs
                        self.lzcs.leading_zeros()
                    } else {
                        // count leading ones of lzcs
                        self.lzcs.leading_ones()
                    }
                }
                _ => panic!("Impossible"),
            }
        } else {
            0
        }
    }

    pub fn data_reg_write(&mut self, reg: u32, val: u32) {
        if self.enabled {
            event!(target: "ps1_emulator::GTE", Level::TRACE, "Data write to reg: {reg} with {:08X}", val);
            match reg {
                0 => {
                    self.v0[0] = (val & 0xFFFF) as i16;
                    self.v0[1] = (val >> 16) as i16;
                }
                1 => self.v0[2] = (val & 0xFFFF) as i16,
                2 => {
                    self.v1[0] = (val & 0xFFFF) as i16;
                    self.v1[1] = (val >> 16) as i16;
                }
                3 => self.v1[2] = (val & 0xFFFF) as i16,
                4 => {
                    self.v2[0] = (val & 0xFFFF) as i16;
                    self.v2[1] = (val >> 16) as i16;
                }
                5 => self.v2[2] = (val & 0xFFFF) as i16,
                6 => self.rgb = val,
                7 => {
                    self.otz = (val & 0xFFFF) as u16;
                }
                8 => self.intermediates[0] = (val & 0xFFFF) as i16,
                9 => self.intermediates[1] = (val & 0xFFFF) as i16,
                10 => self.intermediates[2] = (val & 0xFFFF) as i16,
                11 => self.intermediates[3] = (val & 0xFFFF) as i16,
                12 => {
                    // SXY0
                    self.screenxy[0][0] = (val & 0xFFFF) as i16;
                    self.screenxy[0][1] = (val >> 16) as i16;
                }
                13 => {
                    // SXY1
                    self.screenxy[1][0] = (val & 0xFFFF) as i16;
                    self.screenxy[1][1] = (val >> 16) as i16;
                }
                14 => {
                    // SXY2
                    self.screenxy[2][0] = (val & 0xFFFF) as i16;
                    self.screenxy[2][1] = (val >> 16) as i16;
                }
                15 => {
                    // SXYP
                    let sxp = (val & 0xFFFF) as i16;
                    let syp = (val >> 16) as i16;
                    self.scxy_fifo(sxp, syp);
                }
                16 => self.screenz[0] = (val & 0xFFFF) as u16,
                17 => self.screenz[1] = (val & 0xFFFF) as u16,
                18 => self.screenz[2] = (val & 0xFFFF) as u16,
                19 => self.screenz[3] = (val & 0xFFFF) as u16,
                20 => self.characteristic_color[0] = val,
                21 => self.characteristic_color[1] = val,
                22 => self.characteristic_color[2] = val,
                23 => self.res1 = val,
                24 => self.mac[0] = val as i32,
                25 => self.mac[1] = val as i32,
                26 => self.mac[2] = val as i32,
                27 => self.mac[3] = val as i32,
                28 => {
                    // IRGB
                    let r = val & 0x1F;
                    let g = (val >> 5) & 0x1F;
                    let b = (val >> 10) & 0x1F;
                    self.intermediates[1] = r as i16 * 0x80;
                    self.intermediates[2] = g as i16 * 0x80;
                    self.intermediates[3] = b as i16 * 0x80;
                }
                29 => {} // IRGB is read only mirror of IR1, IR2, IR3
                30 => self.lzcs = val as i32,
                31 => {} // don't write to lzcr. Value is dependent on lzcs,
                _ => panic!("Impossible"),
            }
        }
    }

    pub fn write_command(&mut self, cmd: u32) {
        if !self.enabled {
            return;
        }

        // reset flag on each new GTE command
        self.flag = Flag(0);

        match cmd & 0x3F {
            0x01 => {
                // Perspective Transformation Single: RTPS
                event!(target: "ps1_emulator::GTE", Level::TRACE, "RTPS");
                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;
                self.rtps(sf, lm);
            }
            0x06 => {
                // Normal Clipping
                event!(target: "ps1_emulator::GTE", Level::TRACE, "NCLIP");
                self.nclip();
            }
            0x0C => {
                // Cross Product: OP(sf)
                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;

                self.op(sf, lm);
            }
            0x10 => {
                // Depth Cueing Single: DPCS
                event!(target: "ps1_emulator::GTE", Level::TRACE, "DPCS");
                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;

                self.dpcs(sf, lm);
            }
            0x11 => {
                // Interpolation of a vector and far color vector: INTPL
                event!(target: "ps1_emulator::GTE", Level::TRACE, "INTPL");
                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;

                self.intpl(sf, lm);
            }
            0x12 => {
                let mv = match (cmd >> 17) & 0b11 {
                    0 => MV::Rotation,
                    1 => MV::Light,
                    2 => MV::Color,
                    3 => MV::Reserved,
                    _ => panic!("Impossible"),
                };

                let tv = match (cmd >> 13) & 0b11 {
                    0 => TV::Translation,
                    1 => TV::BackgroundColor,
                    2 => TV::FarColor,
                    3 => TV::None,
                    _ => panic!("Impossible"),
                };

                let vector = match (cmd >> 15) & 0b11 {
                    0 => self.v0,
                    1 => self.v1,
                    2 => self.v2,
                    3 => [
                        self.intermediates[1],
                        self.intermediates[2],
                        self.intermediates[3],
                    ],
                    _ => panic!("Impossible"),
                };

                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;
                event!(target: "ps1_emulator::GTE", Level::TRACE, "MVMVA: 0x{:08X}", cmd);

                self.mvmva(mv, tv, vector, sf, lm);
            }
            0x13 => {
                // Normal Color depth cue single vector: NCDS
                event!(target: "ps1_emulator::GTE", Level::TRACE, "NCDS");
                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;
                self.ncds(sf, lm);
            }
            0x14 => {
                // Color Depth Que: CDP
                event!(target: "ps1_emulator::GTE", Level::TRACE, "CDP");
                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;
                self.cdp(sf, lm);
            }
            0x16 => {
                // Normal color depth cue triple vector: NCDT
                event!(target: "ps1_emulator::GTE", Level::TRACE, "NCDT");
                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;
                self.ncdt(sf, lm);
            }
            0x1B => {
                // Normal Color Col. single vector: NCCS
                event!(target: "ps1_emulator::GTE", Level::TRACE, "NCSS");
                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;

                self.nccs(sf, lm);
            }
            0x1C => {
                // Color Color: CC
                event!(target: "ps1_emulator::GTE", Level::TRACE, "CC");
                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;
                self.cc(sf, lm);
            }
            0x1E => {
                // Normal Color v0: NCS
                event!(target: "ps1_emulator::GTE", Level::TRACE, "NCS");

                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;
                self.ncs(sf, lm);
            }
            0x20 => {
                // Normal Color v0, v2, v2: NCT
                event!(target: "ps1_emulator::GTE", Level::TRACE, "NCT");

                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;

                self.nct(sf, lm);
            }
            0x28 => {
                // Square of Vector IR: SQR(sf)
                event!(target: "ps1_emulator::GTE", Level::TRACE, "SQR");

                let sf = cmd & 0x80000 > 0;
                //let lm = cmd & 0x400 > 0;
                self.sqr(sf);
            }
            0x29 => {
                // Depth Cue Color Light: DCPL
                event!(target: "ps1_emulator::GTE", Level::TRACE, "DCPL");

                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;
                self.dcpl(sf, lm);
            }
            0x2A => {
                // Depth Cueing Single: DPCT
                event!(target: "ps1_emulator::GTE", Level::TRACE, "DPCT");
                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;

                self.dpct(sf, lm);
            }
            0x2D => {
                // AVSZ3 - Average of three Z values
                event!(target: "ps1_emulator::GTE", Level::TRACE, "AVSZ3");
                self.avsz3();
            }
            0x2E => {
                // AVSZ4 - Average of four Z values
                event!(target: "ps1_emulator::GTE", Level::TRACE, "AVSZ4");
                self.avsz4();
            }
            0x30 => {
                // Perspective Transformation Triple: RTPT
                event!(target: "ps1_emulator::GTE", Level::TRACE, "RTPT");
                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;
                self.rtpt(sf, lm);
            }
            0x3D => {
                // General Purpose interpolation: GPF
                event!(target: "ps1_emulator::GTE", Level::TRACE, "GPF");
                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;
                self.gpf(sf, lm);
            }
            0x3E => {
                // General Purpose interpolation with base: GPL
                event!(target: "ps1_emulator::GTE", Level::TRACE, "GPL");
                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;
                self.gpl(sf, lm);
            }
            0x3F => {
                // Normal Color Col. triple vector: NCCT
                event!(target: "ps1_emulator::GTE", Level::TRACE, "NCCT");
                let sf = cmd & 0x80000 > 0;
                let lm = cmd & 0x400 > 0;

                self.ncct(sf, lm);
            }
            _ => {
                event!(target: "ps1_emulator::GTE", Level::ERROR, "No GTE command for 0x{:02X}", cmd & 0x3F);
            }
        }
    }

    fn write_ir3(&mut self, val: i32, lm: bool) {
        let lo = if lm { 0 } else { -0x8000 };
        if val > 0x7FFF {
            self.intermediates[3] = 0x7FFF;
            //self.flag.set_ir_flag(3);
        } else if val < lo {
            self.intermediates[3] = lo as i16;
            if val < -0x8000 {
                //self.flag.set_ir_flag(3);
            }
        } else {
            self.intermediates[3] = val as i16;
        }
    }
    fn write_ir(&mut self, ir: usize, val: i32, lm: bool) {
        let lo = if lm { 0 } else { -0x8000 };
        if val > 0x7FFF {
            self.intermediates[ir] = 0x7FFF;
            self.flag.set_ir_flag(ir);
        } else if val < lo {
            self.intermediates[ir] = lo as i16;
            self.flag.set_ir_flag(ir);
        } else {
            self.intermediates[ir] = val as i16;
        }
    }
    fn write_ir0(&mut self, val: i32) {
        if val > 0x1000 {
            self.intermediates[0] = 0x1000;
            self.flag.set_ir_flag(0);
        } else if val < 0 {
            self.intermediates[0] = 0;
            self.flag.set_ir_flag(0);
        } else {
            self.intermediates[0] = val as i16;
        }
    }

    fn saturate_scxy(&mut self, sxp: i32, syp: i32) -> (i16, i16) {
        let sx2 = if sxp > 0x3FF {
            self.flag.set_scx_flag();
            0x3FF
        } else if sxp < -0x400 {
            self.flag.set_scx_flag();
            -0x400
        } else {
            sxp as i16
        };
        let sy2 = if syp > 0x3FF {
            self.flag.set_scy_flag();
            0x3FF
        } else if syp < -0x400 {
            self.flag.set_scy_flag();
            -0x400
        } else {
            syp as i16
        };

        (sx2, sy2)
    }

    fn scxy_fifo(&mut self, sxp: i16, syp: i16) {
        self.screenxy[0] = self.screenxy[1];
        self.screenxy[1] = self.screenxy[2];
        self.screenxy[2][1] = syp;
        self.screenxy[2][0] = sxp;

        // SXYP is a mirror of SXY2
        self.screenxy[3] = self.screenxy[2];
    }

    fn scz_fifo(&mut self, new_scz: i32) {
        let scz = if !(0..=0xFFFF).contains(&new_scz) {
            self.flag.set_scz_flag();
            new_scz.clamp(0, 0xFFFF) as u16
        } else {
            new_scz as u16
        };

        self.screenz[0] = self.screenz[1];
        self.screenz[1] = self.screenz[2];
        self.screenz[2] = self.screenz[3];
        self.screenz[3] = scz;
    }

    fn rgb_fifo(&mut self) {
        let new_r = self.mac[1] >> 4;
        let new_g = self.mac[2] >> 4;
        let new_b = self.mac[3] >> 4;
        let r = if new_r > 0xFF {
            self.flag.set_color_flag_r();
            0xFF
        } else if new_r < 0 {
            self.flag.set_color_flag_r();
            0
        } else {
            new_r as u32
        };

        let g = if new_g > 0xFF {
            self.flag.set_color_flag_g();
            0xFF
        } else if new_g < 0 {
            self.flag.set_color_flag_g();
            0
        } else {
            new_g as u32
        };

        let b = if new_b > 0xFF {
            self.flag.set_color_flag_b();
            0xFF
        } else if new_b < 0 {
            self.flag.set_color_flag_b();
            0
        } else {
            new_b as u32
        };

        let code = self.rgb & 0xFF000000;
        let new_color = code | (b << 16) | (g << 8) | r;
        self.characteristic_color[0] = self.characteristic_color[1];
        self.characteristic_color[1] = self.characteristic_color[2];
        self.characteristic_color[2] = new_color;
    }

    fn rtps(&mut self, sf: bool, lm: bool) {
        let div = self.perspective_transform(self.v0, sf, lm);
        self.depth_que(div);
    }

    fn rtpt(&mut self, sf: bool, lm: bool) {
        let _ = self.perspective_transform(self.v0, sf, lm);
        let _ = self.perspective_transform(self.v1, sf, lm);
        let div = self.perspective_transform(self.v2, sf, lm);
        self.depth_que(div);
    }

    fn perspective_transform(&mut self, vector: [i16; 3], sf: bool, lm: bool) -> i64 {
        /*
        IR1 = MAC1 = (TRX*1000h + RT11*VX0 + RT12*VY0 + RT13*VZ0) SAR (sf*12)
        IR2 = MAC2 = (TRY*1000h + RT21*VX0 + RT22*VY0 + RT23*VZ0) SAR (sf*12)
        IR3 = MAC3 = (TRZ*1000h + RT31*VX0 + RT32*VY0 + RT33*VZ0) SAR (sf*12)
        SZ3 = MAC3 SAR ((1-sf)*12)                           ;ScreenZ FIFO 0..+FFFFh
        MAC0=(((H*20000h/SZ3)+1)/2)*IR1+OFX, SX2=MAC0/10000h ;ScrX FIFO -400h..+3FFh
        MAC0=(((H*20000h/SZ3)+1)/2)*IR2+OFY, SY2=MAC0/10000h ;ScrY FIFO -400h..+3FFh
        MAC0=(((H*20000h/SZ3)+1)/2)*DQA+DQB, IR0=MAC0/1000h  ;Depth cueing 0..+1000h
        */
        let mut sum1 = self.translation_vec[0] as i64 * 0x1000
            + self.rotation_matrix[0][0] as i64 * vector[0] as i64;
        let mut sum2 = self.translation_vec[1] as i64 * 0x1000
            + self.rotation_matrix[1][0] as i64 * vector[0] as i64;
        let mut sum3 = self.translation_vec[2] as i64 * 0x1000
            + self.rotation_matrix[2][0] as i64 * vector[0] as i64;

        self.flag.mac_flag_check(1, sum1);
        self.flag.mac_flag_check(2, sum2);
        self.flag.mac_flag_check(3, sum3);

        sum1 = (sum1 << 20) >> 20;
        sum2 = (sum2 << 20) >> 20;
        sum3 = (sum3 << 20) >> 20;

        sum1 += self.rotation_matrix[0][1] as i64 * vector[1] as i64;
        sum2 += self.rotation_matrix[1][1] as i64 * vector[1] as i64;
        sum3 += self.rotation_matrix[2][1] as i64 * vector[1] as i64;

        self.flag.mac_flag_check(1, sum1);
        self.flag.mac_flag_check(2, sum2);
        self.flag.mac_flag_check(3, sum3);

        sum1 = (sum1 << 20) >> 20;
        sum2 = (sum2 << 20) >> 20;
        sum3 = (sum3 << 20) >> 20;

        sum1 += self.rotation_matrix[0][2] as i64 * vector[2] as i64;
        sum2 += self.rotation_matrix[1][2] as i64 * vector[2] as i64;
        sum3 += self.rotation_matrix[2][2] as i64 * vector[2] as i64;

        self.flag.mac_flag_check(1, sum1);
        self.flag.mac_flag_check(2, sum2);
        self.flag.mac_flag_check(3, sum3);

        self.mac[1] = (sum1 >> (sf as u8 * 12)) as i32;
        self.mac[2] = (sum2 >> (sf as u8 * 12)) as i32;
        self.mac[3] = (sum3 >> (sf as u8 * 12)) as i32;

        // let sum1 = self.translation_vec[0] as i64 * 0x1000
        //     + self.rotation_matrix[0][0] as i64 * vector[0] as i64
        //     + self.rotation_matrix[0][1] as i64 * vector[1] as i64
        //     + self.rotation_matrix[0][2] as i64 * vector[2] as i64;
        // self.flag.mac_flag_check(1, sum1);
        // self.mac[1] = (sum1 >> (sf as u8 * 12)) as i32;
        //
        // let sum2 = self.translation_vec[1] as i64 * 0x1000
        //     + self.rotation_matrix[1][0] as i64 * vector[0] as i64
        //     + self.rotation_matrix[1][1] as i64 * vector[1] as i64
        //     + self.rotation_matrix[1][2] as i64 * vector[2] as i64;
        // self.flag.mac_flag_check(2, sum2);
        // self.mac[2] = (sum2 >> (sf as u8 * 12)) as i32;
        //
        // let sum3 = self.translation_vec[2] as i64 * 0x1000
        //     + self.rotation_matrix[2][0] as i64 * vector[0] as i64
        //     + self.rotation_matrix[2][1] as i64 * vector[1] as i64
        //     + self.rotation_matrix[2][2] as i64 * vector[2] as i64;
        // self.flag.mac_flag_check(3, sum3);
        // self.mac[3] = (sum3 >> (sf as u8 * 12)) as i32;

        // IR1, IR2, IR3
        self.write_ir(1, self.mac[1], lm);
        self.write_ir(2, self.mac[2], lm);
        if !(-0x8000..=0x7FFF).contains(&(sum3 >> 12)) {
            self.flag.set_ir_flag(3);
        }
        self.write_ir3(self.mac[3], lm);

        // SZ3
        self.scz_fifo((sum3 >> 12) as i32 /*self.mac[3] >> (!sf as u8 * 12)*/);

        // division
        let division_result = if (self.h as u32) < (self.screenz[3] as u32) * 2 {
            let z = self.screenz[3].leading_zeros();
            let mut n = (self.h as u32) << z;
            let mut d = (self.screenz[3] as u32) << z;
            let u = UNR_LKUP[((d - 0x7FC0) >> 7) as usize] + 0x101;
            d = (0x2000080 - (d * u)) >> 8;
            d = (0x0000080 + (d * u)) >> 8;
            let val = ((n as u64 * d as u64) + 0x8000) >> 16;
            n = min(0x1FFFF, val as u32);
            n
        } else {
            self.flag.divide_overflow();
            0x1FFFF
        };

        // MAC0 SCX
        let scx =
            division_result as i64 * self.intermediates[1] as i64 + self.screen_offset[0] as i64;
        self.flag.mac0_flag_check(scx);
        self.mac[0] = scx as i32;
        let sxp = scx >> 16;

        // MAC0 SCY
        let scy =
            division_result as i64 * self.intermediates[2] as i64 + self.screen_offset[1] as i64;
        self.flag.mac0_flag_check(scy);
        self.mac[0] = scy as i32;
        let syp = scy >> 16;

        let (sx2, sy2) = self.saturate_scxy(sxp as i32, syp as i32);
        self.scxy_fifo(sx2, sy2);

        division_result as i64
    }

    fn depth_que(&mut self, division_result: i64) {
        let depth = division_result * self.depth_cue_a as i64 + self.depth_cue_b as i64;
        self.flag.mac0_flag_check(depth);
        self.mac[0] = depth as i32;
        self.write_ir0((depth >> 12) as i32);
    }

    fn nclip(&mut self) {
        // MAC0 =   SX0*SY1 + SX1*SY2 + SX2*SY0 - SX0*SY2 - SX1*SY0 - SX2*SY1
        let sum = self.screenxy[0][0] as i64 * self.screenxy[1][1] as i64
            + self.screenxy[1][0] as i64 * self.screenxy[2][1] as i64
            + self.screenxy[2][0] as i64 * self.screenxy[0][1] as i64
            - self.screenxy[0][0] as i64 * self.screenxy[2][1] as i64
            - self.screenxy[1][0] as i64 * self.screenxy[0][1] as i64
            - self.screenxy[2][0] as i64 * self.screenxy[1][1] as i64;
        self.flag.mac0_flag_check(sum);
        self.mac[0] = sum as i32;
    }

    fn avsz3(&mut self) {
        // MAC0 = ZSF3*(SZ1+SZ2+SZ3)
        // OTZ  = MAC0/1000h
        let sum = self.screenz[1] as i64 + self.screenz[2] as i64 + self.screenz[3] as i64;
        let avg = self.zsf3 as i64 * sum;
        self.flag.mac0_flag_check(avg);
        self.mac[0] = avg as i32;
        let otz = avg / 0x1000;
        if otz > 0xFFFF {
            self.otz = 0xFFFF;
            self.flag.set_scz_flag();
        } else if otz < 0 {
            self.otz = 0;
            self.flag.set_scz_flag();
        } else {
            self.otz = otz as u16;
        }
    }

    fn avsz4(&mut self) {
        // MAC0 = ZSF4*(SZ0+SZ1+SZ2+SZ3)
        // OTZ  = MAC0/1000h
        let sum = self.screenz[0] as i64
            + self.screenz[1] as i64
            + self.screenz[2] as i64
            + self.screenz[3] as i64;
        let avg = self.zsf4 as i64 * sum;
        self.flag.mac0_flag_check(avg);
        self.mac[0] = avg as i32;
        let otz = avg / 0x1000;
        if otz > 0xFFFF {
            self.otz = 0xFFFF;
            self.flag.set_scz_flag();
        } else if otz < 0 {
            self.otz = 0;
            self.flag.set_scz_flag();
        } else {
            self.otz = otz as u16;
        }
    }

    fn mvmva(&mut self, mv: MV, tv: TV, vector: [i16; 3], sf: bool, lm: bool) {
        //   MAC1 = (Tx1*1000h + Mx11*Vx1 + Mx12*Vx2 + Mx13*Vx3) SAR (sf*12)
        //   MAC2 = (Tx2*1000h + Mx21*Vx1 + Mx22*Vx2 + Mx23*Vx3) SAR (sf*12)
        //   MAC3 = (Tx3*1000h + Mx31*Vx1 + Mx32*Vx2 + Mx33*Vx3) SAR (sf*12)
        //   [IR1,IR2,IR3] = [MAC1,MAC2,MAC3]

        // translation vector
        let translation_result = match tv {
            TV::Translation => [
                self.translation_vec[0] as i64 * 0x1000,
                self.translation_vec[1] as i64 * 0x1000,
                self.translation_vec[2] as i64 * 0x1000,
            ],
            TV::FarColor => [
                self.far_color[0] as i64 * 0x1000,
                self.far_color[1] as i64 * 0x1000,
                self.far_color[2] as i64 * 0x1000,
            ],
            TV::BackgroundColor => [
                self.background_color[0] as i64 * 0x1000,
                self.background_color[1] as i64 * 0x1000,
                self.background_color[2] as i64 * 0x1000,
            ],
            TV::None => [0, 0, 0],
        };

        // matrix
        let matrix = match mv {
            MV::Rotation => self.rotation_matrix,
            MV::Light => self.light_matrix,
            MV::Color => self.light_color_matrix,
            MV::Reserved => [
                [
                    -0x10 * (self.rgb & 0xFF) as i16,
                    0x10 * (self.rgb & 0xFF) as i16,
                    self.intermediates[0],
                ],
                [
                    self.rotation_matrix[0][2],
                    self.rotation_matrix[0][2],
                    self.rotation_matrix[0][2],
                ],
                [
                    self.rotation_matrix[1][1],
                    self.rotation_matrix[1][1],
                    self.rotation_matrix[1][1],
                ],
            ],
        };

        let mut sum1 = translation_result[0] + matrix[0][0] as i64 * vector[0] as i64;
        let mut sum2 = translation_result[1] + matrix[1][0] as i64 * vector[0] as i64;
        let mut sum3 = translation_result[2] + matrix[2][0] as i64 * vector[0] as i64;
        let mut adj1 = 0;
        let mut adj2 = 0;
        let mut adj3 = 0;

        self.flag.mac_flag_check(1, sum1);
        self.flag.mac_flag_check(2, sum2);
        self.flag.mac_flag_check(3, sum3);

        sum1 = (sum1 << 20) >> 20;
        sum2 = (sum2 << 20) >> 20;
        sum3 = (sum3 << 20) >> 20;

        if tv == TV::FarColor {
            self.write_ir(1, (sum1 >> (sf as u8 * 12)) as i32, lm);
            self.write_ir(2, (sum2 >> (sf as u8 * 12)) as i32, lm);
            self.write_ir(3, (sum3 >> (sf as u8 * 12)) as i32, lm);
            adj1 = sum1;
            adj2 = sum2;
            adj3 = sum3;
            sum1 = 0;
            sum2 = 0;
            sum3 = 0;
        }

        sum1 += matrix[0][1] as i64 * vector[1] as i64;
        sum2 += matrix[1][1] as i64 * vector[1] as i64;
        sum3 += matrix[2][1] as i64 * vector[1] as i64;

        self.flag.mac_flag_check(1, sum1);
        self.flag.mac_flag_check(2, sum2);
        self.flag.mac_flag_check(3, sum3);

        sum1 = (sum1 << 20) >> 20;
        sum2 = (sum2 << 20) >> 20;
        sum3 = (sum3 << 20) >> 20;

        sum1 += matrix[0][2] as i64 * vector[2] as i64;
        sum2 += matrix[1][2] as i64 * vector[2] as i64;
        sum3 += matrix[2][2] as i64 * vector[2] as i64;

        if tv == TV::FarColor {
            self.flag.mac_flag_check(1, adj1);
            self.flag.mac_flag_check(2, adj2);
            self.flag.mac_flag_check(3, adj3);
        } else {
            self.flag.mac_flag_check(1, sum1);
            self.flag.mac_flag_check(2, sum2);
            self.flag.mac_flag_check(3, sum3);
        }
        sum1 = (sum1 << 20) >> 20;
        sum2 = (sum2 << 20) >> 20;
        sum3 = (sum3 << 20) >> 20;

        self.mac[1] = (sum1 >> (sf as u8 * 12)) as i32;
        self.mac[2] = (sum2 >> (sf as u8 * 12)) as i32;
        self.mac[3] = (sum3 >> (sf as u8 * 12)) as i32;
        self.write_ir(1, self.mac[1], lm);
        self.write_ir(2, self.mac[2], lm);
        self.write_ir(3, self.mac[3], lm);
    }

    fn sqr(&mut self, sf: bool) {
        //  [MAC1,MAC2,MAC3] = [IR1*IR1,IR2*IR2,IR3*IR3] SHR (sf*12)
        //  [IR1,IR2,IR3]    = [MAC1,MAC2,MAC3]    ;IR1,IR2,IR3 saturated to max 7FFFh
        let result1 = self.intermediates[1] as i64 * self.intermediates[1] as i64;
        let result2 = self.intermediates[2] as i64 * self.intermediates[2] as i64;
        let result3 = self.intermediates[3] as i64 * self.intermediates[3] as i64;

        self.flag.mac_flag_check(1, result1);
        self.flag.mac_flag_check(2, result2);
        self.flag.mac_flag_check(3, result3);

        self.mac[1] = (result1 >> (sf as u8 * 12)) as i32;
        self.mac[2] = (result2 >> (sf as u8 * 12)) as i32;
        self.mac[3] = (result3 >> (sf as u8 * 12)) as i32;

        // mac is always positive so lm value does not matter
        self.write_ir(1, self.mac[1], true);
        self.write_ir(2, self.mac[2], true);
        self.write_ir(3, self.mac[3], true);
    }

    fn op(&mut self, sf: bool, lm: bool) {
        // [MAC1,MAC2,MAC3] = [IR3*D2-IR2*D3, IR1*D3-IR3*D1, IR2*D1-IR1*D2] SAR (sf*12)
        let sum1 = self.intermediates[3] as i64 * self.rotation_matrix[1][1] as i64
            - self.intermediates[2] as i64 * self.rotation_matrix[2][2] as i64;

        let sum2 = self.intermediates[1] as i64 * self.rotation_matrix[2][2] as i64
            - self.intermediates[3] as i64 * self.rotation_matrix[0][0] as i64;

        let sum3 = self.intermediates[2] as i64 * self.rotation_matrix[0][0] as i64
            - self.intermediates[1] as i64 * self.rotation_matrix[1][1] as i64;

        self.flag.mac_flag_check(1, sum1);
        self.mac[1] = (sum1 >> (sf as u8 * 12)) as i32;

        self.flag.mac_flag_check(2, sum2);
        self.mac[2] = (sum2 >> (sf as u8 * 12)) as i32;

        self.flag.mac_flag_check(3, sum3);
        self.mac[3] = (sum3 >> (sf as u8 * 12)) as i32;

        // IR1, IR2, IR3
        self.write_ir(1, self.mac[1], lm);
        self.write_ir(2, self.mac[2], lm);
        self.write_ir(3, self.mac[3], lm);
    }

    fn normal_color(&mut self, vector: [i16; 3], sf: bool, lm: bool) {
        //   [IR1,IR2,IR3] = [MAC1,MAC2,MAC3] = (LLM*V0) SAR (sf*12)
        //   [IR1,IR2,IR3] = [MAC1,MAC2,MAC3] = (BK*1000h + LCM*IR) SAR (sf*12)
        self.mvmva(MV::Light, TV::None, vector, sf, lm);
        let irs = [
            self.intermediates[1],
            self.intermediates[2],
            self.intermediates[3],
        ];
        self.mvmva(MV::Color, TV::BackgroundColor, irs, sf, lm);
    }

    fn normal_color_col(&mut self, sf: bool, lm: bool) {
        // [MAC1,MAC2,MAC3] = [R*IR1,G*IR2,B*IR3] SHL 4
        let r = (self.rgb & 0xFF) as i32;
        let g = ((self.rgb >> 8) & 0xFF) as i32;
        let b = ((self.rgb >> 16) & 0xFF) as i32;
        self.mac[1] = (r * self.intermediates[1] as i32) << 4;
        self.mac[2] = (g * self.intermediates[2] as i32) << 4;
        self.mac[3] = (b * self.intermediates[3] as i32) << 4;

        // [MAC1,MAC2,MAC3] = [MAC1,MAC2,MAC3] SAR (sf*12)
        self.mac[1] >>= sf as u8 * 12;
        self.mac[2] >>= sf as u8 * 12;
        self.mac[3] >>= sf as u8 * 12;

        self.write_ir(1, self.mac[1], lm);
        self.write_ir(2, self.mac[2], lm);
        self.write_ir(3, self.mac[3], lm);
    }

    fn ncs(&mut self, sf: bool, lm: bool) {
        self.normal_color(self.v0, sf, lm);
        self.rgb_fifo();
    }

    fn nct(&mut self, sf: bool, lm: bool) {
        self.normal_color(self.v0, sf, lm);
        self.rgb_fifo();

        self.normal_color(self.v1, sf, lm);
        self.rgb_fifo();

        self.normal_color(self.v2, sf, lm);
        self.rgb_fifo();
    }

    fn nccs(&mut self, sf: bool, lm: bool) {
        self.normal_color(self.v0, sf, lm);
        self.normal_color_col(sf, lm);
        self.rgb_fifo();
    }

    fn ncct(&mut self, sf: bool, lm: bool) {
        self.normal_color(self.v0, sf, lm);
        self.normal_color_col(sf, lm);
        self.rgb_fifo();
        self.normal_color(self.v1, sf, lm);
        self.normal_color_col(sf, lm);
        self.rgb_fifo();
        self.normal_color(self.v2, sf, lm);
        self.normal_color_col(sf, lm);
        self.rgb_fifo();
    }

    fn ncds(&mut self, sf: bool, lm: bool) {
        self.normal_color(self.v0, sf, lm);
        self.dcpl(sf, lm);
    }

    fn ncdt(&mut self, sf: bool, lm: bool) {
        self.normal_color(self.v0, sf, lm);
        self.dcpl(sf, lm);
        self.normal_color(self.v1, sf, lm);
        self.dcpl(sf, lm);
        self.normal_color(self.v2, sf, lm);
        self.dcpl(sf, lm);
    }

    fn cc(&mut self, sf: bool, lm: bool) {
        // [IR1,IR2,IR3] = [MAC1,MAC2,MAC3] = (BK*1000h + LCM*IR) SAR (sf*12)
        let irs = [
            self.intermediates[1],
            self.intermediates[2],
            self.intermediates[3],
        ];
        self.mvmva(MV::Color, TV::BackgroundColor, irs, sf, lm);

        // [MAC1,MAC2,MAC3] = [R*IR1,G*IR2,B*IR3] SHL 4
        let r = (self.rgb & 0xFF) as i32;
        let g = ((self.rgb >> 8) & 0xFF) as i32;
        let b = ((self.rgb >> 16) & 0xFF) as i32;

        self.mac[1] = (r * self.intermediates[1] as i32) << 4;
        self.mac[2] = (g * self.intermediates[2] as i32) << 4;
        self.mac[3] = (b * self.intermediates[3] as i32) << 4;

        // [MAC1,MAC2,MAC3] = [MAC1,MAC2,MAC3] SAR (sf*12)
        self.mac[1] >>= sf as u8 * 12;
        self.mac[2] >>= sf as u8 * 12;
        self.mac[3] >>= sf as u8 * 12;

        self.write_ir(1, self.mac[1], lm);
        self.write_ir(2, self.mac[2], lm);
        self.write_ir(3, self.mac[3], lm);

        self.rgb_fifo();
    }

    fn cdp(&mut self, sf: bool, lm: bool) {
        // [IR1,IR2,IR3] = [MAC1,MAC2,MAC3] = (BK*1000h + LCM*IR) SAR (sf*12)
        let irs = [
            self.intermediates[1],
            self.intermediates[2],
            self.intermediates[3],
        ];
        self.mvmva(MV::Color, TV::BackgroundColor, irs, sf, lm);

        // [MAC1,MAC2,MAC3] = [R*IR1,G*IR2,B*IR3] SHL 4
        let r = (self.rgb & 0xFF) as i32;
        let g = ((self.rgb >> 8) & 0xFF) as i32;
        let b = ((self.rgb >> 16) & 0xFF) as i32;

        self.mac[1] = (r * self.intermediates[1] as i32) << 4;
        self.mac[2] = (g * self.intermediates[2] as i32) << 4;
        self.mac[3] = (b * self.intermediates[3] as i32) << 4;

        // [MAC1,MAC2,MAC3] = MAC+(FC-MAC)*IR0
        // [MAC1,MAC2,MAC3] = [MAC1,MAC2,MAC3] SAR (sf*12)
        self.interpolate_macs(sf, lm);

        self.rgb_fifo();
    }

    fn dcpl(&mut self, sf: bool, lm: bool) {
        // [MAC1,MAC2,MAC3] = [R*IR1,G*IR2,B*IR3] SHL 4
        let r = (self.rgb & 0xFF) as i32;
        let g = ((self.rgb >> 8) & 0xFF) as i32;
        let b = ((self.rgb >> 16) & 0xFF) as i32;

        self.mac[1] = (r * self.intermediates[1] as i32) << 4;
        self.mac[2] = (g * self.intermediates[2] as i32) << 4;
        self.mac[3] = (b * self.intermediates[3] as i32) << 4;

        // [MAC1,MAC2,MAC3] = MAC+(FC-MAC)*IR0
        // [MAC1,MAC2,MAC3] = [MAC1,MAC2,MAC3] SAR (sf*12)
        self.interpolate_macs(sf, lm);

        self.rgb_fifo();
    }

    fn dpcs(&mut self, sf: bool, lm: bool) {
        // [MAC1,MAC2,MAC3] = [R,G,B] SHL 16
        let r = (self.rgb & 0xFF) as i32;
        let g = ((self.rgb >> 8) & 0xFF) as i32;
        let b = ((self.rgb >> 16) & 0xFF) as i32;
        self.mac[1] = r << 16;
        self.mac[2] = g << 16;
        self.mac[3] = b << 16;

        // [MAC1,MAC2,MAC3] = MAC+(FC-MAC)*IR0
        // [MAC1,MAC2,MAC3] = [MAC1,MAC2,MAC3] SAR (sf*12)
        self.interpolate_macs(sf, lm);

        self.rgb_fifo()
    }

    fn dpct(&mut self, sf: bool, lm: bool) {
        for _ in 0..3 {
            // [MAC1,MAC2,MAC3] = [R,G,B] SHL 16
            let r = (self.characteristic_color[0] & 0xFF) as i32;
            let g = ((self.characteristic_color[0] >> 8) & 0xFF) as i32;
            let b = ((self.characteristic_color[0] >> 16) & 0xFF) as i32;
            self.mac[1] = r << 16;
            self.mac[2] = g << 16;
            self.mac[3] = b << 16;

            // [MAC1,MAC2,MAC3] = MAC+(FC-MAC)*IR0
            // [MAC1,MAC2,MAC3] = [MAC1,MAC2,MAC3] SAR (sf*12)
            self.interpolate_macs(sf, lm);

            self.rgb_fifo()
        }
    }

    fn intpl(&mut self, sf: bool, lm: bool) {
        // [MAC1,MAC2,MAC3] = [IR1,IR2,IR3] SHL 12
        self.mac[1] = (self.intermediates[1] as i32) << 12;
        self.mac[2] = (self.intermediates[2] as i32) << 12;
        self.mac[3] = (self.intermediates[3] as i32) << 12;

        // [MAC1,MAC2,MAC3] = MAC+(FC-MAC)*IR0
        // [MAC1,MAC2,MAC3] = [MAC1,MAC2,MAC3] SAR (sf*12)
        self.interpolate_macs(sf, lm);

        self.write_ir(1, self.mac[1], lm);
        self.write_ir(2, self.mac[2], lm);
        self.write_ir(3, self.mac[3], lm);

        self.rgb_fifo()
    }

    fn interpolate_macs(&mut self, sf: bool, lm: bool) {
        // [MAC1,MAC2,MAC3] = MAC+(FC-MAC)*IR0
        let diff1 = ((self.far_color[0] as i64) << 12) - self.mac[1] as i64;
        let diff2 = ((self.far_color[1] as i64) << 12) - self.mac[2] as i64;
        let diff3 = ((self.far_color[2] as i64) << 12) - self.mac[3] as i64;

        self.flag.mac_flag_check(1, diff1);
        self.flag.mac_flag_check(2, diff2);
        self.flag.mac_flag_check(3, diff3);

        self.write_ir(1, (diff1 >> (sf as u8 * 12)) as i32, false);
        self.write_ir(2, (diff2 >> (sf as u8 * 12)) as i32, false);
        self.write_ir(3, (diff3 >> (sf as u8 * 12)) as i32, false);

        let sum1 = self.mac[1] as i64 + self.intermediates[1] as i64 * self.intermediates[0] as i64;
        let sum2 = self.mac[2] as i64 + self.intermediates[2] as i64 * self.intermediates[0] as i64;
        let sum3 = self.mac[3] as i64 + self.intermediates[3] as i64 * self.intermediates[0] as i64;

        // [MAC1,MAC2,MAC3] = [MAC1,MAC2,MAC3] SAR (sf*12)
        self.mac[1] = (sum1 >> (sf as u8 * 12)) as i32;
        self.mac[2] = (sum2 >> (sf as u8 * 12)) as i32;
        self.mac[3] = (sum3 >> (sf as u8 * 12)) as i32;
        self.flag.mac_flag_check(1, sum1);
        self.flag.mac_flag_check(2, sum2);
        self.flag.mac_flag_check(3, sum3);

        self.write_ir(1, self.mac[1], lm);
        self.write_ir(2, self.mac[2], lm);
        self.write_ir(3, self.mac[3], lm);
    }
    fn gpf(&mut self, sf: bool, lm: bool) {
        self.mac[1] = 0;
        self.mac[2] = 0;
        self.mac[3] = 0;

        let sum1 = self.intermediates[1] as i64 * self.intermediates[0] as i64;
        let sum2 = self.intermediates[2] as i64 * self.intermediates[0] as i64;
        let sum3 = self.intermediates[3] as i64 * self.intermediates[0] as i64;

        self.flag.mac_flag_check(1, sum1);
        self.flag.mac_flag_check(2, sum2);
        self.flag.mac_flag_check(3, sum3);

        self.mac[1] = (sum1 >> (sf as u8 * 12)) as i32;
        self.mac[2] = (sum2 >> (sf as u8 * 12)) as i32;
        self.mac[3] = (sum3 >> (sf as u8 * 12)) as i32;

        self.write_ir(1, self.mac[1], lm);
        self.write_ir(2, self.mac[2], lm);
        self.write_ir(3, self.mac[3], lm);

        self.rgb_fifo();
    }

    fn gpl(&mut self, sf: bool, lm: bool) {
        let shift1 = (self.mac[1] as i64) << (sf as u8 * 12);
        let shift2 = (self.mac[2] as i64) << (sf as u8 * 12);
        let shift3 = (self.mac[3] as i64) << (sf as u8 * 12);

        self.flag.mac_flag_check(1, shift1);
        self.flag.mac_flag_check(2, shift2);
        self.flag.mac_flag_check(3, shift3);

        let sum1 = self.intermediates[1] as i64 * self.intermediates[0] as i64 + shift1;
        let sum2 = self.intermediates[2] as i64 * self.intermediates[0] as i64 + shift2;
        let sum3 = self.intermediates[3] as i64 * self.intermediates[0] as i64 + shift3;

        self.flag.mac_flag_check(1, sum1);
        self.flag.mac_flag_check(2, sum2);
        self.flag.mac_flag_check(3, sum3);

        self.mac[1] = (sum1 >> (sf as u8 * 12)) as i32;
        self.mac[2] = (sum2 >> (sf as u8 * 12)) as i32;
        self.mac[3] = (sum3 >> (sf as u8 * 12)) as i32;

        self.write_ir(1, self.mac[1], lm);
        self.write_ir(2, self.mac[2], lm);
        self.write_ir(3, self.mac[3], lm);

        self.rgb_fifo();
    }
}

enum MV {
    Rotation,
    Light,
    Color,
    Reserved,
}

#[derive(Debug, PartialEq)]
enum TV {
    Translation,
    BackgroundColor,
    FarColor,
    None,
}

#[derive(Copy, Clone, PartialEq)]
struct Flag(u32);

impl Flag {
    fn mac_flag_check(&mut self, mac_id: u32, proposed_mac: i64) {
        // 2^43 = 8,796,093,022,208
        if proposed_mac >= 8_796_093_022_208 {
            self.0 |= 0x10000000 << (3 - mac_id);
        }

        if proposed_mac < -8_796_093_022_208 {
            self.0 |= 0x2000000 << (3 - mac_id);
        }
    }

    fn mac0_flag_check(&mut self, proposed_mac0: i64) {
        if proposed_mac0.is_positive() && proposed_mac0 >= 2_147_483_648 {
            self.0 |= 0x10000;
        }

        if proposed_mac0.is_negative() && proposed_mac0 < -2_147_483_648 {
            self.0 |= 0x8000;
        }
    }

    fn set_ir_flag(&mut self, ir: usize) {
        if ir > 0 {
            self.0 |= 0x400000 << (3 - ir);
        } else {
            self.0 |= 0x1000;
        }
    }

    fn set_scz_flag(&mut self) {
        self.0 |= 0x40000;
    }

    fn set_scx_flag(&mut self) {
        self.0 |= 0x4000;
    }

    fn set_scy_flag(&mut self) {
        self.0 |= 0x2000;
    }

    fn divide_overflow(&mut self) {
        self.0 |= 0x20000;
    }

    fn set_color_flag_r(&mut self) {
        self.0 |= 0x200000;
    }

    fn set_color_flag_g(&mut self) {
        self.0 |= 0x100000;
    }

    fn set_color_flag_b(&mut self) {
        self.0 |= 0x80000;
    }
}

const UNR_LKUP: [u32; 257] = [
    0xFF, 0xFD, 0xFB, 0xF9, 0xF7, 0xF5, 0xF3, 0xF1, 0xEF, 0xEE, 0xEC, 0xEA, 0xE8, 0xE6, 0xE4, 0xE3,
    0xE1, 0xDF, 0xDD, 0xDC, 0xDA, 0xD8, 0xD6, 0xD5, 0xD3, 0xD1, 0xD0, 0xCE, 0xCD, 0xCB, 0xC9, 0xC8,
    0xC6, 0xC5, 0xC3, 0xC1, 0xC0, 0xBE, 0xBD, 0xBB, 0xBA, 0xB8, 0xB7, 0xB5, 0xB4, 0xB2, 0xB1, 0xB0,
    0xAE, 0xAD, 0xAB, 0xAA, 0xA9, 0xA7, 0xA6, 0xA4, 0xA3, 0xA2, 0xA0, 0x9F, 0x9E, 0x9C, 0x9B, 0x9A,
    0x99, 0x97, 0x96, 0x95, 0x94, 0x92, 0x91, 0x90, 0x8F, 0x8D, 0x8C, 0x8B, 0x8A, 0x89, 0x87, 0x86,
    0x85, 0x84, 0x83, 0x82, 0x81, 0x7F, 0x7E, 0x7D, 0x7C, 0x7B, 0x7A, 0x79, 0x78, 0x77, 0x75, 0x74,
    0x73, 0x72, 0x71, 0x70, 0x6F, 0x6E, 0x6D, 0x6C, 0x6B, 0x6A, 0x69, 0x68, 0x67, 0x66, 0x65, 0x64,
    0x63, 0x62, 0x61, 0x60, 0x5F, 0x5E, 0x5D, 0x5D, 0x5C, 0x5B, 0x5A, 0x59, 0x58, 0x57, 0x56, 0x55,
    0x54, 0x53, 0x53, 0x52, 0x51, 0x50, 0x4F, 0x4E, 0x4D, 0x4D, 0x4C, 0x4B, 0x4A, 0x49, 0x48, 0x48,
    0x47, 0x46, 0x45, 0x44, 0x43, 0x43, 0x42, 0x41, 0x40, 0x3F, 0x3F, 0x3E, 0x3D, 0x3C, 0x3C, 0x3B,
    0x3A, 0x39, 0x39, 0x38, 0x37, 0x36, 0x36, 0x35, 0x34, 0x33, 0x33, 0x32, 0x31, 0x31, 0x30, 0x2F,
    0x2E, 0x2E, 0x2D, 0x2C, 0x2C, 0x2B, 0x2A, 0x2A, 0x29, 0x28, 0x28, 0x27, 0x26, 0x26, 0x25, 0x24,
    0x24, 0x23, 0x22, 0x22, 0x21, 0x20, 0x20, 0x1F, 0x1E, 0x1E, 0x1D, 0x1D, 0x1C, 0x1B, 0x1B, 0x1A,
    0x19, 0x19, 0x18, 0x18, 0x17, 0x16, 0x16, 0x15, 0x15, 0x14, 0x14, 0x13, 0x12, 0x12, 0x11, 0x11,
    0x10, 0x0F, 0x0F, 0x0E, 0x0E, 0x0D, 0x0D, 0x0C, 0x0C, 0x0B, 0x0A, 0x0A, 0x09, 0x09, 0x08, 0x08,
    0x07, 0x07, 0x06, 0x06, 0x05, 0x05, 0x04, 0x04, 0x03, 0x03, 0x02, 0x02, 0x01, 0x01, 0x00, 0x00,
    0x00,
];

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
    flag: u32,
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
            flag: 0,
        }
    }

    pub fn control_reg_read(&self, reg: u32) -> u32 {
        if self.enabled {
            event!(target: "ps1_emulator::GTE", Level::TRACE, "Control read reg: {reg}");
            match reg {
                0 => {
                    ((self.rotation_matrix[0][0] as u32) << 16)
                        + (self.rotation_matrix[0][1] as u32 & 0xFFFF)
                }
                1 => {
                    ((self.rotation_matrix[0][2] as u32) << 16)
                        + (self.rotation_matrix[1][0] as u32 & 0xFFFF)
                }
                2 => {
                    ((self.rotation_matrix[1][1] as u32) << 16)
                        + (self.rotation_matrix[1][2] as u32 & 0xFFFF)
                }
                3 => {
                    ((self.rotation_matrix[2][0] as u32) << 16)
                        + (self.rotation_matrix[2][1] as u32 & 0xFFFF)
                }
                4 => (self.rotation_matrix[2][2] as i32) as u32,
                5 => self.translation_vec[0] as u32,
                6 => self.translation_vec[1] as u32,
                7 => self.translation_vec[2] as u32,
                8 => {
                    ((self.light_matrix[0][0] as u32) << 16)
                        + (self.light_matrix[0][1] as u32 & 0xFFFF)
                }
                9 => {
                    ((self.light_matrix[0][2] as u32) << 16)
                        + (self.light_matrix[1][0] as u32 & 0xFFFF)
                }
                10 => {
                    ((self.light_matrix[1][1] as u32) << 16)
                        + (self.light_matrix[1][2] as u32 & 0xFFFF)
                }
                11 => {
                    ((self.light_matrix[2][0] as u32) << 16)
                        + (self.light_matrix[2][1] as u32 & 0xFFFF)
                }
                12 => (self.light_matrix[2][2] as i32) as u32,
                13 => self.background_color[0] as u32,
                14 => self.background_color[1] as u32,
                15 => self.background_color[2] as u32,
                16 => {
                    ((self.light_matrix[0][0] as u32) << 16)
                        + (self.light_matrix[0][1] as u32 & 0xFFFF)
                }
                17 => {
                    ((self.light_matrix[0][2] as u32) << 16)
                        + (self.light_matrix[1][0] as u32 & 0xFFFF)
                }
                18 => {
                    ((self.light_matrix[1][1] as u32) << 16)
                        + (self.light_matrix[1][2] as u32 & 0xFFFF)
                }
                19 => {
                    ((self.light_matrix[2][0] as u32) << 16)
                        + (self.light_matrix[2][1] as u32 & 0xFFFF)
                }
                20 => (self.light_matrix[2][2] as i32) as u32,
                21 => self.far_color[0] as u32,
                22 => self.far_color[1] as u32,
                23 => self.far_color[2] as u32,
                24 => self.screen_offset[0] as u32,
                25 => self.screen_offset[1] as u32,
                26 => (self.h as i32) as u32,
                27 => self.depth_cue_a as u32,
                28 => self.depth_cue_b as u32,
                29 => self.zsf3 as u32,
                30 => self.zsf4 as u32,
                31 => self.flag,
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
                    self.rotation_matrix[0][1] = (val & 0xFFFF) as i16;
                    self.rotation_matrix[0][0] = (val >> 16) as i16;
                }
                1 => {
                    self.rotation_matrix[1][0] = (val & 0xFFFF) as i16;
                    self.rotation_matrix[0][2] = (val >> 16) as i16;
                }
                2 => {
                    self.rotation_matrix[1][2] = (val & 0xFFFF) as i16;
                    self.rotation_matrix[1][1] = (val >> 16) as i16;
                }
                3 => {
                    self.rotation_matrix[2][1] = (val & 0xFFFF) as i16;
                    self.rotation_matrix[2][0] = (val >> 16) as i16;
                }
                4 => self.rotation_matrix[2][2] = (val & 0xFFFF) as i16,
                5 => self.translation_vec[0] = val as i32,
                6 => self.translation_vec[1] = val as i32,
                7 => self.translation_vec[2] = val as i32,
                8 => {
                    self.light_matrix[0][1] = (val & 0xFFFF) as i16;
                    self.light_matrix[0][0] = (val >> 16) as i16;
                }
                9 => {
                    self.light_matrix[1][0] = (val & 0xFFFF) as i16;
                    self.light_matrix[0][2] = (val >> 16) as i16;
                }
                10 => {
                    self.light_matrix[1][2] = (val & 0xFFFF) as i16;
                    self.light_matrix[1][1] = (val >> 16) as i16;
                }
                11 => {
                    self.light_matrix[2][1] = (val & 0xFFFF) as i16;
                    self.light_matrix[2][0] = (val >> 16) as i16;
                }
                12 => self.light_matrix[2][2] = (val & 0xFFFF) as i16,
                13 => self.background_color[0] = val as i32,
                14 => self.background_color[1] = val as i32,
                15 => self.background_color[2] = val as i32,
                16 => {
                    self.light_color_matrix[0][1] = (val & 0xFFFF) as i16;
                    self.light_color_matrix[0][0] = (val >> 16) as i16;
                }
                17 => {
                    self.light_color_matrix[1][0] = (val & 0xFFFF) as i16;
                    self.light_color_matrix[0][2] = (val >> 16) as i16;
                }
                18 => {
                    self.light_color_matrix[1][2] = (val & 0xFFFF) as i16;
                    self.light_color_matrix[1][1] = (val >> 16) as i16;
                }
                19 => {
                    self.light_color_matrix[2][1] = (val & 0xFFFF) as i16;
                    self.light_color_matrix[2][0] = (val >> 16) as i16;
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
                31 => self.flag = val,
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
                12 => ((self.screenxy[0][0] as u32) << 16) + (self.screenxy[0][1] as u32 & 0xFFFF),
                13 => ((self.screenxy[1][0] as u32) << 16) + (self.screenxy[1][1] as u32 & 0xFFFF),
                14 => ((self.screenxy[2][0] as u32) << 16) + (self.screenxy[2][1] as u32 & 0xFFFF),
                15 => ((self.screenxy[3][0] as u32) << 16) + (self.screenxy[3][1] as u32 & 0xFFFF),
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
                },
                30 => self.lzcs as u32,
                31 => {
                    if self.lzcs > 0 {
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
                7 => {} // OTZ is read only,
                8 => self.intermediates[0] = (val & 0xFFFF) as i16,
                9 => self.intermediates[1] = (val & 0xFFFF) as i16,
                10 => self.intermediates[2] = (val & 0xFFFF) as i16,
                11 => self.intermediates[3] = (val & 0xFFFF) as i16,
                12 => {
                    self.screenxy[0][1] = (val & 0xFFFF) as i16;
                    self.screenxy[0][0] = (val >> 16) as i16;
                }
                13 => {
                    self.screenxy[1][1] = (val & 0xFFFF) as i16;
                    self.screenxy[1][0] = (val >> 16) as i16;
                }
                14 => {
                    self.screenxy[2][1] = (val & 0xFFFF) as i16;
                    self.screenxy[2][0] = (val >> 16) as i16;
                }
                15 => {
                    self.screenxy[3][1] = (val & 0xFFFF) as i16;
                    self.screenxy[3][0] = (val >> 16) as i16;
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

        println!("Here?");

        match cmd & 0x1F {
            0x01 => {
                // Perspective Transformation Single: RTPS
                event!(target: "ps1_emulator::GTE", Level::TRACE, "RTPS");
                let sf = cmd & 0x80000 > 0;
                self.rtps(sf);
            }
            0x06 => {
                // Normal Clipping
                event!(target: "ps1_emulator::GTE", Level::TRACE, "NCLIP");
                self.nclip();
            }
            0x0C => {
                // Cross Product: OP(sf)
                let sf = cmd & 0x80000 > 0;
                let im = cmd & 0x400 > 0;

                self.op(sf, im);
            }
            0x10 => {
                // Depth Cueing Single: DPCS
                event!(target: "ps1_emulator::GTE", Level::TRACE, "DPCS");
                let sf = cmd & 0x80000 > 0;
                self.dpcs(sf);
            }
            0x11 => {
                // Interpolation of a vector and far color vector: INTPL
                event!(target: "ps1_emulator::GTE", Level::TRACE, "INTPL");
                let sf = cmd & 0x80000 > 0;
                self.intpl(sf);
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
                event!(target: "ps1_emulator::GTE", Level::TRACE, "MVMVA: 0x{:08X}", cmd);

                self.mvmva(mv, tv, vector, sf);
            }
            0x13 => {
                // Normal Color depth cue single vector: NCDS
                event!(target: "ps1_emulator::GTE", Level::TRACE, "NCDS");
                let sf = cmd & 0x80000 > 0;
                self.ncds(sf);
            }
            0x14 => {
                // Color Depth Que: CDP
                event!(target: "ps1_emulator::GTE", Level::TRACE, "CDP");
                let sf = cmd & 0x80000 > 0;
                self.cdp(sf);
            }
            0x16 => {
                // Normal color depth cue triple vector: NCDT
                event!(target: "ps1_emulator::GTE", Level::TRACE, "NCDT");
                let sf = cmd & 0x80000 > 0;
                self.ncdt(sf);
            }
            0x1B => {
                // Normal Color Col. single vector: NCCS
                event!(target: "ps1_emulator::GTE", Level::TRACE, "NCSS");
                let sf = cmd & 0x80000 > 0;
                self.nccs(sf);
            }
            0x1C => {
                // Color Color: CC
                event!(target: "ps1_emulator::GTE", Level::TRACE, "CC");
                let sf = cmd & 0x80000 > 0;
                self.cc(sf);
            }
            0x1E => {
                // Normal Color v0: NCS
                event!(target: "ps1_emulator::GTE", Level::TRACE, "NCS");

                let sf = cmd & 0x80000 > 0;
                self.ncs(sf);
            }
            0x20 => {
                // Normal Color v0, v2, v2: NCT
                event!(target: "ps1_emulator::GTE", Level::TRACE, "NCT");

                let sf = cmd & 0x80000 > 0;
                self.nct(sf);
            }
            0x28 => {
                // Square of Vector IR: SQR(sf)
                event!(target: "ps1_emulator::GTE", Level::TRACE, "SQR");

                let sf = cmd & 0x80000 > 0;
                self.sqr(sf);
            }
            0x29 => {
                // Depth Cue Color Light: DCPL
                event!(target: "ps1_emulator::GTE", Level::TRACE, "DCPL");

                let sf = cmd & 0x80000 > 0;
                self.dcpl(sf);
            }
            0x2A => {
                // Depth Cueing Single: DPCT
                event!(target: "ps1_emulator::GTE", Level::TRACE, "DPCT");
                let sf = cmd & 0x80000 > 0;
                self.dpct(sf);
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
                self.rtpt(sf);
            }
            0x3F => {
                // Normal Color Col. triple vector: NCCT
                event!(target: "ps1_emulator::GTE", Level::TRACE, "NCCT");
                let sf = cmd & 0x80000 > 0;
                self.ncct(sf);
            }
            _ => {
                event!(target: "ps1_emulator::GTE", Level::ERROR, "No GTE command for 0x{:02X}", cmd & 0x1F);
            }
        }
    }

    fn scxy_fifo(&mut self, sxp: i16, syp: i16) {
        self.screenxy[0] = self.screenxy[1];
        self.screenxy[1] = self.screenxy[2];
        self.screenxy[2][1] = syp;
        self.screenxy[2][0] = sxp;

        self.screenxy[3] = self.screenxy[2];
    }

    fn scz_fifo(&mut self, new_scz: u32) {
        self.screenz[0] = self.screenz[1];
        self.screenz[1] = self.screenz[2];
        self.screenz[2] = self.screenz[3];
        self.screenz[3] = (new_scz & 0xFFFF) as u16;
    }

    fn rgb_fifo(&mut self, new_rgb: u32) {
        self.characteristic_color[0] = self.characteristic_color[1];
        self.characteristic_color[1] = self.characteristic_color[2];
        self.characteristic_color[2] = new_rgb;
    }

    fn rtps(&mut self, sf: bool) {
        self.perspective_transform(self.v0, sf);
    }

    fn rtpt(&mut self, sf: bool) {
        self.perspective_transform(self.v0, sf);
        self.perspective_transform(self.v1, sf);
        self.perspective_transform(self.v2, sf);
    }

    fn perspective_transform(&mut self, vector: [i16; 3], sf: bool) {
        /*
        IR1 = MAC1 = (TRX*1000h + RT11*VX0 + RT12*VY0 + RT13*VZ0) SAR (sf*12)
        IR2 = MAC2 = (TRY*1000h + RT21*VX0 + RT22*VY0 + RT23*VZ0) SAR (sf*12)
        IR3 = MAC3 = (TRZ*1000h + RT31*VX0 + RT32*VY0 + RT33*VZ0) SAR (sf*12)
        SZ3 = MAC3 SAR ((1-sf)*12)                           ;ScreenZ FIFO 0..+FFFFh
        MAC0=(((H*20000h/SZ3)+1)/2)*IR1+OFX, SX2=MAC0/10000h ;ScrX FIFO -400h..+3FFh
        MAC0=(((H*20000h/SZ3)+1)/2)*IR2+OFY, SY2=MAC0/10000h ;ScrY FIFO -400h..+3FFh
        MAC0=(((H*20000h/SZ3)+1)/2)*DQA+DQB, IR0=MAC0/1000h  ;Depth cueing 0..+1000h
        */
        // MAC1
        self.mac[1] = (self.translation_vec[0] * 0x1000
            + self.rotation_matrix[0][0] as i32 * vector[0] as i32
            + self.rotation_matrix[0][1] as i32 * vector[1] as i32
            + self.rotation_matrix[0][2] as i32 * vector[2] as i32)
            >> (sf as u8 * 12);

        // IR1
        self.intermediates[1] = self.mac[1].clamp(-0x8000, 0x7FFF) as i16;

        // MAC2
        self.mac[2] = (self.translation_vec[1] * 0x1000
            + self.rotation_matrix[1][0] as i32 * vector[0] as i32
            + self.rotation_matrix[1][1] as i32 * vector[1] as i32
            + self.rotation_matrix[1][2] as i32 * vector[2] as i32)
            >> (sf as u8 * 12);

        // IR2
        self.intermediates[2] = self.mac[2].clamp(-0x8000, 0x7FFF) as i16;

        // MAC3
        self.mac[3] = (self.translation_vec[2] * 0x1000
            + self.rotation_matrix[2][0] as i32 * vector[0] as i32
            + self.rotation_matrix[2][1] as i32 * vector[1] as i32
            + self.rotation_matrix[2][2] as i32 * vector[2] as i32)
            >> (sf as u8 * 12);

        // IR3
        self.intermediates[3] = self.mac[3].clamp(-0x8000, 0x7FFF) as i16;

        // SZ3
        self.scz_fifo((self.mac[3] >> (!sf as u8 * 12)) as u32);

        let division_result = if let Some(div) = ((self.h as u32) * 0x10000
            + self.screenz[3] as u32 / 2)
            .checked_div(self.screenz[3] as u32)
        {
            if div > 0x1FFFF { 0x1FFFF } else { div }
        } else {
            0x1FFFF
        };

        // MAC0 SCX
        self.mac[0] = division_result as i32 * self.intermediates[1] as i32 + self.screen_offset[0];
        let sxp = (self.mac[0] / 0x10000).clamp(-0x400, 0x3FF) as i16;

        // MAC0 SCY
        self.mac[0] = division_result as i32 * self.intermediates[2] as i32 + self.screen_offset[1];
        let syp = (self.mac[0] / 0x10000).clamp(-0x400, 0x3FF) as i16;

        self.scxy_fifo(sxp, syp);

        // MAC0 Depth
        self.mac[0] = division_result as i32 * self.depth_cue_a as i32 + self.depth_cue_b;
        self.intermediates[0] = (self.mac[0] / 0x1000) as i16;
    }

    fn nclip(&mut self) {
        // MAC0 =   SX0*SY1 + SX1*SY2 + SX2*SY0 - SX0*SY2 - SX1*SY0 - SX2*SY1
        self.mac[0] = self.screenxy[0][0] as i32 * self.screenxy[1][1] as i32
            + self.screenxy[1][0] as i32 * self.screenxy[2][1] as i32
            + self.screenxy[2][0] as i32 * self.screenxy[0][1] as i32
            - self.screenxy[0][0] as i32 * self.screenxy[2][1] as i32
            - self.screenxy[1][0] as i32 * self.screenxy[0][1] as i32
            - self.screenxy[2][0] as i32 * self.screenxy[1][1] as i32;
    }

    fn avsz3(&mut self) {
        // MAC0 = ZSF3*(SZ1+SZ2+SZ3)
        // OTZ  = MAC0/1000h
        let sum = self.screenz[1] + self.screenz[2] + self.screenz[3];
        self.mac[0] = multiply_fixed_point(self.zsf3, sum as i16) as i32;
        self.otz = (self.mac[0] / 0x1000) as u16;
    }

    fn avsz4(&mut self) {
        // MAC0 = ZSF4*(SZ0+SZ1+SZ2+SZ3)
        // OTZ  = MAC0/1000h
        let sum = self.screenz[0] + self.screenz[1] + self.screenz[2] + self.screenz[3];
        self.mac[0] = multiply_fixed_point(self.zsf4, sum as i16) as i32;
        self.otz = (self.mac[0] / 0x1000) as u16;
    }

    fn mvmva(&mut self, mv: MV, tv: TV, vector: [i16; 3], sf: bool) {
        //   MAC1 = (Tx1*1000h + Mx11*Vx1 + Mx12*Vx2 + Mx13*Vx3) SAR (sf*12)
        //   MAC2 = (Tx2*1000h + Mx21*Vx1 + Mx22*Vx2 + Mx23*Vx3) SAR (sf*12)
        //   MAC3 = (Tx3*1000h + Mx31*Vx1 + Mx32*Vx2 + Mx33*Vx3) SAR (sf*12)
        //   [IR1,IR2,IR3] = [MAC1,MAC2,MAC3]

        // translation vector
        let translation_result = match tv {
            TV::Translation => [
                self.translation_vec[0] * 0x1000,
                self.translation_vec[1] * 0x1000,
                self.translation_vec[2] * 0x1000,
            ],
            TV::FarColor => [
                self.far_color[0] * 0x1000,
                self.far_color[1] * 0x1000,
                self.far_color[2] * 0x1000,
            ],
            TV::BackgroundColor => [
                self.background_color[0] * 0x1000,
                self.background_color[1] * 0x1000,
                self.background_color[2] * 0x1000,
            ],
            TV::None => [0, 0, 0],
        };

        // matrix
        let matrix_results = match mv {
            MV::Rotation => {
                let row1 = self.rotation_matrix[0][0] as i32 * vector[0] as i32
                    + self.rotation_matrix[0][1] as i32 * vector[1] as i32
                    + self.rotation_matrix[0][2] as i32 * vector[2] as i32;

                let row2 = self.rotation_matrix[1][0] as i32 * vector[0] as i32
                    + self.rotation_matrix[1][1] as i32 * vector[1] as i32
                    + self.rotation_matrix[1][2] as i32 * vector[2] as i32;

                let row3 = self.rotation_matrix[2][0] as i32 * vector[0] as i32
                    + self.rotation_matrix[2][1] as i32 * vector[1] as i32
                    + self.rotation_matrix[2][2] as i32 * vector[2] as i32;

                [row1, row2, row3]
            }
            MV::Light => {
                let row1 = self.light_matrix[0][0] as i32 * vector[0] as i32
                    + self.light_matrix[0][1] as i32 * vector[1] as i32
                    + self.light_matrix[0][2] as i32 * vector[2] as i32;

                let row2 = self.light_matrix[1][0] as i32 * vector[0] as i32
                    + self.light_matrix[1][1] as i32 * vector[1] as i32
                    + self.light_matrix[1][2] as i32 * vector[2] as i32;

                let row3 = self.light_matrix[2][0] as i32 * vector[0] as i32
                    + self.light_matrix[2][1] as i32 * vector[1] as i32
                    + self.light_matrix[2][2] as i32 * vector[2] as i32;

                [row1, row2, row3]
            }
            MV::Color => {
                let row1 = self.light_color_matrix[0][0] as i32 * vector[0] as i32
                    + self.light_color_matrix[0][1] as i32 * vector[1] as i32
                    + self.light_color_matrix[0][2] as i32 * vector[2] as i32;

                let row2 = self.light_color_matrix[1][0] as i32 * vector[0] as i32
                    + self.light_color_matrix[1][1] as i32 * vector[1] as i32
                    + self.light_color_matrix[1][2] as i32 * vector[2] as i32;

                let row3 = self.light_color_matrix[2][0] as i32 * vector[0] as i32
                    + self.light_color_matrix[2][1] as i32 * vector[1] as i32
                    + self.light_color_matrix[2][2] as i32 * vector[2] as i32;

                [row1, row2, row3]
            }
            MV::Reserved => {
                let row1 = -10 * (self.rgb & 0xFF) as i32 * vector[0] as i32
                    + (10 * (self.rgb & 0xFF)) as i32 * vector[1] as i32
                    + self.intermediates[0] as i32 * vector[2] as i32;

                let row2 = self.rotation_matrix[0][2] as i32 * vector[0] as i32
                    + self.rotation_matrix[0][2] as i32 * vector[1] as i32
                    + self.rotation_matrix[0][2] as i32 * vector[2] as i32;

                let row3 = self.rotation_matrix[1][1] as i32 * vector[0] as i32
                    + self.rotation_matrix[1][1] as i32 * vector[1] as i32
                    + self.rotation_matrix[1][1] as i32 * vector[2] as i32;

                [row1, row2, row3]
            }
        };

        self.mac[1] = (translation_result[0] + matrix_results[0]) >> (sf as u8 * 12);
        self.mac[2] = (translation_result[1] + matrix_results[1]) >> (sf as u8 * 12);
        self.mac[3] = (translation_result[2] + matrix_results[2]) >> (sf as u8 * 12);

        self.intermediates[1] = self.mac[1].clamp(-0x8000, 0x7FFF) as i16;
        self.intermediates[2] = self.mac[2].clamp(-0x8000, 0x7FFF) as i16;
        self.intermediates[3] = self.mac[3].clamp(-0x8000, 0x7FFF) as i16;
    }

    fn sqr(&mut self, sf: bool) {
        //  [MAC1,MAC2,MAC3] = [IR1*IR1,IR2*IR2,IR3*IR3] SHR (sf*12)
        //  [IR1,IR2,IR3]    = [MAC1,MAC2,MAC3]    ;IR1,IR2,IR3 saturated to max 7FFFh
        self.mac[1] =
            (self.intermediates[1] as i32 * self.intermediates[1] as i32) >> (sf as u8 * 12);
        self.mac[2] =
            (self.intermediates[2] as i32 * self.intermediates[2] as i32) >> (sf as u8 * 12);
        self.mac[3] =
            (self.intermediates[3] as i32 * self.intermediates[3] as i32) >> (sf as u8 * 12);

        self.intermediates[1] = self.mac[1].clamp(-0x8000, 0x7FFF) as i16;
        self.intermediates[2] = self.mac[2].clamp(-0x8000, 0x7FFF) as i16;
        self.intermediates[3] = self.mac[3].clamp(-0x8000, 0x7FFF) as i16;
    }

    fn op(&mut self, sf: bool, _im: bool) {
        self.mac[1] = (self.intermediates[3] as i32 * self.rotation_matrix[1][1] as i32
            - self.intermediates[2] as i32 * self.rotation_matrix[2][2] as i32)
            >> (sf as u8 * 12);

        self.mac[2] = (self.intermediates[1] as i32 * self.rotation_matrix[2][2] as i32
            - self.intermediates[3] as i32 * self.rotation_matrix[0][0] as i32)
            >> (sf as u8 * 12);

        self.mac[3] = (self.intermediates[2] as i32 * self.rotation_matrix[0][0] as i32
            - self.intermediates[2] as i32 * self.rotation_matrix[1][1] as i32)
            >> (sf as u8 * 12);

        self.intermediates[1] = self.mac[1].clamp(-0x8000, 0x7FFF) as i16;
        self.intermediates[2] = self.mac[2].clamp(-0x8000, 0x7FFF) as i16;
        self.intermediates[3] = self.mac[3].clamp(-0x8000, 0x7FFF) as i16;
    }

    fn normal_color(&mut self, vector: [i16; 3], sf: bool) {
        //   [IR1,IR2,IR3] = [MAC1,MAC2,MAC3] = (LLM*V0) SAR (sf*12)
        //   [IR1,IR2,IR3] = [MAC1,MAC2,MAC3] = (BK*1000h + LCM*IR) SAR (sf*12)
        self.mac[1] = (self.light_matrix[0][0] as i32 * vector[0] as i32
            + self.light_matrix[0][1] as i32 * vector[1] as i32
            + self.light_matrix[0][2] as i32 * vector[2] as i32)
            >> (sf as u8 * 12);

        self.mac[2] = (self.light_matrix[1][0] as i32 * vector[0] as i32
            + self.light_matrix[1][1] as i32 * vector[1] as i32
            + self.light_matrix[1][2] as i32 * vector[2] as i32)
            >> (sf as u8 * 12);

        self.mac[2] = (self.light_matrix[2][0] as i32 * vector[0] as i32
            + self.light_matrix[2][1] as i32 * vector[1] as i32
            + self.light_matrix[2][2] as i32 * vector[2] as i32)
            >> (sf as u8 * 12);

        self.intermediates[1] = self.mac[1].clamp(-0x8000, 0x7FFF) as i16;
        self.intermediates[2] = self.mac[2].clamp(-0x8000, 0x7FFF) as i16;
        self.intermediates[3] = self.mac[3].clamp(-0x8000, 0x7FFF) as i16;

        self.mac[1] = (self.background_color[0] * 0x1000
            + self.light_color_matrix[0][0] as i32 * self.intermediates[1] as i32
            + self.light_color_matrix[0][1] as i32 * self.intermediates[2] as i32
            + self.light_color_matrix[0][2] as i32 * self.intermediates[3] as i32)
            >> (sf as u8 * 12);

        self.mac[2] = (self.background_color[1] * 0x1000
            + self.light_color_matrix[1][0] as i32 * self.intermediates[1] as i32
            + self.light_color_matrix[1][1] as i32 * self.intermediates[2] as i32
            + self.light_color_matrix[1][2] as i32 * self.intermediates[3] as i32)
            >> (sf as u8 * 12);

        self.mac[3] = (self.background_color[2] * 0x1000
            + self.light_color_matrix[2][0] as i32 * self.intermediates[1] as i32
            + self.light_color_matrix[2][1] as i32 * self.intermediates[2] as i32
            + self.light_color_matrix[2][2] as i32 * self.intermediates[3] as i32)
            >> (sf as u8 * 12);
    }

    fn normal_color_col(&mut self, sf: bool) {
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
    }

    fn normal_color_depth(&mut self, sf: bool) {
        // [MAC1,MAC2,MAC3] = [R*IR1,G*IR2,B*IR3] SHL 4
        let r = (self.rgb & 0xFF) as i32;
        let g = ((self.rgb >> 8) & 0xFF) as i32;
        let b = ((self.rgb >> 16) & 0xFF) as i32;
        self.mac[1] = (r * self.intermediates[1] as i32) << 4;
        self.mac[2] = (g * self.intermediates[2] as i32) << 4;
        self.mac[3] = (b * self.intermediates[3] as i32) << 4;

        //[MAC1,MAC2,MAC3] = MAC+(FC-MAC)*IR0
        self.mac[1] += (self.far_color[0] - self.mac[1]) * self.intermediates[0] as i32;
        self.mac[2] += (self.far_color[1] - self.mac[2]) * self.intermediates[0] as i32;
        self.mac[3] += (self.far_color[2] - self.mac[3]) * self.intermediates[0] as i32;

        // [MAC1,MAC2,MAC3] = [MAC1,MAC2,MAC3] SAR (sf*12)
        self.mac[1] >>= sf as u8 * 12;
        self.mac[2] >>= sf as u8 * 12;
        self.mac[3] >>= sf as u8 * 12;
    }

    fn ncs(&mut self, sf: bool) {
        self.normal_color(self.v0, sf);
        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
    }

    fn nct(&mut self, sf: bool) {
        // todo: color fifo for each call
        self.normal_color(self.v0, sf);
        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
        self.normal_color(self.v1, sf);
        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
        self.normal_color(self.v2, sf);
        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
    }

    fn nccs(&mut self, sf: bool) {
        self.normal_color(self.v0, sf);
        self.normal_color_col(sf);
        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
    }

    fn ncct(&mut self, sf: bool) {
        // todo: color fifo for each call
        self.normal_color(self.v0, sf);
        self.normal_color_col(sf);
        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
        self.normal_color(self.v1, sf);
        self.normal_color_col(sf);
        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
        self.normal_color(self.v2, sf);
        self.normal_color_col(sf);
        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
    }

    fn ncds(&mut self, sf: bool) {
        self.normal_color(self.v0, sf);
        self.normal_color_depth(sf);
        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
    }

    fn ncdt(&mut self, sf: bool) {
        self.normal_color(self.v0, sf);
        self.normal_color_depth(sf);
        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
        self.normal_color(self.v1, sf);
        self.normal_color_depth(sf);
        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
        self.normal_color(self.v2, sf);
        self.normal_color_depth(sf);
        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
    }

    fn cc(&mut self, sf: bool) {
        // [IR1,IR2,IR3] = [MAC1,MAC2,MAC3] = (BK*1000h + LCM*IR) SAR (sf*12)
        self.mac[1] = (self.background_color[0] * 0x1000
            + self.light_color_matrix[0][0] as i32 * self.intermediates[1] as i32
            + self.light_color_matrix[0][1] as i32 * self.intermediates[2] as i32
            + self.light_color_matrix[0][2] as i32 * self.intermediates[3] as i32)
            >> (sf as u8 * 12);

        self.mac[2] = (self.background_color[1] * 0x1000
            + self.light_color_matrix[1][0] as i32 * self.intermediates[1] as i32
            + self.light_color_matrix[1][1] as i32 * self.intermediates[2] as i32
            + self.light_color_matrix[1][2] as i32 * self.intermediates[3] as i32)
            >> (sf as u8 * 12);

        self.mac[3] = (self.background_color[2] * 0x1000
            + self.light_color_matrix[2][0] as i32 * self.intermediates[1] as i32
            + self.light_color_matrix[2][1] as i32 * self.intermediates[2] as i32
            + self.light_color_matrix[2][2] as i32 * self.intermediates[3] as i32)
            >> (sf as u8 * 12);

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

        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
    }

    fn cdp(&mut self, sf: bool) {
        // [IR1,IR2,IR3] = [MAC1,MAC2,MAC3] = (BK*1000h + LCM*IR) SAR (sf*12)
        self.mac[1] = (self.background_color[0] * 0x1000
            + self.light_color_matrix[0][0] as i32 * self.intermediates[1] as i32
            + self.light_color_matrix[0][1] as i32 * self.intermediates[2] as i32
            + self.light_color_matrix[0][2] as i32 * self.intermediates[3] as i32)
            >> (sf as u8 * 12);

        self.mac[2] = (self.background_color[1] * 0x1000
            + self.light_color_matrix[1][0] as i32 * self.intermediates[1] as i32
            + self.light_color_matrix[1][1] as i32 * self.intermediates[2] as i32
            + self.light_color_matrix[1][2] as i32 * self.intermediates[3] as i32)
            >> (sf as u8 * 12);

        self.mac[3] = (self.background_color[2] * 0x1000
            + self.light_color_matrix[2][0] as i32 * self.intermediates[1] as i32
            + self.light_color_matrix[2][1] as i32 * self.intermediates[2] as i32
            + self.light_color_matrix[2][2] as i32 * self.intermediates[3] as i32)
            >> (sf as u8 * 12);

        // [MAC1,MAC2,MAC3] = [R*IR1,G*IR2,B*IR3] SHL 4
        let r = (self.rgb & 0xFF) as i32;
        let g = ((self.rgb >> 8) & 0xFF) as i32;
        let b = ((self.rgb >> 16) & 0xFF) as i32;

        self.mac[1] = (r * self.intermediates[1] as i32) << 4;
        self.mac[2] = (g * self.intermediates[2] as i32) << 4;
        self.mac[3] = (b * self.intermediates[3] as i32) << 4;

        // [MAC1,MAC2,MAC3] = MAC+(FC-MAC)*IR0
        self.mac[1] += (self.far_color[0] - self.mac[1]) * self.intermediates[0] as i32;
        self.mac[2] += (self.far_color[1] - self.mac[2]) * self.intermediates[0] as i32;
        self.mac[3] += (self.far_color[2] - self.mac[3]) * self.intermediates[0] as i32;

        // [MAC1,MAC2,MAC3] = [MAC1,MAC2,MAC3] SAR (sf*12)
        self.mac[1] >>= sf as u8 * 12;
        self.mac[2] >>= sf as u8 * 12;
        self.mac[3] >>= sf as u8 * 12;

        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
    }

    fn dcpl(&mut self, sf: bool) {
        // [MAC1,MAC2,MAC3] = [R*IR1,G*IR2,B*IR3] SHL 4
        let r = (self.rgb & 0xFF) as i32;
        let g = ((self.rgb >> 8) & 0xFF) as i32;
        let b = ((self.rgb >> 16) & 0xFF) as i32;

        self.mac[1] = (r * self.intermediates[1] as i32) << 4;
        self.mac[2] = (g * self.intermediates[2] as i32) << 4;
        self.mac[3] = (b * self.intermediates[3] as i32) << 4;

        // [MAC1,MAC2,MAC3] = MAC+(FC-MAC)*IR0
        self.mac[1] += (self.far_color[0] - self.mac[1]) * self.intermediates[0] as i32;
        self.mac[2] += (self.far_color[1] - self.mac[2]) * self.intermediates[0] as i32;
        self.mac[3] += (self.far_color[2] - self.mac[3]) * self.intermediates[0] as i32;

        // [MAC1,MAC2,MAC3] = [MAC1,MAC2,MAC3] SAR (sf*12)
        self.mac[1] >>= sf as u8 * 12;
        self.mac[2] >>= sf as u8 * 12;
        self.mac[3] >>= sf as u8 * 12;

        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
    }

    fn dpcs(&mut self, sf: bool) {
        // [MAC1,MAC2,MAC3] = [R,G,B] SHL 16
        let r = (self.rgb & 0xFF) as i32;
        let g = ((self.rgb >> 8) & 0xFF) as i32;
        let b = ((self.rgb >> 16) & 0xFF) as i32;
        self.mac[1] = r << 16;
        self.mac[2] = g << 16;
        self.mac[3] = b << 16;

        // [MAC1,MAC2,MAC3] = MAC+(FC-MAC)*IR0
        self.mac[1] += (self.far_color[0] - self.mac[1]) * self.intermediates[0] as i32;
        self.mac[2] += (self.far_color[1] - self.mac[2]) * self.intermediates[0] as i32;
        self.mac[3] += (self.far_color[2] - self.mac[3]) * self.intermediates[0] as i32;

        // [MAC1,MAC2,MAC3] = [MAC1,MAC2,MAC3] SAR (sf*12)
        self.mac[1] >>= sf as u8 * 12;
        self.mac[2] >>= sf as u8 * 12;
        self.mac[3] >>= sf as u8 * 12;

        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
    }

    fn dpct(&mut self, sf: bool) {
        self.dpcs(sf);
        self.dpcs(sf);
        self.dpcs(sf);
    }

    fn intpl(&mut self, sf: bool) {
        // [MAC1,MAC2,MAC3] = [IR1,IR2,IR3] SHL 12
        self.mac[1] = (self.intermediates[1] as i32) << 12;
        self.mac[2] = (self.intermediates[2] as i32) << 12;
        self.mac[3] = (self.intermediates[3] as i32) << 12;

        // [MAC1,MAC2,MAC3] = MAC+(FC-MAC)*IR0
        self.mac[1] += (self.far_color[0] - self.mac[1]) * self.intermediates[0] as i32;
        self.mac[2] += (self.far_color[1] - self.mac[2]) * self.intermediates[0] as i32;
        self.mac[3] += (self.far_color[2] - self.mac[3]) * self.intermediates[0] as i32;

        // [MAC1,MAC2,MAC3] = [MAC1,MAC2,MAC3] SAR (sf*12)
        self.mac[1] >>= sf as u8 * 12;
        self.mac[2] >>= sf as u8 * 12;
        self.mac[3] >>= sf as u8 * 12;

        let code = ((self.rgb >> 24) & 0xFF) as u8;
        self.rgb_fifo(u32::from_le_bytes([
            (self.mac[1] / 16) as u8,
            (self.mac[2] / 16) as u8,
            (self.mac[3] / 16) as u8,
            code,
        ]));
    }
}

fn multiply_fixed_point(arg1: i16, arg2: i16) -> i16 {
    let arg1 = arg1 as i32;
    let arg2 = arg2 as i32;
    ((arg1 * arg2) >> 12) as i16
}

enum MV {
    Rotation,
    Light,
    Color,
    Reserved,
}

enum TV {
    Translation,
    BackgroundColor,
    FarColor,
    None,
}

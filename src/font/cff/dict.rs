use crate::parse_binary::BinaryParser;

#[derive(Debug)]
pub(crate) struct TopDict {
    pub version: Option<u16>,
    pub notice: Option<u16>,
    pub copyright: Option<u16>,
    pub full_name: Option<u16>,
    pub family_name: Option<u16>,
    pub weight: Option<u16>,
    pub is_fixed_pitch: bool,
    pub italic_angle: f32,
    pub underline_position: f32,
    pub underline_thickness: f32,
    pub paint_type: f32,
    pub charstring_type: u32,
    pub font_matrix: [f32; 6],
    pub unique_id: Option<f32>,
    pub font_b_box: [f32; 4],
    pub stroke_width: f32,
    pub xuid: Option<Vec<f32>>,
    pub charset: u32,
    pub encoding: u32,
    pub char_strings: Option<u32>,
    /// Private DICT size and offset (0)
    pub private: Option<[f32; 2]>,
    pub synthetic_base: Option<f32>,
    pub post_script: Option<u16>,
    pub base_font_name: Option<u16>,
    pub base_font_blend: Option<Vec<f32>>,
}

impl Default for TopDict {
    fn default() -> Self {
        TopDict {
            version: None,
            notice: None,
            copyright: None,
            full_name: None,
            family_name: None,
            weight: None,
            is_fixed_pitch: false,
            italic_angle: 0.0,
            underline_position: -100.0,
            underline_thickness: 50.0,
            paint_type: 0.0,
            charstring_type: 2,
            font_matrix: [0.001, 0.0, 0.0, 0.001, 0.0, 0.0],
            unique_id: None,
            font_b_box: [0.0, 0.0, 0.0, 0.0],
            stroke_width: 0.0,
            xuid: None,
            charset: 0,
            encoding: 0,
            char_strings: None,
            private: None,
            synthetic_base: None,
            post_script: None,
            base_font_name: None,
            base_font_blend: None,
        }
    }
}

#[derive(Debug)]
pub(super) struct PrivateDict {
    pub blue_values: Option<Vec<f32>>,
    pub other_blues: Option<Vec<f32>>,
    pub family_blues: Option<Vec<f32>>,
    pub family_other_blues: Option<Vec<f32>>,
    pub blue_scale: f32,
    pub blue_shift: f32,
    pub blue_fuzz: f32,
    pub std_hw: Option<f32>,
    pub std_vw: Option<f32>,
    pub stem_snap_h: Option<Vec<f32>>,
    pub stem_snap_v: Option<Vec<f32>>,
    pub force_bold: bool,
    pub language_group: f32,
    pub expansion_factor: f32,
    pub initial_random_seed: f32,
    pub subrs: Option<f32>,
    pub default_width_x: f32,
    pub nominal_width_x: f32,
}

impl Default for PrivateDict {
    fn default() -> Self {
        PrivateDict {
            blue_values: None,
            other_blues: None,
            family_blues: None,
            family_other_blues: None,
            blue_scale: 0.039625,
            blue_shift: 7.0,
            blue_fuzz: 1.0,
            std_hw: None,
            std_vw: None,
            stem_snap_h: None,
            stem_snap_v: None,
            force_bold: false,
            language_group: 0.0,
            expansion_factor: 0.06,
            initial_random_seed: 0.0,
            subrs: None,
            default_width_x: 0.0,
            nominal_width_x: 0.0,
        }
    }
}

pub(super) struct CffDictInterpreter<'a> {
    buffer: &'a [u8],
    cursor: usize,
    operand_stack: Vec<f32>,
}

impl<'a> CffDictInterpreter<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            cursor: 0,
            operand_stack: Vec::new(),
        }
    }

    fn push(&mut self, n: f32) {
        self.operand_stack.push(n);
    }

    fn pop(&mut self) -> anyhow::Result<f32> {
        self.operand_stack
            .pop()
            .ok_or(anyhow::anyhow!("stack underflow"))
    }

    fn pop_arr(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.operand_stack)
    }

    fn pop_delta(&mut self) -> Vec<f32> {
        todo!()
    }

    fn pop_bool(&mut self) -> anyhow::Result<bool> {
        let n = self.pop()?;

        anyhow::ensure!(n == 0.0 || n == 1.0);

        Ok(n != 0.0)
    }

    fn pop_u16(&mut self) -> anyhow::Result<u16> {
        let n = self.pop()?;

        anyhow::ensure!(n >= 0.0 && n <= u16::MAX as f32);
        anyhow::ensure!(n.fract() == 0.0);

        Ok(n as u16)
    }

    fn pop_u32(&mut self) -> anyhow::Result<u32> {
        let n = self.pop()?;

        anyhow::ensure!(n >= 0.0 && n <= u32::MAX as f32);
        anyhow::ensure!(n.fract() == 0.0);

        Ok(n as u32)
    }

    fn parse_number(&mut self, b0: u8) -> anyhow::Result<()> {
        match b0 {
            32..=246 => {
                let n = b0 as i32 - 139;
                self.push(n as f32);
            }
            247..=250 => {
                let b0 = b0 as u32;
                let b1 = self.next()? as u32;
                let n = (b0 - 247) * 256 + b1 + 108;
                self.push(n as f32);
            }
            251..=254 => {
                let b0 = b0 as i32;
                let b1 = self.next()? as i32;
                let n = -(b0 - 251) * 256 - b1 - 108;
                self.push(n as f32);
            }
            28 => {
                let b1 = self.next()? as i32;
                let b2 = self.next()? as i32;
                let n = (b1 << 8) | b2;
                self.push(n as f32);
            }
            29 => {
                let b1 = self.next()? as i32;
                let b2 = self.next()? as i32;
                let b3 = self.next()? as i32;
                let b4 = self.next()? as i32;
                let n = (b1 << 24) | (b2 << 16) | (b3 << 8) | b4;
                self.push(n as f32);
            }
            _ => anyhow::bail!("invalid dict operator: {:?}", b0),
        }

        Ok(())
    }

    pub fn parse_top_dict(buffer: &'a [u8]) -> anyhow::Result<TopDict> {
        let mut parser = Self::new(buffer);
        let mut dict = TopDict::default();

        while parser.peek().is_some() {
            match parser.next()? {
                0 => dict.version = Some(parser.pop_u16()?),
                1 => dict.notice = Some(parser.pop_u16()?),
                2 => dict.full_name = Some(parser.pop_u16()?),
                3 => dict.family_name = Some(parser.pop_u16()?),
                4 => dict.weight = Some(parser.pop_u16()?),
                5 => dict.font_b_box = parser.pop_arr().try_into().unwrap(),
                12 => match parser.next()? {
                    0 => dict.copyright = Some(parser.pop_u16()?),
                    1 => dict.is_fixed_pitch = parser.pop_bool()?,
                    2 => dict.italic_angle = parser.pop()?,
                    3 => dict.underline_position = parser.pop()?,
                    4 => dict.underline_thickness = parser.pop()?,
                    5 => dict.paint_type = parser.pop()?,
                    6 => dict.charstring_type = parser.pop_u32()?,
                    7 => dict.font_matrix = parser.pop_arr().try_into().unwrap(),
                    8 => dict.stroke_width = parser.pop()?,
                    20 => dict.synthetic_base = Some(parser.pop()?),
                    21 => dict.post_script = Some(parser.pop_u16()?),
                    22 => dict.base_font_name = Some(parser.pop_u16()?),
                    23 => dict.base_font_blend = Some(parser.pop_delta()),
                    b @ 30..=36 => {
                        anyhow::bail!("unimplemented CID CFF top dict operator: 12 {:?}", b)
                    }
                    b => anyhow::bail!("invalid top dict operator: 12 {:?}", b),
                },
                13 => dict.unique_id = Some(parser.pop()?),
                14 => dict.xuid = Some(parser.pop_arr()),
                15 => dict.charset = parser.pop_u32()?,
                16 => dict.encoding = parser.pop_u32()?,
                17 => dict.char_strings = Some(parser.pop_u32()?),
                18 => dict.private = Some(parser.pop_arr().try_into().unwrap()),
                b0 => parser.parse_number(b0)?,
            }
        }

        anyhow::ensure!(parser.operand_stack.is_empty());

        assert_eq!(dict.charstring_type, 2, "unsupported CFF charstring type");

        Ok(dict)
    }

    pub fn parse_private_dict(buffer: &'a [u8]) -> anyhow::Result<PrivateDict> {
        let mut parser = Self::new(buffer);
        let mut dict = PrivateDict::default();

        while parser.peek().is_some() {
            match parser.next()? {
                6 => dict.blue_values = Some(parser.pop_delta()),
                7 => dict.other_blues = Some(parser.pop_delta()),
                8 => dict.family_blues = Some(parser.pop_delta()),
                9 => dict.family_other_blues = Some(parser.pop_delta()),
                10 => dict.std_hw = Some(parser.pop()?),
                11 => dict.std_vw = Some(parser.pop()?),
                12 => match parser.next()? {
                    9 => dict.blue_scale = parser.pop()?,
                    10 => dict.blue_shift = parser.pop()?,
                    11 => dict.blue_fuzz = parser.pop()?,
                    b => anyhow::bail!("invalid private dict operator: 12 {:?}", b),
                },
                19 => dict.subrs = Some(parser.pop()?),
                b0 => parser.parse_number(b0)?,
            }
        }

        anyhow::ensure!(parser.operand_stack.is_empty());

        Ok(dict)
    }
}

impl<'a> BinaryParser for CffDictInterpreter<'a> {
    fn buffer(&self) -> &[u8] {
        self.buffer
    }
    fn cursor(&self) -> usize {
        self.cursor
    }
    fn cursor_mut(&mut self) -> &mut usize {
        &mut self.cursor
    }
}

#![allow(unused)]

use std::collections::VecDeque;

use anyhow::anyhow;

use crate::{
    font::{true_type::table::TrueTypeGlyph, Glyph},
    geometry::{Outline, Path, Point},
};

use super::{
    graphics_state::{RoundState, TrueTypeGraphicsState, Vector, Zone},
    instruction::TrueTypeInstruction,
    table::OutlineFlag,
    F26Dot6, ParsedTrueTypeFontFile,
};

struct InstructionStream {
    buffer: Vec<u8>,
    cursor: usize,
}

impl InstructionStream {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self { buffer, cursor: 0 }
    }

    fn next_byte(&mut self) -> Option<u8> {
        self.buffer.get(self.cursor).copied().map(|b| {
            self.cursor += 1;
            b
        })
    }

    fn next_instruction(&mut self) -> Option<TrueTypeInstruction> {
        let b = self.next_byte()?;
        Some(match b {
            0x7F => TrueTypeInstruction::AdjustAngle,
            0x64 => TrueTypeInstruction::AbsoluteValue,
            0x60 => TrueTypeInstruction::Add,
            0x27 => TrueTypeInstruction::AlignPoints,
            0x3C => TrueTypeInstruction::AlignToReferencePoint,
            0x5A => TrueTypeInstruction::And,
            0x2B => TrueTypeInstruction::Call,
            0x67 => TrueTypeInstruction::Ceiling,
            0x25 => TrueTypeInstruction::CopyIndex,
            0x22 => TrueTypeInstruction::Clear,
            0x4F => TrueTypeInstruction::Debug,
            0x73 => TrueTypeInstruction::DeltaC1,
            0x74 => TrueTypeInstruction::DeltaC2,
            0x75 => TrueTypeInstruction::DeltaC3,
            0x5D => TrueTypeInstruction::DeltaP1,
            0x71 => TrueTypeInstruction::DeltaP2,
            0x72 => TrueTypeInstruction::DeltaP3,
            0x24 => TrueTypeInstruction::Depth,
            0x62 => TrueTypeInstruction::Div,
            0x20 => TrueTypeInstruction::Dup,
            0x59 => TrueTypeInstruction::EndIf,
            0x1B => TrueTypeInstruction::Else,
            0x2D => TrueTypeInstruction::EndFunctionDefinition,
            0x54 => TrueTypeInstruction::Equal,
            0x57 => TrueTypeInstruction::Even,
            0x2C => TrueTypeInstruction::BeginFunctionDefinition,
            0x4E => TrueTypeInstruction::FlipOff,
            0x4D => TrueTypeInstruction::FlipOn,
            0x80 => TrueTypeInstruction::FlipPoint,
            0x82 => TrueTypeInstruction::FlipRangeOff,
            0x81 => TrueTypeInstruction::FlipRangeOn,
            0x66 => TrueTypeInstruction::Floor,
            0x46..=0x47 => TrueTypeInstruction::GetProjectionVectorCoordinate,
            0x88 => TrueTypeInstruction::GetInfo,
            0x0D => TrueTypeInstruction::GetFreedomVector,
            0x0C => TrueTypeInstruction::GetProjectionVector,
            0x52 => TrueTypeInstruction::GreaterThan,
            0x53 => TrueTypeInstruction::GreaterThanOrEqual,
            0x89 => TrueTypeInstruction::InstructionDefinition,
            0x58 => TrueTypeInstruction::If,
            0x8E => TrueTypeInstruction::InstructionControl,
            0x39 => TrueTypeInstruction::InterpolatePoint,
            0x0F => TrueTypeInstruction::IntersectLines,
            0x30..=0x31 => TrueTypeInstruction::InterpolateUntouchedPoints(b - 0x30),
            0x1C => TrueTypeInstruction::JumpRelative,
            0x79 => TrueTypeInstruction::JumpRelativeOnFalse,
            0x78 => TrueTypeInstruction::JumpRelativeOnTrue,
            0x2A => TrueTypeInstruction::LoopAndCall,
            0x50 => TrueTypeInstruction::LessThan,
            0x51 => TrueTypeInstruction::LessThanOrEqual,
            0x8B => TrueTypeInstruction::Max,
            0x49..=0x4A => TrueTypeInstruction::MeasureDistance,
            0x2E..=0x2F => TrueTypeInstruction::MoveDirectAbsolutePoint(b - 0x2E),
            0xC0..=0xDF => TrueTypeInstruction::MoveDirectRelativePoint(b - 0xC0),
            0x3E..=0x3F => TrueTypeInstruction::MoveIndirectAbsolutePoint(b - 0x3E),
            0x8C => TrueTypeInstruction::Min,
            0x26 => TrueTypeInstruction::MoveIndexed,
            0xE0..=0xFF => TrueTypeInstruction::MoveIndirectRelativePoint(b - 0xE0),
            0x4B => TrueTypeInstruction::MeasurePixelsPerEm,
            0x4C => TrueTypeInstruction::MeasurePointSize,
            0x3A..=0x3B => TrueTypeInstruction::MoveStackIndirectRelativePoint,
            0x63 => TrueTypeInstruction::Multiply,
            0x65 => TrueTypeInstruction::Negate,
            0x55 => TrueTypeInstruction::NotEqual,
            0x5C => TrueTypeInstruction::LogicalNot,
            0x40 => TrueTypeInstruction::PushNBytes,
            0x41 => TrueTypeInstruction::PushNWords,
            0x6C..=0x6F => TrueTypeInstruction::NoRound,
            0x56 => TrueTypeInstruction::Odd,
            0x5B => TrueTypeInstruction::LogicalOr,
            0x21 => TrueTypeInstruction::Pop,
            0xB0..=0xB7 => TrueTypeInstruction::PushBytes(b - 0xB0),
            0xB8..=0xBF => TrueTypeInstruction::PushWords,
            0x45 => TrueTypeInstruction::ReadControlValueTableEntry,
            0x7D => TrueTypeInstruction::RoundDownToGrid,
            0x7A => TrueTypeInstruction::RoundOff,
            0x68..=0x6B => TrueTypeInstruction::Roll,
            0x43 => TrueTypeInstruction::ReadStore,
            0x3D => TrueTypeInstruction::RoundToDoubleGrid,
            0x18 => TrueTypeInstruction::RoundToGrid,
            0x19 => TrueTypeInstruction::RoundToHalfGrid,
            0x7C => TrueTypeInstruction::RoundUpToGrid,
            0x77 => TrueTypeInstruction::SuperRound45Deg,
            0x7E => TrueTypeInstruction::SetAngleWeight,
            0x85 => TrueTypeInstruction::ScanConversionControl,
            0x8D => TrueTypeInstruction::ScanType,
            0x48 => TrueTypeInstruction::SetsCoordinateFromStack,
            0x1D => TrueTypeInstruction::SetControlValueTableCutIn,
            0x5E => TrueTypeInstruction::SetDeltaBase,
            0x86..=0x87 => TrueTypeInstruction::SetDualProjectionVector,
            0x5F => TrueTypeInstruction::SetDeltaShift,
            0x04..=0x05 => TrueTypeInstruction::SetFreedomVectorFromStack,
            0x08..=0x09 => TrueTypeInstruction::SetFreedomVectorToLine,
            0x34..=0x35 => TrueTypeInstruction::SetFreedomVectorToProjectionVector,
            0x32..=0x33 => TrueTypeInstruction::ShiftPointUsingReferencePoint(b - 0x32),
            0x38 => TrueTypeInstruction::ShiftPointByPixels,
            0x36..=0x37 => TrueTypeInstruction::ShiftZoneUsingReferencePoint,
            0x17 => TrueTypeInstruction::SetLoop,
            0x1A => TrueTypeInstruction::SetMinimumDistance,
            0x0A => TrueTypeInstruction::SetProjectionVectorFromStack,
            0x02..=0x03 => TrueTypeInstruction::SetProjectionVectorToCoordinateAxis,
            0x06..=0x07 => TrueTypeInstruction::SetProjectionVectorToLine,
            0x76 => TrueTypeInstruction::SuperRound,
            0x10 => TrueTypeInstruction::SetReferencePoint0,
            0x11 => TrueTypeInstruction::SetReferencePoint1,
            0x12 => TrueTypeInstruction::SetReferencePoint2,
            0x1F => TrueTypeInstruction::SetSingleWidth,
            0x1E => TrueTypeInstruction::SetSingleWidthCutIn,
            0x61 => TrueTypeInstruction::Subtract,
            0x00..=0x01 => TrueTypeInstruction::SetFreedomAndProjectionVectorsToCoordinateAxis(b),
            0x23 => TrueTypeInstruction::Swap,
            0x13 => TrueTypeInstruction::SetZonePointer0,
            0x14 => TrueTypeInstruction::SetZonePointer1,
            0x15 => TrueTypeInstruction::SetZonePointer2,
            0x16 => TrueTypeInstruction::SetZonePointerS,
            0x29 => TrueTypeInstruction::UnTouchPoint,
            0x70 => TrueTypeInstruction::WriteControlValueTableInFunits,
            0x44 => TrueTypeInstruction::WriteControlValueTableInPixels,
            0x42 => TrueTypeInstruction::WriteStore,
            _ => todo!("{:x?}", b),
        })
    }
}

pub struct TrueTypeInterpreter<'a> {
    instruction_stream: InstructionStream,
    interpreter_stack: Vec<u32>,
    graphics_state: TrueTypeGraphicsState,
    ttf_file: ParsedTrueTypeFontFile<'a>,
    storage_area: Vec<u32>,
    original_positions: Vec<Point>,
    glyph_zone: Vec<Point>,
    twilight_zone: Vec<Point>,
}

struct PointIterator<'a> {
    x_coords: &'a [i16],
    y_coords: &'a [i16],
    flags: &'a [u8],
    cursor: usize,
    queue: VecDeque<(Point, u8)>,
    last_point: Point,
    last_on_curve: bool,
    first_point_on_curve: Option<Point>,
}

impl<'a> PointIterator<'a> {
    pub fn new(x_coords: &'a [i16], y_coords: &'a [i16], flags: &'a [u8]) -> Self {
        assert_eq!(x_coords.len(), y_coords.len());
        assert_eq!(x_coords.len(), flags.len());

        if flags.len() >= 2 {
            assert!(flags[0] & OutlineFlag::ON_CURVE != 0 || flags[1] & OutlineFlag::ON_CURVE != 0);
        }

        Self {
            x_coords,
            y_coords,
            flags,
            cursor: 0,
            queue: VecDeque::new(),
            last_point: Point::origin(),
            last_on_curve: false,
            first_point_on_curve: None,
        }
    }

    fn next_point(&mut self) -> Option<Point> {
        if let Some((point, flag)) = self.queue.pop_front() {
            return Some(point);
        }

        let is_last = self.x_coords.get(self.cursor).is_none();

        if is_last && !self.last_on_curve {
            self.last_on_curve = true;
            return self.first_point_on_curve;
        }

        let x = *self.x_coords.get(self.cursor)? as f32 / 2048.0;
        let y = *self.y_coords.get(self.cursor)? as f32 / 2048.0;
        let flag = *self.flags.get(self.cursor)?;

        let mut point = Point { x, y };
        let on_curve = flag & OutlineFlag::ON_CURVE != 0;

        if self.cursor == 0 && !on_curve {
            self.queue.push_back((point, flag));

            self.last_on_curve = true;
            self.cursor += 1;

            self.first_point_on_curve = Some(point.midpoint(Point::origin()));

            return Some(point.midpoint(Point::origin()));
        } else {
            self.first_point_on_curve.get_or_insert(point);
        }

        self.last_point = point;

        if (on_curve && self.last_on_curve) || (!on_curve && !self.last_on_curve) {
            self.queue.push_back((point, flag));

            point = point.midpoint(self.last_point);
        }

        self.last_on_curve = on_curve;

        self.cursor += 1;

        Some(point)
    }
}

impl<'a> Iterator for PointIterator<'a> {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_point()
    }
}

impl<'a> TrueTypeInterpreter<'a> {
    pub fn new(ttf_file: ParsedTrueTypeFontFile<'a>) -> Self {
        let storage_area_size = ttf_file.max_storage();
        let max_twilight_points = ttf_file.max_twilight_points();
        Self {
            instruction_stream: InstructionStream::new(Vec::new()),
            interpreter_stack: Vec::new(),
            graphics_state: TrueTypeGraphicsState::default(),
            ttf_file,
            storage_area: vec![0; storage_area_size as usize],
            original_positions: Vec::new(),
            glyph_zone: Vec::new(),
            twilight_zone: vec![Point::origin(); max_twilight_points as usize],
        }
    }

    pub fn render_glyph(&mut self, char_code: u32) -> anyhow::Result<Glyph> {
        self.reset();

        let ttf_glyph = self.ttf_file.glyph(char_code)?;

        let mut relative = Point::origin();

        let simple = match ttf_glyph {
            TrueTypeGlyph::Simple(simple) => simple,
            TrueTypeGlyph::Compound(_) => todo!(),
        };

        let mut points = PointIterator::new(&simple.x_coords, &simple.y_coords, &simple.flags);

        // todo: initialize glyph zone and initial positions
        self.instruction_stream = InstructionStream::new(simple.instructions);

        let mut paths = Vec::new();

        let mut start = 0;
        for contour_end in simple.end_points_of_contours {
            let mut path = None;
            let contour_end = contour_end as usize;

            let x_coords = &simple.x_coords[start..contour_end];
            let y_coords = &simple.y_coords[start..contour_end];
            let flags = &simple.flags[start..contour_end];

            let mut points = PointIterator::new(x_coords, y_coords, flags);
            // assert_eq!(num_points.len() % 2, 1);

            let p1 = points.next();

            while let Some(c1) = points.next() {
                let p2 = match points.next() {
                    Some(v) => v,
                    None => continue,
                };
                let path = path.get_or_insert_with(|| Path::new(p1.unwrap()));

                path.quadratic_curve_to(c1, p2);
            }

            if let Some(mut path) = path.take() {
                path.close_path();
                paths.push(path);
            }

            start = contour_end;
        }

        // self.execute().unwrap();

        // assert!(self.interpreter_stack.is_empty());

        let outline = Outline { paths };

        Ok(Glyph {
            width_vector: outline.bounding_box().max - outline.bounding_box().min,
            outline,
        })
    }

    pub fn reset(&mut self) {
        self.interpreter_stack.clear();
        self.graphics_state = TrueTypeGraphicsState::default();
        self.storage_area.fill(0);
        self.twilight_zone.fill(Point::origin());
        self.original_positions.clear();
        self.glyph_zone.clear();
    }

    fn pop(&mut self) -> anyhow::Result<u32> {
        self.interpreter_stack
            .pop()
            .ok_or(anyhow!("stack underflow"))
    }

    fn original_position(&self, zone: Zone, idx: u32) -> Point {
        match zone {
            Zone::Twilight => Point::origin(),
            Zone::Glyph => self.original_positions[idx as usize],
        }
    }

    fn zone(&self, zone: Zone) -> &[Point] {
        match zone {
            Zone::Twilight => &self.twilight_zone,
            Zone::Glyph => &self.glyph_zone,
        }
    }

    fn pop_f26dot6(&mut self) -> anyhow::Result<F26Dot6> {
        let n = self.pop()?;

        Ok(F26Dot6::from_bits(i32::from_be_bytes(n.to_be_bytes())))
    }

    fn execute(&mut self) -> anyhow::Result<()> {
        while let Some(inst) = self.instruction_stream.next_instruction() {
            match inst {
                TrueTypeInstruction::AdjustAngle => todo!(),
                TrueTypeInstruction::AbsoluteValue => todo!(),
                TrueTypeInstruction::Add => todo!(),
                TrueTypeInstruction::AlignPoints => todo!(),
                TrueTypeInstruction::AlignToReferencePoint => self.alignrp()?,
                TrueTypeInstruction::And => todo!(),
                TrueTypeInstruction::Call => self.call()?,
                TrueTypeInstruction::Ceiling => todo!(),
                TrueTypeInstruction::CopyIndex => todo!(),
                TrueTypeInstruction::Clear => todo!(),
                TrueTypeInstruction::Debug => todo!(),
                TrueTypeInstruction::DeltaC1 => todo!(),
                TrueTypeInstruction::DeltaC2 => todo!(),
                TrueTypeInstruction::DeltaC3 => todo!(),
                TrueTypeInstruction::DeltaP1 => self.delta_p1()?,
                TrueTypeInstruction::DeltaP2 => self.delta_p2()?,
                TrueTypeInstruction::DeltaP3 => self.delta_p3()?,
                TrueTypeInstruction::Depth => todo!(),
                TrueTypeInstruction::Div => todo!(),
                TrueTypeInstruction::Dup => todo!(),
                TrueTypeInstruction::EndIf => todo!(),
                TrueTypeInstruction::Else => todo!(),
                TrueTypeInstruction::EndFunctionDefinition => todo!(),
                TrueTypeInstruction::Equal => todo!(),
                TrueTypeInstruction::Even => todo!(),
                TrueTypeInstruction::BeginFunctionDefinition => todo!(),
                TrueTypeInstruction::FlipOff => todo!(),
                TrueTypeInstruction::FlipOn => todo!(),
                TrueTypeInstruction::FlipPoint => todo!(),
                TrueTypeInstruction::FlipRangeOff => todo!(),
                TrueTypeInstruction::FlipRangeOn => todo!(),
                TrueTypeInstruction::Floor => todo!(),
                TrueTypeInstruction::GetProjectionVectorCoordinate => todo!(),
                TrueTypeInstruction::GetInfo => todo!(),
                TrueTypeInstruction::GetFreedomVector => todo!(),
                TrueTypeInstruction::GetProjectionVector => todo!(),
                TrueTypeInstruction::GreaterThan => todo!(),
                TrueTypeInstruction::GreaterThanOrEqual => todo!(),
                TrueTypeInstruction::InstructionDefinition => todo!(),
                TrueTypeInstruction::If => todo!(),
                TrueTypeInstruction::InstructionControl => todo!(),
                TrueTypeInstruction::InterpolatePoint => self.ip()?,
                TrueTypeInstruction::IntersectLines => todo!(),
                TrueTypeInstruction::InterpolateUntouchedPoints(a) => self.iup(a)?,
                TrueTypeInstruction::JumpRelative => todo!(),
                TrueTypeInstruction::JumpRelativeOnFalse => todo!(),
                TrueTypeInstruction::JumpRelativeOnTrue => todo!(),
                TrueTypeInstruction::LoopAndCall => todo!(),
                TrueTypeInstruction::LessThan => todo!(),
                TrueTypeInstruction::LessThanOrEqual => todo!(),
                TrueTypeInstruction::Max => todo!(),
                TrueTypeInstruction::MeasureDistance => todo!(),
                TrueTypeInstruction::MoveDirectAbsolutePoint(a) => self.mdap(a)?,
                TrueTypeInstruction::MoveDirectRelativePoint(a) => self.mdrp(a)?,
                TrueTypeInstruction::MoveIndirectAbsolutePoint(a) => self.miap(a)?,
                TrueTypeInstruction::Min => todo!(),
                TrueTypeInstruction::MoveIndexed => todo!(),
                TrueTypeInstruction::MoveIndirectRelativePoint(abcde) => self.mirp(abcde)?,
                TrueTypeInstruction::MeasurePixelsPerEm => todo!(),
                TrueTypeInstruction::MeasurePointSize => todo!(),
                TrueTypeInstruction::MoveStackIndirectRelativePoint => todo!(),
                TrueTypeInstruction::Multiply => todo!(),
                TrueTypeInstruction::Negate => todo!(),
                TrueTypeInstruction::NotEqual => todo!(),
                TrueTypeInstruction::LogicalNot => todo!(),
                TrueTypeInstruction::PushNBytes => self.push_n_bytes()?,
                TrueTypeInstruction::PushNWords => todo!(),
                TrueTypeInstruction::NoRound => todo!(),
                TrueTypeInstruction::Odd => todo!(),
                TrueTypeInstruction::LogicalOr => todo!(),
                TrueTypeInstruction::Pop => todo!(),
                TrueTypeInstruction::PushBytes(abc) => self.push_bytes(abc)?,
                TrueTypeInstruction::PushWords => todo!(),
                TrueTypeInstruction::ReadControlValueTableEntry => todo!(),
                TrueTypeInstruction::RoundDownToGrid => todo!(),
                TrueTypeInstruction::RoundOff => self.roff()?,
                TrueTypeInstruction::Roll => todo!(),
                TrueTypeInstruction::Round => todo!(),
                TrueTypeInstruction::ReadStore => todo!(),
                TrueTypeInstruction::RoundToDoubleGrid => self.rtdg()?,
                TrueTypeInstruction::RoundToGrid => self.rtg()?,
                TrueTypeInstruction::RoundToHalfGrid => self.rthg()?,
                TrueTypeInstruction::RoundUpToGrid => self.rutg()?,
                TrueTypeInstruction::SuperRound45Deg => todo!(),
                TrueTypeInstruction::SetAngleWeight => todo!(),
                TrueTypeInstruction::ScanConversionControl => todo!(),
                TrueTypeInstruction::ScanType => todo!(),
                TrueTypeInstruction::SetsCoordinateFromStack => todo!(),
                TrueTypeInstruction::SetControlValueTableCutIn => todo!(),
                TrueTypeInstruction::SetDeltaBase => self.sdb()?,
                TrueTypeInstruction::SetDualProjectionVector => todo!(),
                TrueTypeInstruction::SetDeltaShift => self.sds()?,
                TrueTypeInstruction::SetFreedomVectorFromStack => todo!(),
                TrueTypeInstruction::SetFreedomVectorToCoordinateAxis => todo!(),
                TrueTypeInstruction::SetFreedomVectorToLine => todo!(),
                TrueTypeInstruction::SetFreedomVectorToProjectionVector => todo!(),
                TrueTypeInstruction::ShiftContourUsingReferencePoint => todo!(),
                TrueTypeInstruction::ShiftPointUsingReferencePoint(a) => self.shp(a)?,
                TrueTypeInstruction::ShiftPointByPixels => todo!(),
                TrueTypeInstruction::ShiftZoneUsingReferencePoint => todo!(),
                TrueTypeInstruction::SetLoop => self.sloop()?,
                TrueTypeInstruction::SetMinimumDistance => todo!(),
                TrueTypeInstruction::SetProjectionVectorFromStack => todo!(),
                TrueTypeInstruction::SetProjectionVectorToCoordinateAxis => todo!(),
                TrueTypeInstruction::SetProjectionVectorToLine => todo!(),
                TrueTypeInstruction::SuperRound => todo!(),
                TrueTypeInstruction::SetReferencePoint0 => self.srp0()?,
                TrueTypeInstruction::SetReferencePoint1 => self.srp1()?,
                TrueTypeInstruction::SetReferencePoint2 => self.srp2()?,
                TrueTypeInstruction::SetSingleWidth => todo!(),
                TrueTypeInstruction::SetSingleWidthCutIn => todo!(),
                TrueTypeInstruction::Subtract => todo!(),
                TrueTypeInstruction::SetFreedomAndProjectionVectorsToCoordinateAxis(a) => {
                    self.svtca(a)?
                }
                TrueTypeInstruction::Swap => todo!(),
                TrueTypeInstruction::SetZonePointer0 => self.szp0()?,
                TrueTypeInstruction::SetZonePointer1 => self.szp1()?,
                TrueTypeInstruction::SetZonePointer2 => self.szp2()?,
                TrueTypeInstruction::SetZonePointerS => self.szps()?,
                TrueTypeInstruction::UnTouchPoint => todo!(),
                TrueTypeInstruction::WriteControlValueTableInFunits => todo!(),
                TrueTypeInstruction::WriteControlValueTableInPixels => todo!(),
                TrueTypeInstruction::WriteStore => todo!(),
            }
        }

        Ok(())
    }
}

impl<'a> TrueTypeInterpreter<'a> {
    fn push_bytes(&mut self, abc: u8) -> anyhow::Result<()> {
        let (a, b, c) = ((abc >> 2) & 1, (abc >> 1) & 1, abc & 1);
        let n = 4 * a + 2 * b + c;

        for _ in 0..=n {
            let b = self.instruction_stream.next_byte().unwrap() as u32;
            self.interpreter_stack.push(b);
        }

        Ok(())
    }

    fn push_n_bytes(&mut self) -> anyhow::Result<()> {
        let n = self.instruction_stream.next_byte().unwrap();

        for _ in 0..n {
            let b = self.instruction_stream.next_byte().unwrap() as u32;
            self.interpreter_stack.push(b);
        }

        Ok(())
    }

    fn szp0(&mut self) -> anyhow::Result<()> {
        let zone_number = self.pop()?;

        self.graphics_state.zp0 = match zone_number {
            0 => Zone::Twilight,
            1 => Zone::Glyph,
            _ => anyhow::bail!("invalid zone: {:?}", zone_number),
        };

        Ok(())
    }

    fn szp1(&mut self) -> anyhow::Result<()> {
        let zone_number = self.pop()?;

        self.graphics_state.zp1 = match zone_number {
            0 => Zone::Twilight,
            1 => Zone::Glyph,
            _ => anyhow::bail!("invalid zone: {:?}", zone_number),
        };

        Ok(())
    }

    fn szp2(&mut self) -> anyhow::Result<()> {
        let zone_number = self.pop()?;

        self.graphics_state.zp2 = match zone_number {
            0 => Zone::Twilight,
            1 => Zone::Glyph,
            _ => anyhow::bail!("invalid zone: {:?}", zone_number),
        };

        Ok(())
    }

    fn szps(&mut self) -> anyhow::Result<()> {
        let zone = match self.pop()? {
            0 => Zone::Twilight,
            1 => Zone::Glyph,
            n => anyhow::bail!("invalid zone: {:?}", n),
        };

        self.graphics_state.zp0 = zone;
        self.graphics_state.zp1 = zone;
        self.graphics_state.zp2 = zone;

        Ok(())
    }

    fn svtca(&mut self, a: u8) -> anyhow::Result<()> {
        match a {
            // set vectors to the y-axis
            0 => {
                self.graphics_state.freedom_vector = Vector::y_axis();
                self.graphics_state.projection_vector = Vector::y_axis();
            }
            // set vectors to the x-axis
            1 => {
                self.graphics_state.freedom_vector = Vector::x_axis();
                self.graphics_state.projection_vector = Vector::x_axis();
            }
            _ => anyhow::bail!("invalid flag {:?}", a),
        }

        Ok(())
    }

    fn sloop(&mut self) -> anyhow::Result<()> {
        let n = self.pop()?;

        self.graphics_state.loop_counter = n;

        Ok(())
    }

    fn srp0(&mut self) -> anyhow::Result<()> {
        let point_number = self.pop()?;

        self.graphics_state.rp0 = point_number;

        Ok(())
    }

    fn srp1(&mut self) -> anyhow::Result<()> {
        let point_number = self.pop()?;

        self.graphics_state.rp1 = point_number;

        Ok(())
    }

    fn srp2(&mut self) -> anyhow::Result<()> {
        let point_number = self.pop()?;

        self.graphics_state.rp2 = point_number;

        Ok(())
    }

    fn rtdg(&mut self) -> anyhow::Result<()> {
        self.graphics_state.round_state = RoundState::DownToGrid;

        Ok(())
    }

    fn rthg(&mut self) -> anyhow::Result<()> {
        self.graphics_state.round_state = RoundState::ToHalfGrid;

        Ok(())
    }

    fn rutg(&mut self) -> anyhow::Result<()> {
        self.graphics_state.round_state = RoundState::UpToGrid;

        Ok(())
    }

    fn roff(&mut self) -> anyhow::Result<()> {
        self.graphics_state.round_state = RoundState::Off;

        Ok(())
    }

    fn rtg(&mut self) -> anyhow::Result<()> {
        self.graphics_state.round_state = RoundState::ToGrid;

        Ok(())
    }

    fn alignrp(&mut self) -> anyhow::Result<()> {
        for _ in 0..self.graphics_state.loop_counter {
            let point_number = self.pop()?;
        }

        println!("ALIGNRP not yet implemented");

        Ok(())
    }

    fn call(&mut self) -> anyhow::Result<()> {
        let function_identifier = self.pop()?;

        println!("CALL not yet implemented");

        Ok(())
    }

    fn delta_p1(&mut self) -> anyhow::Result<()> {
        let n = self.pop()?;

        let mut pairs = Vec::with_capacity(n as usize);

        for _ in 0..n {
            let arg = self.pop()?;
            let point_number = self.pop()?;

            pairs.push((arg, point_number));
        }

        println!("DELTAP1 not yet implemented");

        Ok(())
    }

    fn delta_p2(&mut self) -> anyhow::Result<()> {
        let n = self.pop()?;

        let mut pairs = Vec::with_capacity(n as usize);

        for _ in 0..n {
            let arg = self.pop()?;
            let point_number = self.pop()?;

            pairs.push((arg, point_number));
        }

        println!("DELTAP2 not yet implemented");

        Ok(())
    }

    fn delta_p3(&mut self) -> anyhow::Result<()> {
        let n = self.pop()?;

        let mut pairs = Vec::with_capacity(n as usize);

        for _ in 0..n {
            let arg = self.pop()?;
            let point_number = self.pop()?;

            pairs.push((arg, point_number));
        }

        println!("DELTAP3 not yet implemented");

        Ok(())
    }

    fn ip(&mut self) -> anyhow::Result<()> {
        for _ in 0..self.graphics_state.loop_counter {
            let point_number = self.pop()?;
        }

        println!("IP not yet implemented");

        Ok(())
    }

    fn iup(&mut self, a: u8) -> anyhow::Result<()> {
        println!("IUP not yet implemented");

        Ok(())
    }

    fn sdb(&mut self) -> anyhow::Result<()> {
        let n = self.pop()?;

        self.graphics_state.delta_base = n;

        Ok(())
    }

    fn sds(&mut self) -> anyhow::Result<()> {
        let n = self.pop()?;

        self.graphics_state.delta_shift = n;

        Ok(())
    }

    fn shp(&mut self, a: u8) -> anyhow::Result<()> {
        let delta = match a {
            // uses rp2 in the zone pointed to by zp1
            0 => {
                let original =
                    self.original_position(self.graphics_state.zp1, self.graphics_state.rp2);
                let zone = self.zone(self.graphics_state.zp1);
                let current = zone[self.graphics_state.rp2 as usize];

                current - original
            }
            // uses rp1 in the zone pointed to by zp0
            1 => {
                let original =
                    self.original_position(self.graphics_state.zp0, self.graphics_state.rp1);
                let zone = self.zone(self.graphics_state.zp0);
                let current = zone[self.graphics_state.rp1 as usize];

                current - original
            }
            _ => anyhow::bail!("invalid flag {:?}", a),
        };

        for _ in 0..self.graphics_state.loop_counter {
            let point_number = self.pop()?;

            self.glyph_zone[point_number as usize] += delta;
        }

        Ok(())
    }

    fn mdap(&mut self, a: u8) -> anyhow::Result<()> {
        let point_number = self.pop()?;

        println!("MDAP not yet implemented");

        Ok(())
    }

    fn mdrp(&mut self, a: u8) -> anyhow::Result<()> {
        let point_number = self.pop()?;

        println!("MDRP not yet implemented");

        Ok(())
    }

    fn mirp(&mut self, abcde: u8) -> anyhow::Result<()> {
        let cvt_entry_number = self.pop()?;
        let point_number = self.pop()?;

        println!("MIRP not yet implemented");

        Ok(())
    }

    fn miap(&mut self, a: u8) -> anyhow::Result<()> {
        let cvt_entry_number = self.pop_f26dot6()?;
        let point_number = self.pop()?;

        let cvt_entry = self
            .ttf_file
            .cvt_entry(cvt_entry_number.to_num::<usize>())
            .unwrap();

        self.graphics_state.rp0 = point_number;
        self.graphics_state.rp1 = point_number;

        println!("MIAP not yet implemented");

        Ok(())
    }
}

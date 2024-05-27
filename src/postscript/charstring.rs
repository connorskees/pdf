use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap},
};

use crate::{font::Glyph, geometry::path_builder::PathBuilder};

use super::{
    decode::decrypt_charstring,
    font::{Encoding, Type1PostscriptFont},
    graphics_operator::GraphicsOperator,
    interpreter::PostscriptInterpreter,
    object::{PostScriptArray, PostScriptDictionary, PostScriptObject, PostScriptString},
    PostScriptError, PostScriptResult,
};

#[derive(Debug)]
pub(crate) struct CharString(Vec<CharStringElement>);

#[derive(Debug)]
struct CharStringStack {
    stack: [f32; 24],
    end: u8,
}

impl CharStringStack {
    pub fn new() -> Self {
        Self {
            // we initialize all values to zero, but zero is not used as a
            // sentinel value
            stack: [0.0; 24],
            end: 0,
        }
    }

    pub fn pop(&mut self) -> PostScriptResult<f32> {
        if self.end == 0 {
            anyhow::bail!(PostScriptError::StackUnderflow);
        }

        self.end -= 1;

        Ok(self.stack[self.end as usize])
    }

    pub fn pop_front(&mut self) -> PostScriptResult<f32> {
        if self.end == 0 {
            anyhow::bail!(PostScriptError::StackUnderflow);
        }

        self.end -= 1;

        let v = self.stack[0];

        self.stack.rotate_left(1);

        Ok(v)
    }

    pub fn push(&mut self, n: f32) -> PostScriptResult<()> {
        if self.end >= 24 {
            anyhow::bail!(PostScriptError::StackOverflow);
        }

        self.stack[self.end as usize] = n;

        self.end += 1;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.end == 0
    }

    pub fn clear(&mut self) {
        self.end = 0;
    }
}

#[derive(Debug, Clone, Copy)]
enum CharStringElement {
    Int(i32),
    Op(GraphicsOperator),
}

impl CharString {
    pub fn parse(b: &mut [u8], in_pfb: bool) -> PostScriptResult<Self> {
        let mut b = decrypt_charstring(b);

        // external pfb files always have padding bytes. embedded type 1 fonts
        // somtimes omit them
        if b.get(..4) == Some(&[0, 0, 0, 0]) || in_pfb {
            b = b[4..].to_vec();
        }

        let mut i = 0;
        let mut elems = Vec::new();

        while i < b.len() {
            let byte = b[i];

            i += 1;

            match byte {
                v @ 0..=31 => match v {
                    // y dy hstem (1)
                    1 => elems.push(CharStringElement::Op(GraphicsOperator::HorizontalStem)),

                    // x dx vstem (3)
                    3 => elems.push(CharStringElement::Op(GraphicsOperator::VerticalStem)),

                    // dy vmoveto (4)
                    4 => elems.push(CharStringElement::Op(GraphicsOperator::VerticalMoveTo)),

                    // dx dy rlineto (5)
                    5 => elems.push(CharStringElement::Op(GraphicsOperator::RelativeLineTo)),

                    // dx hlineto (6)
                    6 => elems.push(CharStringElement::Op(GraphicsOperator::HorizontalLineTo)),

                    // dy vlineto (7)
                    7 => elems.push(CharStringElement::Op(GraphicsOperator::VerticalLineTo)),

                    //  dx1 dy1 dx2 dy2 dx3 dy3 rrcurveto (8)
                    8 => elems.push(CharStringElement::Op(
                        GraphicsOperator::RelativeRelativeCurveTo,
                    )),
                    9 => elems.push(CharStringElement::Op(GraphicsOperator::ClosePath)),
                    10 => elems.push(CharStringElement::Op(GraphicsOperator::CallSubroutine)),
                    11 => elems.push(CharStringElement::Op(GraphicsOperator::Return)),
                    12 => {
                        match b[i] {
                            0 => elems.push(CharStringElement::Op(GraphicsOperator::DotSection)),
                            1 => elems.push(CharStringElement::Op(GraphicsOperator::VerticalStem3)),
                            2 => {
                                elems.push(CharStringElement::Op(GraphicsOperator::HorizontalStem3))
                            }
                            6 => elems.push(CharStringElement::Op(
                                GraphicsOperator::StandardEncodingAccentedCharacter,
                            )),
                            7 => elems
                                .push(CharStringElement::Op(GraphicsOperator::SideBearingWidth)),
                            12 => elems.push(CharStringElement::Op(GraphicsOperator::Div)),
                            16 => elems
                                .push(CharStringElement::Op(GraphicsOperator::CallOtherSubroutine)),
                            17 => elems.push(CharStringElement::Op(GraphicsOperator::Pop)),
                            33 => {
                                elems.push(CharStringElement::Op(GraphicsOperator::SetCurrentPoint))
                            }
                            v => todo!("INVALID OP CODE: 12 {}", v),
                        }

                        i += 1;
                    }
                    13 => {
                        elems.push(CharStringElement::Op(
                            GraphicsOperator::HorizontalSideBearingWidth,
                        ));
                    }
                    14 => elems.push(CharStringElement::Op(GraphicsOperator::EndChar)),
                    21 => elems.push(CharStringElement::Op(GraphicsOperator::RelativeMoveTo)),
                    22 => elems.push(CharStringElement::Op(GraphicsOperator::HorizontalMoveTo)),
                    30 => elems.push(CharStringElement::Op(
                        GraphicsOperator::VerticalHorizontalCurveTo,
                    )),
                    31 => elems.push(CharStringElement::Op(
                        GraphicsOperator::HorizontalVerticalCurveTo,
                    )),
                    v => todo!("INVALID OP CODE: {}", v),
                },

                // A charstring byte containing a value, v, between 32 and
                // 246 inclusive, indicates the integer v − 139. Thus, the
                // integer values from −107 through 107 inclusive may be
                // encoded in a single byte
                v @ 32..=246 => elems.push(CharStringElement::Int(v as i32 - 139)),

                // A charstring byte containing a value, v, between 247 and
                // 250 inclusive, indicates an integer involving the next byte,
                // w, according to the formula:
                //
                //   [(v − 247) × 256] + w + 108
                //
                // Thus, the integer values between 108 and 1131 inclusive
                // can be encoded in 2 bytes in this manner
                v @ 247..=250 => {
                    let w = b[i] as i32;
                    let int = ((v as i32 - 247) * 256) + w + 108;

                    i += 1;

                    elems.push(CharStringElement::Int(int));
                }

                // A charstring byte containing a value, v, between 251 and
                // 254 inclusive, indicates an integer involving the next
                // byte, w, according to the formula:
                //
                // − [(v − 251) × 256] − w − 108
                //
                // Thus, the integer values between −1131 and −108 inclusive
                // can be encoded in 2 bytes in this manner
                v @ 251..=254 => {
                    let w = b[i] as i32;
                    let int = -((v as i32 - 251) * 256) - w - 108;

                    i += 1;

                    elems.push(CharStringElement::Int(int));
                }

                // Finally, if the charstring byte contains the value 255,
                // the next four bytes indicate a two’s complement signed integer.
                // The first of these four bytes contains the highest order
                // bits, the second byte contains the next higher order bits
                // and the fourth byte contains the lowest order bits. Thus,
                // any 32-bit signed integer may be encoded in 5 bytes in this
                // manner (the 255 byte plus 4 more bytes)
                255 => {
                    let bytes = &b[i..(i + 4)];

                    i += 5;

                    let int = i32::from_be_bytes(bytes.try_into().unwrap());

                    elems.push(CharStringElement::Int(int));
                }
            }
        }

        Ok(Self(elems))
    }
}

#[derive(Debug)]
pub(crate) struct CharStrings(HashMap<PostScriptString, CharString>);

impl CharStrings {
    pub(super) fn from_dict(
        dict: PostScriptDictionary,
        interpreter: &mut PostscriptInterpreter,
    ) -> PostScriptResult<Self> {
        let mut char_strings = HashMap::new();

        for (key, value) in dict.into_iter() {
            let char_string = match value {
                PostScriptObject::String(s) => {
                    let mut s = interpreter.get_str(s).clone().into_bytes();
                    CharString::parse(&mut s, interpreter.in_pfb)?
                }
                _ => anyhow::bail!(PostScriptError::TypeCheck),
            };

            char_strings.insert(key, char_string);
        }

        Ok(Self(char_strings))
    }

    pub(crate) fn get_by_name(&self, s: &PostScriptString) -> Option<&CharString> {
        self.0.get(s).or_else(|| {
            self.0
                .get(&PostScriptString::from_bytes(b".notdef".to_vec()))
        })
    }

    pub(crate) fn is_codepoint_defined(&self, c: u8) -> bool {
        self.0.contains_key(&PostScriptString::from_bytes(vec![c]))
    }
}

pub(crate) struct CharStringPainter<'a> {
    path_builder: PathBuilder,
    has_current_point: bool,
    subroutines: &'a [CharString],
    other_subroutines: &'a [PostScriptArray],
    operand_stack: CharStringStack,
    interpreter: PostscriptInterpreter<'a>,
    encoding: &'a Encoding,
    char_strings: &'a CharStrings,
    gylph_cache: BTreeMap<u32, Glyph>,
}

impl<'a> CharStringPainter<'a> {
    pub fn new(font: &'a Type1PostscriptFont) -> Self {
        Self {
            path_builder: PathBuilder::new(),
            has_current_point: false,
            subroutines: font.private.subroutines.as_deref().unwrap_or(&[]),
            other_subroutines: font.private.other_subroutines.as_deref().unwrap_or(&[]),
            encoding: &font.encoding,
            char_strings: &font.char_strings,
            operand_stack: CharStringStack::new(),
            interpreter: PostscriptInterpreter::new(&[]),
            gylph_cache: BTreeMap::new(),
        }
    }

    fn reinit(&mut self) {
        self.path_builder = PathBuilder::new();
    }

    pub fn evaluate(&mut self, char_code: u32) -> PostScriptResult<Glyph> {
        if let Some(glyph) = self.gylph_cache.get(&char_code) {
            return Ok(glyph.clone());
        }

        self.reinit();

        let charstring_name = self.encoding.get(char_code);

        if let Some(charstring) = self.char_strings.get_by_name(charstring_name.borrow()) {
            let glyph = self.evaluate_as_subroutine(charstring)?;

            self.gylph_cache.insert(char_code, glyph.clone());

            Ok(glyph)
        } else {
            Ok(Glyph::empty())
        }
    }

    fn evaluate_as_subroutine(&mut self, c: &CharString) -> PostScriptResult<Glyph> {
        for &elem in &c.0 {
            match elem {
                CharStringElement::Int(n) => self.operand_stack.push(n as f32)?,
                CharStringElement::Op(GraphicsOperator::HorizontalStem) => {
                    let y = self.operand_stack.pop_front()?;
                    let dy = self.operand_stack.pop_front()?;

                    self.path_builder.horizontal_stem(y, dy);

                    self.operand_stack.clear();
                }
                CharStringElement::Op(GraphicsOperator::VerticalStem) => {
                    let x = self.operand_stack.pop_front()?;
                    let dx = self.operand_stack.pop_front()?;

                    self.path_builder.vertical_stem(x, dx);

                    self.operand_stack.clear();
                }
                CharStringElement::Op(GraphicsOperator::VerticalMoveTo) => {
                    let dy = self.operand_stack.pop_front()?;

                    self.operand_stack.clear();

                    self.path_builder.relative_move_to(0.0, dy);
                }
                CharStringElement::Op(GraphicsOperator::RelativeLineTo) => {
                    let dx = self.operand_stack.pop_front()?;
                    let dy = self.operand_stack.pop_front()?;

                    self.operand_stack.clear();

                    self.path_builder.relative_line_to(dx, dy);
                }
                CharStringElement::Op(GraphicsOperator::HorizontalLineTo) => {
                    let dx = self.operand_stack.pop_front()?;

                    self.path_builder.horizontal_line_to(dx);

                    self.operand_stack.clear();
                }
                CharStringElement::Op(GraphicsOperator::VerticalLineTo) => {
                    let dy = self.operand_stack.pop_front()?;

                    self.path_builder.vertical_line_to(dy);

                    self.operand_stack.clear();
                }
                CharStringElement::Op(GraphicsOperator::RelativeRelativeCurveTo) => {
                    let dx1 = self.operand_stack.pop_front()?;
                    let dy1 = self.operand_stack.pop_front()?;
                    let dx2 = self.operand_stack.pop_front()?;
                    let dy2 = self.operand_stack.pop_front()?;
                    let dx3 = self.operand_stack.pop_front()?;
                    let dy3 = self.operand_stack.pop_front()?;

                    self.path_builder
                        .relative_relative_curve_to(dx1, dy1, dx2, dy2, dx3, dy3);

                    self.operand_stack.clear();
                }
                CharStringElement::Op(GraphicsOperator::ClosePath) => {
                    self.path_builder.close_path()
                }
                CharStringElement::Op(GraphicsOperator::CallSubroutine) => {
                    let subr_number = self.operand_stack.pop()? as usize;

                    self.call_subroutine(subr_number)?;
                }
                CharStringElement::Op(GraphicsOperator::Return) => break,
                CharStringElement::Op(GraphicsOperator::DotSection) => todo!(),
                CharStringElement::Op(GraphicsOperator::VerticalStem3) => {
                    let x0 = self.operand_stack.pop_front()?;
                    let dx0 = self.operand_stack.pop_front()?;
                    let x1 = self.operand_stack.pop_front()?;
                    let dx1 = self.operand_stack.pop_front()?;
                    let x2 = self.operand_stack.pop_front()?;
                    let dx2 = self.operand_stack.pop_front()?;

                    self.operand_stack.clear();

                    self.path_builder.vertical_stem3(x0, dx0, x1, dx1, x2, dx2);
                }
                CharStringElement::Op(GraphicsOperator::HorizontalStem3) => {
                    let y0 = self.operand_stack.pop_front()?;
                    let dy0 = self.operand_stack.pop_front()?;
                    let y1 = self.operand_stack.pop_front()?;
                    let dy1 = self.operand_stack.pop_front()?;
                    let y2 = self.operand_stack.pop_front()?;
                    let dy2 = self.operand_stack.pop_front()?;

                    self.operand_stack.clear();

                    self.path_builder
                        .horizontal_stem3(y0, dy0, y1, dy1, y2, dy2);
                }
                #[allow(unused)]
                CharStringElement::Op(GraphicsOperator::StandardEncodingAccentedCharacter) => {
                    let asb = self.operand_stack.pop_front()?;
                    let adx = self.operand_stack.pop_front()?;
                    let ady = self.operand_stack.pop_front()?;
                    let bchar = self.operand_stack.pop_front()?;
                    let achar = self.operand_stack.pop_front()?;

                    self.operand_stack.clear();

                    todo!()
                }
                #[allow(unused)]
                CharStringElement::Op(GraphicsOperator::SideBearingWidth) => {
                    let sbx = self.operand_stack.pop_front()?;
                    let sby = self.operand_stack.pop_front()?;
                    let wx = self.operand_stack.pop_front()?;
                    let wy = self.operand_stack.pop_front()?;

                    self.operand_stack.clear();

                    todo!()
                }
                CharStringElement::Op(GraphicsOperator::Div) => {
                    let num1 = self.operand_stack.stack[0];
                    let num2 = self.operand_stack.stack[1];

                    self.operand_stack.push(num1 / num2)?;
                }
                CharStringElement::Op(GraphicsOperator::CallOtherSubroutine) => {
                    let other_subr_number = self.operand_stack.pop()?;
                    let num_of_args = self.operand_stack.pop()?;

                    let mut args = Vec::new();

                    for _ in 0..(num_of_args as u32) {
                        args.push(self.operand_stack.pop()?);
                    }

                    self.call_other_subroutine(other_subr_number as usize, args)?;
                }
                CharStringElement::Op(GraphicsOperator::Pop) => match self.interpreter.pop()? {
                    PostScriptObject::Float(n) => self.operand_stack.push(n)?,
                    PostScriptObject::Int(n) => self.operand_stack.push(n as f32)?,
                    obj => todo!("unexpected operand: {:#?}", obj),
                },
                CharStringElement::Op(GraphicsOperator::SetCurrentPoint) => todo!(),
                CharStringElement::Op(GraphicsOperator::HorizontalSideBearingWidth) => {
                    let side_bearing_x_coord = self.operand_stack.pop_front()?;
                    let width_vector_x_coord = self.operand_stack.pop_front()?;

                    self.path_builder
                        .hsbw(side_bearing_x_coord, width_vector_x_coord);

                    self.operand_stack.clear();
                }
                CharStringElement::Op(GraphicsOperator::EndChar) => {
                    // todo: rest of this operator
                    if !self.path_builder.current_path.subpaths.is_empty() {
                        self.path_builder
                            .outline
                            .paths
                            .push(self.path_builder.current_path.clone());
                    }
                    break;
                }
                CharStringElement::Op(GraphicsOperator::RelativeMoveTo) => {
                    let dx = self.operand_stack.pop_front()?;
                    let dy = self.operand_stack.pop_front()?;

                    self.path_builder.relative_move_to(dx, dy);

                    self.operand_stack.clear();
                }
                CharStringElement::Op(GraphicsOperator::HorizontalMoveTo) => {
                    let dx = self.operand_stack.pop_front()?;

                    self.operand_stack.clear();

                    self.path_builder.relative_move_to(dx, 0.0);
                }
                CharStringElement::Op(GraphicsOperator::VerticalHorizontalCurveTo) => {
                    let dy1 = self.operand_stack.pop_front()?;
                    let dx2 = self.operand_stack.pop_front()?;
                    let dy2 = self.operand_stack.pop_front()?;
                    let dx3 = self.operand_stack.pop_front()?;

                    self.path_builder
                        .vertical_horizontal_curve_to(dy1, dx2, dy2, dx3);

                    self.operand_stack.clear();
                }
                CharStringElement::Op(GraphicsOperator::HorizontalVerticalCurveTo) => {
                    let dx1 = self.operand_stack.pop_front()?;
                    let dx2 = self.operand_stack.pop_front()?;
                    let dy2 = self.operand_stack.pop_front()?;
                    let dy3 = self.operand_stack.pop_front()?;

                    self.path_builder
                        .horizontal_vertical_curve_to(dx1, dx2, dy2, dy3);

                    self.operand_stack.clear();
                }
            }
        }

        assert!(self.operand_stack.is_empty(), "{:?}", self.operand_stack);

        Ok(Glyph {
            outline: self.path_builder.outline.clone(),
            width_vector: self.path_builder.width_vector,
        })
    }

    fn call_subroutine(&mut self, subr_number: usize) -> PostScriptResult<()> {
        match subr_number {
            0..=3 => todo!("call special subroutine {subr_number}"),
            _ => {
                let subr = &self.subroutines[subr_number];

                self.evaluate_as_subroutine(subr)?;
            }
        }

        Ok(())
    }

    fn call_other_subroutine(
        &mut self,
        other_subr_number: usize,
        args: Vec<f32>,
    ) -> PostScriptResult<()> {
        match other_subr_number {
            0..=3 => {
                let subr = &self.other_subroutines[other_subr_number];

                for arg in args {
                    self.interpreter.push(PostScriptObject::Float(arg));
                }

                self.interpreter
                    .execute_procedure(subr.clone().into_inner())
                    .unwrap();
            }
            _ => todo!("use of reserved other subroutine idx"),
        }

        Ok(())
    }
}

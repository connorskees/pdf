use std::collections::VecDeque;

use crate::{font::Glyph, geometry::path_builder::PathBuilder, parse_binary::BinaryParser};

const WARN_ON_UNIMPLEMENTED_HINT: bool = false;

pub struct CffCharStringInterpreter<'a> {
    buffer: &'a [u8],
    cursor: usize,
    operand_stack: VecDeque<f32>,
    path_builder: PathBuilder,
    width: Option<f32>,
    first_stack_clearing_op: bool,

    // todo: track actual hints, not just count
    hint_count: usize,
}

impl<'a> CffCharStringInterpreter<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            cursor: 0,
            operand_stack: VecDeque::new(),
            path_builder: PathBuilder::new(),
            width: None,
            first_stack_clearing_op: true,
            hint_count: 0,
        }
    }

    fn maybe_calculate_width(&mut self) -> anyhow::Result<()> {
        if self.first_stack_clearing_op && self.operand_stack.len() == 1 {
            let width = self.pop()?;
            self.width = Some(width);
        }

        self.first_stack_clearing_op = false;

        anyhow::ensure!(self.operand_stack.is_empty());

        self.operand_stack.clear();

        Ok(())
    }

    fn push(&mut self, n: f32) {
        // println!("parser.push({}.0);", n);
        self.operand_stack.push_back(n);
    }

    fn pop(&mut self) -> anyhow::Result<f32> {
        self.operand_stack
            .pop_back()
            .ok_or(anyhow::anyhow!("stack underflow"))
    }

    fn pop_front(&mut self) -> anyhow::Result<f32> {
        let x = self
            .operand_stack
            .pop_front()
            .ok_or(anyhow::anyhow!("stack underflow"))?;
        Ok(x)
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
            28 => {
                let n = self.parse_i16()?;
                self.push(n as f32);
            }
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
            255 => {
                let n = self.parse_i32()?;
                self.push(n as f32);
            }
            _ => anyhow::bail!("invalid charstring operator: {:?}", b0),
        }

        Ok(())
    }

    pub fn evaluate(buffer: &'a [u8]) -> anyhow::Result<Glyph> {
        let mut parser = Self::new(buffer);

        while parser.peek().is_some() {
            match parser.next()? {
                // y dy {dya dyb}* hstem (1)
                1 => parser.hstem()?,
                // x dx {dxa dxb}* vstem (3)
                3 => parser.vstem()?,
                // dy1 vmoveto (4)
                4 => parser.vmoveto()?,
                // {dxa dya}+ rlineto (5)
                5 => parser.rlineto()?,
                // dx1 {dya dxb}* hlineto (6)
                // {dxa dyb}+ hlineto (6)
                6 => parser.hlineto()?,
                // dy1 {dxa dyb}* vlineto (7)
                // {dya dxb}+ vlineto (7)
                7 => parser.vlineto()?,
                // {dxa dya dxb dyb dxc dyc}+ rrcurveto (8)
                8 => parser.rrcurveto()?,
                // subr# callsubr (10) –
                10 => parser.callsubr()?,
                // – return (11) –
                11 => todo!(),
                12 => match parser.next()? {
                    // num1 num2 and (12 3) 1_or_0
                    3 => todo!(),
                    // num1 num2 or (12 4) 1_or_0
                    4 => todo!(),
                    // num1 not (12 5) 1_or_0
                    5 => todo!(),
                    // num abs (12 9) num2
                    9 => todo!(),
                    // num1 num2 add (12 10) sum
                    10 => todo!(),
                    // num1 num2 sub (12 11) difference
                    11 => todo!(),
                    // num1 num2 div (12 12) quotient
                    12 => todo!(),
                    // num neg (12 14) num2
                    14 => todo!(),
                    // num1 num2 eq (12 15) 1_or_0
                    15 => todo!(),
                    // num drop (12 18)
                    18 => todo!(),
                    // val i put (12 20)
                    20 => todo!(),
                    // i get (12 21) val
                    21 => todo!(),
                    // s1 s2 v1 v2 ifelse (12 22) s1_or_s2
                    22 => todo!(),
                    // random (12 23) num2
                    23 => todo!(),
                    // num1 num2 mul (12 24) product
                    24 => todo!(),
                    // num sqrt (12 26) num2
                    26 => todo!(),
                    // any dup (12 27) any any
                    27 => todo!(),
                    // num1 num2 exch (12 28) num2 num1
                    28 => todo!(),
                    // numX ... num0 i index (12 29) numX ... num0 num
                    29 => todo!(),
                    // num(N–1) ... num0 N J roll (12 30) num((J–1) mod N) ... num0
                    // num(N–1) ... num(J mod N)
                    30 => todo!(),
                    // dx1 dx2 dy2 dx3 dx4 dx5 dx6 hflex (12 34)
                    34 => parser.hflex()?,
                    // dx1 dy1 dx2 dy2 dx3 dy3 dx4 dy4 dx5 dy5 dx6 dy6 fd flex (12 35)
                    35 => parser.flex()?,
                    // dx1 dy1 dx2 dy2 dx3 dx4 dx5 dy5 dx6 hflex1 (12 36)
                    36 => parser.hflex1()?,
                    // dx1 dy1 dx2 dy2 dx3 dy3 dx4 dy4 dx5 dy5 d6 flex1 (12 37)
                    37 => parser.flex1()?,
                    b => anyhow::bail!("invalid top dict operator: 12 {:?}", b),
                },
                // – endchar (14)
                14 => parser.end_char()?,
                // y dy {dya dyb}* hstemhm (18)
                18 => parser.hstemhm()?,
                // hintmask (19 + mask)
                19 => parser.hintmask()?,
                // cntrmask (20 + mask)
                20 => parser.cntrmask()?,
                // dx1 dy1 rmoveto (21)
                21 => parser.rmoveto()?,
                // dx1 hmoveto (22)
                22 => parser.hmoveto()?,
                // x dx {dxa dxb}* vstemhm (23)
                23 => parser.vstemhm()?,
                // {dxa dya dxb dyb dxc dyc}+ dxd dyd rcurveline (24)
                24 => parser.rcurveline()?,
                // {dxa dya}+ dxb dyb dxc dyc dxd dyd rlinecurve (25)
                25 => parser.rlinecurve()?,
                // dx1? {dya dxb dyb dyc}+ vvcurveto (26)
                26 => parser.vvcurveto()?,
                // dy1? {dxa dxb dyb dxc}+ hhcurveto (27)
                27 => parser.hhcurveto()?,
                // globalsubr# callgsubr (29) –
                29 => parser.callgsubr()?,
                // dy1 dx2 dy2 dx3 {dxa dxb dyb dyc dyd dxe dye dxf}* dyf? vhcurveto (30)
                // {dya dxb dyb dxc dxd dxe dye dyf}+ dxf? vhcurveto (30)
                30 => parser.vhcurveto()?,
                // dx1 dx2 dy2 dy3 {dya dxb dyb dxc dxd dxe dye dyf}* dxf? hvcurveto (31)
                // {dxa dxb dyb dyc dyd dxe dye dxf}+ dyf? hvcurveto (31)
                31 => parser.hvcurveto()?,
                b @ 0..=31 => anyhow::bail!("unimplemented operator: {:?}", b),
                b0 => parser.parse_number(b0)?,
            }
        }

        anyhow::ensure!(parser.operand_stack.is_empty());

        Ok(Glyph {
            outline: parser.path_builder.outline.clone(),
            width_vector: parser.path_builder.width_vector,
        })
    }

    /// specifies one or more horizontal stem hints. This allows multiple pairs
    /// of numbers, limited by the stack depth, to be used as arguments to a
    /// single hstem operator
    ///
    /// It is required that the stems are encoded in ascending order (defined by
    /// increasing bottom edge). The encoded values are all relative; in the
    /// first pair, y is relative to 0, and dy specifies the distance from y. The
    /// first value of each subsequent pair is relative to the last edge defined
    /// by the previous pair
    ///
    /// A width of –20 specifies the top edge of an edge hint, and –21 specifies the
    /// bottom edge of an edge hint. All other negative widths have undefined
    /// meaning
    ///
    /// Horizontal stem hints must not overlap each other. If there is any overlap,
    /// the hintmask operator must be used immediately after the hint
    /// declarations to establish the desired nonoverlapping set of hints.
    /// hintmask may be used again later in the path to activate a different set
    /// of non-overlapping hints.
    #[allow(unused_variables)]
    fn hstem(&mut self) -> anyhow::Result<()> {
        let y = self.pop_front()?;
        let dy = self.pop_front()?;
        self.hint_count += 1;

        for _ in 0..self.operand_stack.len() / 2 {
            let dya = self.pop_front()?;
            let dyb = self.pop_front()?;
            self.hint_count += 1;
        }
        // println!("parser.hstem()?;");
        if WARN_ON_UNIMPLEMENTED_HINT {
            println!("unimplemented cff charstring hinting op: hstem");
        }
        self.maybe_calculate_width()?;
        Ok(())
    }

    /// has the same meaning as hstem (1), except that it must be used in place of
    /// hstem if the charstring contains one or more hintmask operators
    #[allow(unused_variables)]
    fn hstemhm(&mut self) -> anyhow::Result<()> {
        let y = self.pop_front()?;
        let dy = self.pop_front()?;
        self.hint_count += 1;

        for _ in 0..self.operand_stack.len() / 2 {
            let dya = self.pop_front()?;
            let dyb = self.pop_front()?;
            self.hint_count += 1;
        }
        // println!("parser.hstemhm()?;");
        if WARN_ON_UNIMPLEMENTED_HINT {
            println!("unimplemented cff charstring hinting op: hstemhm");
        }
        self.maybe_calculate_width()?;
        Ok(())
    }

    /// has the same meaning as vstem (3), except that it must be used in place of
    /// vstem if the charstring contains one or more hintmask operators
    #[allow(unused_variables)]
    fn vstemhm(&mut self) -> anyhow::Result<()> {
        let x = self.pop_front()?;
        let dx = self.pop_front()?;
        self.hint_count += 1;

        for _ in 0..self.operand_stack.len() / 2 {
            let dxa = self.pop_front()?;
            let dxb = self.pop_front()?;
            self.hint_count += 1;
        }
        // println!("parser.vstemhm()?;");
        if WARN_ON_UNIMPLEMENTED_HINT {
            println!("unimplemented cff charstring hinting op: vstemhm");
        }
        self.maybe_calculate_width()?;
        Ok(())
    }

    /// specifies which hints are active and which are not active. If any hints
    /// overlap, hintmask must be used to establish a nonoverlapping subset of
    /// hints. hintmask may occur any number of times in a charstring. Path
    /// operators occurring after a hintmask are influenced by the new hint set,
    /// but the current point is not moved. If stem hint zones overlap and are
    /// not properly managed by use of the hintmask operator, the results are
    /// undefined
    ///
    /// The mask data bytes are defined as follows:
    ///
    ///   * The number of data bytes is exactly the number needed, one bit per
    ///     hint, to reference the number of stem hints declared at the beginning
    ///     of the charstring program
    ///
    ///   * Each bit of the mask, starting with the most-significant bit of the
    ///     first byte, represents the corresponding hint zone in the order in
    ///     which the hints were declared at the beginning of the charstring
    ///
    ///   * For each bit in the mask, a value of ‘1’ specifies that the corresponding
    ///     hint shall be active. A bit value of ‘0’ specifies that the hint
    ///     shall be inactive
    ///
    ///   * Unused bits in the mask, if any, must be zero
    ///
    /// If hstem and vstem hints are both declared at the beginning of a charstring,
    /// and this sequence is followed directly by the hintmask or cntrmask
    /// operators, the vstem hint operator need not be included
    fn hintmask(&mut self) -> anyhow::Result<()> {
        if !self.operand_stack.is_empty() {
            self.vstem()?;
        }
        let num_bytes = (self.hint_count as f32 / 8.0).ceil() as u32;
        // println!("parser.hintmask()?;");
        for _ in 0..num_bytes {
            let _mask = self.next()?;
        }
        if WARN_ON_UNIMPLEMENTED_HINT {
            println!("unimplemented cff charstring hinting op: hintmask");
        }
        self.maybe_calculate_width()?;
        Ok(())
    }

    /// specifies the counter spaces to be controlled, and their relative priority.
    /// The mask bits in the bytes, following the operator, reference the stem
    /// hint declarations; the most significant bit of the first byte refers to
    /// the first stem hint declared, through to the last hint declaration. The
    /// counters to be controlled are those that are delimited by the referenced
    /// stem hints. Bits set to 1 in the first cntrmask command have top
    /// priority; subsequent cntrmask commands specify lower priority counters
    fn cntrmask(&mut self) -> anyhow::Result<()> {
        let num_bytes = (self.hint_count as f32 / 8.0).ceil() as u32;
        // println!("parser.cntrmask()?;");
        for _ in 0..num_bytes {
            let _mask = self.next()?;
        }
        if WARN_ON_UNIMPLEMENTED_HINT {
            println!("unimplemented cff charstring hinting op: cntrmask");
        }
        self.maybe_calculate_width()?;
        Ok(())
    }

    /// specifies one or more vertical stem hints between the x coordinates x and
    /// x+dx, where x is relative to the origin of the coordinate axes.
    ///
    /// It is required that the stems are encoded in ascending order (defined by
    /// increasing left edge). The encoded values are all relative; in the first
    /// pair, x is relative to 0, and dx specifies the distance from x. The first
    /// value of each subsequent pair is relative to the last edge defined by the
    /// previous pair.
    ///
    /// A width of –20 specifies the right edge of an edge hint, and –21 specifies
    /// the left edge of an edge hint. All other negative widths have undefined
    /// meaning.
    ///
    /// Vertical stem hints must not overlap each other. If there is any overlap, the
    /// hintmask operator must be used immediately after the hint declarations to
    /// establish the desired non-overlapping set of hints. hintmask may be used
    /// again later in the path to activate a different set of non-overlapping
    /// hints
    #[allow(unused_variables)]
    fn vstem(&mut self) -> anyhow::Result<()> {
        let x = self.pop_front()?;
        let dx = self.pop_front()?;
        self.hint_count += 1;

        for _ in 0..self.operand_stack.len() / 2 {
            let dxa = self.pop_front()?;
            let dxb = self.pop_front()?;
            self.hint_count += 1;
        }
        // println!("parser.vstem()?;");
        if WARN_ON_UNIMPLEMENTED_HINT {
            println!("unimplemented cff charstring hinting op: vstem");
        }
        self.maybe_calculate_width()?;
        Ok(())
    }

    /// causes the two curves described by the arguments to be rendered as a straight
    /// line when the flex depth is less than 0.5 device pixels, and as curved
    /// lines when the flex depth is greater than or equal to 0.5 device pixels
    ///
    /// The d6 argument will be either a dx or dy value, depending on the curve. To
    /// determine the correct value, compute the distance from the starting point
    /// (x, y), the first point of the first curve, to the last flex control
    /// point (dx5, dy5) by summing all the arguments except d6; call this (dx,
    /// dy). If abs(dx) > abs(dy), then the last point’s x-value is given by d6,
    /// and its y-value is equal to y. Otherwise, the last point’s x-value is
    /// equal to x and its y-value is given by d6
    ///
    /// flex1 is used if the conditions for hflex and hflex1 are not met but all of
    /// the following are true
    ///
    ///   1. the starting and ending points have the same x or y value
    ///
    ///   2. the flex depth is 50
    ///
    fn flex1(&mut self) -> anyhow::Result<()> {
        let dx1 = self.pop_front()?;
        let dy1 = self.pop_front()?;
        let dx2 = self.pop_front()?;
        let dy2 = self.pop_front()?;
        let dx3 = self.pop_front()?;
        let dy3 = self.pop_front()?;

        self.path_builder
            .relative_relative_curve_to(dx1, dy1, dx2, dy2, dx3, dy3);

        let dx4 = self.pop_front()?;
        let dy4 = self.pop_front()?;
        let dx5 = self.pop_front()?;
        let dy5 = self.pop_front()?;
        let d6 = self.pop_front()?;

        // todo: should be condition here
        let (dx6, dy6) = (d6, d6);

        self.path_builder
            .relative_relative_curve_to(dx4, dy4, dx5, dy5, dx6, dy6);

        println!("unimplemented cff charstring op: flex1");

        // println!("parser.flex1()?;");

        self.operand_stack.clear();
        Ok(())
    }

    /// causes the two curves described by the arguments to be rendered as a straight
    /// line when the flex depth is less than 0.5 device pixels, and as curved
    /// lines when the flex depth is greater than or equal to 0.5 device pixels.
    ///
    /// hflex1 is used if the conditions for hflex are not met but all of the
    /// following are true
    ///
    ///   1. the starting and ending points have the same y value
    ///
    ///   2. the joining point and the neighbor control points have the same y
    ///      value
    ///
    ///   3. the flex depth is 50
    ///
    fn hflex1(&mut self) -> anyhow::Result<()> {
        let dx1 = self.pop_front()?;
        let dy1 = self.pop_front()?;
        let dx2 = self.pop_front()?;
        let dy2 = self.pop_front()?;
        let dx3 = self.pop_front()?;

        let dy3 = dy2;

        self.path_builder
            .relative_relative_curve_to(dx1, dy1, dx2, dy2, dx3, dy3);

        let dx4 = self.pop_front()?;
        let dx5 = self.pop_front()?;
        let dy5 = self.pop_front()?;
        let dx6 = self.pop_front()?;

        let dy4 = dy1;
        let dy6 = dy1;

        self.path_builder
            .relative_relative_curve_to(dx4, dy4, dx5, dy5, dx6, dy6);

        if WARN_ON_UNIMPLEMENTED_HINT {
            println!("unimplemented cff charstring hinting op: hflex1");
        }

        // println!("parser.hflex1()?;");

        self.operand_stack.clear();
        Ok(())
    }

    /// causes the two curves described by the arguments dx1...dx6 to be rendered as
    /// a straight line when the flex depth is less than 0.5 (that is, fd is 50)
    /// device pixels, and as curved lines when the flex depth is greater than or
    /// equal to 0.5 device pixels
    ///
    /// hflex is used when the following are all true:
    ///
    ///   1. the starting and ending points, first and last control points have
    ///      the same y value.
    ///
    ///   2. the joining point and the neighbor control points have the same y
    ///      value
    ///
    ///   3. the flex depth is 50
    ///
    fn hflex(&mut self) -> anyhow::Result<()> {
        let dx1 = self.pop_front()?;
        let dx2 = self.pop_front()?;
        let dy2 = self.pop_front()?;
        let dx3 = self.pop_front()?;
        let dx4 = self.pop_front()?;
        let dx5 = self.pop_front()?;
        let dx6 = self.pop_front()?;

        let dy1 = dy2;
        let dy3 = dy2;

        self.path_builder
            .relative_relative_curve_to(dx1, dy1, dx2, dy2, dx3, dy3);

        let dy4 = dy2;
        let dy5 = dy2;
        let dy6 = dy2;

        self.path_builder
            .relative_relative_curve_to(dx4, dy4, dx5, dy5, dx6, dy6);

        if WARN_ON_UNIMPLEMENTED_HINT {
            println!("unimplemented cff charstring hinting op: hflex");
        }

        // println!("parser.hflex()?;");

        self.operand_stack.clear();
        Ok(())
    }

    /// causes two Bézier curves, as described by the arguments, to be rendered as a
    /// straight line when the flex depth is less than fd /100 device pixels, and
    /// as curved lines when the flex depth is greater than or equal to fd/100
    /// device pixels
    ///
    /// The flex depth for a horizontal curve is the distance from the join point to
    /// the line connecting the start and end points on the curve. If the curve
    /// is not exactly horizontal or vertical, it must be determined whether the
    /// curve is more horizontal or vertical by the method described in the flex1
    /// description
    #[allow(unused_variables)]
    fn flex(&mut self) -> anyhow::Result<()> {
        let dx1 = self.pop_front()?;
        let dy1 = self.pop_front()?;
        let dx2 = self.pop_front()?;
        let dy2 = self.pop_front()?;
        let dx3 = self.pop_front()?;
        let dy3 = self.pop_front()?;

        self.path_builder
            .relative_relative_curve_to(dx1, dy1, dx2, dy2, dx3, dy3);

        let dx4 = self.pop_front()?;
        let dy4 = self.pop_front()?;
        let dx5 = self.pop_front()?;
        let dy5 = self.pop_front()?;
        let dx6 = self.pop_front()?;
        let dy6 = self.pop_front()?;

        self.path_builder
            .relative_relative_curve_to(dx4, dy4, dx5, dy5, dx6, dy6);

        let fd = self.pop_front()?;

        // println!("parser.flex()?;");
        if WARN_ON_UNIMPLEMENTED_HINT {
            println!("unimplemented cff charstring hinting op: flex");
        }
        self.operand_stack.clear();
        Ok(())
    }

    /// appends a vertical line of length dy1 to the current point. With an odd
    /// number of arguments, subsequent argument pairs are interpreted as
    /// alternating values of dx and dy, for which additional lineto operators
    /// draw alternating horizontal and vertical lines. With an even number of
    /// arguments, the arguments are interpreted as alternating vertical and
    /// horizontal lines. The number of lines is determined from the number of
    /// arguments on the stack
    fn vlineto(&mut self) -> anyhow::Result<()> {
        // println!("parser.vlineto()?;");
        if self.operand_stack.len() % 2 == 0 {
            for _ in 0..self.operand_stack.len() / 2 {
                let dy = self.pop_front()?;
                let dx = self.pop_front()?;

                self.path_builder.vertical_line_to(dy);
                self.path_builder.horizontal_line_to(dx);
            }
        } else {
            let dy = self.pop_front()?;
            self.path_builder.vertical_line_to(dy);

            for _ in 0..self.operand_stack.len() / 2 {
                let dx = self.pop_front()?;
                let dy = self.pop_front()?;

                self.path_builder.horizontal_line_to(dx);
                self.path_builder.vertical_line_to(dy);
            }
        }

        self.operand_stack.clear();
        Ok(())
    }

    /// appends a Bézier curve, defined by dxa...dyc, to the current point. For each
    /// subsequent set of six arguments, an additional curve is appended to the
    /// current point. The number of curve segments is determined from the number
    /// of arguments on the number stack and is limited only by the size of the
    /// number stack
    fn rrcurveto(&mut self) -> anyhow::Result<()> {
        // println!("parser.rrcurveto()?;");
        assert_eq!(self.operand_stack.len() % 6, 0);
        for _ in 0..self.operand_stack.len() / 6 {
            let dx1 = self.pop_front()?;
            let dy1 = self.pop_front()?;
            let dx2 = self.pop_front()?;
            let dy2 = self.pop_front()?;
            let dx3 = self.pop_front()?;
            let dy3 = self.pop_front()?;

            self.path_builder
                .relative_relative_curve_to(dx1, dy1, dx2, dy2, dx3, dy3);
        }

        self.operand_stack.clear();
        Ok(())
    }

    /// appends a horizontal line of length dx1 to the current point. With an odd
    /// number of arguments, subsequent argument pairs are interpreted as
    /// alternating values of dy and dx, for which additional lineto operators
    /// draw alternating vertical and horizontal lines. With an even number of
    /// arguments, the arguments are interpreted as alternating horizontal and
    /// vertical lines. The number of lines is determined from the number of
    /// arguments on the stack
    fn hlineto(&mut self) -> anyhow::Result<()> {
        // println!("parser.hlineto()?;");
        if self.operand_stack.len() % 2 == 0 {
            for _ in 0..self.operand_stack.len() / 2 {
                let dx = self.pop_front()?;
                let dy = self.pop_front()?;

                self.path_builder.horizontal_line_to(dx);
                self.path_builder.vertical_line_to(dy);
            }
        } else {
            let dx = self.pop_front()?;
            self.path_builder.horizontal_line_to(dx);

            for _ in 0..self.operand_stack.len() / 2 {
                let dy = self.pop_front()?;
                let dx = self.pop_front()?;

                self.path_builder.vertical_line_to(dy);
                self.path_builder.horizontal_line_to(dx);
            }
        }

        self.operand_stack.clear();
        Ok(())
    }

    /// moves the current point dy1 units in the vertical direction
    fn vmoveto(&mut self) -> anyhow::Result<()> {
        let dy1 = self.pop_front()?;
        self.path_builder.relative_move_to(0.0, dy1);

        self.maybe_calculate_width()?;
        Ok(())
    }

    /// appends a line from the current point to a position at the relative
    /// coordinates dxa, dya. Additional rlineto operations are performed for all
    /// subsequent argument pairs. The number of lines is determined from the
    /// number of arguments on the stack.
    fn rlineto(&mut self) -> anyhow::Result<()> {
        // println!("parser.rlineto()?;");
        for _ in 0..self.operand_stack.len() / 2 {
            let dx = self.pop_front()?;
            let dy = self.pop_front()?;

            self.path_builder.relative_line_to(dx, dy);
        }

        self.operand_stack.clear();
        Ok(())
    }

    /// is equivalent to one rrcurveto for each set of six arguments dxa...dyc,
    /// followed by exactly one rlineto using the dxd, dyd arguments. The number
    /// of curves is determined from the count on the argument stack
    fn rcurveline(&mut self) -> anyhow::Result<()> {
        // println!("parser.rcurveline()?;");
        assert_eq!((self.operand_stack.len() - 2) % 6, 0);
        while self.operand_stack.len() >= 6 {
            let dx1 = self.pop_front()?;
            let dy1 = self.pop_front()?;
            let dx2 = self.pop_front()?;
            let dy2 = self.pop_front()?;
            let dx3 = self.pop_front()?;
            let dy3 = self.pop_front()?;

            self.path_builder
                .relative_relative_curve_to(dx1, dy1, dx2, dy2, dx3, dy3);
        }

        let dx = self.pop_front()?;
        let dy = self.pop_front()?;

        self.path_builder.relative_line_to(dx, dy);

        self.operand_stack.clear();
        Ok(())
    }

    /// is equivalent to one rlineto for each pair of arguments beyond the six
    /// arguments dxb...dyd needed for the one rrcurveto command. The number of
    /// lines is determined from the count of items on the argument stack
    fn rlinecurve(&mut self) -> anyhow::Result<()> {
        // {dxa dya}+ dxb dyb dxc dyc dxd dyd
        // println!("parser.rlinecurve()?;");
        assert_eq!(self.operand_stack.len() % 2, 0);
        assert!(self.operand_stack.len() >= 8);
        while self.operand_stack.len() > 6 {
            let dx = self.pop_front()?;
            let dy = self.pop_front()?;

            self.path_builder.relative_line_to(dx, dy);
        }

        let dx1 = self.pop_front()?;
        let dy1 = self.pop_front()?;
        let dx2 = self.pop_front()?;
        let dy2 = self.pop_front()?;
        let dx3 = self.pop_front()?;
        let dy3 = self.pop_front()?;

        self.path_builder
            .relative_relative_curve_to(dx1, dy1, dx2, dy2, dx3, dy3);

        assert_eq!(self.operand_stack.len(), 0);

        self.operand_stack.clear();
        Ok(())
    }

    /// moves the current point dx1 units in the horizontal direction
    fn hmoveto(&mut self) -> anyhow::Result<()> {
        let dx1 = self.pop_front()?;

        self.path_builder.relative_move_to(dx1, 0.0);

        self.maybe_calculate_width()?;
        Ok(())
    }

    /// moves the current point to a position at the relative coordinates (dx1, dy1).
    fn rmoveto(&mut self) -> anyhow::Result<()> {
        // println!("parser.rmoveto()?;");
        let dx = self.pop_front()?;
        let dy = self.pop_front()?;

        self.path_builder.relative_move_to(dx, dy);

        self.maybe_calculate_width()?;
        Ok(())
    }

    /// appends one or more Bézier curves to the current point. The tangent for the
    /// first Bézier must be horizontal, and the second must be vertical (except
    /// as noted below)
    ///
    /// If there is a multiple of four arguments, the curve starts horizontal and
    /// ends vertical. Note that the curves alternate between start horizontal,
    /// end vertical, and start vertical, and end horizontal. The last curve (the
    /// odd argument case) need not end horizontal/vertical
    fn hvcurveto(&mut self) -> anyhow::Result<()> {
        // println!("parser.hvcurveto()?;");
        match self.operand_stack.len() % 8 {
            0 | 1 => {
                // {dxa dxb dyb dyc dyd dxe dye dxf}+ dyf?
                while !self.operand_stack.is_empty() {
                    let dxa = self.pop_front()?;
                    let dxb = self.pop_front()?;
                    let dyb = self.pop_front()?;
                    let dyc = self.pop_front()?;

                    self.path_builder
                        .horizontal_vertical_curve_to(dxa, dxb, dyb, dyc);

                    let dyd = self.pop_front()?;
                    let dxe = self.pop_front()?;
                    let dye = self.pop_front()?;
                    let dxf = self.pop_front()?;

                    if self.operand_stack.len() == 1 {
                        let dyf = self.pop_front()?;
                        self.path_builder
                            .relative_relative_curve_to(0.0, dyd, dxe, dye, dxf, dyf);
                    } else {
                        self.path_builder
                            .vertical_horizontal_curve_to(dyd, dxe, dye, dxf);
                    }
                }
            }
            4 | 5 => {
                // dx1 dx2 dy2 dy3 {dya dxb dyb dxc dxd dxe dye dyf}* dxf?
                let mut dx1 = self.pop_front()?;
                let mut dx2 = self.pop_front()?;
                let mut dy2 = self.pop_front()?;
                let mut dy3 = self.pop_front()?;

                self.path_builder
                    .horizontal_vertical_curve_to(dx1, dx2, dy2, dy3);

                while self.operand_stack.len() > 1 {
                    let dya = self.pop_front()?;
                    let dxb = self.pop_front()?;
                    let dyb = self.pop_front()?;
                    let dxc = self.pop_front()?;

                    self.path_builder
                        .vertical_horizontal_curve_to(dya, dxb, dyb, dxc);

                    let dxd = self.pop_front()?;
                    let dxe = self.pop_front()?;
                    let dye = self.pop_front()?;
                    let dyf = self.pop_front()?;

                    if self.operand_stack.len() == 1 {
                        dx1 = dxd;
                        dx2 = dxe;
                        dy2 = dye;
                        dy3 = dyf;
                    } else {
                        self.path_builder
                            .horizontal_vertical_curve_to(dxd, dxe, dye, dyf);
                    }
                }

                if self.operand_stack.len() == 1 {
                    let dxf = self.pop_front()?;
                    self.path_builder
                        .relative_relative_curve_to(dx1, 0.0, dx2, dy2, dxf, dy3);
                }
            }
            _ => anyhow::bail!("invalid operand stack size {}", self.operand_stack.len()),
        }

        anyhow::ensure!(self.operand_stack.is_empty());

        self.operand_stack.clear();
        Ok(())
    }

    /// appends one or more Bézier curves to the current point, where the first
    /// tangent is vertical and the second tangent is horizontal
    ///
    /// This command is the complement of hvcurveto; see the description of hvcurveto
    /// for more information
    fn vhcurveto(&mut self) -> anyhow::Result<()> {
        // println!("parser.vhcurveto()?;");
        match self.operand_stack.len() % 8 {
            4 | 5 => {
                // dy1 dx2 dy2 dx3 {dxa dxb dyb dyc dyd dxe dye dxf}* dyf? vhcurveto (30)
                let mut dy1 = self.pop_front()?;
                let mut dx2 = self.pop_front()?;
                let mut dy2 = self.pop_front()?;
                let mut dx3 = self.pop_front()?;

                self.path_builder
                    .vertical_horizontal_curve_to(dy1, dx2, dy2, dx3);

                while self.operand_stack.len() > 1 {
                    let dxa = self.pop_front()?;
                    let dxb = self.pop_front()?;
                    let dyb = self.pop_front()?;
                    let dyc = self.pop_front()?;

                    self.path_builder
                        .horizontal_vertical_curve_to(dxa, dxb, dyb, dyc);

                    let dyd = self.pop_front()?;
                    let dxe = self.pop_front()?;
                    let dye = self.pop_front()?;
                    let dxf = self.pop_front()?;

                    if self.operand_stack.len() == 1 {
                        dy1 = dyd;
                        dx2 = dxe;
                        dy2 = dye;
                        dx3 = dxf;
                    } else {
                        self.path_builder
                            .vertical_horizontal_curve_to(dyd, dxe, dye, dxf);
                    }
                }

                if self.operand_stack.len() == 1 {
                    let dyf = self.pop_front()?;
                    self.path_builder
                        .relative_relative_curve_to(0.0, dy1, dx2, dy2, dx3, dyf);
                }
            }
            0 | 1 => {
                // {dya dxb dyb dxc dxd dxe dye dyf}+ dxf? vhcurveto (30)
                while !self.operand_stack.is_empty() {
                    let dya = self.pop_front()?;
                    let dxb = self.pop_front()?;
                    let dyb = self.pop_front()?;
                    let dxc = self.pop_front()?;

                    self.path_builder
                        .vertical_horizontal_curve_to(dya, dxb, dyb, dxc);

                    let dxd = self.pop_front()?;
                    let dxe = self.pop_front()?;
                    let dye = self.pop_front()?;
                    let dyf = self.pop_front()?;

                    if self.operand_stack.len() == 1 {
                        let dxf = self.pop_front()?;
                        self.path_builder
                            .relative_relative_curve_to(dxd, 0.0, dxe, dye, dxf, dyf);
                    } else {
                        self.path_builder
                            .horizontal_vertical_curve_to(dxd, dxe, dye, dyf)
                    }
                }
            }
            _ => anyhow::bail!("invalid operand stack size {}", self.operand_stack.len()),
        }

        self.operand_stack.clear();
        Ok(())
    }

    /// appends one or more curves to the current point. If the argument count is a
    /// multiple of four, the curve starts and ends vertical. If the argument
    /// count is odd, the first curve does not begin with a vertical tangent
    fn vvcurveto(&mut self) -> anyhow::Result<()> {
        // println!("parser.vvcurveto()?;");

        // dx1? {dya dxb dyb dyc}+ vvcurveto (26)
        if self.operand_stack.len() % 4 == 0 {
            // {dya dxb dyb dyc}+
            for _ in 0..self.operand_stack.len() / 4 {
                let dya = self.pop_front()?;
                let dxb = self.pop_front()?;
                let dyb = self.pop_front()?;
                let dyc = self.pop_front()?;

                self.path_builder
                    .relative_relative_curve_to(0.0, dya, dxb, dyb, 0.0, dyc);
            }
        } else {
            // dx1? {dya dxb dyb dyc}+
            let dx1 = self.pop_front()?;
            let dya = self.pop_front()?;
            let dxb = self.pop_front()?;
            let dyb = self.pop_front()?;
            let dyc = self.pop_front()?;

            self.path_builder
                .relative_relative_curve_to(dx1, dya, dxb, dyb, 0.0, dyc);

            for _ in 0..self.operand_stack.len() / 4 {
                let dya = self.pop_front()?;
                let dxb = self.pop_front()?;
                let dyb = self.pop_front()?;
                let dyc = self.pop_front()?;

                self.path_builder
                    .relative_relative_curve_to(0.0, dya, dxb, dyb, 0.0, dyc);
            }
        }

        self.operand_stack.clear();
        Ok(())
    }

    /// calls a charstring subroutine with index subr# (actually the subr number plus
    /// the subroutine bias number) in the Subrs array. Each element of the Subrs
    /// array is a charstring encoded like any other charstring. Arguments pushed
    /// on the Type 2 argument stack prior to calling the subroutine, and results
    /// pushed on this stack by the subroutine, act according to the manner in
    /// which the subroutine is coded. Calling an undefined subr (gsubr) has
    /// undefined results
    ///
    /// These subroutines are generally used to encode sequences of path operators
    /// that are repeated throughout the font program, for example, serif outline
    /// sequences
    fn callsubr(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    /// operates in the same manner as callsubr except that it calls a global
    /// subroutine
    fn callgsubr(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    /// appends one or more Bézier curves, as described by the dxa...dxc set of
    /// arguments, to the current point. For each curve, if there are 4
    /// arguments, the curve starts and ends horizontal. The first curve need not
    /// start horizontal (the odd argument case). Note the argument order for the
    /// odd argument case
    fn hhcurveto(&mut self) -> anyhow::Result<()> {
        // println!("parser.hhcurveto()?;");
        if self.operand_stack.len() % 4 == 0 {
            // {dxa dxb dyb dxc}+
            for _ in 0..self.operand_stack.len() / 4 {
                let dxa = self.pop_front()?;
                let dxb = self.pop_front()?;
                let dyb = self.pop_front()?;
                let dxc = self.pop_front()?;

                self.path_builder
                    .relative_relative_curve_to(dxa, 0.0, dxb, dyb, dxc, 0.0);
            }
        } else {
            // dy1? {dxa dxb dyb dxc}+
            let dy1 = self.pop_front()?;
            let dxa = self.pop_front()?;
            let dxb = self.pop_front()?;
            let dyb = self.pop_front()?;
            let dxc = self.pop_front()?;

            self.path_builder
                .relative_relative_curve_to(dxa, dy1, dxb, dyb, dxc, 0.0);

            for _ in 0..self.operand_stack.len() / 4 {
                let dxa = self.pop_front()?;
                let dxb = self.pop_front()?;
                let dyb = self.pop_front()?;
                let dxc = self.pop_front()?;

                self.path_builder
                    .relative_relative_curve_to(dxa, 0.0, dxb, dyb, dxc, 0.0);
            }
        }

        self.operand_stack.clear();
        Ok(())
    }

    /// finishes a charstring outline definition, and must be the last operator in a
    /// character's outline
    fn end_char(&mut self) -> anyhow::Result<()> {
        if !self.path_builder.current_path.subpaths.is_empty() {
            self.path_builder.close_path();
        }
        // println!("parser.end_char()?;");

        self.maybe_calculate_width()?;

        anyhow::ensure!(self.peek().is_none(), "end char should be last op");

        Ok(())
    }
}

impl<'a> BinaryParser for CffCharStringInterpreter<'a> {
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

pub(crate) mod canvas;
pub(super) mod error;
pub(crate) mod graphics_state;
pub(crate) mod text_state;

use std::rc::Rc;

use crate::{
    catalog::ColorSpace,
    content::{ContentLexer, ContentToken, PdfGraphicsOperator},
    data_structures::Matrix,
    error::PdfResult,
    filter::decode_stream,
    font::{Font, Type1Font},
    geometry::{Path, Point},
    objects::Object,
    page::PageObject,
    pdf_enum,
    postscript::{charstring::CharStringPainter, PostscriptInterpreter},
    xobject::XObject,
    Resolve,
};

use canvas::Canvas;

use self::{error::PdfRenderError, graphics_state::GraphicsState, text_state::TextState};

#[derive(Debug)]
enum FillRule {
    EvenOdd,
    NonZeroWindingNumber,
}

pub struct Renderer<'a, 'b> {
    content: &'b mut ContentLexer<'a>,
    resolver: &'b mut dyn Resolve<'a>,
    canvas: Canvas,
    graphics_state_stack: Vec<GraphicsState>,
    operand_stack: Vec<Object>,
    graphics_state: GraphicsState,

    /// A set of nine graphics state parameters that pertain only to the
    /// painting of text. These include parameters that select the font, scale
    /// the glyphs to an appropriate size, and accomplish other effects.
    text_state: TextState,
    page: Rc<PageObject>,
    current_path: Option<Path>,
}

impl<'a, 'b> Renderer<'a, 'b> {
    fn pop(&mut self) -> PdfResult<Object> {
        Ok(self
            .operand_stack
            .pop()
            .ok_or(PdfRenderError::StackUnderflow)?)
    }

    fn pop_number(&mut self) -> PdfResult<f32> {
        let obj = self.pop()?;

        self.resolver.assert_number(obj)
    }

    fn pop_name(&mut self) -> PdfResult<String> {
        let obj = self.pop()?;

        self.resolver.assert_name(obj)
    }

    fn pop_string(&mut self) -> PdfResult<String> {
        let obj = self.pop()?;

        self.resolver.assert_string(obj)
    }

    fn pop_arr(&mut self) -> PdfResult<Vec<Object>> {
        let obj = self.pop()?;

        self.resolver.assert_arr(obj)
    }
}

impl<'a, 'b> Renderer<'a, 'b> {
    pub fn new(
        content: &'b mut ContentLexer<'a>,
        resolver: &'b mut dyn Resolve<'a>,
        page: Rc<PageObject>,
    ) -> Self {
        Self {
            content,
            resolver,
            canvas: Canvas::new(1000, 1000),
            graphics_state_stack: Vec::new(),
            operand_stack: Vec::new(),
            graphics_state: GraphicsState::default(),
            text_state: TextState::default(),
            page,
            current_path: None,
        }
    }

    pub fn render(mut self) -> PdfResult<()> {
        while let Some(token) = self.content.next() {
            let token = token?;

            match token {
                ContentToken::Object(obj) => self.operand_stack.push(obj),
                ContentToken::Operator(op) => match op {
                    PdfGraphicsOperator::G => self.set_stroking_gray()?,
                    PdfGraphicsOperator::g => self.set_nonstroking_gray()?,
                    PdfGraphicsOperator::BT => self.begin_text()?,
                    PdfGraphicsOperator::Tf => self.set_font_and_size()?,
                    PdfGraphicsOperator::Td => self.move_text_position()?,
                    PdfGraphicsOperator::TJ => self.draw_text_adjusted()?,
                    PdfGraphicsOperator::q => self.save_graphics_state()?,
                    PdfGraphicsOperator::Q => self.restore_graphics_state()?,
                    PdfGraphicsOperator::cm => self.transform_ctm()?,
                    PdfGraphicsOperator::Do => self.draw_xobject()?,
                    PdfGraphicsOperator::w => self.set_line_width()?,
                    PdfGraphicsOperator::re => self.create_rectangle()?,
                    PdfGraphicsOperator::W_star => self.set_clipping_path_even_odd()?,
                    PdfGraphicsOperator::n => self.draw_path_nop()?,
                    PdfGraphicsOperator::RG => self.set_stroking_rgb()?,
                    PdfGraphicsOperator::rg => self.set_nonstroking_rgb()?,
                    PdfGraphicsOperator::ET => self.end_text()?,
                    PdfGraphicsOperator::BDC => self.begin_marked_content_sequence()?,
                    PdfGraphicsOperator::EMC => self.end_marked_content_sequence()?,
                    PdfGraphicsOperator::Tm => self.set_text_matrix()?,
                    PdfGraphicsOperator::Tj => self.draw_text()?,
                    PdfGraphicsOperator::gs => self.set_graphics_state_parameters()?,
                    PdfGraphicsOperator::f | PdfGraphicsOperator::F => {
                        self.fill_path(FillRule::NonZeroWindingNumber)?
                    }
                    PdfGraphicsOperator::f_star => self.fill_path(FillRule::EvenOdd)?,
                    PdfGraphicsOperator::m => self.move_to()?,
                    PdfGraphicsOperator::l => self.line_to()?,
                    PdfGraphicsOperator::h => self.close_path()?,
                    PdfGraphicsOperator::S => self.stroke_path()?,
                    _ => todo!("unimplemented operator: {:?}", op),
                },
            }
        }

        self.canvas.draw();

        Ok(())
    }

    /// Set the specified parameters in the graphics state. dictName shall be
    /// the name of a graphics state parameter dictionary in the ExtGState
    /// subdictionary of the current resource dictionary
    fn set_graphics_state_parameters(&mut self) -> PdfResult<()> {
        let dict_name = self.pop_name()?;

        let graphics_state_parameters = self
            .page
            .resources
            .as_ref()
            .and_then(|res| res.ext_g_state.as_ref())
            .and_then(|state_map| state_map.get(&dict_name));

        graphics_state_parameters
            .unwrap()
            .update_graphics_state(&mut self.graphics_state, &mut self.text_state);

        Ok(())
    }

    /// Stroke the path.
    fn stroke_path(&mut self) -> PdfResult<()> {
        let path = self
            .current_path
            .get_or_insert_with(|| Path::new(Point::new(0.0, 0.0)));

        let color = self
            .graphics_state
            .device_independent
            .color_space
            .stroking
            .as_u32();

        path.apply_transform(
            self.graphics_state
                .device_independent
                .current_transformation_matrix,
        );

        self.canvas.stroke_path(&path, color);

        self.current_path = None;

        Ok(())
    }

    /// Close the current subpath by appending a straight line segment from the
    /// current point to the starting point of the subpath. If the current subpath
    /// is already closed, h shall do nothing.
    ///
    /// This operator terminates the current subpath. Appending another segment
    /// to the current path shall begin a new subpath, even if the new segment
    /// begins at the endpoint reached by the h operation.
    fn close_path(&mut self) -> PdfResult<()> {
        if let Some(path) = self.current_path.as_mut() {
            let current_point = path.current_point;
            path.close_path();

            path.current_point = current_point;
            path.start = current_point;
        }

        Ok(())
    }

    /// Append a straight line segment from the current point to the point (x, y).
    ///
    /// The new current point shall be (x, y).
    fn line_to(&mut self) -> PdfResult<()> {
        let y = self.pop_number()?;
        let x = self.pop_number()?;

        let path = self
            .current_path
            .get_or_insert_with(|| Path::new(Point::new(0.0, 0.0)));

        path.line_to(Point::new(x, y));

        Ok(())
    }

    /// Begin a new subpath by moving the current point to coordinates (x, y),
    /// omitting any connecting line segment. If the previous path construction
    /// operator in the current path was also m, the new m overrides it; no vestige
    /// of the previous m operation remains in the path.
    fn move_to(&mut self) -> PdfResult<()> {
        let y = self.pop_number()?;
        let x = self.pop_number()?;

        let path = self
            .current_path
            .get_or_insert_with(|| Path::new(Point::new(0.0, 0.0)));

        path.move_to(Point::new(x, y));
        path.start = Point::new(x, y);

        Ok(())
    }

    /// Set the stroking colour space to DeviceGray and set the gray level to
    /// use for stroking operations. gray shall be a number between 0.0 (black)
    /// and 1.0 (white).
    fn set_stroking_gray(&mut self) -> PdfResult<()> {
        let gray = self.pop_number()?;
        self.graphics_state.device_independent.color_space.stroking = ColorSpace::DeviceGray(gray);

        Ok(())
    }

    fn fill_path(&mut self, fill_rule: FillRule) -> PdfResult<()> {
        let path = self.current_path.as_ref().unwrap();
        let color = self
            .graphics_state
            .device_independent
            .color_space
            .nonstroking
            .as_u32();

        match fill_rule {
            FillRule::EvenOdd => self.canvas.fill_path_even_odd(path, color),
            FillRule::NonZeroWindingNumber => {
                self.canvas.fill_path_non_zero_winding_number(path, color)
            }
        }

        Ok(())
    }

    /// Same as [Renderer::set_stroking_gray], but used for nonstroking operations
    fn set_nonstroking_gray(&mut self) -> PdfResult<()> {
        let gray = self.pop_number()?;
        self.graphics_state
            .device_independent
            .color_space
            .nonstroking = ColorSpace::DeviceGray(gray);

        Ok(())
    }

    /// Set the stroking colour space to DeviceRGB (or the DefaultRGB colour
    /// space and set the colour to use for stroking operations. Each operand
    /// shall be a number between 0.0 (minimum intensity) and 1.0 (maximum
    /// intensity).
    fn set_stroking_rgb(&mut self) -> PdfResult<()> {
        let blue = self.pop_number()?;
        let green = self.pop_number()?;
        let red = self.pop_number()?;

        self.graphics_state.device_independent.color_space.stroking =
            ColorSpace::DeviceRGB { red, green, blue };

        Ok(())
    }

    /// Same as [Renderer::set_stroking_rgb] but used for nonstroking operations.
    fn set_nonstroking_rgb(&mut self) -> PdfResult<()> {
        let blue = self.pop_number()?;
        let green = self.pop_number()?;
        let red = self.pop_number()?;

        self.graphics_state
            .device_independent
            .color_space
            .nonstroking = ColorSpace::DeviceRGB { red, green, blue };

        Ok(())
    }

    /// Begin a text object, initializing the text matrix, Tm, and the text line
    /// matrix, Tlm, to the identity matrix. Text objects shall not be nested;
    /// a second BT shall not appear before an ET.
    fn begin_text(&mut self) -> PdfResult<()> {
        self.text_state.reinit();

        Ok(())
    }

    /// End a text object, discarding the text matrix.
    fn end_text(&mut self) -> PdfResult<()> {
        self.text_state.text_matrix = Matrix::identity();
        self.text_state.text_line_matrix = Matrix::identity();

        Ok(())
    }

    /// Set the text font to _font_ and the text font size to _size_. _font_
    /// shall be the name of a font resource in the Font subdictionary of the
    /// current resource dictionary; size shall be a number representing a scale
    /// factor. There is no initial value for either font or size; they shall
    /// be specified explicitly by using Tf before any text is shown.
    fn set_font_and_size(&mut self) -> PdfResult<()> {
        let size = self.pop_number()?;
        let font_name = self.pop_name()?;

        let font = self
            .page
            .resources
            .as_ref()
            .and_then(|res| res.font.as_ref())
            .and_then(|fonts| fonts.get(&font_name))
            .map(Rc::clone);

        match font {
            Some(font) => {
                self.text_state.font = Some(font);
                self.text_state.font_size = size;
            }
            None => todo!("could not find font with name {:?}", font_name),
        }

        Ok(())
    }

    /// Move to the start of the next line, offset from the start of the current
    /// line by (t_x, t_y). t_x and t_y shall denote numbers expressed in
    /// unscaled text space units. More precisely, this operator shall perform
    /// these assignments:
    ///
    /// T_m = T_lm = [1 0 0, 0 1 0, t_x t_y 1] * T_lm
    fn move_text_position(&mut self) -> PdfResult<()> {
        let t_y = self.pop_number()?;
        let t_x = self.pop_number()?;

        let matrix = Matrix::new_translation(t_x, t_y) * self.text_state.text_line_matrix;

        self.text_state.text_matrix = matrix;
        self.text_state.text_line_matrix = matrix;

        Ok(())
    }

    /// Show one or more text strings, allowing individual glyph positioning.
    ///
    /// Each element of array shall be either a string or a number. If the element
    /// is a string, this operator shall show the string. If it is a number,
    /// the operator shall adjust the text position by that amount; that is, it
    /// shall translate the text matrix, Tm. The number shall be expressed in
    /// thousandths of a unit of text space. This amount shall be subtracted
    /// from the current horizontal or vertical coordinate, depending on the
    /// writing mode. In the default coordinate system, a positive adjustment has
    /// the effect of moving the next glyph painted either to the left or down
    /// by the given amount.
    fn draw_text_adjusted(&mut self) -> PdfResult<()> {
        let arr = self.pop_arr()?;

        let (font_stream, widths) = match self.text_state.font.as_deref() {
            Some(Font::Type1(Type1Font { base, .. })) => (
                base.font_descriptor.font_file.clone().unwrap(),
                &base.widths,
            ),
            Some(font) => todo!("unimplement font type: {:#?}", font),
            None => todo!("no font selected in text state"),
        };

        let stream = decode_stream(
            &font_stream.stream.stream,
            &font_stream.stream.dict,
            self.resolver,
        )?;

        let mut interpreter = PostscriptInterpreter::new(&stream);

        interpreter.run()?;

        let font = interpreter.fonts.into_values().next().unwrap();

        for obj in arr {
            let obj = self.resolver.resolve(obj)?;

            let s = match obj {
                Object::String(s) => s,
                // todo: consolidate integer/float handling here
                Object::Real(n) => {
                    self.text_state.text_matrix *=
                        Matrix::new_translation((-n * self.text_state.font_size) / 1000.0, 0.0);
                    continue;
                }
                Object::Integer(n) => {
                    self.text_state.text_matrix *= Matrix::new_translation(
                        (-n as f32 * self.text_state.font_size) / 1000.0,
                        0.0,
                    );
                    continue;
                }
                _ => todo!(),
            };

            let mut painter = CharStringPainter::new(&font);

            for c in s.chars() {
                let trm = Matrix::new(
                    self.text_state.font_size * self.text_state.horizontal_scaling,
                    0.0,
                    0.0,
                    self.text_state.font_size,
                    0.0,
                    self.text_state.rise,
                ) * font.font_matrix
                    * self.text_state.text_matrix
                    * self
                        .graphics_state
                        .device_independent
                        .current_transformation_matrix;

                let mut glyph = painter.evaluate(c as u32)?;

                glyph.outline.apply_transform(trm);

                self.canvas.stroke_outline(
                    &glyph.outline,
                    self.graphics_state
                        .device_independent
                        .color_space
                        .stroking
                        .as_u32(),
                );

                self.canvas.refresh();

                let mut x_transform = widths.get(c as u32) * self.text_state.font_size
                    + self.text_state.character_spacing;

                if c == ' ' {
                    x_transform += self.text_state.word_spacing
                }

                x_transform *= self.text_state.horizontal_scaling;

                self.text_state.text_matrix *= Matrix::new_translation(x_transform, 0.0);
            }
        }

        Ok(())
    }

    fn draw_text(&mut self) -> PdfResult<()> {
        let _str = self.pop_string()?;

        todo!("unimplemented operator: draw text unadjusted")
    }

    /// Save the current graphics state on the graphics state stack
    fn save_graphics_state(&mut self) -> PdfResult<()> {
        self.graphics_state_stack.push(self.graphics_state.clone());

        Ok(())
    }

    fn restore_graphics_state(&mut self) -> PdfResult<()> {
        if let Some(state) = self.graphics_state_stack.pop() {
            self.graphics_state = state;
        }

        Ok(())
    }

    /// Modify the current transformation matrix (CTM) by concatenating the
    /// specified matrix. Although the operands specify a matrix, they shall be
    /// written as six separate numbers, not as an array.
    fn transform_ctm(&mut self) -> PdfResult<()> {
        let f = self.pop_number()?;
        let e = self.pop_number()?;
        let d = self.pop_number()?;
        let c = self.pop_number()?;
        let b = self.pop_number()?;
        let a = self.pop_number()?;

        let matrix = Matrix::new(a, b, c, d, e, f);

        self.graphics_state
            .device_independent
            .current_transformation_matrix *= matrix;

        Ok(())
    }

    /// Paint the specified XObject. The operand name shall appear as a key in
    /// the XObject subdictionary of the current resource dictionary. The
    /// associated value shall be a stream whose Type entry, if present, is XObject.
    ///
    /// The effect of `Do` depends on the value of the XObject’s Subtype entry
    fn draw_xobject(&mut self) -> PdfResult<()> {
        let name = self.pop_name()?;

        if let Some(resources) = &self.page.resources {
            let xobject = resources
                .xobject
                .as_ref()
                .and_then(|xobject| xobject.get(&name));

            match xobject {
                Some(XObject::Image(image)) => self.canvas.draw_image(image, self.resolver)?,
                _ => todo!("unimplemented xobject"),
            }
        }

        Ok(())
    }

    /// Set the line width in the graphics state
    fn set_line_width(&mut self) -> PdfResult<()> {
        let line_width = self.pop_number()?;

        self.graphics_state.device_independent.line_width = line_width;

        Ok(())
    }

    /// Append a rectangle to the current path as a complete subpath, with
    /// lower-left corner (x, y) and dimensions width and height in user space.
    ///
    /// The operation `x y width height re` is equivalent to
    ///     x y m
    ///     (x + width) y l
    ///     (x + width) (y + height) l
    ///     x (y + height) l
    ///     h
    fn create_rectangle(&mut self) -> PdfResult<()> {
        let height = self.pop_number()?;
        let width = self.pop_number()?;
        let y = self.pop_number()?;
        let x = self.pop_number()?;

        let path = self
            .current_path
            .get_or_insert_with(|| Path::new(Point::new(0.0, 0.0)));

        path.move_to(Point::new(x, y));
        path.line_to(Point::new(x + width, y));
        path.line_to(Point::new(x + width, y + height));
        path.line_to(Point::new(x, y + height));
        path.close_path();

        Ok(())
    }

    /// Modify the current clipping path by intersecting it with the current
    /// path, using the even-odd rule to determine which regions lie inside the
    /// clipping path.
    fn set_clipping_path_even_odd(&mut self) -> PdfResult<()> {
        dbg!("unimplemented clipping path operator");

        Ok(())
    }

    /// End the path object without filling or stroking it. This operator shall
    /// be a path-painting no-op, used primarily for the side effect of changing
    /// the current clipping path
    fn draw_path_nop(&mut self) -> PdfResult<()> {
        self.current_path = None;

        Ok(())
    }

    /// Begin a marked-content sequence with an associated property list,
    /// terminated by a balancing EMC operator. tag shall be a name object
    /// indicating the role or significance of the sequence. properties shall be
    /// either an inline dictionary containing the property list or a name object
    /// associated with it in the Properties subdictionary of the current
    /// resource dictionary
    fn begin_marked_content_sequence(&mut self) -> PdfResult<()> {
        let _properties = self.pop()?;
        let _tag = self.pop_name()?;

        dbg!("todo: unimplemented marked content operator: BDC");

        Ok(())
    }

    /// End a marked-content sequence begun by a BMC or BDC operator.
    fn end_marked_content_sequence(&mut self) -> PdfResult<()> {
        dbg!("todo: unimplemented marked content operator: EMC");

        Ok(())
    }

    /// Set the text matrix, Tm, and the text line matrix, Tlm:
    ///
    /// T_m = T_lm = [a b 0, c d 0, e f 1]
    fn set_text_matrix(&mut self) -> PdfResult<()> {
        let f = self.pop_number()?;
        let e = self.pop_number()?;
        let d = self.pop_number()?;
        let c = self.pop_number()?;
        let b = self.pop_number()?;
        let a = self.pop_number()?;

        let matrix = Matrix::new(a, b, c, d, e, f);

        self.text_state.text_matrix = matrix;
        self.text_state.text_line_matrix = matrix;

        Ok(())
    }
}

enum GraphicsObject {
    Path,
    Text,
    XObject,
    InlineImage,
    Shading,
}

pdf_enum!(
    int
    #[derive(Debug)]
    enum OverprintMode {
        Zero = 0,
        NonZero = 1,
    }
);

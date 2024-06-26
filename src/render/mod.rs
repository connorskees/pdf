pub(crate) mod canvas;
pub(super) mod error;
pub(crate) mod graphics_state;
pub(crate) mod text_state;
mod wgpu;

use std::{
    borrow::Cow,
    rc::Rc,
    sync::{Arc, RwLock},
};

use crate::{
    color::{ColorSpace, ColorSpaceName},
    content::{ContentLexer, ContentToken, PdfGraphicsOperator},
    data_structures::Matrix,
    error::PdfResult,
    filter::decode_stream,
    font::{
        true_type::{ParsedTrueTypeFontFile, TrueTypeInterpreter},
        CffCharStringInterpreter, CffFile, CffParser, CidFontSubtype, CidFontWidths, CidToGidMap,
        Font, FontEncoding, FontEncodingDict, Glyph, TrueTypeFont, Type0Font, Type1Font, Type3Font,
        Type3FontFile, Widths, BASE_14_FONTS,
    },
    geometry::{Outline, Path, Point},
    objects::Object,
    page::PageObject,
    postscript::{charstring::CharStringPainter, font::Type1PostscriptFont, PostscriptInterpreter},
    resources::{
        graphics_state_parameters::{
            LineCapStyle, LineDashPattern, LineJoinStyle, RenderingIntent,
        },
        Resources,
    },
    xobject::{FormXObject, XObject},
    FromObj, Resolve,
};

use anyhow::Context;
use canvas::Canvas;
#[cfg(feature = "window")]
use minifb::Key;

use self::{
    error::PdfRenderError,
    graphics_state::{ColorSpacePosition, GraphicsState},
    text_state::{TextRenderingMode, TextState},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    EvenOdd,
    NonZeroWindingNumber,
}

pub struct Renderer<'a, 'b: 'a> {
    content: &'a mut ContentLexer<'b>,
    scene: Vec<Renderable>,
    resolver: &'a mut dyn Resolve<'b>,
    canvas: Canvas,
    graphics_state_stack: Vec<GraphicsState<'b>>,
    operand_stack: Vec<Object<'b>>,
    graphics_state: GraphicsState<'b>,

    /// A set of nine graphics state parameters that pertain only to the
    /// painting of text. These include parameters that select the font, scale
    /// the glyphs to an appropriate size, and accomplish other effects.
    text_state: TextState<'b>,
    page: Rc<PageObject<'b>>,
    resources: Option<Rc<Resources<'b>>>,
    current_path: Option<Path>,
    pending_clip: Option<FillRule>,
    marked_content_stack: Vec<MarkedContentMarker<'b>>,
    scale: f32,
}

pub(crate) fn scale_to_fit(width: f32, height: f32) -> f32 {
    let max_val = 8192.0;

    if max_val >= width && max_val >= height {
        return 1.0;
    }

    return (max_val / width).min(max_val / height);
}

#[derive(Debug)]
struct Renderable {
    outline: Outline,
    stroke_color: Option<u32>,
    fill_color: Option<u32>,
    fill_rule: Option<FillRule>,
}

impl<'a, 'b: 'a> Renderer<'a, 'b> {
    fn pop<T: FromObj<'b>>(&mut self) -> PdfResult<T> {
        Ok(self
            .operand_stack
            .pop()
            .map(|obj| T::from_obj(obj, self.resolver))
            .transpose()?
            .ok_or(PdfRenderError::StackUnderflow)?)
    }

    fn pop_number(&mut self) -> PdfResult<f32> {
        let obj = self.pop()?;

        self.resolver.assert_number(obj)
    }

    fn pop_integer(&mut self) -> PdfResult<i32> {
        let obj = self.pop()?;

        self.resolver.assert_integer(obj)
    }

    fn pop_name(&mut self) -> PdfResult<String> {
        let obj = self.pop()?;

        self.resolver.assert_name(obj)
    }

    fn pop_string(&mut self) -> PdfResult<String> {
        let obj = self.pop()?;

        self.resolver.assert_string(obj)
    }

    fn pop_arr(&mut self) -> PdfResult<Vec<Object<'b>>> {
        let obj = self.pop()?;

        self.resolver.assert_arr(obj)
    }
}

impl<'a, 'b: 'a> Renderer<'a, 'b> {
    pub fn new(
        content: &'a mut ContentLexer<'b>,
        resolver: &'a mut dyn Resolve<'b>,
        page: Rc<PageObject<'b>>,
    ) -> Self {
        let media_box = page.media_box().unwrap();

        let mut width = media_box.width().ceil();
        let mut height = media_box.height().ceil();

        let scale = scale_to_fit(width, height);
        width *= scale;
        height *= scale;

        let mut graphics_state = GraphicsState::default();
        graphics_state.device_independent.clipping_path = media_box.as_path();

        Self {
            content,
            resolver,
            canvas: Canvas::new(width as usize, height as usize),
            graphics_state_stack: Vec::new(),
            operand_stack: Vec::new(),
            graphics_state,
            text_state: TextState::default(),
            resources: page.resources(),
            page,
            current_path: None,
            pending_clip: None,
            marked_content_stack: Vec::new(),
            scale,
            scene: Vec::new(),
        }
    }

    fn current_transformation_matrix(&self) -> Matrix {
        self.graphics_state
            .device_independent
            .current_transformation_matrix
    }

    fn render_form_xobject(&mut self, mut form: FormXObject<'b>) -> PdfResult<()> {
        let content_buffer: Cow<'b, [u8]> = decode_stream(
            unsafe { &*(&*form.stream.stream as *const _) },
            &form.stream.dict,
            self.resolver,
        )?;

        let mut form_content = ContentLexer::new(content_buffer);

        self.save_graphics_state()?;
        std::mem::swap(self.content, &mut form_content);
        std::mem::swap(&mut self.resources, &mut form.resources);
        self.graphics_state
            .device_independent
            .current_transformation_matrix *= form.matrix;

        self.render_content_stream()?;

        std::mem::swap(self.content, &mut form_content);
        std::mem::swap(&mut self.resources, &mut form.resources);

        self.restore_graphics_state()?;

        Ok(())
    }

    fn stroking_color(&self) -> &ColorSpace {
        &self.graphics_state.device_independent.color_space.stroking
    }

    fn non_stroking_color(&self) -> &ColorSpace {
        &self
            .graphics_state
            .device_independent
            .color_space
            .nonstroking
    }

    fn render_content_stream(&mut self) -> PdfResult<()> {
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
                    PdfGraphicsOperator::Tj => self.draw_text_unadjusted()?,
                    PdfGraphicsOperator::q => self.save_graphics_state()?,
                    PdfGraphicsOperator::Q => self.restore_graphics_state()?,
                    PdfGraphicsOperator::cm => self.transform_ctm()?,
                    PdfGraphicsOperator::Do => self.draw_xobject()?,
                    PdfGraphicsOperator::w => self.set_line_width()?,
                    PdfGraphicsOperator::re => self.create_rectangle()?,
                    PdfGraphicsOperator::W => self.set_clipping_path_non_zero_winding_number()?,
                    PdfGraphicsOperator::W_star => self.set_clipping_path_even_odd()?,
                    PdfGraphicsOperator::n => self.draw_path_nop()?,
                    PdfGraphicsOperator::RG => self.set_stroking_rgb()?,
                    PdfGraphicsOperator::rg => self.set_nonstroking_rgb()?,
                    PdfGraphicsOperator::ET => self.end_text()?,
                    PdfGraphicsOperator::BDC => {
                        self.begin_marked_content_sequence_with_property_list()?
                    }
                    PdfGraphicsOperator::EMC => self.end_marked_content_sequence()?,
                    PdfGraphicsOperator::Tm => self.set_text_matrix()?,
                    PdfGraphicsOperator::gs => self.set_graphics_state_parameters()?,
                    PdfGraphicsOperator::f | PdfGraphicsOperator::F => {
                        self.fill_path(FillRule::NonZeroWindingNumber)?
                    }
                    PdfGraphicsOperator::f_star => self.fill_path(FillRule::EvenOdd)?,
                    PdfGraphicsOperator::m => self.move_to()?,
                    PdfGraphicsOperator::l => self.line_to()?,
                    PdfGraphicsOperator::h => self.close_path()?,
                    PdfGraphicsOperator::S => self.stroke_path()?,
                    PdfGraphicsOperator::k => self.set_nonstroking_cmyk()?,
                    PdfGraphicsOperator::K => self.set_stroking_cmyk()?,
                    PdfGraphicsOperator::TL => self.set_text_leading()?,
                    PdfGraphicsOperator::c => self.curve_to()?,
                    PdfGraphicsOperator::v => self.curve_to_initial_replicated()?,
                    PdfGraphicsOperator::y => self.curve_to_final_replicated()?,
                    PdfGraphicsOperator::CS => self.set_stroking_color_space()?,
                    PdfGraphicsOperator::cs => self.set_nonstroking_color_space()?,
                    PdfGraphicsOperator::SC | PdfGraphicsOperator::SCN => {
                        self.set_stroking_color()?
                    }
                    PdfGraphicsOperator::sc | PdfGraphicsOperator::scn => {
                        self.set_nonstroking_color()?
                    }
                    PdfGraphicsOperator::i => self.set_flatness_tolerance()?,
                    PdfGraphicsOperator::Tc => self.set_character_spacing()?,
                    PdfGraphicsOperator::Tw => self.set_word_spacing()?,
                    PdfGraphicsOperator::Ts => self.set_text_rise()?,
                    PdfGraphicsOperator::Tz => self.set_horizontal_scaling()?,
                    PdfGraphicsOperator::Tr => self.set_text_rendering_mode()?,
                    PdfGraphicsOperator::TD => self.move_text_position_and_set_leading()?,
                    PdfGraphicsOperator::T_star => self.move_to_next_line()?,
                    PdfGraphicsOperator::BMC => self.begin_marked_content_sequence()?,
                    PdfGraphicsOperator::J => self.set_line_cap_style()?,
                    PdfGraphicsOperator::d => self.set_line_dash_pattern()?,
                    PdfGraphicsOperator::j => self.set_line_join_style()?,
                    PdfGraphicsOperator::B => {
                        self.stroke_and_fill(FillRule::NonZeroWindingNumber)?
                    }
                    PdfGraphicsOperator::B_star => self.stroke_and_fill(FillRule::EvenOdd)?,
                    PdfGraphicsOperator::M => self.set_miter_limit()?,
                    PdfGraphicsOperator::s => self.close_and_stroke_path()?,
                    // compat section is handled in lexer
                    PdfGraphicsOperator::BX | PdfGraphicsOperator::EX => {}
                    PdfGraphicsOperator::sh => self.paint_using_shading_pattern()?,
                    PdfGraphicsOperator::ri => self.set_color_rendering_intent()?,
                    _ => todo!("unimplemented operator: {:?}", op),
                },
            }
        }

        Ok(())
    }

    fn paint_using_shading_pattern(&mut self) -> PdfResult<()> {
        let name = self.pop_name()?;

        if let Some(resources) = &self.resources {
            let shade = resources
                .shading
                .as_ref()
                .and_then(|shading| shading.get(&name));

            match shade {
                Some(shade) => todo!("{:#?}", shade),
                None => todo!("unable to locate shade {:#?}", name),
            }
        }

        Ok(())
    }

    fn stroke_and_fill(&mut self, fill_rule: FillRule) -> PdfResult<()> {
        let stroke_color = self.stroking_color().as_u32();
        let fill_color = self.non_stroking_color().as_u32();

        let ctm = self.current_transformation_matrix();
        let mut path = self
            .current_path
            .take()
            .unwrap_or_else(|| Path::new(Point::origin()));

        path.apply_transform(ctm);

        if let Some(clip) = self.pending_clip.take() {
            println!("unimplemented clipping path operator {:?}", clip);
        }

        // todo: gpu stroke
        self.canvas.stroke_path(&path, stroke_color);
        self.scene.push(Renderable {
            outline: Outline::new(vec![path]),
            stroke_color: None,
            fill_color: Some(fill_color),
            fill_rule: Some(fill_rule),
        });

        Ok(())
    }

    fn render_cpu(&mut self) -> PdfResult<()> {
        #[cfg(feature = "window")]
        {
            let mut transform = Matrix::identity();
            let mut changed = true;

            while self.canvas.window.is_open() && !self.canvas.window.is_key_down(Key::Escape) {
                let start = std::time::Instant::now();
                if changed {
                    self.canvas.clear();
                    for renderable in &self.scene {
                        if let Some(stroke_color) = renderable.stroke_color {
                            let mut outline = renderable.outline.clone();
                            outline.apply_transform(transform);
                            // outline.apply_transform(Matrix::new_scale(self.scale, self.scale));

                            self.canvas.fill_outline_even_odd(&outline, stroke_color);
                            break;
                        }
                    }
                    changed = false;
                }
                dbg!(start.elapsed());

                if let Some((x, y)) = self.canvas.window.get_scroll_wheel() {
                    let is_ctrl = self.canvas.window.is_key_down(Key::LeftSuper)
                        || self.canvas.window.is_key_down(Key::RightSuper);
                    if is_ctrl {
                        let y_pct = y / (self.canvas.height as f32 / 2.0);
                        self.scale = 1.0 + y_pct;
                        self.scale = self.scale.min(1.05).max(0.95);
                        transform *= Matrix::new_scale(self.scale, self.scale);
                    } else {
                        transform *= Matrix::new_translation(x * 1.5, -y * 1.5);
                    }

                    changed = true;
                }

                if self.canvas.window.is_key_down(Key::R) {
                    transform = Matrix::identity();
                }

                self.canvas.refresh();
            }

            self.canvas.draw(&mut self.scale);
        }

        Ok(())
    }

    fn render_gpu(&mut self) -> PdfResult<()> {
        pollster::block_on(wgpu::run(
            &self.scene,
            self.canvas.width as f32,
            self.canvas.height as f32,
        ));

        Ok(())
    }

    pub fn render(mut self) -> PdfResult<()> {
        self.render_content_stream()?;

        let gpu = true;

        if gpu {
            self.render_gpu()?;
        } else {
            self.render_cpu()?;
        }

        Ok(())
    }

    fn get_color_space(&mut self, pos: ColorSpacePosition) -> PdfResult<ColorSpace<'b>> {
        let color_space = self.graphics_state.get_color_space(pos);

        Ok(match color_space {
            ColorSpace::IccBased { stream, .. } => {
                let stream = Rc::clone(stream);

                let mut channels = Vec::new();
                for _ in 0..stream.num_of_color_components {
                    channels.push(self.pop_number()?);
                }

                ColorSpace::IccBased { stream, channels }
            }
            ColorSpace::Separation(space) => {
                let mut space = space.clone();
                let tint = self.pop_number()?;

                space.tint = tint;

                ColorSpace::Separation(space)
            }
            ColorSpace::DeviceN(..) => {
                todo!()
            }
            ColorSpace::Pattern(..) => {
                let name = self.pop_name()?;

                let pattern = self
                    .resources
                    .as_ref()
                    .unwrap()
                    .pattern
                    .as_ref()
                    .unwrap()
                    .get(&name)
                    .map(Rc::clone);

                ColorSpace::Pattern(pattern)
            }
            ColorSpace::Indexed { space, .. } => {
                let space = Rc::clone(space);
                let index = self.pop::<u32>()?;

                ColorSpace::Indexed { index, space }
            }
            ColorSpace::DeviceGray(..) | ColorSpace::CalGray { .. } => {
                todo!()
            }
            ColorSpace::DeviceRGB { .. } => {
                let blue = self.pop_number()?;
                let green = self.pop_number()?;
                let red = self.pop_number()?;

                ColorSpace::DeviceRGB { red, green, blue }
            }
            ColorSpace::Lab { .. } | ColorSpace::CalRGB { .. } => {
                todo!()
            }
            ColorSpace::DeviceCMYK { .. } => {
                let key = self.pop_number()?;
                let yellow = self.pop_number()?;
                let magenta = self.pop_number()?;
                let cyan = self.pop_number()?;

                ColorSpace::DeviceCMYK {
                    cyan,
                    magenta,
                    yellow,
                    key,
                }
            }
        })
    }

    fn set_nonstroking_color(&mut self) -> PdfResult<()> {
        let color_space = self.get_color_space(ColorSpacePosition::Nonstroking)?;

        self.graphics_state
            .device_independent
            .color_space
            .nonstroking = color_space;

        Ok(())
    }

    fn set_stroking_color(&mut self) -> PdfResult<()> {
        let color_space = self.get_color_space(ColorSpacePosition::Stroking)?;

        self.graphics_state.device_independent.color_space.stroking = color_space;

        Ok(())
    }

    /// Move to the start of the next line. This operator has the same effect
    /// as the code
    ///
    /// 0 -Tl Td
    ///
    /// where Tl denotes the current leading parameter in the text state. The
    /// negative of Tl is used here because Tl is the text leading expressed as
    /// a positive number. Going to the next line entails decreasing the y
    /// coordinate.
    fn move_to_next_line(&mut self) -> PdfResult<()> {
        let matrix = Matrix::new_translation(0.0, -self.text_state.leading)
            * self.text_state.text_line_matrix;

        self.text_state.text_matrix = matrix;
        self.text_state.text_line_matrix = matrix;

        Ok(())
    }

    /// Set the current colour space to use for stroking operations. The operand
    /// name shall be a name object. If the colour space is one that can be
    /// specified by a name and no additional parameters (DeviceGray, DeviceRGB,
    /// DeviceCMYK, and certain cases of Pattern), the name may be specified
    /// directly. Otherwise, it shall be a name defined in the ColorSpace subdictionary
    /// of the current resource dictionary; the associated value shall be an
    /// array describing the colour space
    ///
    /// The names DeviceGray, DeviceRGB, DeviceCMYK, and Pattern always identify
    /// the corresponding colour spaces directly; they never refer to resources
    /// in the ColorSpace subdictionary.
    ///
    /// The CS operator shall also set the current stroking colour to its initial
    /// value, which depends on the colour space:
    ///
    /// In a DeviceGray, DeviceRGB, CalGray, or CalRGB colour space, the initial
    /// colour shall have all components equal to 0.0.
    ///
    /// In a DeviceCMYK colour space, the initial colour shall be [0.0 0.0 0.0 1.0].
    ///
    /// In a Lab or ICCBased colour space, the initial colour shall have all
    /// components equal to 0.0 unless that falls outside the intervals specified
    /// by the space’s Range entry, in which case the nearest valid value shall
    /// be substituted.
    ///
    /// In an Indexed colour space, the initial colour value shall be 0.
    ///
    /// In a Separation or DeviceN colour space, the initial tint value shall be
    /// 1.0 for all colorants.
    ///
    /// In a Pattern colour space, the initial colour shall be a pattern object
    /// that causes nothing to be painted.
    fn set_stroking_color_space(&mut self) -> PdfResult<()> {
        let name = self.pop_name()?;
        let color_space = if let Ok(name) = ColorSpaceName::from_str(&name) {
            ColorSpace::init(name)
        } else {
            let color_space = self
                .resources
                .as_ref()
                .unwrap()
                .color_space
                .as_ref()
                .unwrap()
                .get(&name)
                .cloned()
                .unwrap();

            color_space
        };

        self.graphics_state.device_independent.color_space.stroking = color_space;

        Ok(())
    }

    fn set_nonstroking_color_space(&mut self) -> PdfResult<()> {
        let name = self.pop_name()?;
        let color_space = if let Ok(name) = ColorSpaceName::from_str(&name) {
            ColorSpace::init(name)
        } else {
            let color_space = self
                .resources
                .as_ref()
                .unwrap()
                .color_space
                .as_ref()
                .unwrap()
                .get(&name)
                .cloned()
                .unwrap();

            color_space
        };

        self.graphics_state
            .device_independent
            .color_space
            .nonstroking = color_space;

        Ok(())
    }

    /// Append a cubic Bézier curve to the current path. The curve shall extend
    /// from the current point to the point (x3, y3), using the current point and
    /// (x2, y2) as the Bézier control points
    ///
    /// The new current point shall be (x3, y3).
    fn curve_to_initial_replicated(&mut self) -> PdfResult<()> {
        let y3 = self.pop_number()?;
        let x3 = self.pop_number()?;
        let y2 = self.pop_number()?;
        let x2 = self.pop_number()?;

        if let Some(path) = self.current_path.as_mut() {
            path.cubic_curve_to(path.current_point, Point::new(x2, y2), Point::new(x3, y3));
        }

        Ok(())
    }

    /// Append a cubic Bézier curve to the current path. The curve shall extend
    /// from the current point to the point (x3, y3), using (x1, y1) and (x3, y3)
    /// as the Bézier control points
    ///
    /// The new current point shall be (x3, y3).
    fn curve_to_final_replicated(&mut self) -> PdfResult<()> {
        let y3 = self.pop_number()?;
        let x3 = self.pop_number()?;
        let y1 = self.pop_number()?;
        let x1 = self.pop_number()?;

        if let Some(path) = self.current_path.as_mut() {
            path.cubic_curve_to(Point::new(x1, y1), Point::new(x3, y3), Point::new(x3, y3));
        }

        Ok(())
    }

    /// Append a cubic Bézier curve to the current path. The curve shall extend
    /// from the current point to the point (x3, y3), using (x1, y1) and (x2, y2)
    /// as the Bézier control points
    ///
    /// The new current point shall be (x3, y3).
    fn curve_to(&mut self) -> PdfResult<()> {
        let y3 = self.pop_number()?;
        let x3 = self.pop_number()?;
        let y2 = self.pop_number()?;
        let x2 = self.pop_number()?;
        let y1 = self.pop_number()?;
        let x1 = self.pop_number()?;

        if let Some(path) = self.current_path.as_mut() {
            path.cubic_curve_to(Point::new(x1, y1), Point::new(x2, y2), Point::new(x3, y3));
        }

        Ok(())
    }

    /// Set the line dash pattern in the graphics state
    fn set_line_dash_pattern(&mut self) -> PdfResult<()> {
        let dash_phase = self.pop_number()?;
        let dash_array = self
            .pop_arr()?
            .into_iter()
            .map(|obj| self.resolver.assert_number(obj))
            .collect::<PdfResult<Vec<f32>>>()?;

        self.graphics_state.device_independent.line_dash_pattern =
            LineDashPattern::new(dash_phase, dash_array);

        Ok(())
    }

    /// Set the line join style in the graphics state
    fn set_line_join_style(&mut self) -> PdfResult<()> {
        let line_join_style = self.pop_integer()?;

        self.graphics_state.device_independent.line_join_style =
            LineJoinStyle::from_integer(line_join_style)?;

        Ok(())
    }

    fn set_miter_limit(&mut self) -> PdfResult<()> {
        let miter_limit = self.pop_number()?;

        self.graphics_state.device_independent.miter_limit = miter_limit;

        Ok(())
    }

    fn set_color_rendering_intent(&mut self) -> PdfResult<()> {
        let rendering_intent = RenderingIntent::from_str(&self.pop_name()?)?;

        self.graphics_state.device_independent.rendering_intent = rendering_intent;

        Ok(())
    }

    /// Set the line cap style in the graphics state
    fn set_line_cap_style(&mut self) -> PdfResult<()> {
        let line_cap_style = self.pop_integer()?;

        self.graphics_state.device_independent.line_cap_style =
            LineCapStyle::from_integer(line_cap_style)?;

        Ok(())
    }

    fn set_text_leading(&mut self) -> PdfResult<()> {
        let leading = self.pop_number()?;

        self.text_state.leading = leading;

        Ok(())
    }

    fn set_flatness_tolerance(&mut self) -> PdfResult<()> {
        let tolerance = self.pop_number()?;

        self.graphics_state.device_dependent.flatness_tolerance = tolerance;

        Ok(())
    }

    fn set_character_spacing(&mut self) -> PdfResult<()> {
        let spacing = self.pop_number()?;

        self.text_state.character_spacing = spacing;

        Ok(())
    }

    fn set_word_spacing(&mut self) -> PdfResult<()> {
        let spacing = self.pop_number()?;

        self.text_state.word_spacing = spacing;

        Ok(())
    }

    fn set_text_rise(&mut self) -> PdfResult<()> {
        let rise = self.pop_number()?;

        self.text_state.rise = rise;

        Ok(())
    }

    fn set_horizontal_scaling(&mut self) -> PdfResult<()> {
        let horizontal_scaling = self.pop_number()?;

        self.text_state.horizontal_scaling = horizontal_scaling / 100.0;

        Ok(())
    }

    fn set_text_rendering_mode(&mut self) -> PdfResult<()> {
        let rendering_mode = self.pop_integer()?;

        self.text_state.rendering_mode = TextRenderingMode::from_integer(rendering_mode)?;

        Ok(())
    }

    /// Set the stroking colour space to DeviceCMYK (or the DefaultCMYK colour
    /// space) and set the colour to use for stroking operations. Each operand
    /// shall be a number between 0.0 (zero concentration) and 1.0 (maximum
    /// concentration). The behaviour of this operator is affected by the overprint
    /// mode
    fn set_stroking_cmyk(&mut self) -> PdfResult<()> {
        let key = self.pop_number()?;
        let yellow = self.pop_number()?;
        let magenta = self.pop_number()?;
        let cyan = self.pop_number()?;

        self.graphics_state.device_independent.color_space.stroking = ColorSpace::DeviceCMYK {
            cyan,
            magenta,
            yellow,
            key,
        };

        Ok(())
    }

    /// Same as [Renderer::set_stroking_cmyk], but used for nonstroking operations
    fn set_nonstroking_cmyk(&mut self) -> PdfResult<()> {
        let key = self.pop_number()?;
        let yellow = self.pop_number()?;
        let magenta = self.pop_number()?;
        let cyan = self.pop_number()?;

        self.graphics_state
            .device_independent
            .color_space
            .nonstroking = ColorSpace::DeviceCMYK {
            cyan,
            magenta,
            yellow,
            key,
        };

        Ok(())
    }

    /// Set the specified parameters in the graphics state. dictName shall be
    /// the name of a graphics state parameter dictionary in the ExtGState
    /// subdictionary of the current resource dictionary
    fn set_graphics_state_parameters(&mut self) -> PdfResult<()> {
        let dict_name = self.pop_name()?;

        let graphics_state_parameters = self
            .resources
            .as_ref()
            .and_then(|res| res.ext_g_state.as_ref())
            .and_then(|state_map| state_map.get(&dict_name));

        match graphics_state_parameters {
            Some(gsp) => gsp.update_graphics_state(&mut self.graphics_state, &mut self.text_state),
            None => todo!("{}", &dict_name),
        }

        Ok(())
    }

    /// Stroke the path.
    fn stroke_path(&mut self) -> PdfResult<()> {
        let color = self.stroking_color().as_u32();

        let ctm = self.current_transformation_matrix();
        let mut path = self
            .current_path
            .take()
            .unwrap_or_else(|| Path::new(Point::origin()));

        path.apply_transform(ctm);

        if let Some(clip) = self.pending_clip.take() {
            println!("unimplemented clipping path operator {:?}", clip);
        }

        self.canvas.stroke_path(&path, color);

        Ok(())
    }

    /// Close and stroke the path. This operator shall have the same effect as
    /// the sequence `h S`.
    fn close_and_stroke_path(&mut self) -> PdfResult<()> {
        let color = self.stroking_color().as_u32();

        let ctm = self.current_transformation_matrix();
        let mut path = self
            .current_path
            .take()
            .unwrap_or_else(|| Path::new(Point::origin()));

        path.close_path();

        path.apply_transform(ctm);

        if let Some(clip) = self.pending_clip.take() {
            println!("unimplemented clipping path operator {:?}", clip);
        }

        self.canvas.stroke_path(&path, color);

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
            .get_or_insert_with(|| Path::new(Point::origin()));

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
            .get_or_insert_with(|| Path::new(Point::origin()));

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
        let mut path = match self.current_path.take() {
            Some(p) => p,
            None => return Ok(()),
        };

        let color = self
            .graphics_state
            .device_independent
            .color_space
            .nonstroking
            .as_u32();

        path.clip(&self.graphics_state.device_independent.clipping_path);
        path.apply_transform(self.current_transformation_matrix());

        if let Some(clip) = self.pending_clip.take() {
            println!("unimplemented clipping path operator {:?}", clip);
        }

        // todo: don't fill shapes we know to be completely off screen
        if false
            && !path.bounding_box().overlaps(
                self.graphics_state
                    .device_independent
                    .clipping_path
                    .clone()
                    .bounding_box(),
            )
        {
            println!("skipping path outside bounds of clipping path");
            return Ok(());
        }

        self.scene.push(Renderable {
            outline: Outline::new(vec![path]),
            stroke_color: None,
            fill_color: Some(color),
            fill_rule: Some(fill_rule),
        });

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

    /// Move to the start of the next line, offset from the start of the current
    /// line by (t_x, t_y). As a side effect, this operator shall set the leading
    /// parameter in the text state. This operator shall have the same effect as
    /// this code:
    ///
    /// −t_y TL
    /// t_x t_y Td
    ///
    fn move_text_position_and_set_leading(&mut self) -> PdfResult<()> {
        let t_y = self.pop_number()?;
        let t_x = self.pop_number()?;

        self.text_state.leading = -t_y;

        let matrix = Matrix::new_translation(t_x, t_y) * self.text_state.text_line_matrix;

        self.text_state.text_matrix = matrix;
        self.text_state.text_line_matrix = matrix;

        Ok(())
    }

    fn draw_text(&mut self, arr: Vec<Object<'b>>) -> PdfResult<()> {
        // todo: should actually get undefined/default width from the glyph itself
        let default_width = Widths::new(Some(Vec::new()), Some(0), Some(0), 1000.0).unwrap();
        let ffs: Cow<[u8]>;
        let stream: Cow<[u8]>;
        let font: Arc<RwLock<dyn RenderableFont>>;
        let cid_font_widths: CidFontWidths;
        let widths: &dyn FontMetrics;

        match self.text_state.font.as_deref() {
            Some(Font::Type1(Type1Font {
                base, base_font, ..
            })) => {
                let font_file = base
                    .font_descriptor
                    .as_ref()
                    .and_then(|desc| desc.font_file.clone());

                if font_file.is_none() {
                    if base.font_descriptor.is_none() {
                        let base_font = BASE_14_FONTS.get(&base_font.as_ref()).unwrap();

                        font = Arc::<RwLock<Type1PostscriptFont>>::clone(base_font);
                        widths = base.widths.as_ref().unwrap_or(&default_width);
                    } else {
                        let descriptor = base.font_descriptor.as_ref().unwrap();

                        if descriptor.font_file_three.is_none() {
                            let base_font = BASE_14_FONTS.get(&base_font.as_ref()).unwrap();

                            font = Arc::<RwLock<Type1PostscriptFont>>::clone(base_font);
                            widths = base.widths.as_ref().unwrap_or(&default_width);
                        } else {
                            match descriptor.font_file_three.as_ref().unwrap().clone() {
                                Type3FontFile::CompactType1(compact_type1) => {
                                    ffs = compact_type1.stream.stream;
                                    stream = decode_stream(
                                        &ffs,
                                        &compact_type1.stream.dict,
                                        self.resolver,
                                    )?;

                                    let cff_file = CffFile::load(&stream)?;

                                    font = Arc::new(RwLock::new(cff_file));
                                    widths = base.widths.as_ref().unwrap_or(&default_width);
                                }
                                Type3FontFile::CompactType0Cid(_) => todo!(),
                                Type3FontFile::OpenType(_) => todo!(),
                            }
                        }
                    }
                } else {
                    let font_file = font_file.unwrap();
                    ffs = font_file.stream.stream;
                    stream = decode_stream(&ffs, &font_file.stream.dict, self.resolver)?;

                    font = Arc::new(RwLock::new(Type1PostscriptFont::load(&stream)?));
                    widths = base.widths.as_ref().unwrap();
                }
            }
            Some(Font::TrueType(TrueTypeFont { base, .. })) => {
                let font_file = base
                    .font_descriptor
                    .as_ref()
                    .and_then(|desc| desc.font_file_two.clone());

                widths = base.widths.as_ref().unwrap();

                if font_file.is_none() {
                    let base_font = BASE_14_FONTS
                        .get(&base.font_descriptor.as_ref().unwrap().font_name.0.as_ref())
                        .unwrap();

                    font = Arc::<RwLock<Type1PostscriptFont>>::clone(base_font);
                } else {
                    let font_file = font_file.unwrap();
                    ffs = font_file.stream.stream;
                    stream = decode_stream(&ffs, &font_file.stream.dict, self.resolver)?;

                    font = Arc::new(RwLock::new(TrueTypeInterpreter::load(&stream)?));
                }
            }
            Some(Font::Type0(Type0Font {
                descendant_font: [descendant_font],
                ..
            })) => {
                assert_eq!(descendant_font.cid_to_gid_map, CidToGidMap::Identity);

                match descendant_font.subtype {
                    CidFontSubtype::CidFontType0 => {
                        let font_file = descendant_font.font_descriptor.font_file.clone();
                        if font_file.is_none() {
                            println!("skipping unsupported type 0 cid font");
                            return Ok(());
                        }

                        let font_file = font_file.unwrap();
                        ffs = font_file.stream.stream;
                        stream = decode_stream(&ffs, &font_file.stream.dict, self.resolver)?;
                        font = Arc::new(RwLock::new(Type1PostscriptFont::load(&stream)?));
                    }
                    CidFontSubtype::CidFontType2 => {
                        let font_file = descendant_font
                            .font_descriptor
                            .font_file_two
                            .clone()
                            .unwrap();

                        ffs = font_file.stream.stream;

                        stream = decode_stream(&ffs, &font_file.stream.dict, self.resolver)?;

                        font = Arc::new(RwLock::new(TrueTypeInterpreter::load(&stream)?));
                    }
                }

                if descendant_font.default_width != 1000 {
                    cid_font_widths = CidFontWidths::with_default(descendant_font.default_width);
                    widths = &cid_font_widths;
                } else {
                    widths = &descendant_font.widths;
                }
            }
            Some(Font::Type3(type3_font)) => {
                widths = type3_font.base.widths.as_ref().unwrap();
                font = Arc::new(RwLock::new(type3_font.clone()));
            }
            Some(font @ Font::MmType1(_)) => todo!("unimplemented mm font: {:#?}", font),
            None => todo!("no font selected in text state"),
        };

        assert_eq!(self.text_state.rendering_mode, TextRenderingMode::Fill);

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

            for c in s.chars() {
                let trm = Matrix::new(
                    self.text_state.font_size * self.text_state.horizontal_scaling,
                    0.0,
                    0.0,
                    self.text_state.font_size,
                    0.0,
                    self.text_state.rise,
                ) * font.read().unwrap().font_matrix()
                    * self.text_state.text_matrix
                    * self
                        .graphics_state
                        .device_independent
                        .current_transformation_matrix;

                let mut glyph = font.write().unwrap().evaluate(c as u32, self.resolver)?;

                glyph.outline.apply_transform(trm);

                self.scene.push(Renderable {
                    outline: glyph.outline,
                    stroke_color: Some(
                        self.graphics_state
                            .device_independent
                            .color_space
                            .stroking
                            .as_u32(),
                    ),
                    fill_color: None,
                    fill_rule: None,
                });

                let mut x_transform = widths.get(c as u32) * self.text_state.font_size
                    + self.text_state.character_spacing;

                // todo: this should actually consult the cmap
                if c == ' ' {
                    x_transform += self.text_state.word_spacing * self.scale;
                }

                x_transform *= self.text_state.horizontal_scaling;

                self.text_state.text_matrix *= Matrix::new_translation(x_transform, 0.0);
            }
        }

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

        self.draw_text(arr)?;

        Ok(())
    }

    fn draw_text_unadjusted(&mut self) -> PdfResult<()> {
        let s = self.pop_string()?;

        self.draw_text(vec![Object::String(s)])?;

        Ok(())
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

        if let Some(resources) = &self.resources {
            let xobject = resources
                .xobject
                .as_ref()
                .and_then(|xobject| xobject.get(&name));

            match xobject {
                Some(XObject::Image(image)) => self.canvas.draw_image(image, self.resolver)?,
                Some(XObject::Form(form)) => {
                    let form: FormXObject<'b> = FormXObject::clone(form);

                    self.render_form_xobject(form)?
                }
                Some(XObject::PostScript(ps_obj)) => {
                    todo!("unimplemented postscript xobject {:#?}", ps_obj)
                }
                None => todo!("unable to find xobject {:#?}", name),
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
            .get_or_insert_with(|| Path::new(Point::origin()));

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
        self.pending_clip = Some(FillRule::EvenOdd);

        Ok(())
    }

    /// Modify the current clipping path by intersecting it with the current path,
    /// using the nonzero winding number rule to determine which regions lie inside
    /// the clipping path.
    fn set_clipping_path_non_zero_winding_number(&mut self) -> PdfResult<()> {
        self.pending_clip = Some(FillRule::NonZeroWindingNumber);

        Ok(())
    }

    /// End the path object without filling or stroking it. This operator shall
    /// be a path-painting no-op, used primarily for the side effect of changing
    /// the current clipping path
    fn draw_path_nop(&mut self) -> PdfResult<()> {
        if let Some(clip) = self.pending_clip.take() {
            println!("unimplemented clipping path operator {:?}", clip);
        }

        self.current_path = None;

        Ok(())
    }

    /// Begin a marked-content sequence terminated by a balancing EMC operator.
    /// tag shall be a name object indicating the role or significance of the
    /// sequence.
    fn begin_marked_content_sequence(&mut self) -> PdfResult<()> {
        let tag = self.pop_name()?;

        self.marked_content_stack.push(MarkedContentMarker {
            tag,
            properties: None,
        });

        Ok(())
    }

    /// Begin a marked-content sequence with an associated property list,
    /// terminated by a balancing EMC operator. tag shall be a name object
    /// indicating the role or significance of the sequence. properties shall be
    /// either an inline dictionary containing the property list or a name object
    /// associated with it in the Properties subdictionary of the current
    /// resource dictionary
    fn begin_marked_content_sequence_with_property_list(&mut self) -> PdfResult<()> {
        let properties = self.pop::<Object<'b>>()?;
        let tag = self.pop_name()?;

        self.marked_content_stack.push(MarkedContentMarker {
            tag,
            properties: Some(properties),
        });

        Ok(())
    }

    /// End a marked-content sequence begun by a BMC or BDC operator.
    fn end_marked_content_sequence(&mut self) -> PdfResult<()> {
        self.marked_content_stack.pop();

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

#[pdf_enum(Integer)]
enum OverprintMode {
    Zero = 0,
    NonZero = 1,
}

impl<'a, 'b> RenderableFont<'a, 'b> for Type3Font<'b> {
    fn load(_stream: &'a [u8]) -> PdfResult<Self>
    where
        Self: Sized,
    {
        unimplemented!("type 3 fonts are loaded by the pdf parser itself")
    }

    fn evaluate(&mut self, codepoint: u32, resolver: &mut dyn Resolve<'b>) -> PdfResult<Glyph> {
        let name = match &self.encoding {
            FontEncoding::Base(_) => todo!(),
            FontEncoding::Dictionary(FontEncodingDict { differences, .. }) => {
                differences.as_ref().unwrap().0.get(&codepoint).unwrap()
            }
        };

        let proc = self.char_procs.get(name).unwrap();

        let decoded = decode_stream(&proc.stream, &proc.dict, resolver)?;
        let decoded = String::from_utf8_lossy(&decoded);

        // todo: we actually probably want to take in a reference to the renderer itself!
        println!("Skipping type 3 font stream: {}", decoded.trim());

        Ok(Glyph::empty())
    }

    fn font_matrix(&self) -> Matrix {
        self.font_matrix
    }
}

pub trait RenderableFont<'a, 'b> {
    fn load(stream: &'a [u8]) -> PdfResult<Self>
    where
        Self: Sized;
    // todo: optimize this to cache construction of evaluation engine
    fn evaluate(&mut self, codepoint: u32, resolver: &mut dyn Resolve<'b>) -> PdfResult<Glyph>;
    fn font_matrix(&self) -> Matrix;
}

impl<'a, 'b> RenderableFont<'a, 'b> for Type1PostscriptFont {
    fn load(stream: &'a [u8]) -> PdfResult<Self>
    where
        Self: Sized,
    {
        let mut interpreter = PostscriptInterpreter::new(stream);
        interpreter.run()?;

        let font = interpreter.fonts.into_values().next().unwrap();

        Ok(font)
    }

    fn evaluate(&mut self, codepoint: u32, _resolver: &mut dyn Resolve<'b>) -> PdfResult<Glyph> {
        let mut painter = CharStringPainter::new(self);
        painter.evaluate(codepoint)
    }

    fn font_matrix(&self) -> Matrix {
        self.font_matrix
    }
}

impl<'a, 'b: 'a> RenderableFont<'a, 'b> for TrueTypeInterpreter<'a> {
    fn load(stream: &'a [u8]) -> PdfResult<Self>
    where
        Self: Sized,
    {
        let file = ParsedTrueTypeFontFile::new(stream)?;
        Ok(TrueTypeInterpreter::new(file))
    }

    fn evaluate(&mut self, codepoint: u32, _resolver: &mut dyn Resolve<'b>) -> PdfResult<Glyph> {
        self.render_glyph(codepoint)
    }

    fn font_matrix(&self) -> Matrix {
        Matrix::identity()
    }
}

impl<'a, 'b: 'a> RenderableFont<'a, 'b> for CffFile<'a> {
    fn load(stream: &'a [u8]) -> PdfResult<Self>
    where
        Self: Sized,
    {
        CffParser::new(stream).parse().context("cff parsing")
    }

    fn evaluate(&mut self, codepoint: u32, _resolver: &mut dyn Resolve<'b>) -> PdfResult<Glyph> {
        let idx = self.encoding.lookup(codepoint).unwrap_or(0);
        // todo: validate this doesn't need idx + 1
        let cs = match self.charstring_index.get(idx as usize) {
            Some(cs) => cs,
            None => {
                println!(
                    "found invalid cff charstring idx: {:?} for len: {}",
                    idx, self.charstring_index.count
                );
                return Ok(Glyph::empty());
            }
        };

        CffCharStringInterpreter::evaluate(cs)
    }

    fn font_matrix(&self) -> Matrix {
        Matrix::from_arr(self.top_dict.font_matrix)
    }
}

trait FontMetrics {
    fn get(&self, codepoint: u32) -> f32;
}

impl FontMetrics for Widths {
    fn get(&self, codepoint: u32) -> f32 {
        self.get(codepoint)
    }
}

impl FontMetrics for CidFontWidths {
    fn get(&self, codepoint: u32) -> f32 {
        self.map
            .get(&(codepoint as i32))
            .copied()
            .unwrap_or(self.default as f32)
            / 1000.0
    }
}

#[derive(Debug, Clone)]
struct MarkedContentMarker<'a> {
    tag: String,
    // todo: type
    properties: Option<Object<'a>>,
}

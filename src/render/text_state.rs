use std::rc::Rc;

use crate::{data_structures::Matrix, font::Font};

#[derive(Debug, Clone)]
pub struct TextState {
    /// The character-spacing parameter shall be a number specified in unscaled
    /// text space units (although it shall be subject to scaling by the Th
    /// parameter if the writing mode is horizontal). When the glyph for each
    /// character in the string is rendered, Tc shall be added to the horizontal
    /// or vertical component of the glyph’s displacement, depending on the writing
    /// mode. In the default coordinate system, horizontal coordinates increase
    /// from left to right and vertical coordinates from bottom to top. Therefore,
    /// for horizontal writing, a positive value of Tc has the effect of expanding
    /// the distance between glyphs, whereas for vertical writing, a negative
    /// value of Tc has this effect.
    pub character_spacing: f32,

    /// Word spacing works the same way as character spacing but shall apply only
    /// to the ASCII SPACE character (20h). The word-spacing parameter shall be
    /// added to the glyph’s horizontal or vertical displacement (depending on
    /// the writing mode). For horizontal writing, a positive value for Tw has
    /// the effect of increasing the spacing between words. For vertical writing,
    /// a positive value for Tw decreases the spacing between words (and a
    /// negative value increases it), since vertical coordinates increase from
    /// bottom to top
    ///
    /// Word spacing shall be applied to every occurrence of the single-byte
    /// character code 32 in a string when using a simple font or a composite
    /// font that defines code 32 as a single-byte code. It shall not apply to
    /// occurrences of the byte value 32 in multiple-byte codes.
    pub word_spacing: f32,

    /// The horizontal scaling parameter adjusts the width of glyphs by stretching
    /// or compressing them in the horizontal direction. Its value shall be
    /// specified as a percentage of the normal width of the glyphs, with 100
    /// being the normal width. The scaling shall apply to the horizontal
    /// coordinate in text space, independently of the writing mode. It shall
    /// affect both the glyph’s shape and its horizontal displacement (that is,
    /// its displacement vector). If the writing mode is horizontal, it shall also
    /// effect the spacing parameters Tc and Tw, as well as any positioning
    /// adjustments performed by the TJ operator.
    pub horizontal_scaling: f32,

    /// The leading parameter shall be specified in unscaled text space units.
    /// It specifies the vertical distance between the baselines of adjacent
    /// lines of text
    pub leading: f32,
    pub font: Option<Rc<Font>>,
    pub font_size: f32,
    pub rendering_mode: TextRenderingMode,

    /// Text rise, shall specify the distance, in unscaled text space units, to
    /// move the baseline up or down from its default location. Positive values
    /// of text rise shall move the baseline up. Text rise shall apply to the
    /// vertical coordinate in text space, regardless of the writing mode.
    pub rise: f32,
    pub knockout: f32,

    pub text_matrix: Matrix,
    pub text_line_matrix: Matrix,
    pub text_rendering_matrix: Matrix,
}

impl TextState {
    pub fn reinit(&mut self) {
        self.text_matrix = Matrix::identity();
        self.text_line_matrix = Matrix::identity();
        self.text_rendering_matrix = Matrix::identity();
    }
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            character_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 1.0,
            leading: 0.0,
            font: None,
            font_size: 0.0,
            rendering_mode: TextRenderingMode::Fill,
            rise: 0.0,
            knockout: 0.0,
            text_matrix: Matrix::identity(),
            text_line_matrix: Matrix::identity(),
            text_rendering_matrix: Matrix::identity(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextRenderingMode {
    Fill,
    Stroke,
    FillThenStroke,
    Invisible,
    FillAndAddToClipping,
    StrokeAndAddToClipping,
    FillThenStrokeAndAddToClipping,
    AddToClipping,
}

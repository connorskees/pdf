use crate::{
    actions::{Actions, UriAction},
    catalog::Destination,
    error::PdfResult,
    objects::Dictionary,
    pdf_enum, Resolve,
};

use super::BorderStyle;

/// A link annotation represents either a hypertext link to a destination elsewhere
/// in the document or an action to be performed
#[derive(Debug)]
pub(crate) struct LinkAnnotation {
    /// An action that shall be performed when the link annotation is activated
    a: Option<Actions>,

    /// A destination that shall be displayed when the annotation is activated
    // todo: not permitted if `a` is present
    dest: Option<Destination>,

    /// The annotation's highlighting mode, the visual effect that shall be used
    /// when the mouse button is pressed or held down inside its active area
    h: HighlightingMode,

    /// A URI action formerly associated with this annotation. When Web Capture
    /// changes an annotation from a URI to a go-to action, it uses this entry to
    /// save the data from the original URI action so that it can be changed back
    /// in case the target page for the goto action is subsequently deleted.
    pa: Option<UriAction>,

    /// An array of 8 × n numbers specifying the coordinates of n quadrilaterals in
    /// default user space that comprise the region in which the link should be
    /// activated. The coordinates for each quadrilateral are given in the order
    ///
    ///   x1 y1 x2 y2 x3 y3 x4 y4
    ///
    /// specifying the four vertices of the quadrilateral in counterclockwise order.
    /// For orientation purposes, such as when applying an underline border style, the
    /// bottom of a quadrilateral is the line formed by (x1, y1) and (x2, y2).
    ///
    /// If this entry is not present or the conforming reader does not recognize it,
    /// the region specified by the Rect entry should be used. QuadPoints shall be
    /// ignored if any coordinate in the array lies outside the region specified by Rect
    quad_points: Option<Vec<f32>>,

    ///  A border style dictionary specifying the line width and dash pattern to be used
    /// in drawing the annotation's border.
    ///
    /// The annotation dictionary's AP entry, if present, takes precedence over the BS entry
    bs: Option<BorderStyle>,
}

pdf_enum!(
    #[derive(Debug)]
    pub enum HighlightingMode {
        /// No highlighting
        None = "N",

        /// Invert the contents of the annotation rectangle
        Invert = "I",

        /// Invert the annotation's border
        Outline = "O",

        /// Display the annotation as if it were being pushed below the surface of the page.
        Push = "P",
    }
);

impl Default for HighlightingMode {
    fn default() -> Self {
        Self::Invert
    }
}

impl LinkAnnotation {
    const TYPE: &'static str = "Link";

    pub fn from_dict(dict: &mut Dictionary, resolver: &mut impl Resolve) -> PdfResult<Self> {
        let a = dict
            .get_dict("A", resolver)?
            .map(|actions| Actions::from_dict(actions, resolver))
            .transpose()?;

        let dest = dict
            .get_arr("Dest", resolver)?
            .map(|dest| Destination::from_arr(dest, resolver))
            .transpose()?;

        let h = dict
            .get_name("H", resolver)?
            .as_deref()
            .map(HighlightingMode::from_str)
            .transpose()?
            .unwrap_or_default();

        let pa = dict
            .get_dict("PA", resolver)?
            .map(|dict| UriAction::from_dict(dict, resolver))
            .transpose()?;
        let quad_points = None;
        let bs = None;

        Ok(Self {
            a,
            dest,
            h,
            pa,
            quad_points,
            bs,
        })
    }
}

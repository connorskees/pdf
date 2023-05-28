use crate::{
    data_structures::Rectangle,
    date::Date,
    error::{ParseError, PdfResult},
    objects::{Dictionary, Object, Reference},
    optional_content::OptionalContent,
    resources::graphics_state_parameters::LineDashPattern,
    FromObj, Resolve,
};

use subtype::{AnnotationSubType, AnnotationSubTypeKind};

mod link;
mod state;
mod subtype;
mod text;

#[derive(Debug)]
pub struct Annotation<'a> {
    base: BaseAnnotation,
    sub_type: AnnotationSubType<'a>,
}

impl<'a> FromObj<'a> for Annotation<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let mut dict = resolver.assert_dict(obj)?;

        let base = BaseAnnotation::from_dict(&mut dict, resolver)?;
        let sub_type = AnnotationSubType::from_dict(dict, &base, resolver)?;

        Ok(Self { base, sub_type })
    }
}

#[derive(Debug)]
pub(crate) struct BaseAnnotation {
    subtype: AnnotationSubTypeKind,

    /// The annotation rectangle, defining the location of the
    /// annotation on the page in default user space units.
    rect: Rectangle,

    /// Text that shall be displayed for the annotation or, if this type
    /// of annotation does not display text, an alternate description of the
    /// annotation's contents in human-readable form. In either case, this
    /// text is useful when extracting the document's contents in support
    /// of accessibility to users with disabilities or for other purposes
    contents: Option<String>,

    /// An indirect reference to the page object with which this annotation
    /// is associated.
    ///
    /// This entry shall be present in screen annotations associated with
    /// rendition actions
    p: Option<Reference>,

    /// The annotation name, a text string uniquely identifying it among all
    /// the annotations on its page.
    name: Option<String>,

    /// The date and time when the annotation was most recently modified.
    /// The format should be a date string, but conforming readers shall accept
    /// and display a string in any format.
    last_modified: Option<String>,

    /// A set of flags specifying various characteristics of the annotation
    ///
    /// Default value: 0
    flags: AnnotationFlags,

    /// An appearance dictionary specifying how the annotation shall be presented
    /// visually on the page. Individual annotation handlers may ignore this entry
    /// and provide their own appearances.
    ap: Option<Appearance>,

    /// The annotation's appearance state, which selects the applicable appearance
    /// stream from an appearance subdictionary
    appearance_stream_name: Option<String>,

    /// An array specifying the characteristics of the annotation's border, which
    /// shall be drawn as a rounded rectangle.
    ///
    /// (PDF 1.0) The array consists of three numbers defining the horizontal corner
    /// radius, vertical corner radius, and border width, all in default user space
    /// units. If the corner radii are 0, the border has square (not rounded) corners;
    /// if the border width is 0, no border is drawn.
    ///
    /// (PDF 1.1) The array may have a fourth element, an optional dash array defining
    /// a pattern of dashes and gaps that shall be used in drawing the border. The dash
    /// array shall be specified in the same format as in the line dash pattern parameter
    /// of the graphics state.
    ///
    /// EXAMPLE
    ///    A Border value of [0 0 1 [3 2]] specifies a border 1 unit wide, with square
    ///    corners, drawn with 3-unit dashes alternating with 2-unit gaps.
    ///
    /// (PDF 1.2) The dictionaries for some annotation types (such as free text and
    /// polygon annotations) can include the BS entry. That entry specifies a border
    /// style dictionary that has more settings than the array specified for the Border
    /// entry. If an annotation dictionary includes the BS entry, then the Border entry
    /// is ignored.
    ///
    /// Default value: [0 0 1]
    border: Border,

    /// An array of numbers in the range 0.0 to 1.0, representing a colour used for the
    /// following purposes:
    ///  - The background of the annotation's icon when closed
    ///  - The title bar of the annotation's pop-up window
    ///  - The border of a link annotation
    ///
    /// The number of array elements determines the colour space in which the colour shall be defined:
    ///  - 0 No colour; transparent
    ///  - 1 DeviceGray
    ///  - 3 DeviceRGB
    ///  - 4 DeviceCMYK
    c: Option<Vec<f32>>,

    /// The integer key of the annotation's entry in the structural parent tree
    struct_parent: Option<i32>,

    /// An optional content group or optional content membership dictionary specifying the optional
    /// content properties for the annotation. Before the annotation is drawn, its visibility
    /// shall be determined based on this entry as well as the annotation flags specified in the
    /// F entry. If it is determined to be invisible, the annotation shall be skipped, as if it were
    /// not in the document.
    oc: Option<OptionalContent>,

    markup_dict: Option<MarkupAnnotation>,
}

#[derive(Debug)]
struct MarkupAnnotation {
    /// The text label that shall be displayed in the title bar of the annotation's pop-up window
    /// when open and active. This entry shall identify the user who added the annotation.
    t: Option<String>,

    /// An indirect reference to a pop-up annotation for entering or editing the text associated with
    /// this annotation.
    popup: Option<Reference>,

    /// The constant opacity value that shall be used in painting the annotation.
    ///
    /// This value shall apply to all visible elements of the annotation in its closed state (including
    /// its background and border) but not to the pop-up window that appears when the annotation is opened.
    ///
    /// The specified value shall not used if the annotation has an appearance stream; in that case, the
    /// appearance stream shall specify any transparency. (However, if the compliant viewer regenerates
    /// the annotation's appearance stream, it may incorporate the CA value into the stream's
    /// content.) The implicit blend mode is Normal.
    ///
    /// Default value: 1.0.
    ///
    /// If no explicit appearance stream is defined for the annotation, it may be painted by
    /// implementation-dependent means that do not necessarily conform to the PDF imaging model;
    /// in this case, the effect of this entry is implementation-dependent as well.
    ca: f32,

    /// A rich text string that shall be displayed in the pop-up window when the annotation is opened.
    rc: Option<RichTextString>,

    /// The date and time when the annotation was created
    creation_date: Option<Date>,

    /// A reference to the annotation that this annotation is "in reply to."
    ///
    /// Both annotations shall be on the same page of the document. The relationship between the two
    /// annotations shall be specified by the RT entry. If this entry is present in an FDF file, its
    /// type shall not be a dictionary but a text string containing the contents of the NM entry of the
    /// annotation being replied to, to allow for a situation where the annotation being replied to
    /// is not in the same FDF file.
    irt: Option<Reference>,

    /// Text representing a short description of the subject being addressed by the annotation.
    subj: Option<String>,

    /// A name specifying the relationship (the "reply type") between this annotation and one specified by IRT.
    ///
    /// Valid values are:
    ///   * `R` The annotation shall be considered a reply to the annotation specified by IRT.
    ///     Conforming readers shall not display replies to an annotation individually but together
    ///     in the form of threaded comments.
    ///   * `Group` The annotation shall be grouped with the annotation specified by IRT
    ///
    /// Default value: R.
    rt: Option<ReplyType>,

    /// A name describing the intent of the markup annotation. Intents allow conforming readers to distinguish
    /// between different uses and behaviors of a single markup annotation type. If this entry is not present
    /// or its value is the same as the annotation type, the annotation shall have no explicit intent and should
    /// behave in a generic manner in a conforming reader.
    ///
    /// Free text annotations, line annotations, polygon annotations, and polyline annotations have defined
    /// intents, whose values are enumerated in the corresponding tables
    // todo: should this be an enum
    it: Option<String>,

    /// An external data dictionary specifying data that shall be associated with the annotation
    ex_data: Option<ExternalDataDictionary>,
}

// todo: this seems to only be used for 3d stuff
#[derive(Debug, FromObj)]
#[obj_type("ExData")]
struct ExternalDataDictionary {}

#[pdf_enum]
enum ReplyType {
    R = "R",
    Group = "Group",
}

impl MarkupAnnotation {
    pub fn from_dict<'a>(
        dict: &mut Dictionary<'a>,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        let t = dict.get_string("T", resolver)?;
        let popup = dict.get_reference("Popup")?;
        let ca = dict.get_number("CA", resolver)?.unwrap_or(1.0);
        let rc = None;
        let creation_date = dict.get::<Date>("CreationDate", resolver)?;
        let irt = dict.get_reference("IRT")?;
        let subj = dict.get_string("Subj", resolver)?;
        let rt = dict
            .get_name("RT", resolver)?
            .as_deref()
            .map(ReplyType::from_str)
            .transpose()?;
        let it = dict.get_name("IT", resolver)?;
        let ex_data = dict.get("ExData", resolver)?;

        Ok(Self {
            t,
            popup,
            ca,
            rc,
            creation_date,
            irt,
            subj,
            rt,
            it,
            ex_data,
        })
    }
}

impl BaseAnnotation {
    const TYPE: &'static str = "Annot";

    pub fn from_dict<'a>(
        dict: &mut Dictionary<'a>,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        dict.expect_type(Self::TYPE, resolver, false)?;

        let subtype = AnnotationSubTypeKind::from_str(&dict.expect_name("Subtype", resolver)?)?;

        let rect = dict.expect::<Rectangle>("Rect", resolver)?;
        let contents = dict.get_string("Contents", resolver)?;
        let p = dict.get_reference("P")?;
        let name = dict.get_string("NM", resolver)?;
        let last_modified = dict.get_string("M", resolver)?;
        let flags = dict
            .get_integer("F", resolver)?
            .map(AnnotationFlags::from_integer)
            .unwrap_or_default();
        let ap = None;
        let appearance_stream_name = dict.get_name("AS", resolver)?;
        let border = dict
            .get_arr("Border", resolver)?
            .map(|border| Border::from_arr(border, resolver))
            .transpose()?
            .unwrap_or_default();

        let c = dict
            .get_arr("C", resolver)?
            .map(|colors| {
                colors
                    .into_iter()
                    .map(|color| resolver.assert_number(color))
                    .collect::<PdfResult<Vec<f32>>>()
            })
            .transpose()?;

        let struct_parent = dict.get_integer("StructParent", resolver)?;
        let oc = None;
        let markup_dict = if subtype.is_markup() {
            Some(MarkupAnnotation::from_dict(dict, resolver)?)
        } else {
            None
        };

        Ok(Self {
            subtype,
            rect,
            contents,
            p,
            name,
            last_modified,
            flags,
            ap,
            appearance_stream_name,
            border,
            c,
            struct_parent,
            oc,
            markup_dict,
        })
    }
}

#[derive(Debug)]
pub struct AnnotationFlags(u16);

impl AnnotationFlags {
    const INVISIBLE: u16 = 1 << 0;
    const HIDDEN: u16 = 1 << 1;
    const PRINT: u16 = 1 << 2;
    const NO_ZOOM: u16 = 1 << 3;
    const NO_ROTATE: u16 = 1 << 4;
    const NO_VIEW: u16 = 1 << 5;
    const READONLY: u16 = 1 << 6;
    const LOCKED: u16 = 1 << 7;
    const TOGGLE_NO_VIEW: u16 = 1 << 8;
    const LOCKED_CONTENTS: u16 = 1 << 9;

    pub fn from_integer(i: i32) -> Self {
        Self(i as u16)
    }

    /// If set, do not display the annotation if it does not belong to one of the
    /// standard annotation types and no annotation handler is available.
    ///
    /// If clear, display such an unknown annotation using an appearance stream specified
    /// by its appearance dictionary, if any
    pub fn is_invisible(&self) -> bool {
        self.0 & Self::INVISIBLE != 0
    }

    /// If set, do not display or print the annotation or allow it to interact with
    /// the user, regardless of its annotation type or whether an annotation handler
    /// is available.
    ///
    /// In cases where screen space is limited, the ability to hide and show annotations
    /// selectively can be used in combination with appearance streams to display
    /// auxiliary pop-up information similar in function to online help systems.
    pub fn is_hidden(&self) -> bool {
        self.0 & Self::HIDDEN != 0
    }

    /// If set, print the annotation when the page is printed. If clear, never print
    /// the annotation, regardless of whether it is displayed on the screen.
    ///
    /// This can be useful for annotations representing interactive pushbuttons, which
    /// would serve no meaningful purpose on the printed page.
    pub fn is_print(&self) -> bool {
        self.0 & Self::PRINT != 0
    }

    /// If set, do not scale the annotation's appearance to match the magnification of
    /// the page.
    ///
    /// The location of the annotation on the page (defined by the upper-left corner of
    /// its annotation rectangle) shall remain fixed, regardless of the page magnification.
    pub fn is_no_zoom(&self) -> bool {
        self.0 & Self::NO_ZOOM != 0
    }

    /// If set, do not rotate the annotation's appearance to match the rotation of the
    /// page.
    ///
    /// The upper-left corner of the annotation rectangle shall remain in a fixed location
    /// on the page, regardless of the page rotation.
    pub fn is_no_rotate(&self) -> bool {
        self.0 & Self::NO_ROTATE != 0
    }

    /// If set, do not display the annotation on the screen or allow it to interact with
    /// the user.
    ///
    /// The annotation may be printed (depending on the setting of the Print flag) but
    /// should be considered hidden for purposes of on-screen display and user interaction.
    pub fn is_no_view(&self) -> bool {
        self.0 & Self::NO_VIEW != 0
    }

    /// If set, do not allow the annotation to interact with the user.
    ///
    /// The annotation may be displayed or printed (depending on the settings of the NoView
    /// and Print flags) but should not respond to mouse clicks or change its appearance in
    /// response to mouse motions. This flag shall be ignored for widget annotations; its
    /// function is subsumed by the ReadOnly flag of the associated form field.
    pub fn is_readonly(&self) -> bool {
        self.0 & Self::READONLY != 0
    }

    /// If set, do not allow the annotation to be deleted or its properties (including
    /// position and size) to be modified by the user. However, this flag does not restrict
    /// changes to the annotation's contents, such as the value of a form field.
    pub fn is_locked(&self) -> bool {
        self.0 & Self::LOCKED != 0
    }

    /// If set, invert the interpretation of the NoView flag for certain events.
    ///
    /// A typical use is to have an annotation that appears only when a mouse cursor is
    /// held over it.
    pub fn is_toggle_no_view(&self) -> bool {
        self.0 & Self::TOGGLE_NO_VIEW != 0
    }

    /// If set, do not allow the contents of the annotation to be modified by the user.
    ///
    /// This flag does not restrict deletion of the annotation or changes to other annotation
    /// properties, such as position and size.
    pub fn is_locked_contents(&self) -> bool {
        self.0 & Self::LOCKED_CONTENTS != 0
    }
}

impl Default for AnnotationFlags {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Debug)]
struct Appearance;
#[derive(Debug)]
struct RichTextString;

/// An annotation may optionally be surrounded by a border when displayed or printed. If present, the border
/// shall be drawn completely inside the annotation rectangle. In PDF 1.1, the characteristics of the border
/// shall be specified by the Border entry in the annotation dictionary. Beginning with PDF 1.2, the border
/// characteristics for some types of annotations may instead be specified in a border style dictionary designated
/// by the annotation's BS entry. Such dictionaries may also be used to specify the width and dash pattern
/// for the lines drawn by line, square, circle, and ink annotations. If neither the Border nor the BS entry
/// is present, the border shall be drawn as a solid line with a width of 1 point
#[derive(Debug, FromObj)]
#[obj_type("Border")]
pub struct BorderStyle {
    /// The border width in points. If this value is 0, no border shall drawn.
    ///
    /// Default value: 1
    #[field("W")]
    w: u32,

    /// The border style
    #[field("S")]
    s: Option<BorderStyleKind>,

    /// A dash array defining a pattern of dashes and gaps that shall be used in drawing a dashed border (border
    /// style D in the S entry). The dash array shall be specified in the same format as in the line dash pattern
    /// parameter of the graphics state. The dash phase is not specified and shall be assumed to be 0.
    ///
    /// EXAMPLE: A `D` entry of [3 2] specifies a border drawn with 3-point dashes alternating with 2-point gaps.
    ///
    /// Default value: [3].
    #[field("D")]
    d: Option<LineDashPattern>,
}

#[derive(Debug)]
enum BorderStyleKind {
    /// A solid rectangle surrounding the annotation
    Solid,

    /// A dashed rectangle surrounding the annotation
    ///
    /// The dash pattern may be specified by the D entry
    Dashed,

    /// A simulated embossed rectangle that appears to be raised above the surface of the page
    Beveled,

    /// A simulated engraved rectangle that appears to be recessedcbelow the surface of the page
    Inset,

    /// A single line along the bottom of the annotation rectangle
    Underline,

    Other(String),
}

impl<'a> FromObj<'a> for BorderStyleKind {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let s = resolver.assert_name(obj)?;
        Ok(BorderStyleKind::from_str(s))
    }
}

impl BorderStyleKind {
    pub fn from_str(s: String) -> Self {
        match s.as_ref() {
            "S" => Self::Solid,
            "D" => Self::Dashed,
            "B" => Self::Beveled,
            "I" => Self::Inset,
            "U" => Self::Underline,
            _ => Self::Other(s),
        }
    }
}

#[derive(Debug)]
struct Border {
    horizontal_corner_radius: u32,
    vertical_corner_radius: u32,
    border_width: u32,
    dash_array: Option<LineDashPattern>,
}

impl Border {
    pub fn from_arr<'a>(
        mut arr: Vec<Object<'a>>,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        if arr.len() < 3 {
            anyhow::bail!(ParseError::ArrayOfInvalidLength {
                expected: 3,
                // found: arr,
            });
        }

        let dash_array = if arr.len() == 4 {
            let arr = resolver.assert_arr(arr.pop().unwrap())?;
            Some(LineDashPattern::from_arr(arr, resolver)?)
        } else {
            None
        };

        let border_width = resolver.assert_unsigned_integer(arr.pop().unwrap())?;
        let vertical_corner_radius = resolver.assert_unsigned_integer(arr.pop().unwrap())?;
        let horizontal_corner_radius = resolver.assert_unsigned_integer(arr.pop().unwrap())?;

        Ok(Self {
            horizontal_corner_radius,
            vertical_corner_radius,
            border_width,
            dash_array,
        })
    }
}

impl Default for Border {
    fn default() -> Self {
        Self {
            horizontal_corner_radius: 0,
            vertical_corner_radius: 0,
            border_width: 1,
            dash_array: None,
        }
    }
}

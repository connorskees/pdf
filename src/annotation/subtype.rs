use crate::{
    error::PdfResult,
    objects::{Dictionary, Object},
    FromObj, Resolve,
};

use super::{link::LinkAnnotation, text::TextAnnotation, BaseAnnotation};

#[derive(Debug)]
pub(crate) enum AnnotationSubType<'a> {
    Text(TextAnnotation),
    Link(LinkAnnotation<'a>),
}

impl<'a> AnnotationSubType<'a> {
    pub(crate) fn from_dict(
        mut dict: Dictionary<'a>,
        base: &BaseAnnotation,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        Ok(match base.subtype {
            AnnotationSubTypeKind::Text => {
                AnnotationSubType::Text(TextAnnotation::from_dict(&mut dict, resolver)?)
            }
            AnnotationSubTypeKind::Link => AnnotationSubType::Link(LinkAnnotation::from_obj(
                Object::Dictionary(dict),
                resolver,
            )?),
            _ => todo!(),
        })
    }
}

#[pdf_enum]
pub(super) enum AnnotationSubTypeKind {
    Text = "Text",
    Link = "Link",
    FreeText = "FreeText",
    Line = "Line",
    Square = "Square",
    Circle = "Circle",
    Polygon = "Polygon",
    PolyLine = "PolyLine",
    Highlight = "Highlight",
    Underline = "Underline",
    Squiggly = "Squiggly",
    StrikeOut = "StrikeOut",
    Stamp = "Stamp",
    Caret = "Caret",
    Ink = "Ink",
    Popup = "Popup",
    FileAttachment = "FileAttachment",
    Sound = "Sound",
    Movie = "Movie",
    Widget = "Widget",
    Screen = "Screen",
    PrinterMark = "PrinterMark",
    TrapNet = "TrapNet",
    Watermark = "Watermark",
    ThreeD = "3D",
    Redact = "Redact",
}

impl AnnotationSubTypeKind {
    pub fn is_markup(&self) -> bool {
        matches!(
            self,
            Self::Text
                | Self::FreeText
                | Self::Line
                | Self::Square
                | Self::Circle
                | Self::Polygon
                | Self::PolyLine
                | Self::Highlight
                | Self::Underline
                | Self::Squiggly
                | Self::StrikeOut
                | Self::Stamp
                | Self::Caret
                | Self::Ink
                | Self::FileAttachment
                | Self::Sound
                | Self::Redact
        )
    }
}

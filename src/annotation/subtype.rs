use crate::{error::PdfResult, objects::Dictionary, pdf_enum, Resolve};

use super::{link::LinkAnnotation, text::TextAnnotation, BaseAnnotation};

#[derive(Debug)]
pub(crate) enum AnnotationSubType {
    Text(TextAnnotation),
    Link(LinkAnnotation),
}

impl AnnotationSubType {
    pub(crate) fn from_dict(
        dict: &mut Dictionary,
        base: &BaseAnnotation,
        resolver: &mut impl Resolve,
    ) -> PdfResult<Self> {
        Ok(match base.subtype {
            AnnotationSubTypeKind::Text => {
                AnnotationSubType::Text(TextAnnotation::from_dict(dict, resolver)?)
            }
            AnnotationSubTypeKind::Link => {
                AnnotationSubType::Link(LinkAnnotation::from_dict(dict, resolver)?)
            }
            _ => todo!(),
        })
    }
}

pdf_enum!(
    #[derive(Debug)]
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
);

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

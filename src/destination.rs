use crate::{
    assert_reference,
    catalog::assert_len,
    error::{ParseError, PdfResult},
    objects::{Object, ObjectType, Reference},
    FromObj, Resolve,
};

/// A destination defines a particular view of a document, consisting of the following items:
///   * The page of the document that shall be displayed
///   * The location of the document window on that page
///   * The magnification (zoom) factor
#[derive(Debug, Clone)]
pub enum Destination {
    Explicit(ExplicitDestination),

    /// Instead of being defined directly with the explicit syntax, a destination may
    /// be referred to indirectly by means of a name object or a byte string. This
    /// capability is especially useful when the destination is located in another PDF
    /// document
    Named(String),
}

impl<'a> FromObj<'a> for Destination {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let obj = resolver.resolve(obj)?;

        match obj {
            Object::Array(arr) => Ok(Destination::Explicit(ExplicitDestination::from_arr(
                arr, resolver,
            )?)),
            Object::String(s) | Object::Name(s) => Ok(Destination::Named(s)),
            _ => anyhow::bail!(ParseError::MismatchedObjectTypeAny {
                expected: &[ObjectType::Array, ObjectType::String, ObjectType::Name],
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExplicitDestination {
    kind: DestinationKind,
    page_ref: Reference,
}

impl ExplicitDestination {
    pub fn from_arr(mut arr: Vec<Object>, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        if arr.len() < 2 {
            anyhow::bail!(ParseError::ArrayOfInvalidLength { expected: 2 });
        }

        let vals = arr.split_off(2);

        let dimensions = vals
            .iter()
            .cloned()
            .map(|obj| resolver.assert_number_or_null(obj))
            .collect::<PdfResult<Vec<Option<f32>>>>()?;

        let kind_str = resolver.assert_name(arr.pop().unwrap())?;

        let page_ref = assert_reference(arr.pop().unwrap())?;

        let kind = match kind_str.as_str() {
            "XYZ" => {
                assert_len(&vals, 3)?;
                DestinationKind::Xyz {
                    left: dimensions[0],
                    top: dimensions[1],
                    zoom: dimensions[2],
                }
            }
            "Fit" => {
                assert_len(&vals, 0)?;
                DestinationKind::Fit
            }
            "FitH" => {
                assert_len(&vals, 1)?;
                DestinationKind::FitH { top: dimensions[0] }
            }
            "FitV" => {
                assert_len(&vals, 1)?;
                DestinationKind::FitV {
                    left: dimensions[0],
                }
            }
            "FitR" => {
                assert_len(&vals, 4)?;
                DestinationKind::FitR {
                    left: dimensions[0],
                    bottom: dimensions[1],
                    right: dimensions[2],
                    top: dimensions[3],
                }
            }
            "FitB" => {
                assert_len(&vals, 0)?;
                DestinationKind::FitB
            }
            "FitBH" => {
                assert_len(&vals, 1)?;
                DestinationKind::FitBh { top: dimensions[0] }
            }
            "FitBV" => {
                assert_len(&vals, 1)?;
                DestinationKind::FitBv {
                    left: dimensions[0],
                }
            }
            found => {
                anyhow::bail!(ParseError::UnrecognizedVariant {
                    found: found.to_owned(),
                    ty: "DestinationKey",
                })
            }
        };

        Ok(ExplicitDestination { kind, page_ref })
    }
}

#[derive(Debug, Clone, Copy)]
enum DestinationKind {
    /// Display the page designated by page, with the coordinates (left, top) positioned
    /// at the upper-left corner of the window and the contents of the page magnified by
    /// the factor zoom. A null value for any of the parameters left, top, or zoom specifies
    /// that the current value of that parameter shall be retained unchanged. A zoom value
    /// of 0 has the same meaning as a null value.
    Xyz {
        left: Option<f32>,
        top: Option<f32>,
        zoom: Option<f32>,
    },

    /// Display the page designated by page, with its contents magnified just enough to fit
    /// the entire page within the window both horizontally and vertically. If the required
    /// horizontal and vertical magnification factors are different, use the smaller of the
    /// two, centering the page within the window in the other dimension.
    Fit,

    /// Display the page designated by page, with the vertical coordinate top positioned at
    /// the top edge of the window and the contents of the page magnified just enough to fit
    /// the entire width of the page within the window. A null value for top specifies that
    /// the current value of that parameter shall be retained unchanged.
    FitH { top: Option<f32> },

    /// Display the page designated by page, with the horizontal coordinate left positioned
    /// at the left edge of the window and the contents of the page magnified just enough to
    /// fit the entire height of the page within the window. A null value for left specifies
    /// that the current value of that parameter shall be retained unchanged.
    FitV { left: Option<f32> },

    /// Display the page designated by page, with its contents magnified just enough to fit
    /// the rectangle specified by the coordinates left, bottom, right, and top entirely within
    /// the window both horizontally and vertically. If the required horizontal and vertical
    /// magnification factors are different, use the smaller of the two, centering the rectangle
    /// within the window in the other dimension.
    FitR {
        left: Option<f32>,
        bottom: Option<f32>,
        right: Option<f32>,
        top: Option<f32>,
    },

    /// Display the page designated by page, with its contents magnified just enough to fit its
    /// bounding box entirely within the window both horizontally and vertically. If the required
    /// horizontal and vertical magnification factors are different, use the smaller of the two,
    /// centering the bounding box within the window in the other dimension.
    FitB,

    /// Display the page designated by page, with the vertical coordinate top positioned at the
    /// top edge of the window and the contents of the page magnified just enough to fit the entire
    /// width of its bounding box within the window. A null value for top specifies that the
    /// current value of that parameter shall be retained unchanged.
    FitBh { top: Option<f32> },

    /// Display the page designated by page, with the horizontal coordinate left positioned at the
    /// left edge of the window and the contents of the page magnified just enough to fit the entire
    /// height of its bounding box within the window. A null value for left specifies that the
    /// current value of that parameter shall be retained unchanged.
    FitBv { left: Option<f32> },
}

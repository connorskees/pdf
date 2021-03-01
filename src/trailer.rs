use std::convert::TryFrom;

use crate::{
    assert_empty,
    catalog::Encryption,
    error::PdfResult,
    file_specification::FileIdentifier,
    objects::{Dictionary, Reference},
    Resolve,
};

#[derive(Debug)]
pub struct Trailer {
    /// The total number of entries in the
    /// file's cross-reference table, as
    /// defined by the combination of the
    /// original section and all update sections.
    ///
    /// Equivalently, this value shall be 1 greater
    /// than the highest object number defined in the
    /// file. Any object in a cross-reference section
    /// whose number is greater than this value shall
    /// be ignored and defined to be missing by a
    /// conforming reader.
    pub size: usize,

    /// The byte offset in the decoded stream from the
    /// beginning of the file to the beginning of the
    /// previous cross-reference section
    ///
    /// Present only if the file has more than one
    /// cross-reference section.
    pub prev: Option<usize>,

    pub root: Reference,

    pub encryption: Option<Encryption>,
    pub id: Option<FileIdentifier>,
    pub info: Option<Reference>,

    /// LibreOffice specific extension, see <https://bugs.documentfoundation.org/show_bug.cgi?id=66580>
    pub(crate) doc_checksum: Option<String>,
}

impl Trailer {
    pub(crate) fn from_dict(
        mut dict: Dictionary,
        is_previous: bool,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Self> {
        let trailer = Trailer::from_dict_ref(&mut dict, is_previous, resolver)?;

        assert_empty(dict);

        Ok(trailer)
    }

    pub(crate) fn from_dict_ref(
        dict: &mut Dictionary,
        is_previous: bool,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Self> {
        let size = usize::try_from(dict.expect_integer("Size", resolver)?)?;
        let prev = dict
            .get_integer("Prev", resolver)?
            .map(usize::try_from)
            .transpose()?;
        let root = if is_previous {
            dict.get_reference("Root")?.unwrap_or(Reference {
                object_number: 0,
                generation: 0,
            })
        } else {
            dict.expect_reference("Root")?
        };
        // TODO: encryption dicts
        let encryption = None;
        let id = dict
            .get_arr("ID", resolver)?
            .map(|objs| FileIdentifier::from_arr(objs, resolver))
            .transpose()?;
        let info = dict.get_reference("Info")?;
        let doc_checksum = dict.get_name("DocChecksum", resolver)?;

        Ok(Trailer {
            size,
            prev,
            root,
            encryption,
            info,
            id,
            doc_checksum,
        })
    }
}

use crate::{
    assert_empty,
    catalog::InformationDictionary,
    encryption::Encryption,
    error::PdfResult,
    file_specification::FileIdentifier,
    objects::{Dictionary, Reference, TypedReference},
    Resolve,
};

#[derive(Debug)]
pub struct Trailer<'a> {
    /// The total number of entries in the file's cross-reference table, as
    /// defined by the combination of the original section and all update
    /// sections.
    ///
    /// Equivalently, this value shall be 1 greater than the highest object number
    /// defined in the file. Any object in a cross-reference section whose
    /// number is greater than this value shall be ignored and defined to
    /// be missing by a conforming reader.
    pub size: usize,

    /// The byte offset in the decoded stream from the beginning of the file to
    /// the beginning of the previous cross-reference section
    ///
    /// Present only if the file has more than one cross-reference section.
    pub prev: Option<usize>,

    /// The catalog dictionary for the PDF document contained in the file
    ///
    /// Shall be indirect reference
    pub root: Reference,

    /// The document’s encryption dictionary
    pub encryption: Option<TypedReference<'a, Encryption<'a>>>,

    /// An array of two byte-strings constituting a file identifier for the
    /// file.
    ///
    /// If there is an Encrypt entry this array and the two byte-strings shall
    /// be direct objects and shall be unencrypted.
    pub id: Option<FileIdentifier>,

    /// The document’s information dictionary
    pub info: Option<TypedReference<'a, InformationDictionary<'a>>>,

    /// The byte offset in the decoded stream from the beginning of the file of a
    /// cross-reference stream
    pub xref_stream: Option<i32>,

    /// LibreOffice specific extension, see <https://bugs.documentfoundation.org/show_bug.cgi?id=66580>
    pub(crate) doc_checksum: Option<String>,
}

impl<'a> Trailer<'a> {
    pub(crate) fn from_dict(
        mut dict: Dictionary<'a>,
        is_previous: bool,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        let trailer = Trailer::from_dict_ref(&mut dict, is_previous, resolver)?;

        assert_empty(dict);

        Ok(trailer)
    }

    pub(crate) fn from_dict_ref(
        dict: &mut Dictionary<'a>,
        is_previous: bool,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        let size = dict.expect("Size", resolver)?;
        let prev = dict.get("Prev", resolver)?;
        let root = if is_previous {
            dict.get_reference("Root")?.unwrap_or(Reference {
                object_number: 0,
                generation: 0,
            })
        } else {
            dict.expect_reference("Root")?
        };
        let encryption = dict.get("Encrypt", resolver)?;
        let id = dict.get("ID", resolver)?;
        let info = dict.get("Info", resolver)?;
        let doc_checksum = dict.get_name("DocChecksum", resolver)?;
        let xref_stream = dict.get_integer("XRefStm", resolver)?;

        Ok(Trailer {
            size,
            prev,
            root,
            encryption,
            info,
            id,
            xref_stream,
            doc_checksum,
        })
    }
}

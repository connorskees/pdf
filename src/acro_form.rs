use crate::{
    error::PdfResult,
    objects::{Dictionary, Reference, Object},
    FromObj, Resolve,
};

#[derive(Debug, FromObj)]
pub struct AcroForm<'a> {
    /// An array of references to the document’s root fields (those with no
    /// ancestors in the field hierarchy).
    #[field("Fields")]
    fields: Vec<Reference>,

    /// A flag specifying whether to construct appearance streams and appearance
    /// dictionaries for all widget annotations in the document
    ///
    /// Default value: false
    #[field("NeedAppearances", default = false)]
    need_appearances: bool,

    /// A set of flags specifying various document-level characteristics related
    /// to signature fields
    ///
    /// Default value: 0
    #[field("SigFlags", default = SigFlags(0))]
    sig_flags: SigFlags,

    /// An array of indirect references to field dictionaries with calculation
    /// actions, defining the calculation order in which their values will be
    /// recalculated when the value of any field changes
    #[field("CO")]
    co: Option<Vec<Reference>>,

    /// A resource dictionary containing default resources (such as fonts, patterns,
    /// or colour spaces) that shall be used by form field appearance streams.
    /// At a minimum, this dictionary shall contain a Font entry specifying the
    /// resource name and font dictionary of the default font for displaying text.
    #[field("DR")]
    dr: Option<Dictionary<'a>>,

    /// A document-wide default value for the DA attribute of variable text fields
    #[field("DA")]
    da: Option<String>,

    /// A document-wide default value for the Q attribute of variable text fields
    #[field("Q")]
    q: Option<String>,

    /// A stream or array containing an XFA resource, whose format shall be
    /// described by the Data Package (XDP) Specification.
    ///
    /// The value of this entry shall be either a stream representing the entire
    /// contents of the XML Data Package or an array of text string and stream
    /// pairs representing the individual packets comprising the XML Data Package.
    // todo: struct for field
    #[field("XFA")]
    xfa: Option<String>,
}

#[derive(Debug, Clone)]
struct SigFlags(u32);

impl SigFlags {
    /// If set, the document contains at least one signature field. This flag
    /// allows a conforming reader to enable user interface items (such as menu
    /// items or pushbuttons) related to signature processing without having to
    /// scan the entire document for the presence of signature fields.
    pub const SIGNATURES_EXIST: u8 = 0b01;

    /// If set, the document contains signatures that may be invalidated if the
    /// file is saved (written) in a way that alters its previous contents, as
    /// opposed to an incremental update. Merely updating the file by appending
    /// new information to the end of the previous version is safe. Conforming
    /// readers may use this flag to inform a user requesting a full save that
    /// signatures will be invalidated and require explicit confirmation before
    /// continuing with the operation.
    pub const APPEND_ONLY: u8 = 0b10;
}

impl<'a> FromObj<'a> for SigFlags {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(Self(u32::from_obj(obj, resolver)?))
    }
}

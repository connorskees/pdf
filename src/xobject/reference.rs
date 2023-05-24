use crate::{
    error::PdfResult,
    file_specification::{FileIdentifier, FileSpecification},
    objects::Object,
    FromObj, Resolve,
};

#[derive(Debug, Clone, FromObj)]
pub struct ReferenceXObject<'a> {
    /// The file containing the target document
    #[field("F")]
    f: FileSpecification<'a>,

    /// A page index or page label identifying the page of the target
    /// document containing the content to be imported. This reference
    /// is a weak one and may be inadvertently invalidated if the referenced
    /// page is changed or replaced in the target document after the
    /// reference is created
    #[field("Page")]
    page: PageIdentifier,

    /// An array of two byte strings constituting a file identifier for
    /// the file containing the target document. The use of this entry
    /// improves an reader's chances of finding the intended file and
    /// allows it to warn the user if the file has changed since the
    /// reference was created
    #[field("ID")]
    id: Option<FileIdentifier>,
}

#[derive(Debug, Clone)]
enum PageIdentifier {
    Index(usize),
    Label(String),
}

impl<'a> FromObj<'a> for PageIdentifier {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(if let Ok(name) = resolver.assert_string(obj.clone()) {
            PageIdentifier::Label(name)
        } else {
            let idx = resolver.assert_unsigned_integer(obj)?;

            PageIdentifier::Index(idx as usize)
        })
    }
}

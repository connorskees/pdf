use crate::{
    error::PdfResult,
    file_specification::{FileIdentifier, FileSpecification},
    objects::{Dictionary, Object},
    Resolve,
};

#[derive(Debug)]
pub struct ReferenceXObject {
    /// The file containing the target document
    f: FileSpecification,

    /// A page index or page label identifying the page of the target
    /// document containing the content to be imported. This reference
    /// is a weak one and may be inadvertently invalidated if the referenced
    /// page is changed or replaced in the target document after the
    /// reference is created
    page: PageIdentifier,

    /// An array of two byte strings constituting a file identifier for
    /// the file containing the target document. The use of this entry
    /// improves an reader’s chances of finding the intended file and
    /// allows it to warn the user if the file has changed since the
    /// reference was created
    id: Option<FileIdentifier>,
}

#[derive(Debug)]
enum PageIdentifier {
    Index(usize),
    Label(String),
}

impl PageIdentifier {
    pub fn from_obj(obj: Object, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        Ok(if let Ok(name) = resolver.assert_string(obj.clone()) {
            PageIdentifier::Label(name)
        } else {
            let idx = resolver.assert_unsigned_integer(obj)?;

            PageIdentifier::Index(idx as usize)
        })
    }
}

impl ReferenceXObject {
    pub fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let f = FileSpecification::from_obj(dict.expect_object("F", resolver)?, resolver)?;
        let page = PageIdentifier::from_obj(dict.expect_object("Page", resolver)?, resolver)?;
        let id = dict
            .get_arr("ID", resolver)?
            .map(|objs| FileIdentifier::from_arr(objs, resolver))
            .transpose()?;

        Ok(Self { f, page, id })
    }
}

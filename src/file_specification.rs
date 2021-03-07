use crate::{
    assert_empty,
    catalog::{assert_len, Collection},
    error::{ParseError, PdfResult},
    objects::{Dictionary, Object, ObjectType},
    Resolve,
};

#[derive(Debug, Clone)]
pub enum FileSpecification {
    Simple(FileSpecificationString),
    Full(FullFileSpecification),
}

impl FileSpecification {
    pub fn from_obj(obj: Object, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        match resolver.resolve(obj)? {
            Object::String(s) => Ok(FileSpecification::Simple(FileSpecificationString::new(s))),
            Object::Dictionary(dict) => Ok(FileSpecification::Full(
                FullFileSpecification::from_dict(dict, resolver)?,
            )),
            obj => Err(ParseError::MismatchedObjectType {
                found: obj,
                expected: ObjectType::Dictionary,
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FullFileSpecification {
    /// The name of the file system that shall be used to interpret this file
    /// specification.
    ///
    /// If this entry is present, all other entries in the dictionary shall be
    /// interpreted by the designated file system. PDF shall define only one
    /// standard file system name, URL; an application can register other names.
    ///
    /// This entry shall be independent of the F and UF entries.
    file_system: Option<String>,

    /// A file specification string as descibed by the `FileSpecificationString` docs
    /// or (if the file system is URL) a uniform resource locator.
    ///
    /// The UF entry should be used in addition to the F entry. The UF entry provides
    /// cross-platform and cross-language compatibility and the F entry provides backwards
    /// compatibility.
    file_specification_string: Option<FileSpecificationString>,

    /// A Unicode text string that provides file specification. This is a text string
    /// encoded using PDFDocEncoding or UTF-16BE with a leading byte-order marker.
    ///
    /// The F entry should be included along with this entry for backwards compatibility
    /// reasons.
    unicode_file_specification_string: Option<FileSpecificationString>,

    /// An array of two byte strings constituting a file identifier that should be included
    /// in the referenced file.
    ///
    /// NOTE: The use of this entry improves an application's chances of finding the
    /// intended file and allows it to warn the user if the file has changed since the link
    /// was made.
    id: Option<FileIdentifier>,

    /// A flag indicating whether the file referenced by the file specification is volatile
    /// (changes frequently with time).
    ///
    /// If the value is true, applications shall not cache a copy of the file. For example,
    /// a movie annotation referencing a URL to a live video camera could set this flag to
    /// true to notify the conforming reader that it should re-acquire the movie each time
    /// it is played.
    ///
    /// Default value: false.
    is_volatile: bool,

    /// A dictionary containing a subset of the keys F, and UF, corresponding to the entries
    /// by those names in the file specification dictionary.
    ///
    /// The value of each such key shall be an embedded file stream containing the corresponding
    /// file. If this entry is present, the Type entry is required and the file specification
    /// dictionary shall be indirectly referenced.
    ef: Option<Dictionary>,

    /// A dictionary with the same structure as the EF dictionary, which shall be present.
    ///
    /// Each key in the RF dictionary shall also be present in the EF dictionary. Each value
    /// shall be a related files arra identifying files that are related to the corresponding
    /// file in the EF dictionary.
    ///
    /// If this entry is present, the Type entry is required and the file specification dictionary
    /// shall be indirectly referenced.
    rf: Option<Dictionary>,

    /// Descriptive text associated with the file specification.
    ///
    /// It shall be used for files in the EmbeddedFiles name tree
    description: Option<String>,

    /// A collection item dictionary, which shall be used to create the user interface for
    /// portable collections
    collection_item_dict: Option<Collection>,
}

impl FullFileSpecification {
    const TYPE: &'static str = "Typespec";

    pub(crate) fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        dict.expect_type(Self::TYPE, resolver, false)?;

        let file_system = dict.get_name("Fs", resolver)?;
        let file_specification_string = dict
            .get_string("F", resolver)?
            .map(FileSpecificationString::new);
        let unicode_file_specification_string = dict
            .get_string("UF", resolver)?
            .map(FileSpecificationString::new);
        dict.get_string("DOS", resolver)?;
        dict.get_string("Mac", resolver)?;
        dict.get_string("Unix", resolver)?;
        let id = dict
            .get_arr("UF", resolver)?
            .map(|objs| FileIdentifier::from_arr(objs, resolver))
            .transpose()?;
        let is_volatile = dict.get_bool("V", resolver)?.unwrap_or(false);
        let ef = dict.get_dict("EF", resolver)?;
        let rf = dict.get_dict("RF", resolver)?;
        let description = dict.get_string("Desc", resolver)?;
        let collection_item_dict = dict.get_dict("CI", resolver)?.map(|_| todo!());

        assert_empty(dict);

        Ok(FullFileSpecification {
            file_system,
            file_specification_string,
            unicode_file_specification_string,
            id,
            is_volatile,
            ef,
            rf,
            description,
            collection_item_dict,
        })
    }
}

/// The standard format for representing a simple file specification in string form divides
/// the string into component substrings separated by the SOLIDUS character (2Fh) (/). The
/// SOLIDUS is a generic component separator that shall be mapped to the appropriate
/// platform-specific separator when generating a platform-dependent file name. Any of the
/// components may be empty. If a component contains one or more literal SOLIDI, each shall
/// be preceded by a REVERSE SOLIDUS (5Ch) (\), which in turn shall be preceded by another
/// REVERSE SOLIDUS to indicate that it is part of the string and not an escape character.
#[derive(Debug, Clone)]
pub struct FileSpecificationString(String);

impl FileSpecificationString {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, Clone)]
struct EmbeddedFileStream;
#[derive(Debug, Clone)]
struct RelatedFilesArray;

#[derive(Debug, Clone)]
pub struct FileIdentifier(String, String);

impl FileIdentifier {
    pub fn from_arr(arr: Vec<Object>, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        assert_len(&arr, 2)?;

        let mut iter = arr.into_iter().map(|obj| resolver.assert_string(obj));

        Ok(FileIdentifier(iter.next().unwrap()?, iter.next().unwrap()?))
    }
}

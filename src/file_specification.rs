use crate::{
    catalog::Collection,
    error::PdfResult,
    objects::{Dictionary, Object},
    FromObj, Resolve,
};

#[derive(Debug, Clone, PartialEq)]
pub enum FileSpecification<'a> {
    Simple(FileSpecificationString),
    Full(FullFileSpecification<'a>),
}

impl<'a> FromObj<'a> for FileSpecification<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        match resolver.resolve(obj)? {
            Object::String(s) => Ok(FileSpecification::Simple(FileSpecificationString::new(s))),
            obj @ Object::Dictionary(..) => Ok(FileSpecification::Full(
                FullFileSpecification::from_obj(obj, resolver)?,
            )),
            obj => anyhow::bail!("expected dictionary or string, found {:?}", obj),
        }
    }
}

#[derive(Debug, Clone, PartialEq, FromObj)]
#[obj_type("Typespec")]
pub struct FullFileSpecification<'a> {
    /// The name of the file system that shall be used to interpret this file
    /// specification.
    ///
    /// If this entry is present, all other entries in the dictionary shall be
    /// interpreted by the designated file system. PDF shall define only one
    /// standard file system name, URL; an application can register other names.
    ///
    /// This entry shall be independent of the F and UF entries.
    #[field("Fs")]
    file_system: Option<String>,

    /// A file specification string as descibed by the `FileSpecificationString` docs
    /// or (if the file system is URL) a uniform resource locator.
    ///
    /// The UF entry should be used in addition to the F entry. The UF entry provides
    /// cross-platform and cross-language compatibility and the F entry provides backwards
    /// compatibility.
    #[field("F")]
    file_specification_string: Option<FileSpecificationString>,

    /// A Unicode text string that provides file specification. This is a text string
    /// encoded using PDFDocEncoding or UTF-16BE with a leading byte-order marker.
    ///
    /// The F entry should be included along with this entry for backwards compatibility
    /// reasons.
    #[field("UF")]
    unicode_file_specification_string: Option<FileSpecificationString>,

    /// A file specification string representing a DOS file name.
    ///
    /// This entry is obsolescent and should not be used by conforming writers.
    #[field("DOS")]
    dos: Option<FileSpecificationString>,

    /// A file specification string representing a Mac OS file name.
    ///
    /// This entry is obsolescent and should not be used by conforming writers.
    #[field("Mac")]
    mac: Option<String>,

    /// A file specification string representing a UNIX file name.
    ///
    /// This entry is obsolescent and should not be used by conforming writers.
    #[field("Unix")]
    unix: Option<String>,

    /// An array of two byte strings constituting a file identifier that should be included
    /// in the referenced file.
    ///
    /// NOTE: The use of this entry improves an application's chances of finding the
    /// intended file and allows it to warn the user if the file has changed since the link
    /// was made.
    #[field("ID")]
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
    #[field("V", default = false)]
    is_volatile: bool,

    /// A dictionary containing a subset of the keys F, and UF, corresponding to the entries
    /// by those names in the file specification dictionary.
    ///
    /// The value of each such key shall be an embedded file stream containing the corresponding
    /// file. If this entry is present, the Type entry is required and the file specification
    /// dictionary shall be indirectly referenced.
    #[field("EF")]
    ef: Option<Dictionary<'a>>,

    /// A dictionary with the same structure as the EF dictionary, which shall be present.
    ///
    /// Each key in the RF dictionary shall also be present in the EF dictionary. Each value
    /// shall be a related files arra identifying files that are related to the corresponding
    /// file in the EF dictionary.
    ///
    /// If this entry is present, the Type entry is required and the file specification dictionary
    /// shall be indirectly referenced.
    #[field("RF")]
    rf: Option<Dictionary<'a>>,

    /// Descriptive text associated with the file specification.
    ///
    /// It shall be used for files in the EmbeddedFiles name tree
    #[field("Desc")]
    description: Option<String>,

    /// A collection item dictionary, which shall be used to create the user interface for
    /// portable collections
    #[field("CI")]
    collection_item_dict: Option<Collection>,
}

/// The standard format for representing a simple file specification in string form divides
/// the string into component substrings separated by the SOLIDUS character (2Fh) (/). The
/// SOLIDUS is a generic component separator that shall be mapped to the appropriate
/// platform-specific separator when generating a platform-dependent file name. Any of the
/// components may be empty. If a component contains one or more literal SOLIDI, each shall
/// be preceded by a REVERSE SOLIDUS (5Ch) (\), which in turn shall be preceded by another
/// REVERSE SOLIDUS to indicate that it is part of the string and not an escape character.
#[derive(Debug, Clone, PartialEq)]
pub struct FileSpecificationString(String);

impl FileSpecificationString {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}

impl<'a> FromObj<'a> for FileSpecificationString {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(Self(String::from_obj(obj, resolver)?))
    }
}

#[derive(Debug, Clone)]
struct EmbeddedFileStream;
#[derive(Debug, Clone)]
struct RelatedFilesArray;

#[derive(Debug, Clone, PartialEq)]
pub struct FileIdentifier(pub [String; 2]);

// todo: should be derivable
impl<'a> FromObj<'a> for FileIdentifier {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(FileIdentifier(<[String; 2]>::from_obj(obj, resolver)?))
    }
}

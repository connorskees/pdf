use std::collections::HashMap;

use crate::{
    objects::{Dictionary, Name, Object},
    resolve::Resolve,
    FromObj, PdfResult,
};

#[derive(Debug, Clone, FromObj)]
pub struct Encryption<'a> {
    /// The name of the preferred security handler for this document. It shall
    /// be the name of the security handler that was used to encrypt the
    /// document. If SubFilter is not present, only this security handler
    /// shall be used when opening the document. If it is present, a
    /// conforming reader can use any security handler that implements the
    /// format specified by SubFilter.
    #[field("Filter")]
    filter: Name,

    /// A name that completely specifies the format and interpretation of the
    /// contents of the encryption dictionary. It allows security handlers
    /// other than the one specified by Filter to decrypt the document. If
    /// this entry is absent, other security handlers shall not decrypt the
    /// document.
    #[field("SubFilter")]
    sub_filter: Option<Name>,

    /// A code specifying the algorithm to be used in encrypting and decrypting
    /// the document
    #[field("V")]
    v: Option<EncryptionAlgorithm>,

    /// The length of the encryption key, in bits.
    ///
    /// The value shall be a multiple of 8, in the range 40 to 128.
    ///
    /// Default value: 40
    #[field("Length", default = 40)]
    length: i32,

    /// A dictionary whose keys shall be crypt filter names and whose values
    /// shall be the corresponding crypt filter dictionaries. Every crypt
    /// filter used in the document shall have an entry in this dictionary,
    /// except for the standard crypt filter names
    ///
    /// The conforming reader shall ignore entries in CF dictionary with the
    /// keys equal to those listed in Table 26 and use properties of the
    /// respective standard crypt filters.
    #[field("CF")]
    crypt_filters: Option<HashMap<String, CryptFilter>>,

    /// The name of the crypt filter that shall be used by default when decrypting
    /// streams
    ///
    /// The name shall be a key in the CF dictionary or a standard crypt filter
    /// name specified in Table 26. All streams in the document, except for
    /// cross-reference streams or streams that have a Crypt entry in their
    /// Filter array, shall be decrypted by the security handler, using
    /// this crypt filter
    ///
    /// Default value: Identity
    #[field("StmF", default = Name("Identity".to_owned()))]
    stream_filter: Name,

    /// The name of the crypt filter that shall be used when decrypting all
    /// strings in the document
    ///
    /// The name shall be a key in the CF dictionary or a standard crypt filter
    /// name specified in Table 26.
    #[field("StrF", default = Name("Identity".to_owned()))]
    string_filter: Name,

    /// The name of the crypt filter that shall be used when encrypting embedded
    /// file streams that do not have their own crypt filter specifier
    ///
    /// The name shall correspond to a key in the CF dictionary or a standard
    /// crypt filter name specified in Table 26.
    ///
    /// This entry shall be provided by the security handler. Conforming writers
    /// shall respect this value when encrypting embedded files, except for
    /// embedded file streams that have their own crypt filter specifier.
    /// If this entry is not present, and the embedded file stream does not
    /// contain a crypt filter specifier, the stream shall be encrypted
    /// using the default stream crypt filter specified by StmF.
    #[field("EFF", default = Name("Identity".to_owned()))]
    embedded_file_filter: Name,

    // todo: additional fields here
    #[field]
    other: Dictionary<'a>,
}

#[pdf_enum(Integer)]
pub enum EncryptionAlgorithm {
    /// An algorithm that is undocumented. This value shall not be used.
    Undocumented = 0,

    /// Encryption of data using the RC4 or AES algorithms" with an encryption
    /// key length of 40 bits
    Rc4OrAes40Bits = 1,

    /// Encryption of data using the RC4 or AES algorithms but permitting encryption
    /// key lengths greater than 40 bits
    Rc4OrAesGt40Bits = 2,

    /// An unpublished algorithm that permits encryption key lengths ranging from
    /// 40 to 128 bits. This value shall not appear in a conforming PDF file
    Unpublished = 3,

    /// The security handler defines the use of encryption and decryption in the
    /// document, using the rules specified by the CF, StmF, and StrF
    /// entries.
    BasedOnOtherEntries = 4,
}

#[derive(Debug, Clone, FromObj)]
#[obj_type("CryptFilter")]
pub struct CryptFilter {
    /// The method used, if any, by the conforming reader to decrypt data.
    ///
    /// When the value is V2 or AESV2, the application may ask once for this
    /// encryption key and cache the key for subsequent use for streams
    /// that use the same crypt filter. Therefore, there shall be a
    /// one-to-one relationship between a crypt filter name and the
    /// corresponding encryption key.
    #[field("CFM")]
    crypt_filter_method: Option<CryptFilterMethod>,

    /// The event to be used to trigger the authorization that is required to
    /// access encryption keys used by this filter. If authorization fails,
    /// the event shall fail.
    ///
    /// If this filter is used as the value of StrF or StmF in the encryption
    /// dictionary, the conforming reader shall ignore this key and behave
    /// as if the value is DocOpen.
    #[field("AuthEvent")]
    auth_event: Option<AuthEvent>,

    /// The bit length of the encryption key. It shall be a multiple of 8 in the
    /// range of 40 to 128.
    #[field("Length")]
    length: Option<i32>,
}

#[pdf_enum]
#[derive(Default)]
enum AuthEvent {
    /// Authorization shall be required when a document is opened
    #[default]
    DocOpen = "DocOpen",

    /// Authorization shall be required when accessing embedded files
    EFOpen = "EFOpen",
}

#[pdf_enum]
#[derive(Default)]
enum CryptFilterMethod {
    /// The application shall not decrypt data but shall direct the input stream
    /// to the security handler for decryption.
    #[default]
    None = "None",

    /// The application shall ask the security handler for the encryption key and
    /// shall implicitly decrypt data with "Algorithm 1: Encryption of data
    /// using the RC4 or AES algorithms", using the RC4 algorithm.
    V2 = "V2",

    /// The application shall ask the security handler for the encryption key and
    /// shall implicitly decrypt data with "Algorithm 1: Encryption of data
    /// using the RC4 or AES algorithms", using the AES algorithm in Cipher
    /// Block Chaining (CBC) mode with a 16-byte block size and an
    /// initialization vector that shall be randomly generated and placed
    /// as the first 16 bytes in the stream or string.
    AesV2 = "AESV2",
}

struct UserAccessPermissions(u32);

impl<'a> FromObj<'a> for UserAccessPermissions {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(Self(resolver.assert_unsigned_integer(obj)?))
    }
}

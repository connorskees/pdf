#![allow(warnings)]

use std::collections::HashMap;

use aes::cipher::{
    block_padding::{NoPadding, Pkcs7},
    generic_array::GenericArray,
    BlockDecrypt, BlockDecryptMut, KeyInit, KeyIvInit,
};

use crate::{
    file_specification::FileIdentifier,
    objects::{Dictionary, Name, Object, Reference},
    resolve::Resolve,
    stream::Stream,
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

    // todo: below fields should only be in standard security handler, not all
    /// A number specifying which revision of the standard security handler shall
    /// be used to interpret this dictionary
    ///
    /// 2 => if the document is encrypted with a V value less than 2 and does not
    /// have any of the access permissions set to 0 (by means of the P
    /// entry, below) that are designated “Security handlers of revision 3
    /// or greater”
    ///
    /// 3 => if the document is encrypted with a V value of 2 or 3, or has any
    /// “Security handlers of revision 3 or greater” access permissions set
    /// to 0
    ///
    /// 4 => if the document is encrypted with a V value of 4
    #[field("R")]
    revision_number: i32,

    /// A 32-byte string, based on both the owner and user passwords, that shall
    /// be used in computing the encryption key and in determining whether
    /// a valid owner password was entered
    #[field("O")]
    owner: String,

    /// A set of flags specifying which operations shall be permitted when the
    /// document is opened with user access
    #[field("U")]
    user: String,

    /// A set of flags specifying which operations shall be permitted when the
    /// document is opened with user access
    #[field("P")]
    user_permission_flags: UserAccessPermissions,

    /// Indicates whether the document-level metadata stream shall be encrypted
    #[field("EncryptMetadata", default = true)]
    encrypt_metadata: bool,

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
    #[field("CFM", default = CryptFilterMethod::default())]
    crypt_filter_method: CryptFilterMethod,

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

#[derive(Debug, Copy, Clone)]
struct UserAccessPermissions(i32);

impl<'a> FromObj<'a> for UserAccessPermissions {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(Self(resolver.assert_integer(obj)?))
    }
}

const PADDING: [u8; 32] = [
    0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41, 0x64, 0x00, 0x4E, 0x56, 0xFF, 0xFA, 0x01, 0x08,
    0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80, 0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53, 0x69, 0x7A,
];

/// Manages encryption for entire document
pub struct SecurityHandler<'a> {
    encryption: Encryption<'a>,
    file_identifier: FileIdentifier,
}

impl<'a> SecurityHandler<'a> {
    pub fn new(encryption: Encryption<'a>, file_identifier: FileIdentifier) -> Self {
        assert_eq!(encryption.length % 8, 0);

        Self {
            encryption,
            file_identifier,
        }
    }

    fn compute_encryption_key(&self, password: &[u8]) -> Vec<u8> {
        let padded_password = if password.len() >= 32 {
            password[..32].to_owned()
        } else {
            let mut b = password.to_owned();
            b.extend_from_slice(&PADDING[..32 - password.len()]);
            b
        };

        assert_eq!(padded_password.len(), 32);

        let mut hash = md5::Context::new();

        hash.consume(&padded_password);
        hash.consume(&self.encryption.owner.as_bytes());
        hash.consume(&self.encryption.user_permission_flags.0.to_le_bytes());
        hash.consume(&self.file_identifier.0[0].as_bytes());

        if self.encryption.revision_number >= 4 && !self.encryption.encrypt_metadata {
            hash.consume(&[0xFF, 0xFF, 0xFF, 0xFF]);
        }

        let mut hash = hash.compute();

        if self.encryption.revision_number >= 3 {
            let n = self.encryption.length / 8;
            for _ in 0..50 {
                hash = md5::compute(&hash[..n as usize]);
            }
        }

        let n = if self.encryption.revision_number == 2 {
            5
        } else {
            self.encryption.length / 8
        };

        hash[..n as usize].to_vec()
    }

    pub fn decrypt_string(&mut self, s: Vec<u8>) -> PdfResult<String> {
        todo!()
    }

    pub fn decrypt_stream(&self, mut stream: Vec<u8>, reference: Reference) -> PdfResult<Vec<u8>> {
        let filter_name = &self.encryption.stream_filter.0;
        let filter = self
            .encryption
            .crypt_filters
            .as_ref()
            .unwrap()
            .get(filter_name)
            .unwrap();

        assert_eq!(filter.crypt_filter_method, CryptFilterMethod::AesV2);

        dbg!(&self.encryption);

        let mut key = self.compute_encryption_key(&[]);
        let key_len = key.len();

        key.extend_from_slice(&reference.object_number.to_le_bytes()[..3]);
        key.extend_from_slice(&reference.generation.to_le_bytes()[..2]);
        key.extend_from_slice(b"sAlT");

        let key = &md5::compute(&key)[..key_len];

        let mut dec = aes::Aes128Dec::new_from_slice(&key).unwrap();

        let key = &[0; 32];

        assert_eq!(stream.len() % 16, 0);

        type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

        let (iv, ciphertext) = stream.split_at_mut(16);
        let cipher = Aes128CbcDec::new_from_slices(key, iv).unwrap();

        cipher.decrypt_padded_mut::<Pkcs7>(ciphertext).unwrap();

        Ok(stream)
    }
}

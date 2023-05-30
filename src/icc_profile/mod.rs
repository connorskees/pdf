/*!

ICC file parsing and color conversion

See https://www.color.org/icc1v42.pdf

Additionally:
 - https://en.wikipedia.org/wiki/ICC_profile
 - https://en.wikipedia.org/wiki/CIE_1931_color_space
 - https://en.wikipedia.org/wiki/Color_management
 - http://fileformats.archiveteam.org/wiki/ICC_profile

*/

use std::fmt::{self, Write};

use crate::{date::Date, error::PdfResult};

use self::{data_types::XyzNumber, parse::IccProfileParser};

mod data_types;
mod parse;

#[derive(Debug)]
pub struct IccProfile {
    pub header: IccProfileHeader,
    pub tag_table: IccTagTable,
}

impl IccProfile {
    pub fn new(buffer: &[u8]) -> PdfResult<Self> {
        let mut parser = IccProfileParser::new(buffer);

        parser.parse()
    }
}

#[derive(Debug)]
pub struct IccProfileHeader {
    /// The exact size obtained by combining the profile header, the tag table,
    /// and the tagged element data, including the pad bytes for the last
    /// tag
    pub profile_size: u32,

    /// This field may be used to identify the preferred CMM to be used. If used,
    /// it shall match a CMM type signature registered in the ICC registry.
    /// If no preferred CMM is identified, this field shall be set to zero
    pub preferred_cmm_type: IccTagSignature,
    pub profile_version_number: u32,
    pub profile_device_class: IccTagSignature,
    pub colour_space: IccTagSignature,
    pub profile_connection_space: IccTagSignature,
    pub created_at: Date,
    pub primary_platform_signature: IccTagSignature,
    pub profile_flags: u32,
    pub device_manufacturer: IccTagSignature,
    pub device_model: IccTagSignature,
    pub device_attributes: u64,

    /// The rendering intent field shall specify the rendering intent which should be
    /// used (or, in the case of a DeviceLink profile, was used) when this
    /// profile is (was) combined with another profile. In a sequence of more
    /// than two profiles, it applies to the combination of this profile and the
    /// next profile in the sequence and not to the entire sequence. Typically,
    /// the user or application will set the rendering intent dynamically at
    /// runtime or embedding time. Therefore, this flag may not have any meaning
    /// until the profile is used in some context, e.g in a DeviceLink or an
    /// embedded source profile.
    pub rendering_intent: u32,
    xyz_values: XyzNumber,
    pub profile_creator_signature: IccTagSignature,
    pub profile_id: [u8; 16],
    pub reserved: [u8; 28],
}

#[derive(Debug, Clone)]
pub struct IccTagTable {
    tag_count: u32,
    entries: Vec<TagTableEntry>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct TagTableEntry {
    pub(super) signature: u32,
    pub(super) offset: u32,
    pub(super) len: u32,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct IccTagSignature(pub [u8; 4]);

impl IccTagSignature {
    pub const fn new(tag: [u8; 4]) -> Self {
        Self(tag)
    }
}

impl fmt::Debug for IccTagSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('"')?;
        f.write_char(self.0[0] as char)?;
        f.write_char(self.0[1] as char)?;
        f.write_char(self.0[2] as char)?;
        f.write_char(self.0[3] as char)?;
        f.write_char('"')?;

        Ok(())
    }
}

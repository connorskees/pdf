mod cvt;
mod font_directory;
mod glyf;
mod head;
mod loca;
mod maxp;
mod name;
mod table_name;
mod tag;

pub use cvt::CvtTable;
pub use font_directory::{DirectoryTableEntry, FontDirectory, OffsetSubtable, TableDirectory};
pub use glyf::{
    CompoundGlyphPartDescription, GlyfTable, GlyphDescription, SimpleGlyph, TrueTypeGlyph,
};
pub use head::{Head, HeadFlags, MacStyle};
pub use loca::LocaTable;
pub use maxp::{MaxpPostscriptTable, MaxpTable};
pub use name::{NameRecord, NameTable};
pub use table_name::TableName;
pub use tag::TableTag;

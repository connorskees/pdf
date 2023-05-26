mod font_directory;
mod glyf;
mod head;
mod loca;
mod maxp;
mod name;
mod tag;

pub use font_directory::{DirectoryTableEntry, FontDirectory, OffsetSubtable, TableDirectory};
pub use glyf::{
    CompoundGlyphPartDescription, GlyfTable, GlyphDescription, SimpleGlyph, TrueTypeGlyph,
};
pub use head::{Head, HeadFlags, MacStyle};
pub use loca::LocaTable;
pub use maxp::{MaxpPostscriptTable, MaxpTable};
pub use name::TableName;
pub use tag::TableTag;
